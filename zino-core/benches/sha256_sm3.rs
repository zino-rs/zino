pub fn bench(c: &mut criterion::Criterion) {
    c.bench_function("sha256_digest", |b| {
        b.iter(|| {
            use sha2::{Digest, Sha256};

            let data = b"Hellow, world!";
            let mut hasher = Sha256::new();
            hasher.update(data);
            hasher.finalize()
        })
    });
    c.bench_function("sm3_digest", |b| {
        b.iter(|| {
            use sm3::{Digest, Sm3};

            let data = b"Hellow, world!";
            let mut hasher = Sm3::new();
            hasher.update(data);
            hasher.finalize()
        })
    });
    c.bench_function("libsm_digest", |b| {
        b.iter(|| {
            use libsm::sm3::hash::Sm3Hash;

            let data = b"Hellow, world!";
            Sm3Hash::new(data).get_hash()
        })
    });
}
