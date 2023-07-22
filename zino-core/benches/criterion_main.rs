mod base64_simd;
mod box_error;
mod format_duration;
mod hashmap_vec;
mod serde_map;
mod uuid_simd;

criterion::criterion_group!(
    benches,
    base64_simd::bench,
    box_error::bench,
    format_duration::bench,
    hashmap_vec::bench,
    serde_map::bench,
    uuid_simd::bench,
);
criterion::criterion_main!(benches);
