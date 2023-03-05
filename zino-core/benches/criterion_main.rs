mod box_error;
mod serde_map;

criterion::criterion_group!(benches, box_error::bench, serde_map::bench);
criterion::criterion_main!(benches);
