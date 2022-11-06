use async_trait::async_trait;
mod builder;
pub use builder::ComponentBuilder;

#[async_trait]
pub trait Component<P, N, E>: Send
where
    P: Send,
    N: Send,
{
    fn name(&self) -> N;
    async fn init(&mut self) -> Result<(), E>;
    async fn run(self) -> Result<(), E>;
}
