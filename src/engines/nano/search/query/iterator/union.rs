use anyhow::Result;

use super::model::{DocIdIterator, ItDocId, ItScore, ScoringDocIdIterator};
use crate::engines::nano::index::model::SegmentDocId;
use crate::utils::TreeNode;

/// Iterator over document IDs, that returns all IDs present in `inputs`
pub struct UnionDocIdIterator<'a> {
    inputs: Vec<Box<dyn ScoringDocIdIterator + 'a>>,
    current_docid: ItDocId,
}

impl<'a> UnionDocIdIterator<'a> {
    pub fn new(inputs: Vec<Box<dyn ScoringDocIdIterator + 'a>>) -> Self {
        Self {
            inputs,
            current_docid: ItDocId::NotStarted,
        }
    }

    fn advance_internal(&mut self, target: Option<SegmentDocId>) -> Result<()> {
        if self.current_docid.is_exhausted() {
            return Ok(());
        }

        for it in &mut self.inputs {
            if let Some(target) = target {
                it.advance_to(target)?;
            } else if self.current_docid.is_not_started()
                // advance previously matched iterators
                || it.current_docid()? == self.current_docid
            {
                it.advance()?;
            }
        }

        let mut candidate: Option<SegmentDocId> = None;

        // TODO: use min-heap for finding next candidate (?).
        // still hesitant, since it only matters with lots of input iterators.
        // e.g. tantivy used min-heap initially, but later removed it and now
        // uses alternative optimization paths. needs deeper investigation
        for it in &self.inputs {
            if let ItDocId::Active(docid) = it.current_docid()? {
                candidate = Some(candidate.map_or(docid, |c| c.min(docid)));
            }
        }

        if let Some(candidate) = candidate {
            self.current_docid = ItDocId::Active(candidate);
        } else {
            self.current_docid = ItDocId::Exhausted;
        }

        Ok(())
    }
}

impl DocIdIterator for UnionDocIdIterator<'_> {
    fn advance(&mut self) -> Result<()> {
        self.advance_internal(None)
    }

    fn advance_to(&mut self, target: SegmentDocId) -> Result<()> {
        self.advance_internal(Some(target))
    }

    fn current_docid(&self) -> Result<ItDocId> {
        Ok(self.current_docid)
    }

    fn explain(&self) -> TreeNode {
        let mut tree = TreeNode::new("Union");
        for input in &self.inputs {
            tree.add_child(input.explain());
        }
        tree
    }
}

impl ScoringDocIdIterator for UnionDocIdIterator<'_> {
    fn current_score(&self) -> Result<ItScore> {
        Ok(match self.current_docid {
            ItDocId::NotStarted => ItScore::NotStarted,
            ItDocId::Exhausted => ItScore::Exhausted,
            ItDocId::Active(current_docid) => {
                let mut total_score = 0.0;

                for it in &self.inputs {
                    if matches!(
                        it.current_docid()?,
                        ItDocId::Active(docid) if docid == current_docid)
                    {
                        total_score += it.current_score()?.expect_score()?;
                    }
                }

                ItScore::Active(total_score)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use super::*;

    #[test]
    fn test_empty_input() -> Result<()> {
        let mut it = UnionDocIdIterator::new(vec![]);

        assert_eq!(it.current_docid()?, ItDocId::NotStarted);
        assert_eq!(it.current_score()?, ItScore::NotStarted);

        it.advance()?;

        assert_eq!(it.current_docid()?, ItDocId::Exhausted);
        assert_eq!(it.current_score()?, ItScore::Exhausted);

        Ok(())
    }

    #[test]
    fn test_single_input() -> Result<()> {
        assert_eq!(
            create_union_it_and_collect(vec![
                (vec![1, 2], vec![1.0, 2.0]) // 1st input
            ])?,
            &[(1, 1.0), (2, 2.0)]
        );

        Ok(())
    }

    #[test]
    fn test_multiple_inputs() -> Result<()> {
        assert_eq!(
            create_union_it_and_collect(vec![
                (vec![1, 2], vec![1.0, 2.0]), // 1st input
                (vec![2, 3], vec![2.0, 3.0]), // 2nd input
                (vec![2, 4], vec![2.0, 4.0]), // 3rd input
            ])?,
            &[(1, 1.0), (2, 6.0), (3, 3.0), (4, 4.0)]
        );
        Ok(())
    }

    #[test]
    fn test_advance_to() -> Result<()> {
        let mut it = create_union_it(vec![
            (vec![1, 2, 3], vec![1.0, 2.0, 3.0]),
            (vec![2, 3, 4], vec![2.0, 3.0, 4.0]),
        ])?;

        it.advance_to(3)?;

        assert_eq!(it.current_docid()?, ItDocId::Active(3));
        assert_eq!(it.current_score()?, ItScore::Active(6.0));

        it.advance()?;

        assert_eq!(it.current_docid()?, ItDocId::Active(4));
        assert_eq!(it.current_score()?, ItScore::Active(4.0));

        it.advance()?;

        assert_eq!(it.current_docid()?, ItDocId::Exhausted);
        assert_eq!(it.current_score()?, ItScore::Exhausted);

        Ok(())
    }

    #[test]
    fn test_one_input_exhausts_early() -> Result<()> {
        assert_eq!(
            create_union_it_and_collect(vec![
                (vec![1], vec![1.0]),
                (vec![1, 2, 3], vec![1.0, 2.0, 3.0]),
            ])?,
            &[(1, 2.0), (2, 2.0), (3, 3.0)]
        );
        Ok(())
    }

    #[test]
    fn test_empty_input_among_others() -> Result<()> {
        assert_eq!(
            create_union_it_and_collect(vec![
                (vec![], vec![]),
                (vec![1, 2], vec![1.0, 2.0]),
            ])?,
            &[(1, 1.0), (2, 2.0)]
        );
        Ok(())
    }
}
