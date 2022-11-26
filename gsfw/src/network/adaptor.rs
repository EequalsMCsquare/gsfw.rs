use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{Decoder, Encoder, FramedRead, FramedWrite};

#[async_trait]
pub trait Adaptor: Send {
    type RecvItem: Send;
    type Dec: Decoder;
    type Enc: Encoder<Self::RecvItem>;

    async fn ready<R, W>(
        &mut self,
        stream: FramedRead<R, Self::Dec>,
        sink: FramedWrite<W, Self::Enc>,
    ) -> Result<(FramedRead<R, Self::Dec>, FramedWrite<W, Self::Enc>), Box<dyn std::error::Error>>
    where
        R: AsyncRead + Send + Unpin,
        W: AsyncWrite + Send + Unpin;

    async fn send(
        &mut self,
        msg: Result<<Self::Dec as Decoder>::Item, <Self::Dec as Decoder>::Error>,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// None -> connection close by peer or timeout
    /// Err -> error happen while attempting to call Adaptor::recv
    async fn recv(&mut self) -> Result<Option<Self::RecvItem>, Box<dyn std::error::Error + Send>>;
}

#[async_trait]
pub trait AdaptorBuilder: Send + Clone {
    type Adaptor: Adaptor;
    async fn build(self) -> Self::Adaptor;
}
