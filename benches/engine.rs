use std::time::Duration;

use anyhow::Result;
use criterion::{
    BenchmarkGroup, Criterion, criterion_group, criterion_main,
    measurement::WallTime,
};
use tempfile::TempDir;

use nano_search::{
    docs::{self, cisi::model::Query},
    engines::{
        nano::engine::NanoSearchEngine, tantivy::engine::TantivySearchEngine,
    },
    model::{doc::DocsSource, engine::SearchEngine},
    utils::panic_on_error,
};

#[derive(Debug)]
enum IndexType {
    Memory,
    Disk,
}

fn index(c: &mut Criterion) {
    panic_on_error(|| {
        let docs = docs::cisi::load_docs()?;

        let mut group = c.benchmark_group("index");

        for index_type in &[IndexType::Memory, IndexType::Disk] {
            index_with::<NanoSearchEngine>(&mut group, &docs, index_type);
            index_with::<TantivySearchEngine>(&mut group, &docs, index_type);
        }

        group.finish();

        Ok(())
    });
}

fn index_with<SE: SearchEngine>(
    group: &mut BenchmarkGroup<WallTime>,
    docs: &impl DocsSource,
    index_type: &IndexType,
) {
    let bench_id = format!("{}/{:?}", SE::name(), index_type).to_lowercase();

    group.bench_function(bench_id, |bencher| {
        bencher.iter(|| -> Result<()> {
            let dir = TempDir::new()?;
            let mut engine = match index_type {
                IndexType::Memory => SE::create_in_memory()?,
                IndexType::Disk => SE::create_on_disk(&dir)?,
            };
            engine.index_docs(&mut docs.clone().into_iter())?;
            Ok(())
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
    let mut engine = SE::create_on_disk(&dir)?;
    engine.index_docs(&mut docs.clone().into_iter())?;

    group.bench_function(SE::name(), |bencher| {
        bencher.iter(|| SE::open_from_disk(&dir));
    });

    Ok(())
}

fn search(c: &mut Criterion) {
    panic_on_error(|| {
        let docs = docs::cisi::load_docs()?;
        let queries = docs::cisi::load_queries()?;

        let mut group = c.benchmark_group("search");

        for index_type in &[IndexType::Memory, IndexType::Disk] {
            search_with::<NanoSearchEngine>(
                &mut group, &docs, &queries, index_type,
            )?;
            search_with::<TantivySearchEngine>(
                &mut group, &docs, &queries, index_type,
            )?;
        }

        group.finish();

        Ok(())
    });
}

fn search_with<SE: SearchEngine>(
    group: &mut BenchmarkGroup<WallTime>,
    docs: &impl DocsSource,
    queries: &[Query],
    index_type: &IndexType,
) -> Result<()> {
    let dir = TempDir::new()?;
    let mut engine = match index_type {
        IndexType::Memory => SE::create_in_memory()?,
        IndexType::Disk => SE::create_on_disk(&dir)?,
    };
    engine.index_docs(&mut docs.clone().into_iter())?;

    let bench_id = format!("{}/{:?}", SE::name(), index_type).to_lowercase();

    group.bench_function(bench_id, |bencher| {
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
