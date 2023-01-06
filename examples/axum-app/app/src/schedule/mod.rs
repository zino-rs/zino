use std::collections::HashMap;
use zino_core::{AsyncCronJob, CronJob};

mod job;

pub(crate) fn init_jobs() -> HashMap<&'static str, CronJob> {
    let mut jobs = HashMap::new();

    let run_every_15s: CronJob = job::every_15s;
    let run_every_20s: CronJob = job::every_20s;
    jobs.insert("0/15 * * * * *", run_every_15s);
    jobs.insert("0/20 * * * * *", run_every_20s);

    jobs
}

pub(crate) fn init_async_jobs() -> HashMap<&'static str, AsyncCronJob> {
    let mut async_jobs = HashMap::new();

    let run_every_30s: AsyncCronJob = job::every_30s;
    async_jobs.insert("0/30 * * * * *", run_every_30s);

    async_jobs
}
