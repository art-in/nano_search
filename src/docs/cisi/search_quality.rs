use std::collections::HashSet;

use anyhow::Result;

use super::load_queries::load_queries;
use crate::model::{doc::DocId, engine::SearchEngine};

// TODO: abstract test queries and merge with docs source into model/
// (e.g. trait DataSource = get_docs + get_queries?)
// storing this code in cisi/ for now, since simplewiki/ does not have test queries
pub struct SearchQuality {
    pub queries_count: u64,
    pub precision_avg: f64,
    pub recall_avg: f64,
    pub precision_percs: inc_stats::Percentiles<f64>,
    pub recall_percs: inc_stats::Percentiles<f64>,
}

pub struct QuerySearchQuality {
    pub precision: f64,
    pub recall: f64,
}

pub fn search_and_calc_quality(
    engine: &dyn SearchEngine,
) -> Result<SearchQuality> {
    let queries = load_queries()?;

    let mut precision_sum: f64 = 0.0;
    let mut recall_sum: f64 = 0.0;

    let mut precision_percs = inc_stats::Percentiles::new();
    let mut recall_percs = inc_stats::Percentiles::new();

    for query in &queries {
        let found_docids = engine.search(&query.text, 10)?;

        let quality =
            calc_search_result_quality(&found_docids, &query.expected_docids);

        precision_sum += quality.precision;
        recall_sum += quality.recall;

        precision_percs.add(quality.precision);
        recall_percs.add(quality.recall);
    }

    let precision_avg = precision_sum / queries.len() as f64;
    let recall_avg = recall_sum / queries.len() as f64;

    Ok(SearchQuality {
        queries_count: queries.len() as u64,
        precision_avg,
        recall_avg,
        precision_percs,
        recall_percs,
    })
}

fn calc_search_result_quality(
    found_docids: &Vec<DocId>,
    relevant_docids: &HashSet<u64>,
) -> QuerySearchQuality {
    let mut found_relevant_docids_count = 0;
    for found_docid in found_docids {
        if relevant_docids.contains(found_docid) {
            found_relevant_docids_count += 1;
        }
    }

    let precision = {
        if found_docids.is_empty() {
            0.0
        } else {
            found_relevant_docids_count as f64 / found_docids.len() as f64
        }
    };

    let recall = {
        if relevant_docids.is_empty() {
            1.0
        } else {
            found_relevant_docids_count as f64 / relevant_docids.len() as f64
        }
    };

    QuerySearchQuality { precision, recall }
}
