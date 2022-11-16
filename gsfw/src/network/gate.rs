use std::fmt::Debug;
use tokio::net::{TcpListener, TcpStream};
use tower::Service;

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
                Ok((stream, addr)) => {
                    tracing::debug!("incoming connection: {:?}", addr);
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
