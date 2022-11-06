use std::fmt::Debug;

pub use tokio_util::codec::Encoder;
pub use tokio_util::codec::Decoder;

pub trait Codec: Send {
    type EncodeFrom: Send;
    type DecodeTo: Send;
    type Error: From<std::io::Error> + Send + Debug;
    type Decoder: tokio_util::codec::Decoder<Error = Self::Error> + Send;
    type Encoder: tokio_util::codec::Encoder<Self::EncodeFrom, Error = Self::Error> + Send;

    fn encoder(&self) -> Self::Encoder;
    fn decoder(&self) -> Self::Decoder;
}
