use crate::{chanrpc::ChanCtx, component::ComponentBuilder};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Debug,
    marker::PhantomData,
};

pub struct GameBuilder<P, N, B, E, Tx, Rx>
where
    P: crate::chanrpc::Proto,
    N: std::hash::Hash + Eq + Send + Debug,
    B: crate::chanrpc::broker::Broker<P, N, E, Tx, Rx>,
    Tx: crate::chanrpc::broker::Sender<crate::chanrpc::ChanCtx<P, N, E>> + Clone,
    Rx: crate::chanrpc::broker::Receiver<crate::chanrpc::ChanCtx<P, N, E>>,
{
    component_set: HashSet<N>,
    component_builders: Vec<Box<dyn ComponentBuilder<P, N, B, E, Rx>>>,

    _tx: PhantomData<Tx>,
}

impl<P, N, B, E, Tx, Rx> GameBuilder<P, N, B, E, Tx, Rx>
where
    P: crate::chanrpc::Proto + 'static,
    N: std::hash::Hash + Eq + Send + Debug + Clone + 'static,
    B: crate::chanrpc::broker::Broker<P, N, E, Tx, Rx>,
    E: Send + Debug + 'static,
    Tx: crate::chanrpc::broker::Sender<crate::chanrpc::ChanCtx<P, N, E>> + Clone + 'static,
    Rx: crate::chanrpc::broker::Receiver<crate::chanrpc::ChanCtx<P, N, E>>,
{
    pub fn new() -> Self {
        Self {
            component_set: Default::default(),
            component_builders: Default::default(),
            _tx: PhantomData,
        }
    }

    pub fn component<CB>(mut self, component_builder: CB) -> Self
    where
        CB: ComponentBuilder<P, N, B, E, Rx> + 'static,
    {
        if let Some(_) = self.component_set.get(&component_builder.name()) {
            panic!(
                "component[{:?}] already registered",
                component_builder.name()
            );
        }
        self.component_set.insert(component_builder.name());
        self.component_builders.push(Box::new(component_builder));
        self
    }

    pub fn serve(self) -> Result<super::Game<N, E>, crate::error::Error> {
        if self.component_builders.len() == 0 {
            return Err(crate::error::Error::NoComponent);
        }
        let names: Vec<_> = self
            .component_builders
            .iter()
            .map(|builder| builder.name())
            .collect();

        let chans: Vec<_> = self
            .component_builders
            .iter()
            .map(|_| B::channel(1024))
            .collect();
        let txs: Vec<_> = chans.iter().map(|(tx, _)| tx.clone()).collect();
        let mut rxs: VecDeque<_> = chans.into_iter().map(|(_, rx)| rx).collect();

        let tx_pairs: Vec<(N, Tx)> = std::iter::zip(
            names.iter().map(|n| n.clone()),
            txs.iter().map(|tx| tx.clone()),
        )
        .collect();
        let tx_map: HashMap<_, _> = tx_pairs
            .iter()
            .map(|(n, tx)| (n.clone(), tx.clone()))
            .collect();
        let ctrl_c_future = tokio::spawn(async move {
            if let Err(err) = tokio::signal::ctrl_c().await {
                tracing::error!("ctrl_c error: {}", err);
            }
            tracing::info!("CTRL+C pressed, begin to clean up");
            // prevent blocking the task drive thread
            match std::thread::spawn(move || {
                for (k, tx) in tx_pairs {
                    tracing::debug!("sending shutdown to {:?}", k);
                    let k = k.clone();
                    if let Err(err) =
                        tx.blocking_send(ChanCtx::new_cast(P::proto_shutdown(), k.clone()))
                    {
                        tracing::error!("fail to send shutdown to {:?}: {}", k, err);
                    }
                }
            })
            .join()
            {
                Ok(_) => {}
                Err(err) => tracing::error!("fail to join thread: {:?}", err),
            }
        });
        let component_handles = self
            .component_builders
            .into_iter()
            .map(|mut builder| {
                builder.set_broker(B::new(builder.name(), &tx_map));
                builder.set_rx(rxs.pop_front().unwrap());
                tracing::debug!("ComponentBuilder {:?} setup complete", builder.name());
                let mut component = builder.build();
                tracing::debug!("component {:?} setup complete", component.name());
                let name = component.name();
                super::ComponentHandle {
                    join: tokio::spawn(async move {
                        component.init().await.unwrap();
                        component.run().await
                    }),
                    name,
                }
            })
            .collect();
        tracing::info!("all components launch complete, running: {:?}", names);
        tracing::info!("press CTRL+C to terminate the app");
        Ok(super::Game {
            component_handles,
            poll_component: None,
            ctrl_c_future,
            ctrl_c_trigger: false,
        })
    }
}
