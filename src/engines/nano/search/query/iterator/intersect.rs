use anyhow::{Ok, Result, bail};

use super::model::{DocIdIterator, ItDocId, ItScore, ScoringDocIdIterator};
use crate::engines::nano::index::model::SegmentDocId;
use crate::utils::TreeNode;

/// Iterator over document IDs, that returns only IDs present in all `inputs`
pub struct IntersectingDocIdIterator<'a> {
    inputs: Vec<Box<dyn ScoringDocIdIterator + 'a>>,
    current_docid: ItDocId,
}

impl<'a> IntersectingDocIdIterator<'a> {
    pub fn new(inputs: Vec<Box<dyn ScoringDocIdIterator + 'a>>) -> Self {
        Self {
            inputs,
            current_docid: ItDocId::NotStarted,
        }
    }

    fn any_exhausted(&self) -> Result<bool> {
        for it in &self.inputs {
            if it.current_docid()?.is_exhausted() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn max_docid(&self) -> Result<SegmentDocId> {
        let mut max = SegmentDocId::MIN;
        for it in &self.inputs {
            max = max.max(it.current_docid()?.expect_docid()?);
        }
        Ok(max)
    }

    fn advance_internal(&mut self, target: Option<SegmentDocId>) -> Result<()> {
        if self.current_docid.is_exhausted() {
            return Ok(());
        }

        if self.inputs.is_empty() {
            self.current_docid = ItDocId::Exhausted;
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

        if self.any_exhausted()? {
            self.current_docid = ItDocId::Exhausted;
            return Ok(());
        }

        // TODO: use leader-based iteration approach: sort iterators by
        // length/cost (in some kind of query planner or right here in
        // new()), and use shortest one as a leader, advancing all other
        // iterators to current leader docid. this works better when
        // iterator sizes are much different and they have ability to
        // quickly skip gaps - term iterator should use skip lists

        loop {
            let candidate = self.max_docid()?;

            let mut matched = true;

            for it in &mut self.inputs {
                if it.current_docid()?.expect_docid()? < candidate {
                    it.advance_to(candidate)?;
                }

                if it.current_docid()?.is_exhausted() {
                    self.current_docid = ItDocId::Exhausted;
                    return Ok(());
                }

                if it.current_docid()?.expect_docid()? != candidate {
                    // docid is greater than candidate
                    matched = false;
                }
            }

            if matched {
                self.current_docid = ItDocId::Active(candidate);
                return Ok(());
            }
        }
    }
}

impl DocIdIterator for IntersectingDocIdIterator<'_> {
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
        let mut tree = TreeNode::new("Intersection");
        for input in &self.inputs {
            tree.add_child(input.explain());
        }
        tree
    }
}

impl ScoringDocIdIterator for IntersectingDocIdIterator<'_> {
    fn current_score(&self) -> Result<ItScore> {
        Ok(match self.current_docid {
            ItDocId::NotStarted => ItScore::NotStarted,
            ItDocId::Exhausted => ItScore::Exhausted,
            ItDocId::Active(_) => {
                // TODO: cache calculated score (?)
                let mut total_score = 0.0;

                for it in &self.inputs {
                    if let ItScore::Active(score) = it.current_score()? {
                        total_score += score;
                    } else {
                        bail!("all iterators should have score");
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
        let mut it = IntersectingDocIdIterator::new(vec![]);

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
            create_interecting_it_and_collect(vec![(
                vec![1, 2],     // docids
                vec![1.0, 2.0]  // scores
            )])?,
            &[(1, 1.0), (2, 2.0)]
        );

        Ok(())
    }

    #[test]
    fn test_multiple_inputs() -> Result<()> {
        assert_eq!(
            create_interecting_it_and_collect(vec![
                (vec![1, 2, 3], vec![1.0, 2.0, 3.0]), // 1st input
                (vec![2, 3, 4], vec![2.0, 3.0, 4.0])  // 2nd input
            ])?,
            &[(2, 4.0), (3, 6.0)] // intersection
        );
        Ok(())
    }

    #[test]
    fn test_advance_to() -> Result<()> {
        let mut it = create_intersecting_it(vec![
            (vec![1, 2, 3], vec![1.0, 2.0, 3.0]),
            (vec![2, 3, 4], vec![2.0, 3.0, 4.0]),
        ])?;

        it.advance_to(3)?;

        assert_eq!(it.current_docid()?, ItDocId::Active(3));
        assert_eq!(it.current_score()?, ItScore::Active(6.0));

        it.advance()?;

        assert_eq!(it.current_docid()?, ItDocId::Exhausted);
        assert_eq!(it.current_score()?, ItScore::Exhausted);

        Ok(())
    }

    #[test]
    fn test_no_intersecting_items() -> Result<()> {
        assert_eq!(
            create_interecting_it_and_collect(vec![
                (vec![1, 2, 3], vec![1.0, 2.0, 3.0]),
                (vec![4, 5, 6], vec![4.0, 5.0, 6.0])
            ])?,
            &[]
        );

        Ok(())
    }
}
