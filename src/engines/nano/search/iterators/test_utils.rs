/// This is a set of utilities to support unit testing of iterators.
use anyhow::{Result, ensure};

use super::exclude::ExcludingDocIdIterator;
use super::intersect::IntersectingDocIdIterator;
use super::model::{DocIdIterator, ItDocId, ItScore, ScoringDocIdIterator};
use super::union::UnionDocIdIterator;
use crate::engines::nano::index::model::SegmentDocId;
use crate::utils::TreeNode;

pub struct TestScoringDocIdIterator {
    docids: Vec<SegmentDocId>,
    scores: Vec<f64>,
    current_idx: Option<usize>,
}

impl TestScoringDocIdIterator {
    pub fn new(docids: Vec<SegmentDocId>, scores: Vec<f64>) -> Result<Self> {
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
        TreeNode::new("Test")
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
        TreeNode::new("Test")
    }
}

pub fn collect_from_it(
    mut it: impl ScoringDocIdIterator,
) -> Result<Vec<(SegmentDocId, f64)>> {
    let mut res = Vec::new();

    if it.current_docid()?.is_not_started() {
        it.advance()?;
    }

    while !it.current_docid()?.is_exhausted() {
        res.push((
            it.current_docid()?.expect_val()?,
            it.current_score()?.expect_val()?,
        ));
        it.advance()?;
    }

    Ok(res)
}

pub fn create_intersecting_it(
    docids_and_scores: Vec<(Vec<SegmentDocId>, Vec<f64>)>,
) -> Result<IntersectingDocIdIterator<'static>> {
    let mut input: Vec<Box<dyn ScoringDocIdIterator>> = Vec::new();
    for (docids, scores) in docids_and_scores {
        let it = Box::new(TestScoringDocIdIterator::new(docids, scores)?);
        input.push(it);
    }
    Ok(IntersectingDocIdIterator::new(input))
}

pub fn create_interecting_it_and_collect(
    docids_and_scores: Vec<(Vec<SegmentDocId>, Vec<f64>)>,
) -> Result<Vec<(SegmentDocId, f64)>> {
    collect_from_it(create_intersecting_it(docids_and_scores)?)
}

pub fn create_excluding_it(
    docids_and_scores: (Vec<SegmentDocId>, Vec<f64>),
    exclude_docids: Vec<SegmentDocId>,
) -> Result<ExcludingDocIdIterator<'static>> {
    let (docids, scores) = docids_and_scores;
    let input = Box::new(TestScoringDocIdIterator::new(docids, scores)?);
    let exclude = Box::new(TestDocIdIterator::new(exclude_docids));
    Ok(ExcludingDocIdIterator::new(input, exclude))
}

pub fn create_excluding_it_and_collect(
    docids_and_scores: (Vec<SegmentDocId>, Vec<f64>),
    exclude_docids: Vec<SegmentDocId>,
) -> Result<Vec<(SegmentDocId, f64)>> {
    collect_from_it(create_excluding_it(docids_and_scores, exclude_docids)?)
}

pub fn create_union_it(
    docids_and_scores: Vec<(Vec<SegmentDocId>, Vec<f64>)>,
) -> Result<UnionDocIdIterator<'static>> {
    let mut input: Vec<Box<dyn ScoringDocIdIterator>> = Vec::new();
    for (docids, scores) in docids_and_scores {
        let it = Box::new(TestScoringDocIdIterator::new(docids, scores)?);
        input.push(it);
    }
    Ok(UnionDocIdIterator::new(input))
}

pub fn create_union_it_and_collect(
    docids_and_scores: Vec<(Vec<SegmentDocId>, Vec<f64>)>,
) -> Result<Vec<(SegmentDocId, f64)>> {
    collect_from_it(create_union_it(docids_and_scores)?)
}
