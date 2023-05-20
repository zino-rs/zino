use zino::prelude::*;

mod job;

pub(crate) fn jobs() -> Vec<(&'static str, CronJob)> {
    vec![
        ("0/15 * * * * *", job::every_15s as CronJob),
        ("0/20 * * * * *", job::every_20s as CronJob),
    ]
}

pub(crate) fn async_jobs() -> Vec<(&'static str, AsyncCronJob)> {
    vec![("0/30 * * * * *", job::every_30s as AsyncCronJob)]
}
