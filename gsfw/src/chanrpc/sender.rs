use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::error::Error as BrokerError;

pub trait Sender<T>: Send {
    fn blocking_send(&self, msg: T) -> Result<(), BrokerError>;
}

impl<T> Sender<T> for mpsc::Sender<T> 
where T: Send
{
    fn blocking_send(&self, msg: T) -> Result<(), BrokerError> {
        mpsc::Sender::blocking_send(self, msg)
            .map_err(|err| BrokerError::SendError(err.to_string()))
    }
}

impl<T> Sender<T> for crossbeam::channel::Sender<T> 
where T: Send
{
    fn blocking_send(&self, msg: T) -> Result<(), BrokerError> {
        self.send(msg)
            .map_err(|err| BrokerError::SendError(err.to_string()))
    }
}

#[async_trait]
pub trait AsyncSender<T>: Sender<T> + Send + Sync {
    async fn send(&self, msg: T) -> Result<(), BrokerError>;
}

#[async_trait]
impl<T> AsyncSender<T> for mpsc::Sender<T>
where
    T: Send,
{
    async fn send(&self, msg: T) -> Result<(), BrokerError> {
        mpsc::Sender::send(self, msg)
            .await
            .map_err(|err| BrokerError::SendError(err.to_string()))
    }
}
