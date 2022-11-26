use super::ChanCtx;
use tokio::sync::{mpsc, oneshot};

pub struct CallTx<P, N, E> {
    // name of caller
    from: N,
    // send of call to
    tx: mpsc::Sender<ChanCtx<P, N, E>>,
}

impl<P, N, E> CallTx<P, N, E> {
    pub fn new(from: N, tx: mpsc::Sender<ChanCtx<P, N, E>>) -> Self {
        Self { from, tx }
    }
}

impl<P, N, E> CallTx<P, N, E>
where
    P: super::Proto,
    N: super::Name,
{
    pub async fn call(&self, msg: P) -> oneshot::Receiver<Result<P, E>> {
        let (ctx, rx) = ChanCtx::new_call(msg, self.from.clone());
        if let Err(err) = self.tx.send(ctx).await {
            tracing::error!("fail to call. {}", err);
        }
        rx
    }
}
