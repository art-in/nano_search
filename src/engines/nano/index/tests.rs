use anyhow::{Context, Result};
use tempfile::TempDir;

use super::model::IndexType;
use super::*;
use crate::engines::nano::index::model::{DocPosting, Index};
use crate::model::engine::IndexStats;
use crate::utils::test_docs::{ID, create_cat_mouse_docs_iterator};

#[test]
fn test_build_memory_index() -> Result<()> {
    test_build_index(model::IndexType::MemoryIndex)
}

#[test]
fn test_build_fs_index() -> Result<()> {
    let dir = TempDir::new()?;
    test_build_index(model::IndexType::FsIndex(dir.path().to_path_buf()))
}

fn test_build_index(index_type: IndexType) -> Result<()> {
    // setup
    let index: Box<dyn Index + 'static> =
        build_index(&index_type, &mut create_cat_mouse_docs_iterator())?;

    {
        // execute
        let res = index.get_doc_postings_for_term(&"xxx".to_string())?;

        // assert
        assert!(res.is_none(), "postings for term 'xxx' should not be found");
    };

    {
        // execute
        let doc_postings_for_term = index
            .get_doc_postings_for_term(&"cat".to_string())?
            .context("postings for term 'cat' should be found")?;

        // assert
        let doc_postings =
            doc_postings_for_term.iterator.collect::<Vec<DocPosting>>();

        assert_eq!(doc_postings_for_term.count, doc_postings.len());
        assert!(doc_postings.iter().map(|posting| posting.docid).is_sorted());
        assert_eq!(
            doc_postings,
            Vec::from([
                DocPosting {
                    docid: ID.cat,
                    term_count: 1,
                    total_terms_count: 1
                },
                DocPosting {
                    docid: ID.cat_dog,
                    term_count: 1,
                    total_terms_count: 2
                },
                DocPosting {
                    docid: ID.cat_mouse,
                    term_count: 1,
                    total_terms_count: 2
                },
                DocPosting {
                    docid: ID.cat_mouse_cat,
                    term_count: 2,
                    total_terms_count: 3
                }
            ])
        );
    };

    assert_eq!(
        index.get_stats(),
        &IndexStats {
            indexed_docs_count: 7,
            max_posting_list_size: 4,
            terms_count_per_doc_avg: 1.7142857142857142,
        }
    );

    Ok(())
}
