use std::borrow::Cow;

use anyhow::{Context, Result};

use super::model::{DocIdIterator, ItDocId, ItScore, ScoringDocIdIterator};
use crate::engines::nano::index::model::{
    DocPosting, DocPostingsForTerm, IndexSegment, SegmentDocId,
};
use crate::engines::nano::search::scoring;

pub struct TermDocIdIterator<'a> {
    postings: DocPostingsForTerm<'a>,
    segment: &'a dyn IndexSegment,
    current_posting: Option<Cow<'a, DocPosting>>,
    is_exhausted: bool,
}

impl<'a> TermDocIdIterator<'a> {
    pub fn create_for_segment(
        segment: &'a dyn IndexSegment,
        term: &str,
    ) -> Result<Self> {
        let postings = segment
            .get_doc_postings_for_term(term)?
            .context("should have term")?;

        Ok(Self {
            postings,
            segment,
            current_posting: None,
            is_exhausted: false,
        })
    }

    fn advance_internal(&mut self, target: Option<SegmentDocId>) -> Result<()> {
        if self.is_exhausted {
            return Ok(());
        }

        if let Some(target) = target {
            if self.current_posting.is_none() {
                self.current_posting =
                    self.postings.iterator.next().transpose()?;
            }

            // TODO: use skip list for fast advancing to target
            while matches!(&self.current_posting, Some(posting) if posting.docid < target)
            {
                self.current_posting =
                    self.postings.iterator.next().transpose()?;
            }
        } else {
            self.current_posting = self.postings.iterator.next().transpose()?;
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
        } else if let Some(current_posting) = &self.current_posting {
            Ok(ItDocId::Active(current_posting.docid))
        } else {
            Ok(ItDocId::NotStarted)
        }
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

        let score = scoring::calc_bm25(
            scoring::ScoringParams {
                doc_term_freq: posting.term_freq,
                doc_total_terms_count: *self
                    .segment
                    .get_doc_terms_count(posting.docid)?,
                docs_with_term_count: self.postings.count as u64,
                docs_total_count: self.segment.get_stats().indexed_docs_count,
            },
            self.segment.get_stats().terms_count_per_doc_avg,
        );

        Ok(ItScore::Active(score))
    }
}
