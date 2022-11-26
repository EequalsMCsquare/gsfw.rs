use tokio::sync::mpsc;

use crate::chanrpc::{broker::Broker, ChanCtx};

pub trait ComponentBuilder<B>
where
    B: Broker,
{
    // component name
    fn name(&self) -> B::Name;
    fn build(self: Box<Self>) -> Box<dyn super::Component<B>>;
    fn set_rx(&mut self, rx: mpsc::Receiver<ChanCtx<B::Proto, B::Name, B::Err>>);
    fn set_broker(&mut self, broker: B);
    fn runtime(&self) -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap()
    }
}
