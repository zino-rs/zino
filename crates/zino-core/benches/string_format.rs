pub fn bench(c: &mut criterion::Criterion) {
    c.bench_function("string_format", |b| {
        b.iter(|| {
            let query = String::from("SELECT * from user");
            let filters = String::from("status = 'Active'");
            format!("{query} WHERE {filters}")
        })
    });
    c.bench_function("string_add", |b| {
        b.iter(|| {
            let mut query = String::from("SELECT * from user");
            query += " WHERE ";
            query += &String::from("status = 'Active'");
            query
        })
    });
    c.bench_function("string_push", |b| {
        b.iter(|| {
            let mut query = String::from("SELECT * from user");
            query.push_str(" WHERE ");
            query.push_str(&String::from("status = 'Active'"));
            query
        })
    });
}
