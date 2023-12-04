pub fn bench(c: &mut criterion::Criterion) {
    c.bench_function("str_format", |b| {
        b.iter(|| {
            let model_name = "user";
            let field = "tags";
            format!("{model_name}.{field}")
        })
    });
    c.bench_function("str_join", |b| {
        b.iter(|| {
            let model_name = "user";
            let field = "tags";
            [model_name, field].join(".")
        })
    });
    c.bench_function("str_concat", |b| {
        b.iter(|| {
            let model_name = "user.";
            let field = "tags";
            [model_name, field].concat()
        })
    });
    c.bench_function("str_add", |b| {
        b.iter(|| {
            let model_name = "user";
            let field = "tags";
            String::from(model_name) + "." + field
        })
    });
}
