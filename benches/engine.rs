use criterion::{criterion_group, criterion_main, Criterion};
use nano_search::{
    docs, engines::nano::engine::NanoSearchEngine, model::engine::SearchEngine,
};

// TODO: add benchmarks for TantivySearchEngine

// use index dir inside workspace, instead of /tmp dir
// - performance of indexing is much different:
//   x130 times slower for workspace dir comparing to /tmp dir
// - temporarely choosing workspace dir (i.e. slower path), since it should not
//   be that slow, and I want to fix that (e.g. tantivy is only x1.5 slower)
// - workspace dir is much slower only in dev docker container.
//   reason: workspace dir is mounted to host fs, while file access in mounted
//   volumes is extremely slow in docker containers
//   https://github.com/docker/for-mac/issues/77
static INDEX_DIR: &str = "index_nano";

fn create_index(c: &mut Criterion) {
    let docs = docs::cisi::load_docs().expect("cisi docs should be loaded");

    c.bench_function("create_index", |bencher| {
        bencher.iter(|| {
            let mut engine = NanoSearchEngine::create_index(INDEX_DIR)
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
    let mut engine = NanoSearchEngine::create_index(INDEX_DIR)
        .expect("index should be created");
    engine
        .index_docs(&mut docs.clone().into_iter())
        .expect("docs should be indexed");

    c.bench_function("open_index", |bencher| {
        bencher.iter(|| {
            NanoSearchEngine::open_index(INDEX_DIR)
                .expect("index should be opened")
        });
    });
}

fn search(c: &mut Criterion) {
    let docs = docs::cisi::load_docs().expect("cisi docs should be loaded");
    let mut engine = NanoSearchEngine::create_index(INDEX_DIR)
        .expect("index should be created");
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
    name = slow_benches;
    // reduce number of samples to minium, otherwise default 100 samples takes
    // too long to gather (e.g. ~30 minutes)
    config = Criterion::default().sample_size(10);
    targets = create_index
}
criterion_group! {
    name = fast_benches;
    config = Criterion::default();
    targets = open_index, search
}
criterion_main!(slow_benches, fast_benches);
