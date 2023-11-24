use crate::model::User;
use zino::prelude::*;

pub fn every_15s(job_id: Uuid, job_data: &mut Map, last_tick: DateTime) {
    let counter = job_data
        .get("counter")
        .map(|c| c.as_u64().unwrap_or_default() + 1)
        .unwrap_or_default();
    job_data.upsert("counter", counter);
    job_data.upsert("current", DateTime::now());
    job_data.upsert("last_tick", last_tick);
    job_data.upsert("job_id", job_id.to_string());
}

pub fn every_20s(job_id: Uuid, job_data: &mut Map, last_tick: DateTime) {
    let counter = job_data
        .get("counter")
        .map(|c| c.as_u64().unwrap_or_default() + 1)
        .unwrap_or_default();
    job_data.upsert("counter", counter);
    job_data.upsert("current", DateTime::now());
    job_data.upsert("last_tick", last_tick);
    job_data.upsert("job_id", job_id.to_string());
}

pub fn every_hour(job_id: Uuid, job_data: &mut Map, last_tick: DateTime) -> BoxFuture {
    let counter = job_data
        .get("counter")
        .map(|c| c.as_u64().unwrap_or_default() + 1)
        .unwrap_or_default();
    job_data.upsert("counter", counter);
    job_data.upsert("current", DateTime::now());
    job_data.upsert("last_tick", last_tick);
    job_data.upsert("job_id", job_id.to_string());
    Box::pin(async {
        let query = Query::default();
        let columns = [("*", true), ("roles", true)];
        if let Ok(mut map) = User::count_many(&query, &columns).await {
            job_data.append(&mut map);
        }
    })
}
