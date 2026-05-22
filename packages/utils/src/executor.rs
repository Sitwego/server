use std::{future::Future, time::Duration};

#[derive(Clone)]
pub struct Executor;

impl Executor {
    pub fn spawn_detached_task<F>(&self, future: F)
    where
        F: 'static + Send + Future<Output = ()>,
    {
        tokio::spawn(future);
    }

    /// Sleep for the given duration.
    pub fn sleep(&self, duration: Duration) -> impl Future<Output = ()> {
        tokio::time::sleep(duration)
    }
}
