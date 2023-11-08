use zino::prelude::*;

mod job;

pub fn jobs() -> StaticRecord<CronJob> {
    let mut record = StaticRecord::new();
    record.add("0/15 * * * * *", job::every_15s as CronJob);
    record.add("0/20 * * * * *", job::every_20s as CronJob);
    record
}

pub fn async_jobs() -> StaticRecord<AsyncCronJob> {
    let mut record = StaticRecord::new();
    record.add("0 0 * * * *", job::every_hour as AsyncCronJob);
    record
}
