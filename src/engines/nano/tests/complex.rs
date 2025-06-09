use anyhow::{Context, Result};
use tempfile::TempDir;

use crate::docs;
use crate::engines::nano::engine::NanoSearchEngine;
use crate::model::engine::SearchEngine;

#[test]
fn test_cisi_create_in_memory() -> Result<()> {
    let docs = docs::cisi::load_docs()?;

    let mut engine = NanoSearchEngine::create_in_memory()?;
    engine.index_docs(&mut docs.into_iter())?;

    assert_search_quality(&engine)?;

    Ok(())
}

#[test]
fn test_cisi_create_on_disk() -> Result<()> {
    let docs = docs::cisi::load_docs()?;
    let dir = TempDir::new()?;

    let mut engine = NanoSearchEngine::create_on_disk(&dir)?;
    engine.index_docs(&mut docs.into_iter())?;

    assert_search_quality(&engine)?;

    Ok(())
}

#[test]
fn test_cisi_create_on_disk_and_open() -> Result<()> {
    let docs = docs::cisi::load_docs()?;
    let dir = TempDir::new()?;

    {
        let mut engine = NanoSearchEngine::create_on_disk(&dir)?;
        engine.index_docs(&mut docs.into_iter())?;
    }

    let engine = NanoSearchEngine::open_from_disk(&dir)?;

    assert_search_quality(&engine)?;

    Ok(())
}

fn assert_search_quality(engine: &NanoSearchEngine) -> Result<()> {
    let quality = docs::cisi::search_quality::search_and_calc_quality(engine)?;

    assert_eq!(quality.queries_count, 112);

    // assert precision
    let precision_percs = quality
        .precision_percs
        .percentiles([0.5, 0.9, 1.0])?
        .context("percentiles should exist in result")?;

    assert_eq!(quality.precision_avg, 0.18839285714285722);
    assert_eq!(precision_percs, [0.1, 0.5900000000000005, 0.9]);

    // assert recall
    let recall_percs = quality
        .recall_percs
        .percentiles([0.5, 0.9, 1.0])?
        .context("percentiles should exist in result")?;

    assert_eq!(quality.recall_avg, 0.3950924054027019);
    assert_eq!(recall_percs, [0.11596638655462185, 1.0, 1.0]);

    Ok(())
}
