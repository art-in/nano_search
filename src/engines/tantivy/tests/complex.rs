use anyhow::Result;
use assert_float_eq::assert_float_relative_eq;
use tempfile::TempDir;

use crate::dataset_readers::cisi::CisiDatasetReader;
use crate::engines::tantivy::engine::TantivySearchEngine;
use crate::eval::evaluate_search_quality;
use crate::eval::model::QueriesSource;
use crate::model::doc::DocsSource;
use crate::model::engine::{CreateOnDiskOptions, SearchEngine};

#[test]
fn test_eval_create_in_memory() -> Result<()> {
    let dataset = CisiDatasetReader::new("datasets/cisi");

    let mut engine = TantivySearchEngine::create_in_memory()?;
    engine.index_docs(&mut dataset.docs()?)?;

    assert_search_quality(&engine)?;

    Ok(())
}

#[test]
fn test_eval_create_on_disk() -> Result<()> {
    let dataset = CisiDatasetReader::new("datasets/cisi");
    let dir = TempDir::new()?;

    let mut engine = TantivySearchEngine::create_on_disk(
        CreateOnDiskOptions::builder()
            .index_dir(dir.path())
            .index_threads(1)
            .build(),
    )?;
    engine.index_docs(&mut dataset.docs()?)?;

    assert_search_quality(&engine)?;

    Ok(())
}

#[test]
fn test_eval_create_on_disk_and_open() -> Result<()> {
    let dataset = CisiDatasetReader::new("datasets/cisi");
    let dir = TempDir::new()?;

    {
        let mut engine = TantivySearchEngine::create_on_disk(
            CreateOnDiskOptions::builder()
                .index_dir(dir.path())
                .index_threads(1)
                .build(),
        )?;
        engine.index_docs(&mut dataset.docs()?)?;
    }

    let engine = TantivySearchEngine::open_from_disk(&dir)?;

    assert_search_quality(&engine)?;

    Ok(())
}

fn assert_search_quality(engine: &impl SearchEngine) -> Result<()> {
    let dataset = CisiDatasetReader::new("datasets/cisi");
    let quality = evaluate_search_quality(&mut dataset.queries()?, engine, 10)?;

    assert_eq!(quality.queries_count, 112);
    assert_eq!(quality.search_limit, 10);

    assert_float_relative_eq!(quality.ndcg_avg, 0.179_194_254_335_665_13, 0.01);

    assert_float_relative_eq!(quality.precision_avg, 0.158_928_571_428_571_4);
    assert_float_relative_eq!(quality.precision_p50, 0.1);
    assert_float_relative_eq!(quality.precision_p90, 0.490_000_000_000_000_55);
    assert_float_relative_eq!(quality.precision_p100, 0.7);

    assert_float_relative_eq!(quality.recall_avg, 0.381_079_248_316_754_6);
    assert_float_relative_eq!(quality.recall_p50, 0.096_875);
    assert_float_relative_eq!(quality.recall_p90, 1.0);
    assert_float_relative_eq!(quality.recall_p100, 1.0);

    Ok(())
}
