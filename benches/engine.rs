use anyhow::Result;
use criterion::{
    BenchmarkGroup, Criterion, criterion_group, criterion_main,
    measurement::WallTime,
};
use nano_search::{
    docs::{self, cisi::model::Query},
    engines::{
        nano::engine::NanoSearchEngine, tantivy::engine::TantivySearchEngine,
    },
    model::{doc::DocsSource, engine::SearchEngine},
    utils::panic_on_error,
};
use std::time::Duration;
use tempfile::TempDir;

fn index(c: &mut Criterion) {
    panic_on_error(|| {
        let docs = docs::cisi::load_docs()?;

        let mut group = c.benchmark_group("index");

        index_with::<NanoSearchEngine>(&mut group, &docs);
        index_with::<TantivySearchEngine>(&mut group, &docs);

        group.finish();

        Ok(())
    });
}

fn index_with<SE: SearchEngine>(
    group: &mut BenchmarkGroup<WallTime>,
    docs: &impl DocsSource,
) {
    group.bench_function(SE::name(), |bencher| {
        bencher.iter(|| {
            let dir = TempDir::new()?;
            let mut engine = SE::create_index(&dir)?;
            engine.index_docs(&mut docs.clone().into_iter())
        });
    });
}

fn open_index(c: &mut Criterion) {
    panic_on_error(|| {
        let docs = docs::cisi::load_docs()?;

        let mut group = c.benchmark_group("open_index");

        open_index_with::<NanoSearchEngine>(&mut group, &docs)?;
        open_index_with::<TantivySearchEngine>(&mut group, &docs)?;

        group.finish();

        Ok(())
    });
}

fn open_index_with<SE: SearchEngine>(
    group: &mut BenchmarkGroup<WallTime>,
    docs: &impl DocsSource,
) -> Result<()> {
    let dir = TempDir::new()?;
    let mut engine = SE::create_index(&dir)?;
    engine.index_docs(&mut docs.clone().into_iter())?;

    group.bench_function(SE::name(), |bencher| {
        bencher.iter(|| SE::open_index(&dir));
    });

    Ok(())
}

fn search(c: &mut Criterion) {
    panic_on_error(|| {
        let docs = docs::cisi::load_docs()?;
        let queries = docs::cisi::load_queries()?;

        let mut group = c.benchmark_group("search");

        search_with::<NanoSearchEngine>(&mut group, &docs, &queries)?;
        search_with::<TantivySearchEngine>(&mut group, &docs, &queries)?;

        group.finish();

        Ok(())
    });
}

fn search_with<SE: SearchEngine>(
    group: &mut BenchmarkGroup<WallTime>,
    docs: &impl DocsSource,
    queries: &[Query],
) -> Result<()> {
    let dir = TempDir::new()?;
    let mut engine = SE::create_index(&dir)?;
    engine.index_docs(&mut docs.clone().into_iter())?;

    group.bench_function(SE::name(), |bencher| {
        bencher.iter(|| {
            queries
                .iter()
                .map(|query| engine.search(&query.text, 10))
                .collect::<Vec<_>>()
        });
    });

    Ok(())
}

criterion_group! {
    name = benches;
    // increase measurement time / iterations count to avoid false positives,
    // which happens a lot, most likely due to variability of file operations
    config = Criterion::default()
        .measurement_time(Duration::from_secs(60))
        .noise_threshold(0.05);
    targets = index, open_index, search
}
criterion_main!(benches);
