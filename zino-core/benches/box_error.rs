use zino_core::{error::Error, BoxError};

pub fn bench(c: &mut criterion::Criterion) {
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
}
