use crate::{BoxFuture, DateTime, Map, Uuid};
use chrono::Local;
use cron::Schedule;
use std::{str::FromStr, time::Duration};

/// Cron job.
pub type CronJob = fn(Uuid, &mut Map);

/// Async cron job.
pub type AsyncCronJob = for<'a> fn(Uuid, &'a mut Map) -> BoxFuture<'a>;

/// Exectuable job.
enum ExecutableJob {
    Fn(CronJob),
    AsyncFn(AsyncCronJob),
}

/// A schedulable `Job`.
pub struct Job {
    id: Uuid,
    data: Map,
    schedule: Schedule,
    run: ExecutableJob,
    last_tick: Option<chrono::DateTime<Local>>,
}

impl Job {
    /// Creates a new `Job`.
    #[inline]
    pub fn new(cron_expr: &str, exec: CronJob) -> Self {
        let schedule = Schedule::from_str(cron_expr).unwrap();
        Job {
            id: Uuid::new_v4(),
            data: Map::new(),
            schedule,
            run: ExecutableJob::Fn(exec),
            last_tick: None,
        }
    }

    /// Creates a new async `Job`.
    #[inline]
    pub fn new_async(cron_expr: &str, exec: AsyncCronJob) -> Self {
        let schedule = Schedule::from_str(cron_expr).unwrap();
        Job {
            id: Uuid::new_v4(),
            data: Map::new(),
            schedule,
            run: ExecutableJob::AsyncFn(exec),
            last_tick: None,
        }
    }

    /// Returns the job ID.
    #[inline]
    pub fn job_id(&self) -> Uuid {
        self.id
    }

    /// Returns a reference to the job data.
    #[inline]
    pub fn job_data(&self) -> &Map {
        &self.data
    }

    /// Sets last tick.
    #[inline]
    pub fn set_last_tick(&mut self, last_tick: impl Into<Option<DateTime>>) {
        self.last_tick = last_tick.into().map(|dt| dt.into());
    }

    /// Executes missed runs.
    pub fn tick(&mut self) {
        let now = Local::now();
        if let Some(ref last_tick) = self.last_tick {
            for event in self.schedule.after(last_tick) {
                if event > now {
                    break;
                }
                match self.run {
                    ExecutableJob::Fn(exec) => exec(self.id, &mut self.data),
                    ExecutableJob::AsyncFn(_exec) => tracing::warn!("job {} is async", self.id),
                }
            }
        }
        self.last_tick = Some(now);
    }

    /// Executes missed runs asynchronously.
    pub async fn tick_async(&mut self) {
        let now = Local::now();
        if let Some(ref last_tick) = self.last_tick {
            for event in self.schedule.after(last_tick) {
                if event > now {
                    break;
                }
                match self.run {
                    ExecutableJob::Fn(_exec) => tracing::warn!("job {} is not async", self.id),
                    ExecutableJob::AsyncFn(exec) => exec(self.id, &mut self.data).await,
                }
            }
        }
        self.last_tick = Some(now);
    }
}

/// A type contains and executes the scheduled jobs.
#[derive(Default)]
pub struct JobScheduler {
    jobs: Vec<Job>,
}

impl JobScheduler {
    /// Creates a new `JobScheduler`.
    #[inline]
    pub fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    /// Adds a job to the `JobScheduler` and returns the job ID.
    pub fn add(&mut self, job: Job) -> Uuid {
        let job_id = job.id;
        self.jobs.push(job);
        job_id
    }

    /// Removes a job by ID from the `JobScheduler`.
    pub fn remove(&mut self, job_id: Uuid) -> bool {
        let position = self.jobs.iter().position(|job| job.id == job_id);
        match position {
            Some(index) => {
                self.jobs.remove(index);
                true
            }
            None => false,
        }
    }

    /// The `tick` method increments time for the `JobScheduler` and executes
    /// any pending jobs. It is recommended to sleep for at least 500
    /// milliseconds between invocations of this method.
    pub fn tick(&mut self) {
        for job in &mut self.jobs {
            job.tick();
        }
    }

    /// The `tick_async` method increments time for the `JobScheduler` and executes
    /// any pending jobs asynchronously. It is recommended to sleep for at least 500
    /// milliseconds between invocations of this method.
    pub async fn tick_async(&mut self) {
        for job in &mut self.jobs {
            job.tick_async().await;
        }
    }

    /// The `time_till_next_job` method returns the duration till the next job
    /// is supposed to run. This can be used to sleep until then without waking
    /// up at a fixed interval.
    pub fn time_till_next_job(&self) -> Duration {
        if self.jobs.is_empty() {
            Duration::from_millis(500)
        } else {
            let mut duration = chrono::Duration::zero();
            let now = Local::now();
            for job in self.jobs.iter() {
                for event in job.schedule.after(&now).take(1) {
                    let d = event - now;
                    if duration.is_zero() || d < duration {
                        duration = d;
                    }
                }
            }
            duration.to_std().unwrap_or(Duration::from_millis(500))
        }
    }
}
