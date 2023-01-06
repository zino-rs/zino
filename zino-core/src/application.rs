use crate::{AsyncCronJob, CronJob, Job, JobScheduler};
use std::{collections::HashMap, io, thread, time::Instant};

/// Application.
pub trait Application {
    /// Router.
    type Router;

    /// Creates a new application.
    fn new() -> Self;

    /// Returns the start time.
    fn start_time(&self) -> Instant;

    /// Registers routes.
    fn register(self, routes: HashMap<&'static str, Self::Router>) -> Self;

    /// Spawns a new thread to run jobs.
    fn spawn(self, jobs: HashMap<&'static str, CronJob>) -> Self
    where
        Self: Sized,
    {
        let mut scheduler = JobScheduler::new();
        for (cron_expr, exec) in jobs {
            scheduler.add(Job::new(cron_expr, exec));
        }
        thread::spawn(move || loop {
            scheduler.tick();
            thread::sleep(scheduler.time_till_next_job());
        });
        self
    }

    /// Runs the application.
    fn run(self, async_jobs: HashMap<&'static str, AsyncCronJob>) -> io::Result<()>;
}
