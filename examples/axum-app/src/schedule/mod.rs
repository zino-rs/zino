use zino::prelude::*;

mod job;

pub fn job_scheduler() -> JobScheduler {
    let mut scheduler = JobScheduler::new();

    let job = Job::new("0/15 * * * * *", job::every_15s as CronJob)
        .data(Map::new())
        .disable(true);
    scheduler.add(job);

    let job = Job::new("0/20 * * * * *", job::every_20s as CronJob)
        .data(Map::new())
        .max_ticks(3);
    scheduler.add(job);

    scheduler
}

pub fn async_job_scheduler() -> AsyncJobScheduler {
    let mut scheduler = AsyncJobScheduler::new();

    let job = AsyncJob::new("0 0 * * * *", job::every_hour as AsyncCronJob)
        .name("count_users")
        .data(Map::new())
        .immediate(true);
    scheduler.add(job);

    scheduler
}
