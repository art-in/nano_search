use anyhow::Result;

use super::model::{Collector, SegmentCollector};
use crate::engines::nano::index::model::{IndexSegment, SegmentDocId};

/// Collector that returns total count of matching documents.
pub struct CountCollector;

impl<'a> Collector<'a> for CountCollector {
    type SegmentCollector = CountSegmentCollector;
    type SegmentOutput = usize;
    type Output = usize;

    fn create_segment_collector(
        &self,
        _: &'a dyn IndexSegment,
    ) -> Result<Self::SegmentCollector> {
        Ok(CountSegmentCollector::default())
    }

    fn merge_segment_outputs(
        &self,
        outputs: Vec<Self::SegmentOutput>,
    ) -> Result<Self::Output> {
        Ok(outputs.into_iter().sum())
    }
}

#[derive(Default)]
pub struct CountSegmentCollector {
    count: usize,
}

impl SegmentCollector for CountSegmentCollector {
    type SegmentOutput = usize;

    fn requires_score(&self) -> bool {
        false
    }

    fn add_docid(&mut self, _: SegmentDocId) -> Result<()> {
        self.count += 1;
        Ok(())
    }

    fn extract_output(self) -> Result<Self::SegmentOutput> {
        Ok(self.count)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::bail;

    use super::*;

    // TODO: move to utils
    fn err<T>(input: Result<T>) -> Result<String> {
        match input {
            Ok(_) => bail!("should return error"),
            Err(message) => Ok(message.to_string()),
        }
    }

    #[test]
    fn test_segment_collector_score_passed() -> Result<()> {
        let mut segment_collector = CountSegmentCollector::default();

        assert!(!segment_collector.requires_score());
        assert_eq!(
            err(segment_collector.add_docid_and_score(0, 1.0))?,
            "score should not be passed"
        );

        Ok(())
    }

    #[test]
    fn test_segment_collector() -> Result<()> {
        {
            let segment_collector = CountSegmentCollector::default();
            assert_eq!(segment_collector.extract_output()?, 0);
        }
        {
            let mut segment_collector = CountSegmentCollector::default();
            segment_collector.add_docid(0)?;
            assert_eq!(segment_collector.extract_output()?, 1);
        }
        {
            let mut segment_collector = CountSegmentCollector::default();
            segment_collector.add_docid(0)?;
            segment_collector.add_docid(1)?;
            assert_eq!(segment_collector.extract_output()?, 2);
        }
        Ok(())
    }

    #[test]
    fn test_collector_merge() -> Result<()> {
        let collector = CountCollector;

        assert_eq!(collector.merge_segment_outputs(vec![])?, 0);
        assert_eq!(collector.merge_segment_outputs(vec![0])?, 0);
        assert_eq!(collector.merge_segment_outputs(vec![0, 1, 2, 3])?, 6);

        Ok(())
    }
}
