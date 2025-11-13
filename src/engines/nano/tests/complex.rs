use anyhow::Result;
use tempfile::TempDir;

use crate::dataset_readers::cisi;
use crate::engines::nano::engine::NanoSearchEngine;
use crate::eval::evaluate_search_quality;
use crate::model::doc::DocsSource;
use crate::model::engine::SearchEngine;
use crate::utils::GetPercentile;

#[test]
fn test_eval_create_in_memory() -> Result<()> {
    let docs = cisi::load_docs()?;

    let mut engine = NanoSearchEngine::create_in_memory()?;
    engine.index_docs(&mut docs.docs()?)?;

    assert_search_quality(&engine)?;

    Ok(())
}

#[test]
fn test_eval_create_on_disk() -> Result<()> {
    let docs = cisi::load_docs()?;
    let dir = TempDir::new()?;

    let mut engine = NanoSearchEngine::create_on_disk(&dir)?;
    engine.index_docs(&mut docs.docs()?)?;

    assert_search_quality(&engine)?;

    Ok(())
}

#[test]
fn test_eval_create_on_disk_and_open() -> Result<()> {
    let docs = cisi::load_docs()?;
    let dir = TempDir::new()?;

    {
        let mut engine = NanoSearchEngine::create_on_disk(&dir)?;
        engine.index_docs(&mut docs.docs()?)?;
    }

    let engine = NanoSearchEngine::open_from_disk(&dir)?;

    assert_search_quality(&engine)?;

    Ok(())
}

fn assert_search_quality(engine: &impl SearchEngine) -> Result<()> {
    let queries = cisi::load_queries()?;
    let quality =
        evaluate_search_quality(&mut queries.iter().cloned(), engine, 10)?;

    assert_eq!(quality.queries_count, 112);

    // assert precision
    assert_eq!(quality.precision_avg, 0.18839285714285722);
    assert_eq!(quality.precisions.perc(0.5)?, 0.1);
    assert_eq!(quality.precisions.perc(0.9)?, 0.5900000000000005);
    assert_eq!(quality.precisions.perc(1.0)?, 0.9);

    // assert recall
    assert_eq!(quality.recall_avg, 0.3950924054027019);
    assert_eq!(quality.recalls.perc(0.5)?, 0.11596638655462185);
    assert_eq!(quality.recalls.perc(0.9)?, 1.0);
    assert_eq!(quality.recalls.perc(1.0)?, 1.0);

    Ok(())
}
