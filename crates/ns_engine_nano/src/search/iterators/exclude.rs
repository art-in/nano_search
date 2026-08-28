use anyhow::Result;

use super::model::{DocIdIterator, ItDocId, ItScore, ScoringDocIdIterator};
use crate::index::model::SegmentDocId;
use crate::utils::TreeNode;

/// Iterator over document IDs, that filters `include` iterator by skipping any
/// IDs also present in `exclude` iterator.
///
/// The excluding has no impact on scoring.
///
/// In Lucene it is called `EqExclScorer`, in Tantivy - `Exclude`.
pub struct ExcludingDocIdIterator<'a> {
    include: Box<dyn ScoringDocIdIterator + 'a>,
    exclude: Box<dyn DocIdIterator + 'a>,
    current_docid: ItDocId,
}

impl<'a> ExcludingDocIdIterator<'a> {
    pub fn new(
        include: Box<dyn ScoringDocIdIterator + 'a>,
        exclude: Box<dyn DocIdIterator + 'a>,
    ) -> Self {
        Self {
            include,
            exclude,
            current_docid: ItDocId::NotStarted,
        }
    }

    fn advance_internal(&mut self, target: Option<SegmentDocId>) -> Result<()> {
        if self.current_docid.is_exhausted() {
            return Ok(());
        }

        if let Some(target) = target {
            self.include.advance_to(target)?;
            self.exclude.advance_to(target)?;
        } else {
            if self.include.current_docid()?.is_not_started()
                // advance previously matched iterator
                || self.include.current_docid()? == self.current_docid
            {
                self.include.advance()?;
            }
            if self.exclude.current_docid()?.is_not_started() {
                self.exclude.advance()?;
            }
        }

        if self.include.current_docid()?.is_exhausted() {
            self.current_docid = ItDocId::Exhausted;
            return Ok(());
        }

        loop {
            let candidate = self.include.current_docid()?.expect()?;

            if matches!(self.exclude.current_docid()?, ItDocId::Active(d) if d < candidate)
            {
                self.exclude.advance_to(candidate)?;
            }

            if matches!(self.exclude.current_docid()?, ItDocId::Active(d) if d == candidate)
            {
                // candidate is excluded, keep advancing
                self.include.advance()?;
                if self.include.current_docid()?.is_exhausted() {
                    self.current_docid = ItDocId::Exhausted;
                    return Ok(());
                }
            } else {
                self.current_docid = ItDocId::Active(candidate);
                return Ok(());
            }
        }
    }
}

impl DocIdIterator for ExcludingDocIdIterator<'_> {
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
        TreeNode::new("Exclusion")
            .with_keyed_child("include", self.include.explain())
            .with_keyed_child("exclude", self.exclude.explain())
    }
}

impl ScoringDocIdIterator for ExcludingDocIdIterator<'_> {
    fn current_score(&self) -> Result<ItScore> {
        self.include.current_score()
    }
}

#[cfg(test)]
mod tests {
    use ns_engine::Score;

    use super::super::test_utils::*;
    use super::*;

    pub fn create_it(
        docids_and_scores: (Vec<SegmentDocId>, Vec<Score>),
        exclude_docids: Vec<SegmentDocId>,
    ) -> Result<ExcludingDocIdIterator<'static>> {
        let (docids, scores) = docids_and_scores;
        let input = Box::new(TestScoringDocIdIterator::new(docids, scores)?);
        let exclude = Box::new(TestDocIdIterator::new(exclude_docids));
        Ok(ExcludingDocIdIterator::new(input, exclude))
    }

    #[test]
    fn test_empty_input() -> Result<()> {
        let it = create_it((vec![], vec![]), vec![1, 2])?;
        assert!(collect(it)?.is_empty());
        Ok(())
    }

    #[test]
    fn test_no_exclusions() -> Result<()> {
        assert_eq!(
            collect(create_it((vec![1, 2, 3], vec![1.0, 2.0, 3.0]), vec![])?)?,
            &[(1, 1.0), (2, 2.0), (3, 3.0)]
        );
        Ok(())
    }

    #[test]
    fn test_exclude_some() -> Result<()> {
        assert_eq!(
            collect(create_it(
                (vec![1, 2, 3, 4], vec![1.0, 2.0, 3.0, 4.0]),
                vec![2, 3]
            )?)?,
            &[(1, 1.0), (4, 4.0)]
        );
        Ok(())
    }

    #[test]
    fn test_exclude_all() -> Result<()> {
        assert_eq!(
            collect(create_it(
                (vec![1, 2, 3], vec![1.0, 2.0, 3.0]),
                vec![1, 2, 3]
            )?)?,
            &[]
        );
        Ok(())
    }

    #[test]
    fn test_exclude_none_matching() -> Result<()> {
        assert_eq!(
            collect(create_it(
                (vec![1, 2, 3], vec![1.0, 2.0, 3.0]),
                vec![10, 20]
            )?)?,
            &[(1, 1.0), (2, 2.0), (3, 3.0)]
        );
        Ok(())
    }

    #[test]
    fn test_exclude_first_and_last() -> Result<()> {
        assert_eq!(
            collect(create_it(
                (vec![1, 2, 3], vec![1.0, 2.0, 3.0]),
                vec![1, 3]
            )?)?,
            &[(2, 2.0)]
        );
        Ok(())
    }

    #[test]
    fn test_consecutive_exclusions_then_exhaustion() -> Result<()> {
        assert_eq!(
            collect(create_it(
                (vec![1, 2, 3], vec![1.0, 2.0, 3.0]),
                vec![2, 3]
            )?)?,
            &[(1, 1.0)]
        );
        Ok(())
    }

    #[test]
    fn test_exclude_iterator_extends_past_include_iterator() -> Result<()> {
        assert_eq!(
            collect(create_it(
                (vec![1, 2], vec![1.0, 2.0]),
                vec![1, 2, 3, 4, 5]
            )?)?,
            &[]
        );
        Ok(())
    }

    #[test]
    fn test_exclude_docid_before_first_include_docid() -> Result<()> {
        assert_eq!(
            collect(create_it(
                (vec![5, 6, 7], vec![5.0, 6.0, 7.0]),
                vec![1, 2, 3, 4, 5]
            )?)?,
            &[(6, 6.0), (7, 7.0)]
        );
        Ok(())
    }

    #[test]
    fn test_exclude_iterator_sparser_than_include_iterator() -> Result<()> {
        assert_eq!(
            collect(create_it(
                (
                    vec![1, 2, 3, 4, 5, 6, 7],
                    vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]
                ),
                vec![5]
            )?)?,
            &[(1, 1.0), (2, 2.0), (3, 3.0), (4, 4.0), (6, 6.0), (7, 7.0)]
        );
        Ok(())
    }

    #[test]
    fn test_advance_to() -> Result<()> {
        let mut it =
            create_it((vec![1, 2, 3, 4], vec![1.0, 2.0, 3.0, 4.0]), vec![2])?;

        it.advance_to(2)?;

        assert_eq!(collect_rest(it)?, &[(3, 3.0), (4, 4.0)]);

        Ok(())
    }
}
