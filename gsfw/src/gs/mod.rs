use futures::{ready, Future, FutureExt};
use pin_project::pin_project;
use std::error::Error as StdError;
use std::{collections::VecDeque, fmt::Debug};
use tokio::task::JoinHandle;

mod builder;
pub use builder::GameBuilder;

#[derive(Debug)]
struct ComponentHandle<N> {
    // rt: tokio::runtime::Runtime,
    // join: JoinHandle<Result<(), Box<dyn StdError + Send>>>,
    join: std::thread::JoinHandle<Result<(), Box<dyn StdError + Send>>>,
    name: N,
}

#[derive(Debug)]
#[pin_project]
pub struct Game<N>
where
    N: Send + Debug,
{
    component_handles: VecDeque<ComponentHandle<N>>,
    poll_component: Option<ComponentHandle<N>>,
    ctrl_c_future: JoinHandle<()>,
    ctrl_c_trigger: bool,
}

impl<N> Future for Game<N>
where
    N: Send + Debug,
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
        while let Some(handle) = this.poll_component.take() {
            // match ready!(handle
            //     .join
            //     .join()
            //     .expect("handle's thread join error")
            //     .poll_unpin(cx))
            // {
            //     Ok(_) => tracing::info!("[{:?}] join success", handle.name),
            //     Err(err) => tracing::error!("error occur while wait for component join: {}", err),
            // }
            match handle.join.join().expect("component thread join error") {
                Ok(_) => tracing::info!("[{:?}] join success", handle.name),
                Err(err) => tracing::error!("error occur while wait for component join: {}", err),
            }

            // shutdown component's runtime
            // handle.rt.shutdown_background();
            *this.poll_component = this.component_handles.pop_front();
        }
        std::task::Poll::Ready(())
    }
}
