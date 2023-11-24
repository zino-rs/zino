//! Scheduler for sync and async cron jobs.

use std::{future::Future, time::Duration};

mod async_job;
mod job;

pub use async_job::{AsyncCronJob, AsyncJob, AsyncJobScheduler};
pub use job::{CronJob, Job, JobScheduler};

/// An interface for scheduling sync jobs.
pub trait Scheduler {
    /// Returns the duration till the next job is supposed to run.
    fn time_till_next_job(&self) -> Duration;

    /// Increments time for the scheduler and executes any pending jobs.
    fn tick(&mut self);
}

/// An interface for scheduling async jobs.
pub trait AsyncScheduler {
    /// Returns the duration till the next job is supposed to run.
    fn time_till_next_job(&self) -> Duration;

    /// Increments time for the scheduler and executes any pending jobs asynchronously.
    fn tick(&mut self) -> impl Future<Output = ()> + Send;
}
