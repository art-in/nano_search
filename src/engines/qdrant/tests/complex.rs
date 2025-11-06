use anyhow::Result;

use crate::dataset_readers::cisi;
use crate::engines::qdrant::engine::QdrantSearchEngine;
use crate::eval::evaluate_search_quality;
use crate::model::doc::DocsSource;
use crate::model::engine::SearchEngine;
use crate::utils::GetPercentile;

#[test]
fn test_eval_create_on_disk_and_open() -> Result<()> {
    let docs = cisi::load_docs()?;

    {
        let mut engine = QdrantSearchEngine::create_on_disk("")?;
        engine.index_docs(&mut docs.docs())?;
    }

    let engine = QdrantSearchEngine::open_from_disk("")?;

    assert_search_quality(&engine)?;

    Ok(())
}

fn assert_search_quality(engine: &QdrantSearchEngine) -> Result<()> {
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

    Ok(())
}
