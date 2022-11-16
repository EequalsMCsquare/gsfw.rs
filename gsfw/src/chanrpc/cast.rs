use std::marker::PhantomData;

use super::{
    broker::{AsyncSender, Sender},
    ChanCtx, Proto,
};

pub struct CastTx<P, N, E, Tx> {
    from: N,
    tx: Tx,

    _p: PhantomData<P>,
    _e: PhantomData<E>,
}

impl<P, N, E, Tx> CastTx<P, N, E, Tx> {
    pub fn new(from: N, tx: Tx) -> Self {
        Self {
            from,
            tx,
            _p: PhantomData,
            _e: PhantomData,
        }
    }
}

impl<P, N, E, Tx> CastTx<P, N, E, Tx>
where
    P: Proto,
    N: Clone,
    Tx: AsyncSender<ChanCtx<P, N, E>>,
{
    pub async fn cast(&self, msg: P) {
        if let Err(err) = self.tx.send(ChanCtx::new_cast(msg, self.from.clone())).await {
            tracing::error!("fail to cast. {}", err)
        }
    }
}

impl<P, N, E, Tx> CastTx<P, N, E, Tx>
where
    P: Proto,
    N: Clone,
    Tx: Sender<ChanCtx<P, N, E>>,
{
    pub fn blocking_cast(&self, msg: P) {
        if let Err(err) = self
            .tx
            .blocking_send(ChanCtx::new_cast(msg, self.from.clone()))
        {
            tracing::error!("fail to cast. {}", err)
        }
    }
}
