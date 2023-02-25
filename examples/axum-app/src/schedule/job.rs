use zino::{BoxFuture, DateTime, Map, Query, Schema, Uuid};
use zino_model::User;

pub(super) fn every_15s(job_id: Uuid, job_data: &mut Map, _last_tick: DateTime) {
    let counter = job_data
        .get("counter")
        .map(|c| c.as_u64().unwrap_or_default() + 1)
        .unwrap_or_default();
    job_data.insert("current".to_owned(), DateTime::now().to_string().into());
    job_data.insert("counter".to_owned(), counter.into());
    tracing::debug!(
        job_data = format!("{job_data:?}"),
        "job {job_id} is executed every 15 seconds"
    );
}

pub(super) fn every_20s(job_id: Uuid, job_data: &mut Map, _last_tick: DateTime) {
    let counter = job_data
        .get("counter")
        .map(|c| c.as_u64().unwrap_or_default() + 1)
        .unwrap_or_default();
    job_data.insert("current".to_owned(), DateTime::now().to_string().into());
    job_data.insert("counter".to_owned(), counter.into());
    tracing::debug!(
        job_data = format!("{job_data:?}"),
        "job {job_id} is executed every 20 seconds"
    );
}

pub(super) fn every_30s(job_id: Uuid, job_data: &mut Map, _last_tick: DateTime) -> BoxFuture {
    let counter = job_data
        .get("counter")
        .map(|c| c.as_u64().unwrap_or_default() + 1)
        .unwrap_or_default();
    job_data.insert("current".to_owned(), DateTime::now().to_string().into());
    job_data.insert("counter".to_owned(), counter.into());
    tracing::debug!(
        job_data = format!("{job_data:?}"),
        "async job {job_id} is executed every 30 seconds"
    );

    Box::pin(async {
        let query = Query::new();
        let columns = [("*", true), ("roles", true)];
        match User::count_as(&query, &columns).await {
            Ok(mut map) => job_data.append(&mut map),
            Err(err) => tracing::error!("failed to count users: {err}"),
        }
    })
}
