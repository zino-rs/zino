use zino::prelude::*;

mod job;
mod user;

pub fn job_scheduler() -> JobScheduler {
    let mut scheduler = JobScheduler::new();

    let job = Job::new("0/15 * * * * *", job::every_15s)
        .data(Map::new())
        .disable(true);
    scheduler.add(job);

    let job = Job::new("0/20 * * * * *", job::every_20s)
        .data(Map::new())
        .max_ticks(3);
    scheduler.add(job);

    scheduler
}

pub fn async_job_scheduler() -> AsyncJobScheduler {
    let mut scheduler = AsyncJobScheduler::new();

    let initial_account = AsyncJob::new("0 0 * * * *", user::create_initial_account)
        .name("create_initial_account")
        .immediate(true)
        .once();
    scheduler.add(initial_account);

    let count_users = AsyncJob::new("0 1 * * * *", job::every_hour)
        .name("count_users")
        .data(Map::new());
    scheduler.add(count_users);

    scheduler
}
