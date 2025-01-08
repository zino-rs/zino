use zino::prelude::*;
use zino_model::User;

pub fn every_15s(ctx: &mut JobContext) {
    if let Some(job_data) = ctx.get_data_mut::<Map>() {
        let counter = job_data
            .get("counter")
            .map(|c| c.as_u64().unwrap_or_default() + 1)
            .unwrap_or_default();
        job_data.upsert("counter", counter);
        job_data.upsert("current", DateTime::now());
    }
}

pub fn every_20s(ctx: &mut JobContext) {
    if let Some(job_data) = ctx.get_data_mut::<Map>() {
        let counter = job_data
            .get("counter")
            .map(|c| c.as_u64().unwrap_or_default() + 1)
            .unwrap_or_default();
        job_data.upsert("counter", counter);
        job_data.upsert("current", DateTime::now());
    }
}

pub fn every_hour(ctx: &mut JobContext) -> BoxFuture {
    if let Some(job_data) = ctx.get_data_mut::<Map>() {
        let counter = job_data
            .get("counter")
            .map(|c| c.as_u64().unwrap_or_default() + 1)
            .unwrap_or_default();
        job_data.upsert("counter", counter);
        job_data.upsert("current", DateTime::now());
    }
    Box::pin(async {
        let query = Query::default();
        let columns = [("*", true), ("roles", true)];
        if let Ok(mut map) = User::count_many(&query, &columns).await {
            if let Some(job_data) = ctx.get_data_mut::<Map>() {
                job_data.append(&mut map);
            }
        }
    })
}
