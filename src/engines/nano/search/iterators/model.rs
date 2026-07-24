use anyhow::{Result, bail};

use crate::engines::nano::index::model::SegmentDocId;
use crate::utils::TreeNode;

/// Iterator over document IDs.
///
/// This is general abstraction over any source of document IDs, whether it is
/// serialized IDs on disk or in memory array or upstream iterators.
///
/// IDs should increase over the course of advancing.
///
/// Initially it should return [`ItDocId::NotStarted`], and needs to be warmed
/// up with first advance call.
///
/// In lucene/tantivy it's called `DocSet`.
pub trait DocIdIterator {
    fn advance(&mut self) -> Result<()>;
    fn advance_to(&mut self, target: SegmentDocId) -> Result<()>;
    fn current_docid(&self) -> Result<ItDocId>;
    fn explain(&self) -> TreeNode;
}

/// Iterator over document IDs with ability to score current document.
///
/// In lucene/tantivy it's called `Scorer`.
pub trait ScoringDocIdIterator: DocIdIterator {
    fn current_score(&self) -> Result<ItScore>;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ItDocId {
    NotStarted,
    Active(SegmentDocId),
    Exhausted,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ItScore {
    NotStarted,
    Active(f64),
    Exhausted,
}

impl ItDocId {
    pub fn is_not_started(self) -> bool {
        self == Self::NotStarted
    }

    pub fn is_exhausted(self) -> bool {
        self == Self::Exhausted
    }

    pub fn expect_val(self) -> Result<SegmentDocId> {
        if let Self::Active(docid) = self {
            Ok(docid)
        } else {
            bail!("should have Active state with a docid");
        }
    }
}

impl ItScore {
    pub fn expect_val(self) -> Result<f64> {
        if let Self::Active(score) = self {
            Ok(score)
        } else {
            bail!("should have Active state with a score");
        }
    }
}
