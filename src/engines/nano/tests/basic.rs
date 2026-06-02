use anyhow::Result;
use tempfile::TempDir;

use crate::engines::nano::engine::NanoSearchEngine;
use crate::model::engine::{CreateOnDiskOptions, SearchEngine};
use crate::utils::test_docs::create_cat_mouse_docs_iterator;
use crate::utils::test_docs::docs::*;

#[test]
fn test_search_fails_on_uninitialized_index() -> Result<()> {
    // setup
    let dir = TempDir::new()?;
    let engine = NanoSearchEngine::create_on_disk(
        CreateOnDiskOptions::builder()
            .index_dir(dir.path())
            .index_threads(1)
            .build(),
    )?;

    // execute
    let res = engine.search("cat", 10);

    // assert
    assert!(res.is_err());
    Ok(())
}

#[test]
fn test_open_index() -> Result<()> {
    // setup
    let dir = TempDir::new()?;

    // 1. create index in dir
    {
        let mut engine = NanoSearchEngine::create_on_disk(
            CreateOnDiskOptions::builder()
                .index_dir(dir.path())
                .index_threads(1)
                .build(),
        )?;
        engine.index_docs(&mut create_cat_mouse_docs_iterator())?;
    };

    // 2. open index from dir
    let engine = NanoSearchEngine::open_from_disk(dir.as_ref())?;

    // execute
    let docids = engine.search("cat", 10)?;

    // assert
    assert_eq!(docids.len(), 4);
    Ok(())
}

#[test]
fn test_search_limit() -> Result<()> {
    // setup
    let dir = TempDir::new()?;
    let mut engine = NanoSearchEngine::create_on_disk(
        CreateOnDiskOptions::builder()
            .index_dir(dir.path())
            .index_threads(1)
            .build(),
    )?;
    engine.index_docs(&mut create_cat_mouse_docs_iterator())?;

    // execute
    let docids = engine.search("cat", 2)?;

    // assert
    assert_eq!(
        docids,
        vec![
            CAT.id,           // 1st - full query match
            CAT_MOUSE_CAT.id, // 2nd - contains query term (twice)
        ]
    );
    Ok(())
}

#[test]
fn test_search_unknown_word() -> Result<()> {
    // setup
    let dir = TempDir::new()?;
    let mut engine = NanoSearchEngine::create_on_disk(
        CreateOnDiskOptions::builder()
            .index_dir(dir.path())
            .index_threads(1)
            .build(),
    )?;
    engine.index_docs(&mut create_cat_mouse_docs_iterator())?;

    // execute
    let docids = engine.search("unknown", 10)?;

    // assert
    assert_eq!(docids.len(), 0);
    Ok(())
}

#[test]
fn test_search() -> Result<()> {
    // setup
    let dir = TempDir::new()?;
    let mut engine = NanoSearchEngine::create_on_disk(
        CreateOnDiskOptions::builder()
            .index_dir(dir.path())
            .index_threads(1)
            .build(),
    )?;
    engine.index_docs(&mut create_cat_mouse_docs_iterator())?;

    // execute
    let docids = engine.search("cat", 10)?;

    // assert
    assert_eq!(
        docids,
        vec![
            CAT.id,           // 1st - full query match
            CAT_MOUSE_CAT.id, // 2nd - contains query term (twice)
            CAT_DOG.id,       // 3nd - contains query term (once)
            CAT_MOUSE.id      // 4rd - contains query term (once)
        ]
    );
    Ok(())
}

#[test]
fn test_search_with_multiple_words_query() -> Result<()> {
    // setup
    let dir = TempDir::new()?;
    let mut engine = NanoSearchEngine::create_on_disk(
        CreateOnDiskOptions::builder()
            .index_dir(dir.path())
            .index_threads(1)
            .build(),
    )?;
    engine.index_docs(&mut create_cat_mouse_docs_iterator())?;

    // execute
    let docids = engine.search("cat mouse", 10)?;

    // assert
    assert_eq!(
        docids,
        vec![
            CAT_MOUSE_CAT.id, // 1st - full query match (one term twice)
            CAT_MOUSE.id,     // 2nd - full query match
            CAT.id,           // 3rd - match with one of query terms
            MOUSE.id,         // 4th - match with one of query terms
            CAT_DOG.id,       // 5th - contains one of query terms
            DOG_MOUSE.id      // 6th - contains one of query terms
        ]
    );
    Ok(())
}
