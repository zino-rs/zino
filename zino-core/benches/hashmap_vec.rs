use std::collections::{BTreeMap, HashMap};

pub fn bench(c: &mut criterion::Criterion) {
    c.bench_function("btreemap_lookup", |b| {
        b.iter(|| {
            let mut map = BTreeMap::new();
            map.insert("en-US", "Welcome!");
            map.insert("zh-CN", "欢迎！");
            map.insert("zh-HK", "歡迎！");
            map.get("zh-CN").is_some()
        })
    });
    c.bench_function("hashmap_lookup", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            map.insert("en-US", "Welcome!");
            map.insert("zh-CN", "欢迎！");
            map.insert("zh-HK", "歡迎！");
            map.get("zh-CN").is_some()
        })
    });
    c.bench_function("vec_lookup", |b| {
        b.iter(|| {
            let vec = vec![
                ("en-US", "Welcome!"),
                ("zh-CN", "欢迎！"),
                ("zh-HK", "歡迎！"),
            ];
            vec.iter()
                .find_map(|(lang, text)| (lang == &"zh-CN").then_some(text))
                .is_some()
        })
    });
    c.bench_function("arrayvec_lookup", |b| {
        b.iter(|| {
            let mut vec = arrayvec::ArrayVec::<_, 3>::new();
            vec.push(("en-US", "Welcome!"));
            vec.push(("zh-CN", "欢迎！"));
            vec.push(("zh-HK", "歡迎！"));
            vec.iter()
                .find_map(|(lang, text)| (lang == &"zh-CN").then_some(text))
                .is_some()
        })
    });
    c.bench_function("smallvec_lookup", |b| {
        b.iter(|| {
            let mut vec = smallvec::SmallVec::<[_; 3]>::new();
            vec.push(("en-US", "Welcome!"));
            vec.push(("zh-CN", "欢迎！"));
            vec.push(("zh-HK", "歡迎！"));
            vec.iter()
                .find_map(|(lang, text)| (lang == &"zh-CN").then_some(text))
                .is_some()
        })
    });
    c.bench_function("tinyvec_lookup", |b| {
        b.iter(|| {
            let mut vec = tinyvec::TinyVec::<[_; 3]>::new();
            vec.push(("en-US", "Welcome!"));
            vec.push(("zh-CN", "欢迎！"));
            vec.push(("zh-HK", "歡迎！"));
            vec.iter()
                .find_map(|(lang, text)| (lang == &"zh-CN").then_some(text))
                .is_some()
        })
    });
}
