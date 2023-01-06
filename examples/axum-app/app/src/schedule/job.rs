use zino_core::{BoxFuture, DateTime, Map, Query, Schema, Uuid};
use zino_model::User;

pub(super) fn every_15s(job_id: Uuid, job_data: &mut Map) {
    let counter = job_data
        .get("counter")
        .map(|c| c.as_u64().unwrap_or_default() + 1)
        .unwrap_or_default();
    job_data.insert("current".to_string(), DateTime::now().to_string().into());
    job_data.insert("counter".to_string(), counter.into());
    println!("Job {job_id} is executed every 15 seconds: {job_data:?}");
}

pub(super) fn every_20s(job_id: Uuid, job_data: &mut Map) {
    let counter = job_data
        .get("counter")
        .map(|c| c.as_u64().unwrap_or_default() + 1)
        .unwrap_or_default();
    job_data.insert("current".to_string(), DateTime::now().to_string().into());
    job_data.insert("counter".to_string(), counter.into());
    println!("Job {job_id} is executed every 20 seconds: {job_data:?}");
}

pub(super) fn every_30s(job_id: Uuid, job_data: &mut Map) -> BoxFuture {
    let counter = job_data
        .get("counter")
        .map(|c| c.as_u64().unwrap_or_default() + 1)
        .unwrap_or_default();
    job_data.insert("current".to_string(), DateTime::now().to_string().into());
    job_data.insert("counter".to_string(), counter.into());
    println!("Job {job_id} is executed every 45 seconds: {job_data:?}");

    Box::pin(async {
        let query = Query::new();
        let users = User::find(query).await.unwrap();
        job_data.insert("users".to_string(), users.len().into());
    })
}
