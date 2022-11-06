use super::adaptor::Adaptor;
use crate::codec;
use std::{fmt::Debug, marker::PhantomData};
use tokio::net::{TcpListener, TcpStream};
use tower::Service;

pub struct Server<Enc, Dec, A, RM> {
    inner: TcpListener,
    adaptor: A,
    make_encoder: fn() -> Enc,
    make_decoder: fn() -> Dec,

    rm: PhantomData<RM>,
}

impl<Enc, Dec, A, RM> Server<Enc, Dec, A, RM>
where
    Enc: codec::Encoder<RM> + Send + 'static,
    Dec: codec::Decoder + Send + 'static,
    A: Adaptor<Result<Dec::Item, Dec::Error>, RM> + 'static,
    RM: Send + 'static,
{
    pub fn new<T>(
        listen_addr: T,
        make_encoder: fn() -> Enc,
        make_decoder: fn() -> Dec,
        adaptor: A,
    ) -> Self
    where
        T: std::net::ToSocketAddrs + Debug,
    {
        Self {
            inner: tokio::net::TcpListener::from_std(
                std::net::TcpListener::bind(&listen_addr)
                    .expect(format!("fail to listen on {:?}", listen_addr).as_str()),
            )
            .unwrap(),
            make_encoder,
            make_decoder,
            adaptor,
            rm: PhantomData,
        }
    }

    pub async fn serve<S>(self, mut service: S)
    where
        S: Service<TcpStream, Response = ()>,
        S::Error: Send + 'static,
        S::Future: Send + 'static,
    {
        loop {
            match self.inner.accept().await {
                Ok((stream, addr)) => {
                    tracing::debug!("incoming connection: {:?}", addr);
                    // let (rd, wr) = tokio::io::split(stream);
                    // let agent = AgentFuture::<_, _, _, _>::new(
                    //     FramedRead::with_capacity(rd, (self.make_decoder)(), 1024),
                    //     FramedWrite::new(wr, (self.make_encoder)()),
                    //     self.adaptor.clone(),
                    // );
                    let fut = service.call(stream);
                    tokio::spawn(fut);
                }
                Err(err) => {
                    tracing::debug!("fail to accpect. {}", err);
                }
            }
        }
    }
}
