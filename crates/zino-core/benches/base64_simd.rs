use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};

pub fn bench(c: &mut criterion::Criterion) {
    c.bench_function("base64_encode", |b| {
        b.iter(|| {
            let bytes = b"hello world";
            STANDARD_NO_PAD.encode(bytes)
        })
    });
    c.bench_function("base64_encode_simd", |b| {
        b.iter(|| {
            let bytes = b"hello world";
            base64_simd::STANDARD_NO_PAD.encode_to_string(bytes)
        })
    });
    c.bench_function("data_encoding_base64_encode", |b| {
        b.iter(|| {
            let bytes = b"hello world";
            data_encoding::BASE64_NOPAD.encode(bytes)
        })
    });
    c.bench_function("base64_decode", |b| {
        b.iter(|| {
            let encoded = "Er/DkSLyeOsUiHXHK4hO7E8fdl1g8Qwy2Ef8mR1/4BQ";
            STANDARD_NO_PAD.decode(encoded)
        })
    });
    c.bench_function("base64_decode_simd", |b| {
        b.iter(|| {
            let encoded = "Er/DkSLyeOsUiHXHK4hO7E8fdl1g8Qwy2Ef8mR1/4BQ";
            base64_simd::STANDARD_NO_PAD.decode_to_vec(encoded)
        })
    });
    c.bench_function("base64_forgiving_decode_simd", |b| {
        b.iter(|| {
            let encoded = "Er/DkSLyeOsUiHXHK4hO7E8fdl1g8Qwy2Ef8mR1/4BQ";
            base64_simd::forgiving_decode_to_vec(encoded.as_bytes())
        })
    });
    c.bench_function("data_encoding_base64_decode", |b| {
        b.iter(|| {
            let encoded = "Er/DkSLyeOsUiHXHK4hO7E8fdl1g8Qwy2Ef8mR1/4BQ";
            data_encoding::BASE64_NOPAD.decode(encoded.as_bytes())
        })
    });
}
