use anyhow::Result;
use assert_float_eq::assert_float_relative_eq;
use tempfile::TempDir;

use crate::dataset_readers::cisi::CisiDatasetReader;
use crate::engines::tantivy::engine::TantivySearchEngine;
use crate::eval::evaluate_search_quality;
use crate::eval::model::QueriesSource;
use crate::model::doc::DocsSource;
use crate::model::engine::{CreateOnDiskOptions, SearchEngine};
use crate::utils::GetPercentile;

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

    // assert precision
    assert_eq!(quality.precision_avg, 0.18839285714285717);
    assert_eq!(quality.precisions.perc(0.5)?, 0.1);
    assert_eq!(quality.precisions.perc(0.9)?, 0.5);
    assert_eq!(quality.precisions.perc(1.0)?, 0.9);

    // assert recall
    assert_eq!(quality.recall_avg, 0.3975600431151478);
    assert_eq!(quality.recalls.perc(0.5)?, 0.13392857142857142);
    assert_eq!(quality.recalls.perc(0.9)?, 1.0);
    assert_eq!(quality.recalls.perc(1.0)?, 1.0);

    // assert NDCG
    assert_float_relative_eq!(quality.ndcg_avg, 0.212, 0.01);

    Ok(())
}
