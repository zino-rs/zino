//! Scheduler for sync and async cron jobs.

use std::{future::Future, io, time::Duration};

mod async_job;
mod job;

pub use async_job::{AsyncCronJob, AsyncJob, AsyncJobScheduler};
pub use job::{CronJob, Job, JobScheduler};

/// An interface for scheduling sync jobs.
pub trait Scheduler {
    /// Returns `true` if the scheduler is ready to run.
    fn is_ready(&self) -> bool;

    /// Returns the duration till the next job is supposed to run.
    fn time_till_next_job(&self) -> Option<Duration>;

    /// Increments time for the scheduler and executes any pending jobs.
    fn tick(&mut self);
}

/// An interface for scheduling async jobs.
pub trait AsyncScheduler {
    /// Returns `true` if the scheduler is ready to run.
    fn is_ready(&self) -> bool;

    /// Returns `true` if the scheduler is blocking.
    fn is_blocking(&self) -> bool;

    /// Returns the duration till the next job is supposed to run.
    fn time_till_next_job(&self) -> Option<Duration>;

    /// Increments time for the scheduler and executes any pending jobs asynchronously.
    fn tick(&mut self) -> impl Future<Output = ()> + Send;

    /// Runs the scheduler and returns an `std::io::Error` if failed.
    fn run(self) -> impl Future<Output = io::Result<()>> + Send;
}

#[cfg(feature = "apalis")]
impl AsyncScheduler for apalis::prelude::Monitor {
    #[inline]
    fn is_ready(&self) -> bool {
        true
    }

    #[inline]
    fn is_blocking(&self) -> bool {
        true
    }

    #[inline]
    fn time_till_next_job(&self) -> Option<Duration> {
        None
    }

    #[inline]
    async fn tick(&mut self) {}

    #[inline]
    async fn run(self) -> io::Result<()> {
        self.run().await
    }
}
