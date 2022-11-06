use super::{adaptor::Adaptor, agent::AgentFuture};
use crate::{
    codec::{Decoder, Encoder},
    error,
};

use std::{marker::PhantomData, task::Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadHalf, WriteHalf};
use tokio_util::codec::{FramedRead, FramedWrite};
use tower::Service;

pub struct AgentService<Enc, Dec, A, RM>
where
    Enc: Encoder<RM>,
    Dec: Decoder,
{
    enc: Enc,
    dec: Dec,
    adaptor: A,
    _rm: PhantomData<RM>,
}

impl<Request, Enc, Dec, A, RM> Service<Request> for AgentService<Enc, Dec, A, RM>
where
    Request: AsyncRead + AsyncWrite,
    Enc: Encoder<RM> + Clone,
    Dec: Decoder + Clone,
    A: Adaptor<std::result::Result<Dec::Item, Dec::Error>, RM>,
{
    type Response = ();

    type Error = error::Error;

    type Future = AgentFuture<
        FramedRead<ReadHalf<Request>, Dec>,
        FramedWrite<WriteHalf<Request>, Enc>,
        A,
        RM,
    >;

    fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let (rd, wr) = tokio::io::split(req);
        let stream = FramedRead::with_capacity(rd, self.dec.clone(), 1024);
        let sink = FramedWrite::new(wr, self.enc.clone());
        AgentFuture::new(stream, sink, self.adaptor.clone())
    }
}
