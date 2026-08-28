use anyhow::Result;
use assert_float_eq::assert_float_relative_eq;
use ns_datasets::{CisiDatasetReader, DocsSource, QueriesSource};
use ns_engine::{CreateOnDiskOptions, SearchEngine};
use ns_eval::evaluate_search_quality;
use tempfile::TempDir;

use crate::engine::TantivySearchEngine;

#[test]
fn test_eval_create_in_memory() -> Result<()> {
    let dataset = CisiDatasetReader::new("../../datasets/cisi");

    let mut engine = TantivySearchEngine::create_in_memory()?;
    engine.index_docs(&mut dataset.docs()?)?;

    assert_search_quality(&engine)?;

    Ok(())
}

#[test]
fn test_eval_create_on_disk() -> Result<()> {
    let dataset = CisiDatasetReader::new("../../datasets/cisi");
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
    let dataset = CisiDatasetReader::new("../../datasets/cisi");
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
    let dataset = CisiDatasetReader::new("../../datasets/cisi");
    let quality = evaluate_search_quality(&mut dataset.queries()?, engine, 10)?;

    assert_eq!(quality.queries_count, 112);
    assert_eq!(quality.search_limit, 10);

    assert_float_relative_eq!(quality.ndcg_avg, 0.183_004_074_811_264_68, 0.01);

    assert_float_relative_eq!(quality.precision_avg, 0.164_285_714_285_714_23);
    assert_float_relative_eq!(quality.precision_p50, 0.1);
    assert_float_relative_eq!(quality.precision_p90, 0.5);
    assert_float_relative_eq!(quality.precision_p100, 0.7);

    assert_float_relative_eq!(quality.recall_avg, 0.382_841_911_804_789);
    assert_float_relative_eq!(quality.recall_p50, 0.093_386_627_906_976_74);
    assert_float_relative_eq!(quality.recall_p90, 1.0);
    assert_float_relative_eq!(quality.recall_p100, 1.0);

    Ok(())
}
