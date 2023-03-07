use zino_core::error::Error;

pub fn bench(c: &mut criterion::Criterion) {
    type BoxError = Box<dyn std::error::Error + 'static>;

    c.bench_function("static_str_into_box_error", |b| {
        b.iter(|| {
            let message = "a string error";
            BoxError::from(message)
        })
    });
    c.bench_function("string_into_box_error", |b| {
        b.iter(|| {
            let message = String::from("a string error");
            BoxError::from(message)
        })
    });
    c.bench_function("parse_int_error_into_box_error", |b| {
        b.iter(|| {
            let err = "12.15".parse::<i32>().unwrap_err();
            BoxError::from(err)
        })
    });
    c.bench_function("static_str_into_anyhow_error", |b| {
        b.iter(|| {
            let message = "a string error";
            anyhow::Error::msg(message)
        })
    });
    c.bench_function("string_into_anyhow_error", |b| {
        b.iter(|| {
            let message = String::from("a string error");
            anyhow::Error::msg(message)
        })
    });
    c.bench_function("parse_int_error_into_anyhow_error", |b| {
        b.iter(|| {
            let err = "12.15".parse::<i32>().unwrap_err();
            anyhow::Error::new(err)
        })
    });
    c.bench_function("static_str_into_zino_error", |b| {
        b.iter(|| {
            let message = "a string error";
            Error::new(message)
        })
    });
    c.bench_function("string_into_zino_error", |b| {
        b.iter(|| {
            let message = String::from("a string error");
            Error::new(message)
        })
    });
    c.bench_function("parse_int_error_into_zino_error", |b| {
        b.iter(|| {
            let err = "12.15".parse::<i32>().unwrap_err();
            Error::from(err)
        })
    });
}
