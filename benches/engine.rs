use criterion::{criterion_group, criterion_main, Criterion};
use nano_search::{
    docs, engines::nano::engine::NanoSearchEngine, model::engine::SearchEngine,
};
use std::time::Duration;
use tempfile::TempDir;

// TODO: add benchmarks for TantivySearchEngine

fn create_index(c: &mut Criterion) {
    let docs = docs::cisi::load_docs().expect("cisi docs should be loaded");

    c.bench_function("create_index", |bencher| {
        bencher.iter(|| {
            let dir = TempDir::new().expect("temp dir should be created");
            let mut engine = NanoSearchEngine::create_index(&dir)
                .expect("index should be created");

            // TODO: avoid docs.clone()
            let mut docs_iterator = docs.clone().into_iter();

            engine
                .index_docs(&mut docs_iterator)
                .expect("docs should be indexed")
        });
    });
}

fn open_index(c: &mut Criterion) {
    let docs = docs::cisi::load_docs().expect("cisi docs should be loaded");
    let dir = TempDir::new().expect("temp dir should be created");
    let mut engine =
        NanoSearchEngine::create_index(&dir).expect("index should be created");
    engine
        .index_docs(&mut docs.clone().into_iter())
        .expect("docs should be indexed");

    c.bench_function("open_index", |bencher| {
        bencher.iter(|| {
            NanoSearchEngine::open_index(&dir).expect("index should be opened")
        });
    });
}

fn search(c: &mut Criterion) {
    let docs = docs::cisi::load_docs().expect("cisi docs should be loaded");
    let dir = TempDir::new().expect("temp dir should be created");
    let mut engine =
        NanoSearchEngine::create_index(&dir).expect("index should be created");
    engine
        .index_docs(&mut docs.clone().into_iter())
        .expect("docs should be indexed");
    let queries =
        docs::cisi::load_queries().expect("cisi queries should be loaded");

    c.bench_function("search", |bencher| {
        bencher.iter(|| {
            for query in &queries {
                engine.search(&query.text, 10).expect("should search");
            }
        });
    });
}

criterion_group! {
    name = benches;
    // increase measurement time / iterations count to avoid false positives,
    // which happens a lot, most likely due to variability of file operations
    config = Criterion::default().measurement_time(Duration::from_secs(60));
    targets = create_index, open_index, search
}
criterion_main!(benches);
