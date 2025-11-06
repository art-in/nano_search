use std::collections::HashSet;

use anyhow::{Result, bail};

use super::model::{QuerySearchQuality, SearchQuality};
use crate::eval::model::Query;
use crate::model::doc::DocId;
use crate::model::engine::SearchEngine;

pub fn evaluate_search_quality(
    queries: &mut dyn Iterator<Item = Query>,
    engine: &dyn SearchEngine,
    search_limit: u64,
) -> Result<SearchQuality> {
    let mut precision_sum: f64 = 0.0;
    let mut recall_sum: f64 = 0.0;

    let mut precisions = inc_stats::Percentiles::new();
    let mut recalls = inc_stats::Percentiles::new();

    let mut queries_count: usize = 0;

    for query in queries {
        let found_docids = engine.search(&query.text, search_limit)?;

        let quality = evaluate_search_quality_for_query(
            &found_docids,
            &query.relevant_docids,
        );

        precision_sum += quality.precision;
        recall_sum += quality.recall;

        precisions.add(quality.precision);
        recalls.add(quality.recall);

        queries_count += 1;
    }

    if queries_count == 0 {
        bail!("no queries to evaluate search quality");
    }

    let precision_avg = precision_sum / queries_count as f64;
    let recall_avg = recall_sum / queries_count as f64;

    Ok(SearchQuality {
        queries_count: queries_count as u64,
        precision_avg,
        recall_avg,
        precisions,
        recalls,
    })
}

pub fn evaluate_search_quality_for_query(
    found_docids: &[DocId],
    relevant_docids: &HashSet<DocId>,
) -> QuerySearchQuality {
    let found_relevant_docids_count: usize = found_docids
        .iter()
        .filter(|docid| relevant_docids.contains(*docid))
        .count();

    let precision = if found_docids.is_empty() {
        if relevant_docids.is_empty() { 1.0 } else { 0.0 }
    } else {
        found_relevant_docids_count as f64 / found_docids.len() as f64
    };

    let recall = if relevant_docids.is_empty() {
        1.0
    } else {
        found_relevant_docids_count as f64 / relevant_docids.len() as f64
    };

    QuerySearchQuality { precision, recall }
}
