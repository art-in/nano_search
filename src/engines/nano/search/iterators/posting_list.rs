use std::borrow::Cow;

use anyhow::{Context, Result};

use super::model::{DocIdIterator, ItDocId, ItScore, ScoringDocIdIterator};
use crate::engines::nano::index::model::{
    DocPosting, DocPostingsForTerm, IndexSegment, SegmentDocId,
};
use crate::engines::nano::search::scoring;
use crate::utils::TreeNode;

// Iterator over document IDs, that reads posting list of concrete term from
// index segment.
pub struct PostingListIterator<'a> {
    segment: &'a dyn IndexSegment,
    term: String,
    postings: Option<DocPostingsForTerm<'a>>,
    current_posting: Option<Cow<'a, DocPosting>>,
    is_exhausted: bool,
}

impl<'a> PostingListIterator<'a> {
    pub fn create_for_segment(
        segment: &'a dyn IndexSegment,
        term: &str,
    ) -> Result<Self> {
        // TODO: ignore stop words
        let postings = segment.get_doc_postings_for_term(term)?;

        Ok(Self {
            segment,
            term: term.to_string(),
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
            TreeNode::new("Term").with_attr("term", self.term.clone());

        if self.postings.is_none() {
            node.add_attr("unknown_term", "true");
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

        Ok(ItScore::Active(score))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::nano::index::MemoryIndex;

    #[test]
    fn test_unknown_term() -> Result<()> {
        let segment = MemoryIndex::default();
        let mut it =
            PostingListIterator::create_for_segment(&segment, "unknown")?;

        assert!(matches!(it.current_docid()?, ItDocId::NotStarted));
        assert!(matches!(it.current_score()?, ItScore::NotStarted));

        it.advance()?;

        assert!(matches!(it.current_docid()?, ItDocId::Exhausted));
        assert!(matches!(it.current_score()?, ItScore::Exhausted));

        Ok(())
    }
}
