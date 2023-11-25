use zino::prelude::*;

mod job;

pub fn job_scheduler() -> JobScheduler {
    let mut scheduler = JobScheduler::new();

    let job = Job::new("0/15 * * * * *", job::every_15s as CronJob).disable(true);
    scheduler.add(job);

    let job = Job::new("0/20 * * * * *", job::every_20s as CronJob).immediate(true);
    scheduler.add(job);

    scheduler
}

pub fn async_job_scheduler() -> AsyncJobScheduler {
    let mut scheduler = AsyncJobScheduler::new();

    let job = AsyncJob::new("0 0 * * * *", job::every_hour as AsyncCronJob).immediate(true);
    scheduler.add(job);

    scheduler
}
