use async_trait::async_trait;

#[async_trait]
pub trait Adaptor<S, R>: Send + Clone
{
    async fn send(&mut self, msg: S);
    async fn recv(&mut self) -> Option<R>;
}
