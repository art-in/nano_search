use anyhow::{Context, Result};
use tempfile::TempDir;

use super::disk::DiskIndexOptions;
use super::model::IndexMedium;
use super::*;
use crate::engines::nano::index::model::{
    DocPosting, IndexSegment, IndexSegmentStats,
};
use crate::utils::test_docs::{ID, create_cat_mouse_docs_iterator};

#[test]
fn test_build_memory_index() -> Result<()> {
    test_build_index(IndexMedium::Memory)
}

#[test]
fn test_build_disk_index() -> Result<()> {
    let dir = TempDir::new()?;
    test_build_index(IndexMedium::Disk(DiskIndexOptions::new(&dir)))
}

fn test_build_index(index_medium: IndexMedium) -> Result<()> {
    // setup
    let mut docs_it = create_cat_mouse_docs_iterator();

    // execute
    let index = build_index(&index_medium, &mut docs_it)?;
    let segments = index.get_segments();
    let segment = segments[0];

    // assert no postings for unknown term
    {
        let res = segment.get_doc_postings_for_term(&"xxx".to_string())?;
        assert!(res.is_none(), "postings for term 'xxx' should not be found");
    }

    // assert correct postings for known term
    assert_postings_for_term(
        segment,
        "cat",
        vec![
            DocPosting {
                docid: ID.cat,
                term_count: 1,
                total_terms_count: 1,
            },
            DocPosting {
                docid: ID.cat_dog,
                term_count: 1,
                total_terms_count: 2,
            },
            DocPosting {
                docid: ID.cat_mouse,
                term_count: 1,
                total_terms_count: 2,
            },
            DocPosting {
                docid: ID.cat_mouse_cat,
                term_count: 2,
                total_terms_count: 3,
            },
        ],
    )?;

    // assert correct index statistics
    assert_eq!(
        segment.get_stats(),
        &IndexSegmentStats {
            indexed_docs_count: 7,
            max_posting_list_size: 4,
            terms_count_per_doc_avg: 12.0 / 7.0,
        }
    );

    Ok(())
}

#[test]
fn test_build_disk_index_with_multiple_segments() -> Result<()> {
    // setup
    let dir = TempDir::new()?;
    let options = DiskIndexOptions::new(&dir).set_max_segment_docs(4);
    let index_medium = IndexMedium::Disk(options);
    let mut docs_it = create_cat_mouse_docs_iterator();

    // execute
    let index = build_index(&index_medium, &mut docs_it)?;
    let segments = index.get_segments();

    // assert
    // basically we index 7 documents with max 4 documents in one segment,
    // and expect there's is going to be 2 segments with 4 and 3 docs
    assert_eq!(create_cat_mouse_docs_iterator().count(), 7);
    assert_eq!(segments.len(), 2);

    // assert correct first segment
    {
        let first_segment = segments[0];

        assert_postings_for_term(
            first_segment,
            "cat",
            vec![
                DocPosting {
                    docid: ID.cat,
                    term_count: 1,
                    total_terms_count: 1,
                },
                DocPosting {
                    docid: ID.cat_dog,
                    term_count: 1,
                    total_terms_count: 2,
                },
            ],
        )?;

        assert_eq!(
            first_segment.get_stats(),
            &IndexSegmentStats {
                indexed_docs_count: 4,
                max_posting_list_size: 2,
                terms_count_per_doc_avg: 5.0 / 4.0
            }
        );
    }

    // assert correct second segment
    {
        let second_segment = segments[1];

        assert_postings_for_term(
            second_segment,
            "cat",
            vec![
                DocPosting {
                    docid: ID.cat_mouse,
                    term_count: 1,
                    total_terms_count: 2,
                },
                DocPosting {
                    docid: ID.cat_mouse_cat,
                    term_count: 2,
                    total_terms_count: 3,
                },
            ],
        )?;

        assert_eq!(
            second_segment.get_stats(),
            &IndexSegmentStats {
                indexed_docs_count: 3,
                max_posting_list_size: 3,
                terms_count_per_doc_avg: 7.0 / 3.0
            }
        );
    }

    Ok(())
}

fn assert_postings_for_term(
    segment: &dyn IndexSegment,
    term: &str,
    expected_postings: Vec<DocPosting>,
) -> Result<()> {
    let postings_it = segment
        .get_doc_postings_for_term(&term.to_string())?
        .context(format!("postings for term '{term}' should be found"))?;
    let postings: Vec<DocPosting> = postings_it.iterator.collect();

    assert_eq!(postings_it.count, postings.len());
    assert!(postings.iter().map(|p| p.docid).is_sorted());
    assert_eq!(postings, expected_postings);

    Ok(())
}
