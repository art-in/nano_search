use anyhow::Result;
use tempfile::TempDir;

use super::engine::NanoSearchEngine;
use crate::{
    model::engine::SearchEngine,
    utils::test_docs::{ID, create_cat_mouse_docs_iterator},
};

#[test]
fn test_search_fails_on_uninitialized_index() -> Result<()> {
    // setup
    let dir = TempDir::new()?;
    let engine = NanoSearchEngine::create_on_disk(dir.as_ref())?;

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
        let mut engine = NanoSearchEngine::create_on_disk(dir.as_ref())?;
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
    let mut engine = NanoSearchEngine::create_on_disk(dir.as_ref())?;
    engine.index_docs(&mut create_cat_mouse_docs_iterator())?;

    // execute
    let docids = engine.search("cat", 2)?;

    // assert
    assert_eq!(
        docids,
        vec![
            ID.cat,           // 1st - full match with query
            ID.cat_mouse_cat, // 2nd - contains query term (twice)
        ]
    );
    Ok(())
}

#[test]
fn test_search_unknown_word() -> Result<()> {
    // setup
    let dir = TempDir::new()?;
    let mut engine = NanoSearchEngine::create_on_disk(dir.as_ref())?;
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
    let mut engine = NanoSearchEngine::create_on_disk(dir.as_ref())?;
    engine.index_docs(&mut create_cat_mouse_docs_iterator())?;

    // execute
    let docids = engine.search("cat", 10)?;

    // assert
    assert_eq!(
        docids,
        vec![
            ID.cat,           // 1st - full match with query
            ID.cat_mouse_cat, // 2nd - contains query term (twice)
            ID.cat_dog,       // 3nd - contains query term (once)
            ID.cat_mouse      // 4rd - contains query term (once)
        ]
    );
    Ok(())
}

#[test]
fn test_search_with_multiple_words_query() -> Result<()> {
    // setup
    let dir = TempDir::new()?;
    let mut engine = NanoSearchEngine::create_on_disk(dir.as_ref())?;
    engine.index_docs(&mut create_cat_mouse_docs_iterator())?;

    // execute
    let docids = engine.search("cat mouse", 10)?;

    // assert
    assert_eq!(
        docids,
        vec![
            ID.cat_mouse_cat, // 1st - full match with query (one term occures twice)
            ID.cat_mouse,     // 2nd - full match with query
            ID.cat,           // 3rd - match with one of query terms
            ID.mouse,         // 4th - match with one of query terms
            ID.cat_dog,       // 5th - contains one of query terms
            ID.dog_mouse      // 6th - contains one of query terms
        ]
    );
    Ok(())
}
