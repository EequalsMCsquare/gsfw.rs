use std::{collections::VecDeque, fmt::Debug};
use tokio::task::JoinHandle;

use futures::{ready, Future, FutureExt};
use pin_project::pin_project;

mod builder;
pub use builder::GameBuilder;

#[derive(Debug)]
struct ComponentHandle<N, E>
where
    N: Debug,
    E: Debug,
{
    join: JoinHandle<Result<(), E>>,
    name: N,
}

#[derive(Debug)]
#[pin_project]
pub struct Game<N, E>
where
    N: Send + Debug,
    E: Debug,
{
    component_handles: VecDeque<ComponentHandle<N, E>>,
    poll_component: Option<ComponentHandle<N, E>>,
    ctrl_c_future: JoinHandle<()>,
    ctrl_c_trigger: bool,
}

impl<N, E> Future for Game<N, E>
where
    N: Send + Debug,
    E: Debug,
{
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        if !*this.ctrl_c_trigger {
            match ready!(this.ctrl_c_future.poll_unpin(cx)) {
                Ok(_) => *this.ctrl_c_trigger = true,
                Err(err) => tracing::error!("ctrl_c polling error: {}", err),
            };
            this.poll_component
                .replace(this.component_handles.pop_front().unwrap());
        }
        while let Some(handle) = this.poll_component {
            match ready!(handle.join.poll_unpin(cx)) {
                Ok(_) => tracing::info!("[{:?}] join success", handle.name),
                Err(err) => tracing::error!("error occur while wait for component join: {}", err),
            }
            *this.poll_component = this.component_handles.pop_front();
        }
        std::task::Poll::Ready(())
    }
}
