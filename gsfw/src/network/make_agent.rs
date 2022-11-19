use super::{adaptor::Adaptor, AdaptorBuilder};
use crate::{
    codec::{Decoder, Encoder},
    error,
};

use futures::{Future, SinkExt, StreamExt};
use std::{fmt::Debug, marker::PhantomData, pin::Pin, task::Poll};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{FramedRead, FramedWrite};
use tower::Service;

pub struct AgentService<Enc, Dec, AB, RM> {
    enc: Enc,
    dec: Dec,
    adaptor_builder: AB,
    _rm: PhantomData<RM>,
}

impl<Enc, Dec, AB, RM> AgentService<Enc, Dec, AB, RM> {
    pub fn new(encoder: Enc, decoder: Dec, adaptor_builder: AB) -> Self {
        Self {
            enc: encoder,
            dec: decoder,
            adaptor_builder,
            _rm: PhantomData,
        }
    }
}

impl<Request, Enc, Dec, AB, RM> Service<Request> for AgentService<Enc, Dec, AB, RM>
where
    Request: AsyncRead + AsyncWrite + 'static + Send,
    Enc: Encoder<RM> + Clone + 'static + Send,
    Dec: Decoder + Clone + 'static + Send,
    Enc::Error: Debug,
    AB: AdaptorBuilder + 'static,
    AB::Adaptor: Adaptor<RecvItem = RM, Dec = Dec, Enc = Enc>,
    RM: Send,
    Dec::Error: Send,
    Dec::Item: Send,
{
    type Response = ();

    type Error = error::Error;

    type Future = Pin<Box<dyn Future<Output = Result<(), crate::error::Error>> + 'static + Send>>;

    fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let (rd, wr) = tokio::io::split(req);
        let stream = FramedRead::with_capacity(rd, self.dec.clone(), 1024);
        let sink = FramedWrite::new(wr, self.enc.clone());
        let adaptor_builder = self.adaptor_builder.clone();
        Box::pin(async move {
            let mut adaptor = adaptor_builder.build().await;
            let (mut stream, mut sink) = match adaptor.ready(stream, sink).await {
                Ok(pair) => pair,
                Err(err) => {
                    tracing::error!("fail to call Adaptor::ready: {:?}", err);
                    return Err(crate::error::Error::AdaptorReady);
                }
            };
            loop {
                tokio::select! {
                    frame = stream.next() => {
                        if let Some(frame) = frame {
                            if let Err(err) = adaptor.send(frame).await {
                                tracing::error!("fail to call Adaptor::send: {:?}", err);
                                return Err(crate::error::Error::AdaptorSend)
                            }
                        } else {
                            return Result::<(), _>::Err(crate::error::Error::ReadZero)
                        }
                    },
                    sc = adaptor.recv() => {
                        match sc {
                            Ok(sc) => {
                                if let Some(sc) = sc {
                                    if let Err(err) = sink.send(sc).await {
                                        tracing::error!("fail to call Sink::send: {:?}", err);
                                        return Err(crate::error::Error::SinkSend)
                                    }
                                }
                            },
                            Err(err) => {
                                tracing::error!("fail to call Adaptor::recv: {:?}", err);
                                return Err(crate::error::Error::AdaptorRecv)
                            }
                        }
                    }
                }
            }
        })
    }
}
