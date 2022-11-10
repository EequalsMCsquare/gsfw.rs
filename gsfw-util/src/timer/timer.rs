use futures::{ready, Future, FutureExt};
use pin_project::pin_project;
use std::{
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
};

static TIMER_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
pub struct Snapshot {
    pub id: u64,
    pub start: std::time::Instant,
    pub end: std::time::Instant,
}

#[derive(Debug)]
pub struct Meta<T> {
    pub id: u64,
    pub start: std::time::Instant,
    pub end: std::time::Instant,
    pub data: Option<T>,
}

impl<T> Meta<T> {
    pub fn new(start: std::time::Instant, end: std::time::Instant, data: T) -> Self {
        Self {
            id: TIMER_ID.fetch_add(1, Ordering::Acquire),
            start,
            end,
            data: Some(data),
        }
    }
}

#[pin_project]
pub struct Timer<T> {
    meta: Meta<T>,
    sleep: Pin<Box<tokio::time::Sleep>>,
}

impl<T> Timer<T>
where
    T: 'static,
{
    pub fn new(start: std::time::Instant, end: std::time::Instant, data: T) -> Self {
        Self {
            sleep: Box::pin(tokio::time::sleep_until(end.into())),
            meta: Meta {
                id: TIMER_ID.fetch_add(1, Ordering::Acquire),
                start,
                end,
                data: Some(data),
            },
        }
    }
}

impl<T> Future for Timer<T> {
    type Output = Result<T, super::Error<T>>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        ready!(this.sleep.poll_unpin(cx));
        if this.meta.data.is_none() {
            return std::task::Poll::Ready(Err(super::Error::TimerFinish));
        }
        let data = this.meta.data.take().unwrap();
        std::task::Poll::Ready(Ok(data))
    }
}
