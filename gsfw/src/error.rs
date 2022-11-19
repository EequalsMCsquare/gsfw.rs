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
    AdaptorReady
}
