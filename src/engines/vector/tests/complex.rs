use anyhow::Result;
use assert_float_eq::assert_float_relative_eq;
use tempfile::TempDir;

use crate::dataset_readers::cisi::CisiDatasetReader;
use crate::engines::vector::engine::VectorSearchEngine;
use crate::eval::evaluate_search_quality;
use crate::eval::model::QueriesSource;
use crate::model::doc::DocsSource;
use crate::model::engine::{CreateOnDiskOptions, SearchEngine};

#[test]
fn test_eval_create_on_disk_and_open() -> Result<()> {
    let dataset = CisiDatasetReader::new("datasets/cisi");
    let dir = TempDir::new()?;

    {
        let mut engine = VectorSearchEngine::create_on_disk(
            CreateOnDiskOptions::builder()
                .index_dir(dir.path())
                .index_threads(1)
                .build(),
        )?;

        // reduce amount of docs to index to make test run faster
        let docs = &mut dataset.docs()?.take(100);

        engine.index_docs(docs)?;
    }

    let engine = VectorSearchEngine::open_from_disk(&dir)?;

    assert_search_quality(&engine, &dataset)?;

    Ok(())
}

fn assert_search_quality(
    engine: &dyn SearchEngine,
    dataset: &CisiDatasetReader,
) -> Result<()> {
    let quality = evaluate_search_quality(&mut dataset.queries()?, engine, 10)?;

    assert_eq!(quality.queries_count, 112);
    assert_eq!(quality.search_limit, 10);

    assert_float_relative_eq!(quality.ndcg_avg, 0.167_688_336_365_987_57);

    assert_float_relative_eq!(quality.precision_avg, 0.145_535_714_285_714_27);
    assert_float_relative_eq!(quality.precision_p50, 0.05);
    assert_float_relative_eq!(quality.precision_p90, 0.5);
    assert_float_relative_eq!(quality.precision_p100, 0.8);

    assert_float_relative_eq!(quality.recall_avg, 0.367_113_610_871_619_03);
    assert_float_relative_eq!(quality.recall_p50, 0.095_894_607_843_137_25);
    assert_float_relative_eq!(quality.recall_p90, 1.0);
    assert_float_relative_eq!(quality.recall_p100, 1.0);

    Ok(())
}
