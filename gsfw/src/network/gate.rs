use std::fmt::Debug;
use tokio::net::{TcpListener, TcpStream};
use tower::Service;
use tracing::{instrument, Instrument};

pub struct Gate {
    inner: TcpListener,
}

impl Gate {
    pub fn new<T>(listen_addr: T) -> Self
    where
        T: std::net::ToSocketAddrs + Debug,
    {
        Self {
            inner: tokio::net::TcpListener::from_std(
                std::net::TcpListener::bind(&listen_addr)
                    .expect(format!("fail to listen on {:?}", listen_addr).as_str()),
            )
            .unwrap(),
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
                Ok((stream, remote)) => {
                    // tracing::trace!("incoming connection: {:?}", remote);
                    let fut = service.call(stream);
                    tokio::spawn(
                        fut.instrument(tracing::trace_span!("gate_agent", ?remote).or_current()),
                    );
                }
                Err(err) => {
                    tracing::error!("fail to accpect. {}", err);
                }
            }
        }
    }
}
