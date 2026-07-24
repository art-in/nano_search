use std::collections::BinaryHeap;

use anyhow::Result;

use super::model::{Collector, SegmentCollector};
use crate::engines::nano::index::model::{IndexSegment, SegmentDocId};
use crate::engines::nano::search::model::DocCandidate;
use crate::model::doc::ExternalDocId;

/// Collector that returns top-K documents with highest scores.
pub struct TopCollector {
    doc_count: usize,
}

impl TopCollector {
    pub const fn new(doc_count: usize) -> Self {
        Self { doc_count }
    }
}

impl<'a> Collector<'a> for TopCollector {
    type SegmentCollector = TopSegmentCollector<'a>;
    type SegmentOutput = Vec<DocCandidate<ExternalDocId>>;
    type Output = Vec<ExternalDocId>;

    fn create_segment_collector(
        &self,
        segment: &'a dyn IndexSegment,
    ) -> Result<Self::SegmentCollector> {
        Ok(TopSegmentCollector::new(segment, self.doc_count))
    }

    fn merge_segment_outputs(
        &self,
        outputs: Vec<Self::SegmentOutput>,
    ) -> Result<Self::Output> {
        let mut min_heap: BinaryHeap<DocCandidate<ExternalDocId>> =
            BinaryHeap::new();

        for output in outputs {
            for candidate in output {
                if min_heap.len() < self.doc_count {
                    min_heap.push(candidate);
                } else if matches!(min_heap.peek(), Some(min) if candidate < *min)
                {
                    min_heap.pop();
                    min_heap.push(candidate);
                }
            }
        }

        let mut res: Vec<ExternalDocId> = Vec::with_capacity(self.doc_count);

        while let Some(candidate) = min_heap.pop() {
            res.push(candidate.id);
        }

        res.reverse();

        Ok(res)
    }
}

pub struct TopSegmentCollector<'a> {
    segment: &'a dyn IndexSegment,
    doc_count: usize,
    min_heap: BinaryHeap<DocCandidate<SegmentDocId>>,
}

impl<'a> TopSegmentCollector<'a> {
    pub fn new(segment: &'a dyn IndexSegment, doc_count: usize) -> Self {
        Self {
            segment,
            doc_count,
            min_heap: BinaryHeap::new(),
        }
    }
}

