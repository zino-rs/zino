use std::collections::HashMap;
use zino::{AsyncCronJob, CronJob};

mod job;

pub(crate) fn init_jobs() -> HashMap<&'static str, CronJob> {
    let mut jobs = HashMap::new();
    jobs.insert("0/15 * * * * *", job::every_15s as CronJob);
    jobs.insert("0/20 * * * * *", job::every_20s as CronJob);
    jobs
}

pub(crate) fn init_async_jobs() -> HashMap<&'static str, AsyncCronJob> {
    let mut async_jobs = HashMap::new();
    async_jobs.insert("0/30 * * * * *", job::every_30s as AsyncCronJob);
    async_jobs
}
