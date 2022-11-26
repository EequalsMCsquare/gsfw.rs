use crate::chanrpc::{self, broker};

pub trait ComponentBuilder<P, N, B, E, Rx>
where
    P: chanrpc::Proto,
    N: Send,
    E: Send,
    Rx: broker::Receiver<chanrpc::ChanCtx<P, N, E>>,
{
    // component name
    fn name(&self) -> N;
    fn build(self: Box<Self>) -> Box<dyn super::Component<P, N, E>>;
    fn set_rx(&mut self, rx: Rx);
    fn set_broker(&mut self, broker: B);
    fn runtime(&self) -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap()
    }
}
