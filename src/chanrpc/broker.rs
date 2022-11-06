pub use super::{
    receiver::{AsyncReceiver, Receiver},
    sender::{AsyncSender, Sender},
};
use super::{ChanCtx, Proto};
use async_trait::async_trait;
use std::collections::HashMap;

pub trait Broker<P, N, E, Tx>
where
    P: Proto,
    Tx: Sender<ChanCtx<P, N, E>>,
{
    fn new(name: N, tx_map: &HashMap<N, Tx>) -> Self;
    fn name(&self) -> N;
    fn tx(&self, name: N) -> &Tx;

    fn blocking_cast(&self, to: N, msg: P) {
        let tx = self.tx(to);
        if let Err(err) = tx.blocking_send(ChanCtx::new_cast(msg, self.name())) {
            tracing::error!("fail to cast. {:?}", err)
        }
    }

    fn blocking_call(&self, to: N, msg: P) -> Result<P, E> {
        let (ctx, rx) = ChanCtx::new_call(msg, self.name());
        let tx = self.tx(to);
        if let Err(err) = tx.blocking_send(ctx) {
            tracing::error!("fail to request. {}", err);
            panic!("{}", err);
        }
        rx.blocking_recv().expect("blocking_recv reply error")
    }
}

#[async_trait]
pub trait AsyncBroker<P, N, E, Tx>: Broker<P, N, E, Tx>
where
    P: Proto,
    N: Send,
    E: Send,
    Tx: AsyncSender<ChanCtx<P, N, E>>,
{
    async fn cast(&self, to: N, msg: P)
    where
        P: 'async_trait,
        N: 'async_trait,
    {
        let tx = self.tx(to);
        if let Err(err) = tx.send(ChanCtx::new_cast(msg, self.name())).await {
            tracing::error!("fail to cast. {}", err)
        }
    }

    async fn call(&self, to: N, msg: P) -> Result<P, E>
    where
        P: 'async_trait,
        N: 'async_trait,
    {
        let (ctx, rx) = ChanCtx::new_call(msg, self.name());
        let tx = self.tx(to);
        if let Err(err) = tx.send(ctx).await {
            tracing::error!("fail to request. {}", err);
            panic!("{}", err);
        }
        rx.await.expect("blocking_recv reply error")
    }
}

impl<T, P, N, E, Tx> AsyncBroker<P, N, E, Tx> for T
where
    P: Proto,
    N: Send,
    E: Send,
    Tx: AsyncSender<ChanCtx<P, N, E>>,
    T: Broker<P, N, E, Tx>,
{
}

#[cfg(test)]
mod test_mpsc_broker {
    use tokio::sync::mpsc;
    use crate::chanrpc::{ChanCtx, Proto};
    use super::Broker;

    enum TestProto {
        CtrlShutdown,
    }
    impl Proto for TestProto {
        fn proto_shutdown() -> Self {
            Self::CtrlShutdown
        }
    }

    #[derive(Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
    enum ComponentName {
        ComponentA,
        ComponentB,
        ComponentC,
    }

    struct TestBroker {
        a: mpsc::Sender<ChanCtx<TestProto, ComponentName, anyhow::Error>>,
        b: mpsc::Sender<ChanCtx<TestProto, ComponentName, anyhow::Error>>,
        c: mpsc::Sender<ChanCtx<TestProto, ComponentName, anyhow::Error>>,
    }

    impl
        Broker<
            TestProto,
            ComponentName,
            anyhow::Error,
            mpsc::Sender<ChanCtx<TestProto, ComponentName, anyhow::Error>>,
        > for TestBroker
    {
        fn new(
            _name: ComponentName,
            tx_map: &std::collections::HashMap<
                ComponentName,
                mpsc::Sender<ChanCtx<TestProto, ComponentName, anyhow::Error>>,
            >,
        ) -> Self {
            Self {
                a: tx_map.get(&ComponentName::ComponentA).unwrap().clone(),
                b: tx_map.get(&ComponentName::ComponentB).unwrap().clone(),
                c: tx_map.get(&ComponentName::ComponentC).unwrap().clone(),
            }
        }

        fn name(&self) -> ComponentName {
            ComponentName::ComponentA
        }

        fn tx(
            &self,
            name: ComponentName,
        ) -> &mpsc::Sender<ChanCtx<TestProto, ComponentName, anyhow::Error>> {
            match name {
                ComponentName::ComponentA => &self.a,
                ComponentName::ComponentB => &self.b,
                ComponentName::ComponentC => &self.c,
            }
        }
    }
}

#[cfg(test)]
mod test_mpsc_async_broker {}
