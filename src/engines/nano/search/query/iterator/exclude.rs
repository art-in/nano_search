use anyhow::Result;

use super::model::{DocIdIterator, ItDocId, ItScore, ScoringDocIdIterator};
use crate::engines::nano::index::model::SegmentDocId;

/// Iterator over document IDs, that filters `input` iterator by removing any
/// IDs also present in `exclude` iterator.
///
/// The excluding has no impact on scoring.
///
/// It's called `ReqExclScorer` in Lucene and `Exclude` in Tantivy.
pub struct ExcludingDocIdIterator<'a> {
    input: Box<dyn ScoringDocIdIterator + 'a>,
    exclude: Box<dyn DocIdIterator + 'a>,
    current_docid: ItDocId,
}

impl<'a> ExcludingDocIdIterator<'a> {
    pub fn new(
        input: Box<dyn ScoringDocIdIterator + 'a>,
        exclude: Box<dyn DocIdIterator + 'a>,
    ) -> Self {
        Self {
            input,
            exclude,
            current_docid: ItDocId::NotStarted,
        }
    }

    fn advance_internal(&mut self, target: Option<SegmentDocId>) -> Result<()> {
        if self.current_docid.is_exhausted() {
            return Ok(());
        }

        if let Some(target) = target {
            self.input.advance_to(target)?;
            self.exclude.advance_to(target)?;
        } else {
            if self.input.current_docid()?.is_not_started()
                // advance previously matched iterator
                || self.input.current_docid()? == self.current_docid
            {
                self.input.advance()?;
            }
            if self.exclude.current_docid()?.is_not_started() {
                self.exclude.advance()?;
            }
        }

        if self.input.current_docid()?.is_exhausted() {
            self.current_docid = ItDocId::Exhausted;
            return Ok(());
        }

        loop {
            let candidate = self.input.current_docid()?.expect_docid()?;

            if matches!(self.exclude.current_docid()?, ItDocId::Active(d) if d < candidate)
            {
                self.exclude.advance_to(candidate)?;
            }

            if matches!(self.exclude.current_docid()?, ItDocId::Active(d) if d == candidate)
            {
                // candidate is excluded, keep advancing input
                self.input.advance()?;
                if self.input.current_docid()?.is_exhausted() {
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
}

impl ScoringDocIdIterator for ExcludingDocIdIterator<'_> {
    fn current_score(&self) -> Result<ItScore> {
        self.input.current_score()
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use super::*;

    #[test]
    fn test_empty_input() -> Result<()> {
        assert_eq!(
            create_excluding_it_and_collect((vec![], vec![]), vec![1, 2])?,
            &[]
        );
        Ok(())
    }

    #[test]
    fn test_not_started_before_first_advance() -> Result<()> {
        let it = create_excluding_it((vec![1, 2], vec![1.0, 2.0]), vec![])?;

        assert_eq!(it.current_docid()?, ItDocId::NotStarted);
        assert_eq!(it.current_score()?, ItScore::NotStarted);

        Ok(())
    }

    #[test]
    fn test_advance_after_exhaustion_is_noop() -> Result<()> {
        let mut it = create_excluding_it((vec![1], vec![1.0]), vec![])?;

        it.advance()?;
        assert_eq!(it.current_docid()?, ItDocId::Active(1));

        it.advance()?;
        assert_eq!(it.current_docid()?, ItDocId::Exhausted);

        it.advance()?;
        assert_eq!(it.current_docid()?, ItDocId::Exhausted);

        Ok(())
    }

    #[test]
    fn test_no_exclusions() -> Result<()> {
        assert_eq!(
            create_excluding_it_and_collect(
                (vec![1, 2, 3], vec![1.0, 2.0, 3.0]),
                vec![]
            )?,
            &[(1, 1.0), (2, 2.0), (3, 3.0)]
        );
        Ok(())
    }

    #[test]
    fn test_exclude_some() -> Result<()> {
        assert_eq!(
            create_excluding_it_and_collect(
                (vec![1, 2, 3, 4], vec![1.0, 2.0, 3.0, 4.0]),
                vec![2, 3]
            )?,
            &[(1, 1.0), (4, 4.0)]
        );
        Ok(())
    }

    #[test]
    fn test_exclude_all() -> Result<()> {
        assert_eq!(
            create_excluding_it_and_collect(
                (vec![1, 2, 3], vec![1.0, 2.0, 3.0]),
                vec![1, 2, 3]
            )?,
            &[]
        );
        Ok(())
    }

    #[test]
    fn test_exclude_none_matching() -> Result<()> {
        assert_eq!(
            create_excluding_it_and_collect(
                (vec![1, 2, 3], vec![1.0, 2.0, 3.0]),
                vec![10, 20]
            )?,
            &[(1, 1.0), (2, 2.0), (3, 3.0)]
        );
        Ok(())
    }

    #[test]
    fn test_exclude_first_and_last() -> Result<()> {
        assert_eq!(
            create_excluding_it_and_collect(
                (vec![1, 2, 3], vec![1.0, 2.0, 3.0]),
                vec![1, 3]
            )?,
            &[(2, 2.0)]
        );
        Ok(())
    }

    #[test]
    fn test_consecutive_exclusions_then_exhaustion() -> Result<()> {
        assert_eq!(
            create_excluding_it_and_collect(
                (vec![1, 2, 3], vec![1.0, 2.0, 3.0]),
                vec![2, 3]
            )?,
            &[(1, 1.0)]
        );
        Ok(())
    }

    #[test]
    fn test_exclude_iterator_extends_past_input() -> Result<()> {
        assert_eq!(
            create_excluding_it_and_collect(
                (vec![1, 2], vec![1.0, 2.0]),
                vec![1, 2, 3, 4, 5]
            )?,
            &[]
        );
        Ok(())
    }

    #[test]
    fn test_exclude_docid_before_first_input_docid() -> Result<()> {
        assert_eq!(
            create_excluding_it_and_collect(
                (vec![5, 6, 7], vec![5.0, 6.0, 7.0]),
                vec![1, 2, 3, 4, 5]
            )?,
            &[(6, 6.0), (7, 7.0)]
        );
        Ok(())
    }

    #[test]
    fn test_exclude_iterator_sparser_than_input() -> Result<()> {
        assert_eq!(
            create_excluding_it_and_collect(
                (
                    vec![1, 2, 3, 4, 5, 6, 7],
                    vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]
                ),
                vec![5]
            )?,
            &[(1, 1.0), (2, 2.0), (3, 3.0), (4, 4.0), (6, 6.0), (7, 7.0)]
        );
        Ok(())
    }

    #[test]
    fn test_advance_to_skips_excluded_target() -> Result<()> {
        let mut it = create_excluding_it(
            (vec![1, 2, 3, 4], vec![1.0, 2.0, 3.0, 4.0]),
            vec![2],
        )?;

        it.advance_to(2)?;

        assert_eq!(it.current_docid()?, ItDocId::Active(3));
        assert_eq!(it.current_score()?, ItScore::Active(3.0));

        Ok(())
    }
}
