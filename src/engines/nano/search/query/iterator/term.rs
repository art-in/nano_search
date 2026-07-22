use std::borrow::Cow;

use anyhow::{Context, Result};

use super::model::{DocIdIterator, ItDocId, ItScore, ScoringDocIdIterator};
use crate::engines::nano::index::model::{
    DocPosting, DocPostingsForTerm, IndexSegment, SegmentDocId,
};
use crate::engines::nano::search::scoring;
use crate::utils::TreeNode;

// TODO: rename PostingListIterator (?)
pub struct TermDocIdIterator<'a> {
    segment: &'a dyn IndexSegment,
    term: String,
    postings: Option<DocPostingsForTerm<'a>>,
    current_posting: Option<Cow<'a, DocPosting>>,
    is_exhausted: bool,
}

impl<'a> TermDocIdIterator<'a> {
    pub fn create_for_segment(
        segment: &'a dyn IndexSegment,
        term: &str,
    ) -> Result<Self> {
        let postings = segment.get_doc_postings_for_term(term)?;
        let is_exhausted = postings.is_none();

        Ok(Self {
            segment,
            term: term.to_string(),
            postings,
            current_posting: None,
            is_exhausted,
        })
    }

    fn advance_internal(&mut self, target: Option<SegmentDocId>) -> Result<()> {
        if self.is_exhausted {
            return Ok(());
        }

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

impl DocIdIterator for TermDocIdIterator<'_> {
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
            node.add_attr("unknown", "true");
        }

        node
    }
}

impl ScoringDocIdIterator for TermDocIdIterator<'_> {
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
            TermDocIdIterator::create_for_segment(&segment, "unknown")?;

        assert!(matches!(it.current_docid()?, ItDocId::Exhausted));
        assert!(matches!(it.current_score()?, ItScore::Exhausted));

        it.advance()?;

        assert!(matches!(it.current_docid()?, ItDocId::Exhausted));
        assert!(matches!(it.current_score()?, ItScore::Exhausted));

        Ok(())
    }
}
