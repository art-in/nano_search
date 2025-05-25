use criterion::{Criterion, criterion_group, criterion_main};
use nano_search::{
    docs, engines::nano::engine::NanoSearchEngine, model::engine::SearchEngine,
    utils::panic_on_error,
};
use std::time::Duration;
use tempfile::TempDir;

// TODO: add benchmarks for TantivySearchEngine

fn create_index(c: &mut Criterion) {
    panic_on_error(|| {
        let docs = docs::cisi::load_docs()?;

        c.bench_function("create_index", |bencher| {
            bencher.iter(|| {
                let dir = TempDir::new()?;
                let mut engine = NanoSearchEngine::create_index(&dir)?;

                // TODO: avoid docs.clone()
                let mut docs_iterator = docs.clone().into_iter();

                engine.index_docs(&mut docs_iterator)
            });
        });

        Ok(())
    });
}

fn open_index(c: &mut Criterion) {
    panic_on_error(|| {
        let docs = docs::cisi::load_docs()?;
        let dir = TempDir::new()?;
        let mut engine = NanoSearchEngine::create_index(&dir)?;
        engine.index_docs(&mut docs.clone().into_iter())?;

        c.bench_function("open_index", |bencher| {
            bencher.iter(|| NanoSearchEngine::open_index(&dir));
        });

        Ok(())
    });
}

fn search(c: &mut Criterion) {
    panic_on_error(|| {
        let docs = docs::cisi::load_docs()?;
        let dir = TempDir::new()?;
        let mut engine = NanoSearchEngine::create_index(&dir)?;
        engine.index_docs(&mut docs.clone().into_iter())?;
        let queries = docs::cisi::load_queries()?;

        c.bench_function("search", |bencher| {
            bencher.iter(|| {
                queries.iter().map(|query| engine.search(&query.text, 10))
            });
        });

        Ok(())
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
