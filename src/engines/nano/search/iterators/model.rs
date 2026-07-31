use anyhow::{Result, bail};

use crate::engines::nano::index::model::SegmentDocId;
use crate::utils::TreeNode;

/// Iterator over document IDs.
///
/// This is general abstraction over any source of document IDs, whether it is
/// serialized IDs on disk or in memory array or other upstream iterators in a
/// tree.
///
/// IDs should increase over the course of advancing.
///
/// Initially it should return [`ItDocId::NotStarted`], and needs to be warmed
/// up with first advance call.
///
/// In Lucene/Tantivy it's called `DocSet`.
pub trait DocIdIterator {
    fn advance(&mut self) -> Result<()>;
    fn advance_to(&mut self, target: SegmentDocId) -> Result<()>;
    fn current_docid(&self) -> Result<ItDocId>;
    #[allow(unused)]
    fn explain(&self) -> TreeNode;
}

/// Iterator over document IDs with ability to score current document.
///
/// In Lucene/Tantivy it's called `Scorer`.
pub trait ScoringDocIdIterator: DocIdIterator {
    /// Returns score of current document.
    ///
    /// Splitting docid and score getters allows us to perform potentially heavy
    /// score calculation only when it is really required, e.g. for candidates
    /// that reached root iterator of the tree, and we need to sort them by
    /// score.
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

    pub fn expect(self) -> Result<SegmentDocId> {
        if let Self::Active(docid) = self {
            Ok(docid)
        } else {
            bail!("should have Active state with a docid");
        }
    }
}

impl ItScore {
    #[allow(unused)]
    pub fn is_not_started(self) -> bool {
        self == Self::NotStarted
    }

    #[allow(unused)]
    pub fn is_exhausted(self) -> bool {
        self == Self::Exhausted
    }

    pub fn expect(self) -> Result<f64> {
        if let Self::Active(score) = self {
            Ok(score)
        } else {
            bail!("should have Active state with a score");
        }
    }
}
