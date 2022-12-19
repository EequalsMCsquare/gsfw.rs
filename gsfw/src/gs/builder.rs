use tokio::sync::mpsc;
use tracing::{instrument, Instrument};

use crate::{
    chanrpc::{broker::Broker, ChanCtx},
    component::ComponentBuilder,
};
use std::collections::{HashMap, HashSet, VecDeque};

pub struct GameBuilder<B: Broker> {
    component_set: HashSet<B::Name>,
    component_builders: Vec<Box<dyn ComponentBuilder<B>>>,
}

impl<B: Broker + 'static> GameBuilder<B> {
    pub fn new() -> Self {
        Self {
            component_set: Default::default(),
            component_builders: Default::default(),
        }
    }

    #[instrument(level="info", skip_all, name="add_component", fields(name=?component_builder.name()))]
    pub fn component<CB>(mut self, component_builder: CB) -> Self
    where
        CB: ComponentBuilder<B> + 'static,
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

    #[instrument(level = "info", skip(self), name="game_serve")]
    pub fn serve(self) -> Result<super::Game<B::Name>, crate::error::Error> {
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
            .map(|_| mpsc::channel(1024))
            .collect();
        let txs: Vec<_> = chans.iter().map(|(tx, _)| tx.clone()).collect();
        let mut rxs: VecDeque<_> = chans.into_iter().map(|(_, rx)| rx).collect();

        let tx_pairs: Vec<(_, _)> = std::iter::zip(
            names.iter().map(|n| n.clone()),
            txs.iter().map(|tx| tx.clone()),
        )
        .collect();
        let tx_map: HashMap<_, _> = tx_pairs
            .iter()
            .map(|(n, tx)| (n.clone(), tx.clone()))
            .collect();

        // future of SIGINT event
        let ctrl_c_future = tokio::spawn(
            async move {
                if let Err(err) = tokio::signal::ctrl_c().await {
                    tracing::error!("ctrl_c error: {}", err);
                }
                tracing::debug!("CTRL+C pressed, begin to clean up");
                // prevent blocking the task drive thread
                for (k, tx) in tx_pairs {
                    tracing::trace!("sending shutdown to {:?}", k);
                    if let Err(err) = tx
                        .send(ChanCtx::new_cast(
                            <B::Proto as crate::chanrpc::Proto>::proto_shutdown(),
                            k,
                        ))
                        .await
                    {
                        tracing::error!("fail to send shutdown. {}", err);
                    }
                }
            }
            .instrument(tracing::info_span!("waiting for ctrl_c...").or_current()),
        );

        let component_handles = self
            .component_builders
            .into_iter()
            .map(|mut builder| {
                builder.set_broker(B::new(builder.name(), &tx_map));
                builder.set_rx(rxs.pop_front().unwrap());
                tracing::debug!("ComponentBuilder {:?} setup complete", builder.name());
                let rt = builder.runtime();
                let component = builder.build();
                let component_name = component.name();
                tracing::debug!("component {:?} setup complete", component.name());
                let name = component.name();
                let join = std::thread::spawn(move || {
                    let ret = rt.block_on(
                        async move { component.init().await?.run().await }.instrument(
                            tracing::debug_span!("component", name=?component_name)
                                .or_current(),
                        ),
                    );
                    rt.shutdown_background();
                    ret
                });
                super::ComponentHandle { join, name }
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
