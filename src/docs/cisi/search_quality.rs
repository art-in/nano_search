use super::query::get_queries;
use crate::model::{doc::DocId, engine::SearchEngine};
use std::collections::HashSet;

// TODO: abstract test queries and merge with docs source into model/
// (e.g. trait DataSource = get_docs + get_queries?)
// storing this code in cisi/ for now, since simplewiki/ does not have test queries
pub struct SearchQuality {
    pub precision: f64,
    pub recall: f64,
}

pub fn get_search_quality(engine: &dyn SearchEngine) -> SearchQuality {
    let queries = get_queries();

    let mut precision_sum: f64 = 0.0;
    let mut recall_sum: f64 = 0.0;
    let mut queries_count = 0;

    for query in &queries {
        // skip long queries, since tantivy panics on parsing long queries
        if query.text.len() > 100 {
            continue;
        }

        let found_docids = engine.search(&query.text, 5);

        let quality =
            calc_search_result_quality(&found_docids, &query.expected_docids);

        precision_sum += quality.precision;
        recall_sum += quality.recall;
        queries_count += 1;
    }

    let precision_avg = precision_sum / queries_count as f64;
    let recall_avg = recall_sum / queries_count as f64;

    SearchQuality {
        precision: precision_avg,
        recall: recall_avg,
    }
}

fn calc_search_result_quality(
    found_docids: &Vec<DocId>,
    relevant_docids: &HashSet<u64>,
) -> SearchQuality {
    let mut found_relevant_docids_count = 0;
    for found_docid in found_docids {
        if relevant_docids.contains(found_docid) {
            found_relevant_docids_count += 1;
        }
    }

    let precision =
        found_relevant_docids_count as f64 / found_docids.len() as f64;
    let recall =
        found_relevant_docids_count as f64 / relevant_docids.len() as f64;

    SearchQuality { precision, recall }
}
