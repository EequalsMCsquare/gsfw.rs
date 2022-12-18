use async_trait::async_trait;
mod builder;
pub use builder::ComponentBuilder;
use std::error::Error as StdError;

use crate::chanrpc::broker::Broker;

#[async_trait]
pub trait Component<B>: Send
where
    B: Broker,
{
    fn name(&self) -> B::Name;
    async fn init(self: Box<Self>) -> Result<Box<dyn Component<B>>, Box<dyn StdError + Send>>;
    async fn run(self: Box<Self>) -> Result<(), Box<dyn StdError + Send>>;
}
