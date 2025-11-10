use std::collections::HashMap;

use anyhow::{Result, bail};

use super::metrics::{precision, recall};
use super::model::{QuerySearchQuality, SearchQuality};
use crate::eval::metrics::ndcg;
use crate::eval::model::{Query, Relevance};
use crate::model::doc::DocId;
use crate::model::engine::SearchEngine;

pub fn evaluate_search_quality(
    queries: &mut dyn Iterator<Item = Query>,
    engine: &dyn SearchEngine,
    search_limit: u64,
) -> Result<SearchQuality> {
    let mut precision_sum: f64 = 0.0;
    let mut recall_sum: f64 = 0.0;
    let mut ndcg_sum: f64 = 0.0;

    let mut precisions = inc_stats::Percentiles::new();
    let mut recalls = inc_stats::Percentiles::new();

    let mut queries_count = 0;

    for query in queries {
        let found_docids = engine.search(&query.text, search_limit)?;

        let quality = evaluate_search_quality_for_query(
            &found_docids,
            &query.relevant_docs,
            search_limit,
        )?;

        precision_sum += quality.precision;
        recall_sum += quality.recall;
        ndcg_sum += quality.ndcg;

        precisions.add(quality.precision);
        recalls.add(quality.recall);

        queries_count += 1;
    }

    if queries_count == 0 {
        bail!("no queries to evaluate search quality");
    }

    let precision_avg = precision_sum / queries_count as f64;
    let recall_avg = recall_sum / queries_count as f64;
    let ndcg_avg = ndcg_sum / queries_count as f64;

    Ok(SearchQuality {
        queries_count,
        search_limit,
        precision_avg,
        recall_avg,
        ndcg_avg,
        precisions,
        recalls,
    })
}

pub fn evaluate_search_quality_for_query(
    found_docids: &[DocId],
    relevant_docs: &HashMap<DocId, Relevance>,
    search_limit: u64,
) -> Result<QuerySearchQuality> {
    let precision = precision(found_docids, relevant_docs);
    let recall = recall(found_docids, relevant_docs);
    let ndcg = ndcg(found_docids, relevant_docs, search_limit)?;

    Ok(QuerySearchQuality {
        precision,
        recall,
        ndcg,
    })
}
