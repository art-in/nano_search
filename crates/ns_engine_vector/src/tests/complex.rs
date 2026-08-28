use anyhow::Result;
use assert_float_eq::assert_float_relative_eq;
use ns_datasets::{CisiDatasetReader, DocsSource, QueriesSource};
use ns_engine::{CreateOnDiskOptions, SearchEngine};
use ns_eval::evaluate_search_quality;
use tempfile::TempDir;

use crate::engine::VectorSearchEngine;

#[test]
fn test_eval_create_on_disk_and_open() -> Result<()> {
    let dataset = CisiDatasetReader::new("../../datasets/cisi");
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

    assert_float_relative_eq!(quality.ndcg_avg, 0.167_069_088_996_724_85, 0.01);

    assert_float_relative_eq!(quality.precision_avg, 0.142_857_142_857_142_85);
    assert_float_relative_eq!(quality.precision_p50, 0.05);
    assert_float_relative_eq!(quality.precision_p90, 0.5);
    assert_float_relative_eq!(quality.precision_p100, 0.7);

    assert_float_relative_eq!(quality.recall_avg, 0.366_338_918_611_863_9);
    assert_float_relative_eq!(quality.recall_p50, 0.095_894_607_843_137_25);
    assert_float_relative_eq!(quality.recall_p90, 1.0);
    assert_float_relative_eq!(quality.recall_p100, 1.0);

    Ok(())
}
