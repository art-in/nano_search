use anyhow::Result;
use tempfile::TempDir;

use crate::dataset_readers::cisi;
use crate::engines::vector::engine::VectorSearchEngine;
use crate::eval::evaluate_search_quality;
use crate::model::doc::DocsSource;
use crate::model::engine::SearchEngine;
use crate::utils::GetPercentile;

#[test]
fn test_eval_create_on_disk_and_open() -> Result<()> {
    let docs = cisi::load_docs()?;
    let dir = TempDir::new()?;

    {
        let mut engine = VectorSearchEngine::create_on_disk(&dir)?;
        engine.index_docs(&mut docs.docs()?)?;
    }

    let engine = VectorSearchEngine::open_from_disk(&dir)?;

    assert_search_quality(&engine)?;

    Ok(())
}

fn assert_search_quality(engine: &dyn SearchEngine) -> Result<()> {
    let queries = cisi::load_queries()?;
    let quality =
        evaluate_search_quality(&mut queries.iter().cloned(), engine, 10)?;

    assert_eq!(quality.queries_count, 112);

    // assert precision
    assert_eq!(quality.precision_avg, 0.2598214285714286);
    assert_eq!(quality.precisions.perc(0.5)?, 0.1);
    assert_eq!(quality.precisions.perc(0.9)?, 0.8);
    assert_eq!(quality.precisions.perc(1.0)?, 1.0);

    // assert recall
    assert_eq!(quality.recall_avg, 0.407004555378722);
    assert_eq!(quality.recalls.perc(0.5)?, 0.16666666666666666);
    assert_eq!(quality.recalls.perc(0.9)?, 1.0);
    assert_eq!(quality.recalls.perc(1.0)?, 1.0);

    // assert NDCG
    assert_eq!(quality.ndcg_avg, 0.28298675475553986);

    Ok(())
}
