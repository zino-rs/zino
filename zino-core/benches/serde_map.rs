use zino_core::{extension::JsonObjectExt, Map, Uuid};

pub fn bench(c: &mut criterion::Criterion) {
    c.bench_function("new_map", |b| {
        b.iter(|| {
            let mut map = Map::new();
            map.upsert("status_code", 200);
            map.upsert("request_id", Uuid::new_v4().to_string());
            map.upsert("data", b"OK".to_vec());
            map
        })
    });
    c.bench_function("serde_map_from_value", |b| {
        b.iter(|| {
            let mut map = Map::new();
            map.upsert("status_code", 200);
            map.upsert("request_id", Uuid::new_v4().to_string());
            map.upsert("data", b"OK".to_vec());
            serde_json::from_value::<Map>(map.into())
        })
    });
}
