//! Scheduler for sync and async cron jobs.

use super::AsyncScheduler;
use crate::{datetime::DateTime, BoxFuture, Map, Uuid};
use chrono::Local;
use cron::Schedule;
use std::{str::FromStr, time::Duration};

/// A function pointer of the async cron job.
pub type AsyncCronJob =
    for<'a> fn(id: Uuid, data: &'a mut Map, last_tick: DateTime) -> BoxFuture<'a>;

/// An async schedulable job.
pub struct AsyncJob {
    /// Job ID.
    id: Uuid,
    /// Job data.
    data: Map,
    /// Flag to indicate whether the job is disabled.
    disabled: bool,
    /// Flag to indicate whether the job is executed immediately.
    immediate: bool,
    /// Cron expression parser.
    schedule: Schedule,
    /// Cron job to run.
    run: AsyncCronJob,
    /// Last time when running the job.
    last_tick: Option<chrono::DateTime<Local>>,
}

impl AsyncJob {
    /// Creates a new instance.
    #[inline]
    pub fn new(cron_expr: &str, exec: AsyncCronJob) -> Self {
        let schedule = Schedule::from_str(cron_expr)
            .unwrap_or_else(|err| panic!("invalid cron expression `{cron_expr}`: {err}"));
        Self {
            id: Uuid::now_v7(),
            data: Map::new(),
            disabled: false,
            immediate: false,
            schedule,
            run: exec,
            last_tick: None,
        }
    }

    /// Enables the flag to indicate whether the job is disabled.
    #[inline]
    pub fn disable(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Enables the flag to indicate whether the job is executed immediately.
    #[inline]
    pub fn immediate(mut self, immediate: bool) -> Self {
        self.immediate = immediate;
        self
    }

    /// Returns the job ID.
    #[inline]
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Returns a reference to the job data.
    #[inline]
    pub fn data(&self) -> &Map {
        &self.data
    }

    /// Returns a mutable reference to the job data.
    #[inline]
    pub fn data_mut(&mut self) -> &mut Map {
        &mut self.data
    }

    /// Returns `true` if the job is disabled.
    #[inline]
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Returns `true` if the job is is executed immediately.
    #[inline]
    pub fn is_immediate(&self) -> bool {
        self.immediate
    }

    /// Pauses the job by setting the `disabled` flag to `true`.
    #[inline]
    pub fn pause(&mut self) {
        self.disabled = true;
    }

    /// Resumes the job by setting the `disabled` flag to `false`.
    #[inline]
    pub fn resume(&mut self) {
        self.disabled = false;
    }

    /// Sets the last tick when the job was executed.
    #[inline]
    pub fn set_last_tick(&mut self, last_tick: Option<DateTime>) {
        self.last_tick = last_tick.map(|dt| dt.into());
    }

    /// Executes the missed runs asynchronously.
    pub async fn tick(&mut self) {
        let now = Local::now();
        let disabled = self.disabled;
        let run = self.run;
        if let Some(last_tick) = self.last_tick {
            for event in self.schedule.after(&last_tick) {
                if event > now {
                    break;
                }
                if !disabled {
                    run(self.id, &mut self.data, last_tick.into()).await;
                }
            }
        } else if !disabled && self.immediate {
            run(self.id, &mut self.data, now.into()).await;
        }
        self.last_tick = Some(now);
    }

    /// Executes the job manually.
    pub async fn execute(&mut self) {
        let now = Local::now();
        let run = self.run;
        run(self.id, &mut self.data, now.into()).await;
        self.last_tick = Some(now);
    }
}

/// A type contains and executes the async scheduled jobs.
#[derive(Default)]
pub struct AsyncJobScheduler {
    /// A list of async jobs.
    jobs: Vec<AsyncJob>,
}

impl AsyncJobScheduler {
    /// Creates a new instance.
    #[inline]
    pub fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    /// Adds an async job to the scheduler and returns the job ID.
    pub fn add(&mut self, job: AsyncJob) -> Uuid {
        let job_id = job.id;
        self.jobs.push(job);
        job_id
    }

    /// Removes an async job by ID from the scheduler.
    pub fn remove(&mut self, job_id: Uuid) -> bool {
        let position = self.jobs.iter().position(|job| job.id == job_id);
        if let Some(index) = position {
            self.jobs.remove(index);
            true
        } else {
            false
        }
    }

    /// Returns a reference to the job with the ID.
    #[inline]
    pub fn get(&self, job_id: Uuid) -> Option<&AsyncJob> {
        self.jobs.iter().find(|job| job.id == job_id)
    }

    /// Returns a mutable reference to the job with the ID.
    #[inline]
    pub fn get_mut(&mut self, job_id: Uuid) -> Option<&mut AsyncJob> {
        self.jobs.iter_mut().find(|job| job.id == job_id)
    }

    /// Returns the duration till the next job is supposed to run.
    pub fn time_till_next_job(&self) -> Duration {
        if self.jobs.is_empty() {
            Duration::from_millis(500)
        } else {
            let mut duration = chrono::Duration::zero();
            let now = Local::now();
            for job in self.jobs.iter() {
                for event in job.schedule.after(&now).take(1) {
                    let interval = event - now;
                    if duration.is_zero() || interval < duration {
                        duration = interval;
                    }
                }
            }
            duration
                .to_std()
                .unwrap_or_else(|_| Duration::from_millis(500))
        }
    }

    /// Increments time for the scheduler and executes any pending jobs asynchronously.
    /// It is recommended to sleep for at least 500 milliseconds between invocations of this method.
    #[inline]
    pub async fn tick(&mut self) {
        for job in &mut self.jobs {
            job.tick().await;
        }
    }

    /// Executes all the job manually.
    pub async fn execute(&mut self) {
        for job in &mut self.jobs {
            job.execute().await;
        }
    }
}

impl AsyncScheduler for AsyncJobScheduler {
    #[inline]
    fn time_till_next_job(&self) -> Duration {
        self.time_till_next_job()
    }

    #[inline]
    async fn tick(&mut self) {
        self.tick().await;
    }
}