impl SegmentCollector for TopSegmentCollector<'_> {
    type SegmentOutput = Vec<DocCandidate<ExternalDocId>>;

    fn requires_score(&self) -> bool {
        true
    }

    fn add_docid_and_score(
        &mut self,
        docid: SegmentDocId,
        score: f64,
    ) -> Result<()> {
        let candidate = DocCandidate {
            id: docid,
            relevance: score,
        };

        if self.min_heap.len() < self.doc_count {
            self.min_heap.push(candidate);
        } else if matches!(self.min_heap.peek(), Some(min) if candidate < *min)
        {
            self.min_heap.pop();
            self.min_heap.push(candidate);
        }

        Ok(())
    }

    fn extract_output(mut self) -> Result<Self::SegmentOutput> {
        let mut res = Vec::with_capacity(self.doc_count);

        while let Some(item) = self.min_heap.pop() {
            res.push(DocCandidate {
                id: self.segment.get_stored_doc(item.id)?.docid,
                relevance: item.relevance,
            });
        }

        res.reverse();

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::bail;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::engines::nano::index::MemoryIndex;
    use crate::engines::nano::index::model::StoredDoc;
    use crate::model::doc::ExternalDocId;

    // TODO: move to utils
    fn err<T>(input: Result<T>) -> Result<String> {
        match input {
            Ok(_) => bail!("should return error"),
            Err(message) => Ok(message.to_string()),
        }
    }

    fn create_segment(doc_count: usize) -> Box<dyn IndexSegment> {
        let mut segment = MemoryIndex::default();

        for i in 0..doc_count {
            segment.docs.push(StoredDoc {
                docid: i as ExternalDocId,
            });
        }

        Box::new(segment)
    }

    #[test]
    fn test_segment_collector() -> Result<()> {
        let segment = create_segment(3);
        let mut segment_collector =
            TopSegmentCollector::new(segment.as_ref(), 3);

        segment_collector.add_docid_and_score(1, 1.0)?;
        segment_collector.add_docid_and_score(0, 0.0)?;
        segment_collector.add_docid_and_score(2, 2.0)?;

        assert_eq!(
            segment_collector.extract_output()?,
            &[
                DocCandidate::new(2, 2.0),
                DocCandidate::new(1, 1.0),
                DocCandidate::new(0, 0.0)
            ]
        );

        Ok(())
    }

    #[test]
    fn test_segment_collector_requested_more_than_exists() -> Result<()> {
        let segment = create_segment(3);
        let mut segment_collector =
            TopSegmentCollector::new(segment.as_ref(), 4);

        segment_collector.add_docid_and_score(1, 1.0)?;
        segment_collector.add_docid_and_score(2, 2.0)?;
        segment_collector.add_docid_and_score(0, 0.0)?;

        assert_eq!(
            segment_collector.extract_output()?,
            &[
                DocCandidate::new(2, 2.0),
                DocCandidate::new(1, 1.0),
                DocCandidate::new(0, 0.0)
            ]
        );

        Ok(())
    }

    #[test]
    fn test_segment_collector_requested_less_than_exists() -> Result<()> {
        let segment = create_segment(3);
        let mut segment_collector =
            TopSegmentCollector::new(segment.as_ref(), 2);

        segment_collector.add_docid_and_score(0, 0.0)?;
        segment_collector.add_docid_and_score(2, 2.0)?;
        segment_collector.add_docid_and_score(1, 1.0)?;

        assert_eq!(
            segment_collector.extract_output()?,
            &[DocCandidate::new(2, 2.0), DocCandidate::new(1, 1.0),]
        );

        Ok(())
    }

    #[test]
    fn test_segment_collector_requested_zero() -> Result<()> {
        let segment = create_segment(2);
        let mut segment_collector =
            TopSegmentCollector::new(segment.as_ref(), 0);

        segment_collector.add_docid_and_score(0, 0.0)?;
        segment_collector.add_docid_and_score(1, 1.0)?;

        assert_eq!(segment_collector.extract_output()?, &[]);

        Ok(())
    }

    #[test]
    fn test_segment_collector_score_not_passed() -> Result<()> {
        let segment = create_segment(1);
        let mut segment_collector =
            TopSegmentCollector::new(segment.as_ref(), 1);

        assert!(segment_collector.requires_score());
        assert_eq!(
            err(segment_collector.add_docid(0))?,
            "score should be passed"
        );

        Ok(())
    }

    #[test]
    fn test_segment_collector_same_scores() -> Result<()> {
        let segment = create_segment(4);
        let mut segment_collector =
            TopSegmentCollector::new(segment.as_ref(), 3);

        segment_collector.add_docid_and_score(3, 10.0)?;
        segment_collector.add_docid_and_score(1, 10.0)?;
        segment_collector.add_docid_and_score(0, 10.0)?;
        segment_collector.add_docid_and_score(2, 10.0)?;

        // candidates with same score should be sorted by ID
        assert_eq!(
            segment_collector.extract_output()?,
            &[
                DocCandidate::new(0, 10.0),
                DocCandidate::new(1, 10.0),
                DocCandidate::new(2, 10.0),
            ]
        );

        Ok(())
    }

    #[test]
    fn test_collector_merge_single_output() -> Result<()> {
        let output = vec![DocCandidate::new(2, 2.0), DocCandidate::new(1, 1.0)];

        assert_eq!(
            TopCollector::new(2).merge_segment_outputs(vec![output])?,
            &[2, 1]
        );

        Ok(())
    }

    #[test]
    fn test_collector_merge_multiple_outputs() -> Result<()> {
        let output_a =
            vec![DocCandidate::new(3, 3.0), DocCandidate::new(0, 0.0)];
        let output_b =
            vec![DocCandidate::new(2, 1.0), DocCandidate::new(1, 5.0)];

        assert_eq!(
            TopCollector::new(3)
                .merge_segment_outputs(vec![output_a, output_b])?,
            &[1, 3, 2]
        );

        Ok(())
    }

    #[test]
    fn test_collector_merge_requested_zero() -> Result<()> {
        let output = vec![DocCandidate::new(0, 0.0)];

        assert!(
            TopCollector::new(0)
                .merge_segment_outputs(vec![output])?
                .is_empty()
        );

        Ok(())
    }

    #[test]
    fn test_collector_merge_no_outputs() -> Result<()> {
        assert!(
            TopCollector::new(5)
                .merge_segment_outputs(vec![])?
                .is_empty()
        );
        Ok(())
    }

    #[test]
    fn test_collector_merge_requested_more_than_in_outputs() -> Result<()> {
        let output = vec![DocCandidate::new(0, 0.0)];

        assert_eq!(
            TopCollector::new(5).merge_segment_outputs(vec![output])?,
            &[0]
        );

        Ok(())
    }

    #[test]
    fn test_collector_merge_same_scores() -> Result<()> {
        let output_a =
            vec![DocCandidate::new(1, 10.0), DocCandidate::new(3, 10.0)];
        let output_b =
            vec![DocCandidate::new(0, 10.0), DocCandidate::new(2, 10.0)];

        // candidates with same score should be sorted by ID
        assert_eq!(
            TopCollector::new(3)
                .merge_segment_outputs(vec![output_a, output_b])?,
            &[0, 1, 2]
        );
        Ok(())
    }
}
