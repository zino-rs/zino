use std::time::Duration;

pub fn bench(c: &mut criterion::Criterion) {
    c.bench_function("format_duration_secs", |b| {
        b.iter(|| {
            let duration = Duration::from_secs_f64(0.0024635);
            let millis = duration.as_secs_f64() * 1000.0;
            let duration_millis = format!("{:.3}", millis);
            format!("dur={}", duration_millis.trim_end_matches(['.', '0']))
        })
    });
    c.bench_function("format_duration_micros", |b| {
        b.iter(|| {
            let duration = Duration::from_secs_f64(0.0024635);
            let millis = (duration.as_micros() as f64) / 1000.0;
            format!("dur={}", millis)
        })
    });
    c.bench_function("format_duration_micros_ryu", |b| {
        b.iter(|| {
            let duration = Duration::from_secs_f64(0.0024635);
            let millis = (duration.as_micros() as f64) / 1000.0;
            let mut buffer = ryu::Buffer::new();
            format!("dur={}", buffer.format_finite(millis))
        })
    });
}
