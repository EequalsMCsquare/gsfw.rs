use super::adaptor::Adaptor;
use crate::{
    codec::{Decoder, Encoder},
    error,
};

use futures::{Future, SinkExt, StreamExt};
use std::{marker::PhantomData, pin::Pin, task::Poll};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{FramedRead, FramedWrite};
use tower::Service;

pub struct AgentService<Enc, Dec, A, RM> {
    enc: Enc,
    dec: Dec,
    adaptor: A,
    _rm: PhantomData<RM>,
}

impl<Enc, Dec, A, RM> AgentService<Enc, Dec, A, RM> {
    pub fn new(encoder: Enc, decoder: Dec, adaptor: A) -> Self {
        Self {
            enc: encoder,
            dec: decoder,
            adaptor,
            _rm: PhantomData,
        }
    }
}

impl<Request, Enc, Dec, A, RM> Service<Request> for AgentService<Enc, Dec, A, RM>
where
    Request: AsyncRead + AsyncWrite + 'static,
    Enc: Encoder<RM> + Clone + 'static,
    Dec: Decoder + Clone + 'static,
    A: Adaptor<std::result::Result<Dec::Item, Dec::Error>, RM> + 'static,
{
    type Response = ();

    type Error = error::Error;

    type Future = Pin<Box<dyn Future<Output = Result<(), crate::error::Error>> + 'static>>;

    fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let (rd, wr) = tokio::io::split(req);
        let mut stream = FramedRead::with_capacity(rd, self.dec.clone(), 1024);
        let mut sink = FramedWrite::new(wr, self.enc.clone());
        let mut adaptor = self.adaptor.clone();
        Box::pin(async move {
            loop {
                tokio::select! {
                    frame = stream.next() => {
                        if let Some(frame) = frame {
                            adaptor.send(frame).await;
                        } else {
                            return Result::<(), _>::Err(crate::error::Error::ReadZero)
                        }
                    },
                    Some(sc) = adaptor.recv() => {
                        if let Err(_) = sink.send(sc).await {
                            tracing::error!("fail to call Adaptor::send")
                        }
                    }
                }
            }
        })
    }
}
