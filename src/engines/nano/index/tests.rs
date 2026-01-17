use std::borrow::Cow;

use anyhow::{Context, Result};
use tempfile::TempDir;

use super::disk::DiskIndexOptions;
use super::model::IndexMedium;
use super::*;
use crate::engines::nano::index::model::{
    DocPosting, Index, IndexSegment, IndexSegmentStats,
};
use crate::utils::test_docs::{ID, create_cat_mouse_docs_iterator};

#[test]
fn test_build_memory_index() -> Result<()> {
    // setup
    let mut docs_it = create_cat_mouse_docs_iterator();

    // execute
    let index = build_index(&IndexMedium::Memory, &mut docs_it)?;

    // assert
    assert_one_segment_index(index.as_ref())
}

#[test]
fn test_build_disk_index() -> Result<()> {
    // setup
    let mut docs_it = create_cat_mouse_docs_iterator();
    let dir = TempDir::new()?;
    let medium = IndexMedium::Disk(
        DiskIndexOptions::builder()
            .index_dir(dir.path())
            .index_threads(1)
            .build(),
    );

    // execute
    let index = build_index(&medium, &mut docs_it)?;

    // assert
    assert_one_segment_index(index.as_ref())
}

#[test]
fn test_build_disk_index_and_open() -> Result<()> {
    // setup
    let mut docs_it = create_cat_mouse_docs_iterator();
    let dir = TempDir::new()?;
    let medium = IndexMedium::Disk(
        DiskIndexOptions::builder()
            .index_dir(dir.path())
            .index_threads(1)
            .build(),
    );

    // execute
    build_index(&medium, &mut docs_it)?;
    let index = open_index(&medium)?;

    // assert
    assert_one_segment_index(index.as_ref())
}

fn assert_one_segment_index(index: &dyn Index) -> Result<()> {
    let segments = index.get_segments();
    assert_eq!(segments.len(), 1);
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
        &[
            DocPosting {
                docid: ID.cat,
                term_count: 1,
            },
            DocPosting {
                docid: ID.cat_dog,
                term_count: 1,
            },
            DocPosting {
                docid: ID.cat_mouse,
                term_count: 1,
            },
            DocPosting {
                docid: ID.cat_mouse_cat,
                term_count: 2,
            },
        ],
    )?;

    // assert correct doc terms counts
    assert_eq!(segment.get_doc_terms_count(ID.cat)?, 1);
    assert_eq!(segment.get_doc_terms_count(ID.dog)?, 1);
    assert_eq!(segment.get_doc_terms_count(ID.mouse)?, 1);
    assert_eq!(segment.get_doc_terms_count(ID.cat_dog)?, 2);
    assert_eq!(segment.get_doc_terms_count(ID.dog_mouse)?, 2);
    assert_eq!(segment.get_doc_terms_count(ID.cat_mouse)?, 2);
    assert_eq!(segment.get_doc_terms_count(ID.cat_mouse_cat)?, 3);

    // assert correct index statistics
    assert_eq!(
        segment.get_stats(),
        &IndexSegmentStats {
            indexed_docs_count: 7,
            max_posting_list_size: 4, // docs with "cat" term
            terms_count_per_doc_avg: 12.0 / 7.0,
        }
    );

    Ok(())
}

#[test]
fn test_build_disk_index_with_multiple_segments() -> Result<()> {
    // setup
    let mut docs_it = create_cat_mouse_docs_iterator();
    let dir = TempDir::new()?;
    let medium = IndexMedium::Disk(
        DiskIndexOptions::builder()
            .index_dir(dir.path())
            .index_threads(1)
            .max_segment_docs(4)
            .build(),
    );

    // execute
    let index = build_index(&medium, &mut docs_it)?;

    // assert
    assert_multiple_segments_index(index.as_ref())
}

#[test]
fn test_build_disk_index_with_multiple_segments_and_open() -> Result<()> {
    // setup
    let mut docs_it = create_cat_mouse_docs_iterator();
    let dir = TempDir::new()?;
    let medium = IndexMedium::Disk(
        DiskIndexOptions::builder()
            .index_dir(dir.path())
            .index_threads(1)
            .max_segment_docs(4)
            .build(),
    );

    // execute
    build_index(&medium, &mut docs_it)?;
    let index = open_index(&medium)?;

    // assert
    assert_multiple_segments_index(index.as_ref())
}

fn assert_multiple_segments_index(index: &dyn Index) -> Result<()> {
    let segments = index.get_segments();

    // assert
    // basically we index 7 documents with max 4 documents in one segment,
    // and expect there's is going to be 2 segments with 4 and 3 docs
    assert_eq!(create_cat_mouse_docs_iterator().count(), 7);
    assert_eq!(segments.len(), 2);

    // when opening index from disk, segment sequence is undetermined, since
    // their names on disk are randomized. so we have to normalize it first
    let mut first_segment = segments[0];
    let mut second_segment = segments[1];

    if first_segment.get_stats().indexed_docs_count == 3 {
        std::mem::swap(&mut first_segment, &mut second_segment);
    }

    // assert correct first segment
    {
        assert_postings_for_term(
            first_segment,
            "cat",
            &[
                DocPosting {
                    docid: ID.cat,
                    term_count: 1,
                },
                DocPosting {
                    docid: ID.cat_dog,
                    term_count: 1,
                },
            ],
        )?;

        assert_eq!(first_segment.get_doc_terms_count(ID.cat)?, 1);
        assert_eq!(first_segment.get_doc_terms_count(ID.dog)?, 1);
        assert_eq!(first_segment.get_doc_terms_count(ID.mouse)?, 1);
        assert_eq!(first_segment.get_doc_terms_count(ID.cat_dog)?, 2);

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
        assert_postings_for_term(
            second_segment,
            "cat",
            &[
                DocPosting {
                    docid: ID.cat_mouse,
                    term_count: 1,
                },
                DocPosting {
                    docid: ID.cat_mouse_cat,
                    term_count: 2,
                },
            ],
        )?;

        assert_eq!(second_segment.get_doc_terms_count(ID.dog_mouse)?, 2);
        assert_eq!(second_segment.get_doc_terms_count(ID.cat_mouse)?, 2);
        assert_eq!(second_segment.get_doc_terms_count(ID.cat_mouse_cat)?, 3);

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
    expected_postings: &[DocPosting],
) -> Result<()> {
    let postings_it = segment
        .get_doc_postings_for_term(&term.to_string())?
        .context(format!("postings for term '{term}' should be found"))?;
    let postings = postings_it
        .iterator
        .collect::<Result<Vec<Cow<DocPosting>>>>()?;

    let expected_postings = expected_postings
        .iter()
        .map(Cow::Borrowed)
        .collect::<Vec<Cow<'_, DocPosting>>>();

    assert_eq!(postings_it.count, postings.len());
    assert!(postings.iter().map(|p| p.docid).is_sorted());
    assert_eq!(postings, expected_postings);

    Ok(())
}
