use anyhow::{Result, ensure};

use super::SegmentCollector;
use crate::engines::nano::search::iterators::ScoringDocIdIterator;

pub fn collect<T>(
    collector: &mut dyn SegmentCollector<SegmentOutput = T>,
    iterator: &mut dyn ScoringDocIdIterator,
) -> Result<()> {
    ensure!(
        iterator.current_docid()?.is_not_started(),
        "docid iterator should not be started initially"
    );

    iterator.advance()?;

    while !iterator.current_docid()?.is_exhausted() {
        let docid = iterator.current_docid()?.expect()?;

        if collector.requires_score() {
            let score = iterator.current_score()?.expect()?;
            collector.add_docid_and_score(docid, score)?;
        } else {
            collector.add_docid(docid)?;
        }

        iterator.advance()?;
    }

    Ok(())
}
