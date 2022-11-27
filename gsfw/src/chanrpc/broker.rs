use super::{calltx::CallTx, casttx::CastTx, ChanCtx};
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::mpsc;

#[async_trait]
pub trait Broker {
    type Proto: super::Proto;
    type Name: super::Name;
    type Err: Send;

    fn new(
        name: Self::Name,
        tx_map: &HashMap<Self::Name, mpsc::Sender<ChanCtx<Self::Proto, Self::Name, Self::Err>>>,
    ) -> Self;
    fn name(&self) -> Self::Name;
    fn tx(&self, name: Self::Name) -> &mpsc::Sender<ChanCtx<Self::Proto, Self::Name, Self::Err>>;

    fn cast_tx(&self, name: Self::Name) -> CastTx<Self::Proto, Self::Name, Self::Err> {
        CastTx::new(self.name(), self.tx(name).clone())
    }

    fn call_tx(&self, name: Self::Name) -> CallTx<Self::Proto, Self::Name, Self::Err> {
        CallTx::new(self.name(), self.tx(name).clone())
    }

    async fn cast(&self, to: Self::Name, msg: Self::Proto) {
        let tx = self.tx(to);
        if let Err(err) = tx.send(ChanCtx::new_cast(msg, self.name())).await {
            tracing::error!("fail to cast. {}", err)
        }
    }

    fn blocking_cast(&self, to: Self::Name, msg: Self::Proto) {
        let tx = self.tx(to);
        if let Err(err) = tx.blocking_send(ChanCtx::new_cast(msg, self.name())) {
            tracing::error!("fail to cast. {}", err)
        }
    }

    async fn call(&self, to: Self::Name, msg: Self::Proto) -> Result<Self::Proto, Self::Err> {
        let (ctx, rx) = ChanCtx::new_call(msg, self.name());
        let tx = self.tx(to);
        if let Err(err) = tx.send(ctx).await {
            tracing::error!("fail to request. {}", err);
            panic!("{}", err);
        }
        rx.await.expect("blocking_recv reply error")
    }

    fn blocking_call(&self, to: Self::Name, msg: Self::Proto) -> Result<Self::Proto, Self::Err> {
        let (ctx, rx) = ChanCtx::new_call(msg, self.name());
        let tx = self.tx(to);
        if let Err(err) = tx.blocking_send(ctx) {
            tracing::error!("fail to request. {}", err);
            panic!("{}", err);
        }
        rx.blocking_recv().expect("blocking_recv reply error")
    }
}
