use super::{model::IndexType, *};
use crate::{
    model::{doc::Doc, engine::IndexStats},
    search_engines::nano::index::model::DocPosting,
};
use anyhow::{Context, Result};

struct TestDocsIterator {
    docs: Vec<Doc>,
    position: usize,
}

impl Iterator for TestDocsIterator {
    type Item = Doc;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.docs.len() {
            let doc = self.docs[self.position].clone();
            self.position += 1;
            Some(doc)
        } else {
            None
        }
    }
}

impl TestDocsIterator {
    fn new() -> Self {
        let doc_texts = Vec::from([
            (0, "cat"),
            (1, "dog"),
            (2, "mouse"),
            (3, "cat dog"),
            (4, "dog mouse"),
            (5, "cat mouse cat"),
        ]);

        TestDocsIterator {
            docs: doc_texts
                .iter()
                .map(|(id, text)| Doc {
                    id: *id,
                    text: text.to_string(),
                })
                .collect(),
            position: 0,
        }
    }
}

#[test]
fn test_build_memory_index() -> Result<()> {
    test_build_index_with_type(model::IndexType::MemoryIndex)
}

#[test]
fn test_build_fs_index() -> Result<()> {
    let dir = tempfile::tempdir()?;
    test_build_index_with_type(model::IndexType::FsIndex(
        dir.path().to_path_buf(),
    ))
}

fn test_build_index_with_type(index_type: IndexType) -> Result<()> {
    let index = build_index(&index_type, &mut TestDocsIterator::new())?;

    let res = index.get_doc_postings_for_term(&"xxx".to_string())?;
    assert!(res.is_none(), "postings for term 'xxx' should not be found");

    let doc_postings_for_term = index
        .get_doc_postings_for_term(&"cat".to_string())?
        .context("postings for term 'cat' should be found")?;

    assert_eq!(doc_postings_for_term.count, 3);
    assert_eq!(
        doc_postings_for_term.iterator.collect::<Vec<DocPosting>>(),
        Vec::from([
            DocPosting {
                docid: 0,
                term_count: 1,
                total_terms_count: 1
            },
            DocPosting {
                docid: 3,
                term_count: 1,
                total_terms_count: 2
            },
            DocPosting {
                docid: 5,
                term_count: 2,
                total_terms_count: 3
            }
        ])
    );

    assert_eq!(
        index.get_index_stats(),
        &IndexStats {
            indexed_docs_count: 6,
            posting_lists_count: 3,
            max_posting_list_size: 3,
            terms_count_per_doc_avg: 1.6666666666666667,
        }
    );

    Ok(())
}
