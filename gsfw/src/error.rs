#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("read 0 bytes. connection close")]
    ReadZero,
    #[error("send error: {0}")]
    SendError(String),
    #[error("recv error: {0}")]
    RecvError(String),
    #[error("invalid frame format")]
    FrameFormat,
    #[error(transparent)]
    IO(#[from] tokio::io::Error),
    #[error("no component. you must register at least one component to the Game.")]
    NoComponent,
    #[error("sink send error occur. close agent")]
    SinkSend,
    #[error("adaptor send error occur. close agent")]
    AdaptorSend,
    #[error("adaptor recv error occur. close agent")]
    AdaptorRecv,
    #[error("adaptor ready error occur. close agent")]
    AdaptorReady,
    #[error("unknown protocol. MSG_ID: {0}")]
    UnknownPB(i32),
    #[error("decode error. {0}")]
    Decode(String),
    #[error("encode error. {0}")]
    Encode(String),
    #[error("mismatch variant when cast to {0}")]
    VariantCast(&'static str),
}
