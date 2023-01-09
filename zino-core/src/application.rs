use crate::{AsyncCronJob, CronJob, Job, JobScheduler, Map, State};
use std::{collections::HashMap, thread};
use toml::value::Table;

/// Application.
pub trait Application {
    /// Router.
    type Router;

    /// Creates a new application.
    fn new() -> Self;

    /// Returns a reference to the shared application state.
    fn shared() -> &'static State;

    /// Registers routes.
    fn register(self, routes: HashMap<&'static str, Self::Router>) -> Self;

    /// Runs the application.
    fn run(self, async_jobs: HashMap<&'static str, AsyncCronJob>);

    /// Returns the application env.
    #[inline]
    fn env() -> &'static str {
        Self::shared().env()
    }

    /// Returns a reference to the shared application config.
    #[inline]
    fn config() -> &'static Table {
        Self::shared().config()
    }

    /// Returns a reference to the shared application state data.
    #[inline]
    fn state_data() -> &'static Map {
        Self::shared().data()
    }

    /// Spawns a new thread to run cron jobs.
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
}
