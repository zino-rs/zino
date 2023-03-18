use std::collections::HashMap;

pub fn bench(c: &mut criterion::Criterion) {
    c.bench_function("hashmap_lookup_", |b| {
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
            let mut vec = Vec::new();
            vec.push(("en-US", "Welcome!"));
            vec.push(("zh-CN", "欢迎！"));
            vec.push(("zh-HK", "歡迎！"));
            vec.iter()
                .find_map(|(lang, text)| (lang == &"zh-CN").then_some(text))
                .is_some()
        })
    });
}
