use anyhow::Result;

use super::collectors::{SegmentCollector, collect};
use super::plan::plan_query_for_segment;
use super::query::{Lexer, Parser, normalize};
use crate::engines::nano::index::model::Index;
use crate::engines::nano::search::collectors::{Collector, TopCollector};
use crate::model::ExternalDocId;

pub fn search(
    query: &str,
    index: &dyn Index,
    limit: u64,
) -> Result<Vec<ExternalDocId>> {
    let tokens = Lexer::new(query);
    let query_ast = Parser::new(tokens).parse()?;
    let query_ast = normalize(&query_ast)?;

    let Some(query_ast) = query_ast else {
        return Ok(vec![]);
    };

    let collector = TopCollector::new(limit as usize);

    let mut segment_outputs = Vec::new();

    for segment in index.get_segments() {
        let mut it = plan_query_for_segment(&query_ast, segment)?;
        let mut col = collector.create_segment_collector(segment)?;

        collect(&mut col, it.as_mut())?;
        let output = col.extract_output()?;

        segment_outputs.push(output);
    }

    let output = collector.merge_segment_outputs(segment_outputs)?;

    Ok(output)
}
