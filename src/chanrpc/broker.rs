use super::{cast::CastTx, ChanCtx, Proto};
pub use super::{
    receiver::{AsyncReceiver, Receiver},
    sender::{AsyncSender, Sender},
};
use async_trait::async_trait;
use std::collections::HashMap;

pub trait Broker<P, N, E, Tx, Rx>
where
    P: Proto,
    Tx: Sender<ChanCtx<P, N, E>> + Clone,
    Rx: Receiver<ChanCtx<P, N, E>>,
{
    fn new(name: N, tx_map: &HashMap<N, Tx>) -> Self;
    fn name(&self) -> N;
    fn tx(&self, name: N) -> &Tx;
    fn channel(size: usize) -> (Tx, Rx);

    fn cast_tx(&self, name: N) -> super::cast::CastTx<P, N, E, Tx> {
        CastTx::new(self.name(), self.tx(name).clone())
    }

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
pub trait AsyncBroker<P, N, E, Tx, Rx>: Broker<P, N, E, Tx, Rx>
where
    P: Proto,
    N: Send,
    E: Send,
    Tx: AsyncSender<ChanCtx<P, N, E>> + Clone,
    Rx: AsyncReceiver<ChanCtx<P, N, E>>,
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

impl<T, P, N, E, Tx, Rx> AsyncBroker<P, N, E, Tx, Rx> for T
where
    P: Proto,
    N: Send,
    E: Send,
    Tx: AsyncSender<ChanCtx<P, N, E>> + Clone,
    Rx: AsyncReceiver<ChanCtx<P, N, E>>,
    T: Broker<P, N, E, Tx, Rx>,
{
}
