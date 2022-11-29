use std::{cell::RefCell, fmt::Debug, hash::Hash};

use tokio::sync::oneshot;

type ReplySender<P, E> = oneshot::Sender<Result<P, E>>;
type ReplyReceiver<P, E> = oneshot::Receiver<Result<P, E>>;

pub trait Proto: Send {
    fn proto_shutdown() -> Self;
}

pub trait Name: Send + Hash + Eq + Clone + Debug {}

#[derive(Debug)]
pub struct ChanCtx<P, N, E> {
    payload: RefCell<Option<P>>,
    from: N,
    reply_chan: Option<ReplySender<P, E>>,
}

unsafe impl<P: Send, N: Send, E: Send> Send for ChanCtx<P, N, E> {}
unsafe impl<P: Sync, N: Sync, E:Sync> Sync for ChanCtx<P, N, E> {}

#[allow(dead_code)]
impl<P, N, E> ChanCtx<P, N, E>
where
    P: Proto,
{
    pub fn new_call(msg: P, from: N) -> (ChanCtx<P, N, E>, ReplyReceiver<P, E>) {
        let (tx, rx) = oneshot::channel();
        (
            Self {
                payload: RefCell::new(msg.into()),
                from,
                reply_chan: Some(tx),
            },
            rx,
        )
    }

    pub fn new_cast(msg: P, from: N) -> ChanCtx<P, N, E> {
        Self {
            payload: RefCell::new(Some(msg)),
            from,
            reply_chan: None,
        }
    }

    pub fn from(&self) -> &N {
        &self.from
    }

    pub fn payload(&self) -> P {
        let mut p = self.payload.borrow_mut();
        p.take().expect("calling twice payload()")
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
