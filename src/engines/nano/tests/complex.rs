use anyhow::Result;
use assert_float_eq::assert_float_relative_eq;
use tempfile::TempDir;

use crate::dataset_readers::cisi::CisiDatasetReader;
use crate::engines::nano::engine::NanoSearchEngine;
use crate::eval::evaluate_search_quality;
use crate::eval::model::QueriesSource;
use crate::model::{CreateOnDiskOptions, DocsSource, SearchEngine};

#[test]
fn test_eval_create_in_memory() -> Result<()> {
    let dataset = CisiDatasetReader::new("datasets/cisi");

    let mut engine = NanoSearchEngine::create_in_memory()?;
    engine.index_docs(&mut dataset.docs()?)?;

    assert_search_quality(&engine)?;

    Ok(())
}

#[test]
fn test_eval_create_on_disk() -> Result<()> {
    let dataset = CisiDatasetReader::new("datasets/cisi");
    let dir = TempDir::new()?;

    let mut engine = NanoSearchEngine::create_on_disk(
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
        let mut engine = NanoSearchEngine::create_on_disk(
            CreateOnDiskOptions::builder()
                .index_dir(dir.path())
                .index_threads(1)
                .build(),
        )?;
        engine.index_docs(&mut dataset.docs()?)?;
    }

    let engine = NanoSearchEngine::open_from_disk(&dir)?;

    assert_search_quality(&engine)?;

    Ok(())
}

fn assert_search_quality(engine: &impl SearchEngine) -> Result<()> {
    let dataset = CisiDatasetReader::new("datasets/cisi");
    let quality = evaluate_search_quality(&mut dataset.queries()?, engine, 10)?;

    assert_eq!(quality.queries_count, 112);
    assert_eq!(quality.search_limit, 10);

    assert_float_relative_eq!(quality.ndcg_avg, 0.220_955_568_892_424_04);

    assert_float_relative_eq!(quality.precision_avg, 0.188_392_857_142_857_22);
    assert_float_relative_eq!(quality.precision_p50, 0.1);
    assert_float_relative_eq!(quality.precision_p90, 0.590_000_000_000_000_5);
    assert_float_relative_eq!(quality.precision_p100, 0.9);

    assert_float_relative_eq!(quality.recall_avg, 0.395_092_405_402_701_9);
    assert_float_relative_eq!(quality.recall_p50, 0.115_966_386_554_621_85);
    assert_float_relative_eq!(quality.recall_p90, 1.0);
    assert_float_relative_eq!(quality.recall_p100, 1.0);

    Ok(())
}
