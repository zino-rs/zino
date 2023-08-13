use zino::prelude::*;
use zino_model::User;

pub fn every_15s(job_id: Uuid, job_data: &mut Map, _last_tick: DateTime) {
    let counter = job_data
        .get("counter")
        .map(|c| c.as_u64().unwrap_or_default() + 1)
        .unwrap_or_default();
    job_data.upsert("current", DateTime::now().to_string());
    job_data.upsert("counter", counter);
    tracing::debug!(
        job_data = format!("{job_data:?}"),
        "job {job_id} is executed every 15 seconds"
    );
}

pub fn every_20s(job_id: Uuid, job_data: &mut Map, _last_tick: DateTime) {
    let counter = job_data
        .get("counter")
        .map(|c| c.as_u64().unwrap_or_default() + 1)
        .unwrap_or_default();
    job_data.upsert("current", DateTime::now().to_string());
    job_data.upsert("counter", counter);
    tracing::debug!(
        job_data = format!("{job_data:?}"),
        "job {job_id} is executed every 20 seconds"
    );
}

pub fn every_hour(job_id: Uuid, job_data: &mut Map, _last_tick: DateTime) -> BoxFuture {
    let counter = job_data
        .get("counter")
        .map(|c| c.as_u64().unwrap_or_default() + 1)
        .unwrap_or_default();
    job_data.upsert("current", DateTime::now().to_string());
    job_data.upsert("counter", counter);
    tracing::debug!(
        job_data = format!("{job_data:?}"),
        "async job {job_id} is executed every hour"
    );

    Box::pin(async {
        let query = Query::default();
        let columns = [("*", true), ("roles", true)];
        match User::count_many(&query, &columns).await {
            Ok(mut map) => job_data.append(&mut map),
            Err(err) => tracing::error!("fail to count users: {err}"),
        }
    })
}
