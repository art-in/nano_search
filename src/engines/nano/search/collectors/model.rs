use anyhow::{Result, bail};

use crate::engines::nano::index::model::{IndexSegment, SegmentDocId};

/// Collector aggregates partial search results from multiple index segments
/// into single index-wide search result.
///
/// This trait abstracts any possible way to gather and form search result from
/// series of matching documents, a.k.a. candidates.
///
/// For example:
/// - gathering 10 best documents
/// - counting total number of candidates
///
/// Splitting collector into index-wide part `Collector` and segment-specific
/// parts `SegmentCollector` reflects the index structure itself. Binding them
/// with associated types keeps relationship obvious and type-aligned.
pub trait Collector<'a> {
    type SegmentCollector: SegmentCollector<SegmentOutput = Self::SegmentOutput>;
    type SegmentOutput;
    type Output;

    fn create_segment_collector(
        &self,
        segment: &'a dyn IndexSegment,
    ) -> Result<Self::SegmentCollector>;

    fn merge_segment_outputs(
        &self,
        outputs: Vec<Self::SegmentOutput>,
    ) -> Result<Self::Output>;
}

/// Segment collector forms partial search result for concrete segment.
///
/// The caller is expected to pass all candidates through the collector.
/// In the process it will gather all data, required for this implementation.
///
/// It doesn't drive doc iterator inside of it, it expects document data to be
/// passed to it instead. This approach allows us to use multiple collectors
/// simultaneously for the same doc iteration (e.g. when gathering both top
/// results and stats for faceted queries).
pub trait SegmentCollector {
    type SegmentOutput;

    /// Indicates whether score should be passed with [`add_docid_and_score`] or
    /// omitted with [`add_docid`]. Calling side should respect this flag and
    /// choose appropriate adding method, otherwise error will be returned.
    // impementation note:
    // - we wouldn't need this if collector would drive the iterator and take
    //   [`ScoringDocIdIterator::current_score()`] or not inside of it, but this
    //   breaks multiple-collectors-per-iteration scenario we want to support
    // - also, this rules could be encoded into types, to avoid any runtime
    //   checking and errors, but that would make lots of typing bloat
    fn requires_score(&self) -> bool;

    fn add_docid(&mut self, _docid: SegmentDocId) -> Result<()> {
        bail!("score should be passed");
    }
    fn add_docid_and_score(
        &mut self,
        _docid: SegmentDocId,
        _score: f64,
    ) -> Result<()> {
        bail!("score should not be passed");
    }

    fn extract_output(self) -> Result<Self::SegmentOutput>;
}
