use super::adaptor::Adaptor;
use futures::{FutureExt, Sink, SinkExt, Stream, StreamExt};
use pin_project::pin_project;
use std::{future::Future, marker::PhantomData, task::Poll};

#[pin_project]
pub struct AgentFuture<FR, FW, A, RM> {
    pub(crate) stream: FR,
    pub(crate) sink: FW,
    pub(crate) adaptor: A,

    _rm: PhantomData<RM>,
}

impl<FR, FW, A, RM> AgentFuture<FR, FW, A, RM>
where
    FR: Stream,
    FW: Sink<RM>,
    A: Adaptor<FR::Item, RM>,
{
    pub fn new(stream: FR, sink: FW, adaptor: A) -> Self {
        Self {
            stream,
            sink,
            adaptor,
            _rm: PhantomData,
        }
    }
}

impl<FR, FW, A, RM> Future for AgentFuture<FR, FW, A, RM>
where
    FR: Stream + Unpin,
    FW: Sink<RM> + Unpin,
    A: Adaptor<FR::Item, RM>,
{
    type Output = Result<(), crate::error::Error>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let mut select_future = Box::pin(async {
            loop {
                tokio::select! {
                    frame = this.stream.next() => {
                        if let Some(frame) = frame {
                            this.adaptor.send(frame).await;
                        } else {
                            return Result::<(), _>::Err(crate::error::Error::ReadZero)
                        }
                    },
                    Some(sc) = this.adaptor.recv() => {
                        if let Err(_) = this.sink.send(sc).await {
                            tracing::error!("fail to call Adaptor::send")
                        }
                    }
                }
            }
        });
        select_future.poll_unpin(cx)
    }
}