//! Set of utilities to support unit testing of iterators.
use anyhow::{Result, ensure};
use ns_engine::Score;

use super::model::{DocIdIterator, ItDocId, ItScore, ScoringDocIdIterator};
use crate::index::model::SegmentDocId;
use crate::utils::TreeNode;

pub struct TestScoringDocIdIterator {
    docids: Vec<SegmentDocId>,
    scores: Vec<Score>,
    current_idx: Option<usize>,
}

impl TestScoringDocIdIterator {
    pub fn new(docids: Vec<SegmentDocId>, scores: Vec<Score>) -> Result<Self> {
        ensure!(docids.len() == scores.len());
        ensure!(docids.is_sorted());
        Ok(Self {
            docids,
            scores,
            current_idx: None,
        })
    }
}

impl DocIdIterator for TestScoringDocIdIterator {
    fn advance(&mut self) -> Result<()> {
        self.current_idx = Some(self.current_idx.map_or(0, |v| v + 1));
        Ok(())
    }

    fn advance_to(&mut self, target: SegmentDocId) -> Result<()> {
        let mut current_idx = self.current_idx.map_or(0, |v| v);
        while current_idx < self.docids.len()
            && self.docids[current_idx] < target
        {
            current_idx += 1;
        }
        self.current_idx = Some(current_idx);
        Ok(())
    }

    fn current_docid(&self) -> Result<ItDocId> {
        Ok(self.current_idx.map_or_else(
            || ItDocId::NotStarted,
            |current_idx| {
                if current_idx < self.docids.len() {
                    ItDocId::Active(self.docids[current_idx])
                } else {
                    ItDocId::Exhausted
                }
            },
        ))
    }

    fn explain(&self) -> TreeNode {
        TreeNode::new("Test").with_attr("scoring", "true")
    }
}

impl ScoringDocIdIterator for TestScoringDocIdIterator {
    fn current_score(&self) -> Result<ItScore> {
        Ok(self.current_idx.map_or_else(
            || ItScore::NotStarted,
            |current_idx| {
                if current_idx < self.scores.len() {
                    ItScore::Active(self.scores[current_idx])
                } else {
                    ItScore::Exhausted
                }
            },
        ))
    }
}

pub struct TestDocIdIterator {
    docids: Vec<SegmentDocId>,
    current_idx: Option<usize>,
}

impl TestDocIdIterator {
    pub fn new(docids: Vec<SegmentDocId>) -> Self {
        Self {
            docids,
            current_idx: None,
        }
    }
}

impl DocIdIterator for TestDocIdIterator {
    fn advance(&mut self) -> Result<()> {
        self.current_idx = Some(self.current_idx.map_or(0, |v| v + 1));
        Ok(())
    }

    fn advance_to(&mut self, target: SegmentDocId) -> Result<()> {
        let mut current_idx = self.current_idx.map_or(0, |v| v);
        while current_idx < self.docids.len()
            && self.docids[current_idx] < target
        {
            current_idx += 1;
        }
        self.current_idx = Some(current_idx);
        Ok(())
    }

    fn current_docid(&self) -> Result<ItDocId> {
        Ok(self.current_idx.map_or_else(
            || ItDocId::NotStarted,
            |current_idx| {
                if current_idx < self.docids.len() {
                    ItDocId::Active(self.docids[current_idx])
                } else {
                    ItDocId::Exhausted
                }
            },
        ))
    }

    fn explain(&self) -> TreeNode {
        TreeNode::new("Test").with_attr("scoring", "false")
    }
}

pub fn collect(
    mut it: impl ScoringDocIdIterator,
) -> Result<Vec<(SegmentDocId, Score)>> {
    // make sure it is in not-started state before first advance
    assert!(it.current_docid()?.is_not_started());
    assert!(it.current_score()?.is_not_started());

    it.advance()?;

    collect_rest(it)
}

pub fn collect_rest(
    mut it: impl ScoringDocIdIterator,
) -> Result<Vec<(SegmentDocId, Score)>> {
    let mut res = Vec::new();

    while !it.current_docid()?.is_exhausted() {
        res.push((
            it.current_docid()?.expect()?,
            it.current_score()?.expect()?,
        ));
        it.advance()?;
    }

    // make sure it stays exhausted on further advances
    it.advance()?;

    assert!(it.current_docid()?.is_exhausted());
    assert!(it.current_score()?.is_exhausted());

    Ok(res)
}
