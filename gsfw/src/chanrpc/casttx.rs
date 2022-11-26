use tokio::sync::mpsc;
use super::ChanCtx;

pub struct CastTx<P, N, E> {
    from: N,
    tx: mpsc::Sender<ChanCtx<P, N, E>>,
}

impl<P, N, E> CastTx<P, N, E> {
    pub fn new(from: N, tx: mpsc::Sender<ChanCtx<P, N, E>>) -> Self {
        Self { from, tx }
    }
}

impl<P, N, E> CastTx<P, N, E>
where
    P: super::Proto,
    N: super::Name,
{
    pub async fn cast(&self, msg: P) {
        if let Err(err) = self
            .tx
            .send(ChanCtx::new_cast(msg, self.from.clone()))
            .await
        {
            tracing::error!("fail to cast. {}", err)
        }
    }
}
