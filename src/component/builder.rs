use crate::chanrpc::{self, broker};

pub trait ComponentBuilder<P, N, B, E, Tx, Rx>
where
    B: broker::Broker<P, N, E, Tx>,
    P: chanrpc::Proto,
    N: Send,
    E: Send,
    Tx: broker::Sender<chanrpc::ChanCtx<P, N, E>>,
    Rx: broker::Receiver<chanrpc::ChanCtx<P, N, E>>,
{
    // component name
    fn name(&self) -> N;
    fn set_rx(&mut self, rx: Rx);
    fn set_broker(&mut self, broker: B);
}
