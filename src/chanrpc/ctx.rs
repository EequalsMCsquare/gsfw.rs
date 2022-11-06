use tokio::sync::oneshot;

type ReplySender<P, E> = oneshot::Sender<Result<P, E>>;
type ReplyReceiver<P, E> = oneshot::Receiver<Result<P, E>>;

pub trait Proto: Send {
    fn proto_shutdown() -> Self;
}

#[derive(Debug)]
pub struct ChanCtx<P, N, E> {
    pub payload: P,
    pub from: N,
    reply_chan: Option<ReplySender<P, E>>,
}

#[allow(dead_code)]
impl<P, N, E> ChanCtx<P, N, E>
where
    P: Proto,
{
    pub fn new_call(msg: P, from: N) -> (ChanCtx<P, N, E>, ReplyReceiver<P, E>) {
        let (tx, rx) = oneshot::channel();
        (
            Self {
                payload: msg,
                from,
                reply_chan: Some(tx),
            },
            rx,
        )
    }

    pub fn new_cast(msg: P, from: N) -> ChanCtx<P, N, E> {
        Self {
            payload: msg,
            from,
            reply_chan: None,
        }
    }

    pub fn ok(self, reply: P) {
        if let Some(reply_chan) = self.reply_chan {
            if let Err(_) = reply_chan.send(Ok(reply)) {
                tracing::error!("ChanRpc fail to reply with Ok. receiver dropped");
            }
            return;
        }
        tracing::warn!("attempt to reply to a non request ctx");
    }

    pub fn err(self, err: E) {
        if let Some(reply_chan) = self.reply_chan {
            if let Err(_) = reply_chan.send(Err(err)) {
                tracing::error!("ChanRpc fail to reply with Err. receiver dropped");
            }
            return;
        }
        tracing::warn!("attempt to reply to a non request ctx");
    }
}
