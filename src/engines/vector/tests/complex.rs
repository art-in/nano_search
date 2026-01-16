use anyhow::Result;
use tempfile::TempDir;

use crate::dataset_readers::cisi::CisiDatasetReader;
use crate::engines::vector::engine::VectorSearchEngine;
use crate::eval::evaluate_search_quality;
use crate::eval::model::QueriesSource;
use crate::model::doc::DocsSource;
use crate::model::engine::{CreateOnDiskOptions, SearchEngine};
use crate::utils::GetPercentile;

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
        engine.index_docs(&mut dataset.docs()?)?;
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

    // assert precision
    assert_eq!(quality.precision_avg, 0.259_821_428_571_428_6);
    assert_eq!(quality.precisions.perc(0.5)?, 0.1);
    assert_eq!(quality.precisions.perc(0.9)?, 0.8);
    assert_eq!(quality.precisions.perc(1.0)?, 1.0);

    // assert recall
    assert_eq!(quality.recall_avg, 0.407_004_555_378_722);
    assert_eq!(quality.recalls.perc(0.5)?, 0.166_666_666_666_666_66);
    assert_eq!(quality.recalls.perc(0.9)?, 1.0);
    assert_eq!(quality.recalls.perc(1.0)?, 1.0);

    // assert NDCG
    assert_eq!(quality.ndcg_avg, 0.282_986_754_755_539_86);

    Ok(())
}
