use std::time::Duration;

use anyhow::Result;
use criterion::measurement::WallTime;
use criterion::{BenchmarkGroup, Criterion, criterion_group, criterion_main};
use nano_search::dataset_readers::cisi::CisiDatasetReader;
use nano_search::engines::nano::engine::NanoSearchEngine;
use nano_search::engines::tantivy::engine::TantivySearchEngine;
use nano_search::eval::model::{QueriesSource, Query};
use nano_search::model::doc::{Doc, DocsSource};
use nano_search::model::engine::{CreateOnDiskOptions, SearchEngine};
use nano_search::utils::panic_on_error;
use tempfile::TempDir;

#[derive(Debug)]
enum IndexType {
    Memory,
    Disk,
}

fn index(c: &mut Criterion) {
    panic_on_error(|| {
        let dataset = CisiDatasetReader::new("datasets/cisi");

        let mut group = c.benchmark_group("index");

        for index_type in &[IndexType::Memory, IndexType::Disk] {
            index_with::<NanoSearchEngine>(&mut group, &dataset, index_type)?;
            index_with::<TantivySearchEngine>(
                &mut group, &dataset, index_type,
            )?;
        }

        group.finish();

        Ok(())
    });
}

fn index_with<SE: SearchEngine>(
    group: &mut BenchmarkGroup<WallTime>,
    dataset: &impl DocsSource,
    index_type: &IndexType,
) -> Result<()> {
    // serve docs from memory, to not affect benchmark with extra disk IO
    let docs = dataset.docs()?.collect::<Vec<Doc>>();

    let bench_id = format!("{}/{:?}", SE::name(), index_type).to_lowercase();

    group.bench_function(bench_id, |bencher| {
        bencher.iter(|| -> Result<()> {
            let dir = TempDir::new()?;
            let mut engine = match index_type {
                IndexType::Memory => SE::create_in_memory()?,
                IndexType::Disk => SE::create_on_disk(
                    CreateOnDiskOptions::builder()
                        .index_dir(dir.path())
                        .index_threads(1)
                        .build(),
                )?,
            };
            engine.index_docs(&mut docs.iter().cloned())?;
            Ok(())
        });
    });
    Ok(())
}

fn open_index(c: &mut Criterion) {
    panic_on_error(|| {
        let dataset = CisiDatasetReader::new("datasets/cisi");

        let mut group = c.benchmark_group("open_index");

        open_index_with::<NanoSearchEngine>(&mut group, &dataset)?;
        open_index_with::<TantivySearchEngine>(&mut group, &dataset)?;

        group.finish();

        Ok(())
    });
}

fn open_index_with<SE: SearchEngine>(
    group: &mut BenchmarkGroup<WallTime>,
    dataset: &impl DocsSource,
) -> Result<()> {
    let dir = TempDir::new()?;
    let mut engine = SE::create_on_disk(
        CreateOnDiskOptions::builder()
            .index_dir(dir.path())
            .index_threads(1)
            .build(),
    )?;
    engine.index_docs(&mut dataset.docs()?)?;

    group.bench_function(SE::name(), |bencher| {
        bencher.iter(|| SE::open_from_disk(&dir));
    });

    Ok(())
}

fn search(c: &mut Criterion) {
    panic_on_error(|| {
        let dataset = CisiDatasetReader::new("datasets/cisi");

        let mut group = c.benchmark_group("search");

        for index_type in &[IndexType::Memory, IndexType::Disk] {
            search_with::<NanoSearchEngine>(&mut group, &dataset, index_type)?;
            search_with::<TantivySearchEngine>(
                &mut group, &dataset, index_type,
            )?;
        }

        group.finish();

        Ok(())
    });
}

fn search_with<SE: SearchEngine>(
    group: &mut BenchmarkGroup<WallTime>,
    dataset: &(impl DocsSource + QueriesSource),
    index_type: &IndexType,
) -> Result<()> {
    let dir = TempDir::new()?;
    let mut engine = match index_type {
        IndexType::Memory => SE::create_in_memory()?,
        IndexType::Disk => SE::create_on_disk(
            CreateOnDiskOptions::builder()
                .index_dir(dir.path())
                .index_threads(1)
                .build(),
        )?,
    };
    engine.index_docs(&mut dataset.docs()?)?;

    // serve queries from memory, to not affect benchmark with extra disk IO
    let queries = dataset.queries()?.collect::<Vec<Query>>();

    let bench_id = format!("{}/{:?}", SE::name(), index_type).to_lowercase();

    group.bench_function(bench_id, |bencher| {
        bencher.iter(|| {
            queries.iter().map(|query| engine.search(&query.text, 10))
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
