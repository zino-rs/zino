use zino::prelude::*;

mod job;

pub fn jobs() -> Vec<(&'static str, CronJob)> {
    vec![
        ("0/15 * * * * *", job::every_15s as CronJob),
        ("0/20 * * * * *", job::every_20s as CronJob),
    ]
}

pub fn async_jobs() -> Vec<(&'static str, AsyncCronJob)> {
    vec![("0 0 * * * *", job::every_hour as AsyncCronJob)]
}
