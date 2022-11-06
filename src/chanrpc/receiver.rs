use async_trait::async_trait;
use tokio::sync::mpsc;
pub trait Receiver<T> {
    fn blocking_recv(&mut self) -> Option<T>;
}

impl<T> Receiver<T> for mpsc::Receiver<T> {
    fn blocking_recv(&mut self) -> Option<T> {
        mpsc::Receiver::blocking_recv(self)
    }
}

impl<T> Receiver<T> for crossbeam::channel::Receiver<T> {
    fn blocking_recv(&mut self) -> Option<T> {
        crossbeam::channel::Receiver::recv(&self).ok()
    }
}

#[async_trait]
pub trait AsyncReceiver<T>: Receiver<T>
where
    T: Send,
{
    async fn recv(&mut self) -> Option<T>;
}

#[async_trait]
impl<T> AsyncReceiver<T> for mpsc::Receiver<T>
where
    T: Send,
{
    async fn recv(&mut self) -> Option<T> {
        mpsc::Receiver::recv(self).await
    }
}
