use std::borrow::Cow;

use anyhow::{Context, Result};
use ns_engine::Score;

use super::model::{DocIdIterator, ItDocId, ItScore, ScoringDocIdIterator};
use crate::index::model::{
    DocPosting, DocPostingsForTerm, IndexSegment, SegmentDocId,
};
use crate::search::scoring;
use crate::utils::TreeNode;

// Iterator over document IDs, that reads posting list of concrete term from
// index segment.
pub struct PostingListIterator<'a> {
    segment: &'a dyn IndexSegment,
    term: String,
    term_frequency: usize,
    postings: Option<DocPostingsForTerm<'a>>,
    current_posting: Option<Cow<'a, DocPosting>>,
    is_exhausted: bool,
}

impl<'a> PostingListIterator<'a> {
    pub fn create_for_segment(
        segment: &'a dyn IndexSegment,
        term: &str,
        term_frequency: usize,
    ) -> Result<Self> {
        let postings = segment.get_doc_postings_for_term(term)?;

        Ok(Self {
            segment,
            term: term.to_string(),
            term_frequency,
            postings,
            current_posting: None,
            is_exhausted: false,
        })
    }

    fn advance_internal(&mut self, target: Option<SegmentDocId>) -> Result<()> {
        if self.is_exhausted {
            return Ok(());
        }

        if self.postings.is_none() {
            self.is_exhausted = true;
            return Ok(());
        }

        // TODO: optimize non-scoring iteration.
        //
        // reading scoring data (e.g. term frequencies) may require additional
        // work while advancing docids. when scores are not needed (e.g. NOT
        // branches or a counting collector), it should be possible to skip
        // reading them.
        //
        // approaches:
        //
        // - runtime ScoringMode::On/Off flag propagated through the iterator
        //   tree. non-scoring branches (such as the excluded side of Exclusion)
        //   would always use Off. simple, but current_score() remains callable
        //   and would have to fail at runtime
        //
        // - encode scoring capability in the type system, making
        //   current_score() unavailable on non-scoring iterators. more
        //   type-safe, but requires additional iterator/planner abstractions
        //   and lots of boilerplate
        //
        // Lucene and Tantivy use a runtime flag approach, but calling score()
        // when scoring is disabled does not fail; it returns a constant (dummy)
        // score instead
        let postings = self.postings.as_mut().context("should exist")?;

        if let Some(target) = target {
            if self.current_posting.is_none() {
                self.current_posting = postings.iterator.next().transpose()?;
            }

            // TODO: use skip list for fast advancing to target
            while matches!(
                &self.current_posting,
                Some(posting) if posting.docid < target
            ) {
                self.current_posting = postings.iterator.next().transpose()?;
            }
        } else {
            self.current_posting = postings.iterator.next().transpose()?;
        }

        if self.current_posting.is_none() {
            self.is_exhausted = true;
        }

        Ok(())
    }
}

impl DocIdIterator for PostingListIterator<'_> {
    fn advance(&mut self) -> Result<()> {
        self.advance_internal(None)
    }

    fn advance_to(&mut self, target: SegmentDocId) -> Result<()> {
        self.advance_internal(Some(target))
    }

    fn current_docid(&self) -> Result<ItDocId> {
        if self.is_exhausted {
            Ok(ItDocId::Exhausted)
        } else if let Some(posting) = &self.current_posting {
            Ok(ItDocId::Active(posting.docid))
        } else {
            Ok(ItDocId::NotStarted)
        }
    }

    fn explain(&self) -> TreeNode {
        let mut node =
            TreeNode::new("PostingList").with_attr("term", self.term.clone());

        if self.postings.is_none() {
            node.add_attr("unknown_term", "true");
        }

        if self.term_frequency > 1 {
            node.add_attr("freq", self.term_frequency.to_string());
        }

        node
    }
}

impl ScoringDocIdIterator for PostingListIterator<'_> {
    fn current_score(&self) -> Result<ItScore> {
        if self.is_exhausted {
            return Ok(ItScore::Exhausted);
        }

        let Some(posting) = &self.current_posting else {
            return Ok(ItScore::NotStarted);
        };

        let postings = self.postings.as_ref().context("should exist")?;

        let score = scoring::calc_bm25(
            scoring::ScoringParams {
                doc_term_freq: posting.term_freq,
                doc_total_terms_count: *self
                    .segment
                    .get_doc_terms_count(posting.docid)?,
                docs_with_term_count: postings.count as u64,
                docs_total_count: self.segment.get_stats().indexed_docs_count,
            },
            self.segment.get_stats().terms_count_per_doc_avg,
        );

        Ok(ItScore::Active(score * self.term_frequency as Score))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ns_engine::Score;

    use super::super::test_utils::collect;
    use super::*;
    use crate::index::MemoryIndex;

    fn create_segment(doc_count: usize) -> MemoryIndex {
        let mut segment = MemoryIndex::default();
        let mut posting_list = BTreeMap::new();

        for i in 0..doc_count {
            let docid = i as SegmentDocId;
            posting_list.insert(
                docid,
                DocPosting {
                    docid,
                    term_freq: 1,
                },
            );
            segment.doc_term_counts.push(10);
        }

        segment.terms.insert("a".to_string(), posting_list);

        segment.stats.indexed_docs_count = doc_count as u64;
        segment.stats.terms_count_per_doc_avg = 10.0;
        segment.stats.max_posting_list_size = doc_count as u64;

        segment
    }

    #[test]
    fn test_basic() -> Result<()> {
        let segment = create_segment(5);
        let it = PostingListIterator::create_for_segment(&segment, "a", 1)?;

        let (docids, scores): (Vec<SegmentDocId>, Vec<Score>) =
            collect(it)?.into_iter().unzip();

        assert_eq!(docids, vec![0, 1, 2, 3, 4]);

        // do not care much about concrete score values in this test
        assert!(scores.iter().all(|s| *s > 0.0));

        Ok(())
    }

    #[test]
    fn test_term_frequency() -> Result<()> {
        let segment = create_segment(5);

        let it_one = PostingListIterator::create_for_segment(&segment, "a", 1)?;
        let it_two = PostingListIterator::create_for_segment(&segment, "a", 2)?;

        let (docids_one, scores_one): (Vec<SegmentDocId>, Vec<Score>) =
            collect(it_one)?.into_iter().unzip();
        let (docids_two, scores_two): (Vec<SegmentDocId>, Vec<Score>) =
            collect(it_two)?.into_iter().unzip();

        // docids are the same
        assert_eq!(docids_one, docids_two);

        // scores are doubled
        assert!(
            scores_one
                .iter()
                .zip(scores_two)
                .into_iter()
                .all(|(score_one, score_two)| (score_two / *score_one) == 2.0)
        );

        Ok(())
    }

    #[test]
    fn test_unknown_term() -> Result<()> {
        let segment = MemoryIndex::default();
        let it =
            PostingListIterator::create_for_segment(&segment, "unknown", 1)?;

        assert!(collect(it)?.is_empty());

        Ok(())
    }
}
