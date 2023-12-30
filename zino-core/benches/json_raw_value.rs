use zino_core::{extension::JsonObjectExt, Map, Uuid};

pub fn bench(c: &mut criterion::Criterion) {
    c.bench_function("serialize_json_value", |b| {
        b.iter(|| {
            let mut res = Map::new();
            res.upsert("status_code", 200);
            res.upsert("request_id", Uuid::new_v4().to_string());

            let mut data = Map::new();
            data.upsert("name", "alice");
            data.upsert("age", 18);
            data.upsert("roles", vec!["admin", "worker"]);
            res.upsert("data", serde_json::to_value(&data).unwrap());
            serde_json::to_vec(&res)
        })
    });
    c.bench_function("serialize_json_raw_value", |b| {
        b.iter(|| {
            let mut res = Map::new();
            res.upsert("status_code", 200);
            res.upsert("request_id", Uuid::new_v4().to_string());

            let mut data = Map::new();
            data.upsert("name", "alice");
            data.upsert("age", 18);
            data.upsert("roles", vec!["admin", "worker"]);
            res.upsert(
                "data",
                serde_json::value::to_raw_value(&data).unwrap().get(),
            );
            serde_json::to_vec(&res)
        })
    });
    c.bench_function("serialize_json_object", |b| {
        b.iter(|| {
            let mut res = Map::new();
            res.upsert("status_code", 200);
            res.upsert("request_id", Uuid::new_v4().to_string());

            let mut data = Map::new();
            data.upsert("name", "alice");
            data.upsert("age", 18);
            data.upsert("roles", vec!["admin", "worker"]);
            res.upsert("data", data);
            serde_json::to_vec(&res)
        })
    });
}
