mod box_error;
mod hashmap_vec;
mod serde_map;

criterion::criterion_group!(
    benches,
    box_error::bench,
    hashmap_vec::bench,
    serde_map::bench
);
criterion::criterion_main!(benches);
