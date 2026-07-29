use std::collections::HashMap;

use anyhow::{Context, Result, ensure};

use super::metrics::{precision, recall};
use super::model::{QuerySearchQuality, SearchQuality};
use crate::eval::metrics::ndcg;
use crate::eval::model::{Query, Relevance};
use crate::model::doc::ExternalDocId;
use crate::model::engine::SearchEngine;
use crate::utils::GetPercentile;

pub fn evaluate_search_quality(
    queries: &mut dyn Iterator<Item = Result<Query>>,
    engine: &dyn SearchEngine,
    search_limit: u64,
) -> Result<SearchQuality> {
    let mut ndcg_sum: f64 = 0.0;
    let mut precision_sum: f64 = 0.0;
    let mut recall_sum: f64 = 0.0;

    let mut precisions = inc_stats::Percentiles::new();
    let mut recalls = inc_stats::Percentiles::new();

    let mut queries_count = 0;

    for query in queries {
        let query = query.context("query should be valid")?;

        let query_text = strip_query_syntax(&query.text);

        let found_docids = engine
            .search(&query_text, search_limit)
            .with_context(|| format!("query: '{query_text}'"))?;

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

    ensure!(queries_count > 0, "queries source should not be empty");

    let ndcg_avg = ndcg_sum / queries_count as f64;
    let precision_avg = precision_sum / queries_count as f64;
    let recall_avg = recall_sum / queries_count as f64;

    Ok(SearchQuality {
        queries_count,
        search_limit,

        ndcg_avg,

        precision_avg,
        precision_p50: precisions.perc(0.5)?,
        precision_p90: precisions.perc(0.9)?,
        precision_p100: precisions.perc(1.0)?,

        recall_avg,
        recall_p50: recalls.perc(0.5)?,
        recall_p90: recalls.perc(0.9)?,
        recall_p100: recalls.perc(1.0)?,
    })
}

pub fn evaluate_search_quality_for_query(
    found_docids: &[ExternalDocId],
    relevant_docs: &HashMap<ExternalDocId, Relevance>,
    search_limit: u64,
) -> Result<QuerySearchQuality> {
    let ndcg = ndcg(found_docids, relevant_docs, search_limit)?;
    let precision = precision(found_docids, relevant_docs);
    let recall = recall(found_docids, relevant_docs);

    Ok(QuerySearchQuality {
        ndcg,
        precision,
        recall,
    })
}

// strip query syntax, because evaluation queries are arbitrary texts that may
// accidentally contain operators or grouping characters.
// e.g. "1) GUNS AND ROSES 2) BEATLES" contains the boolean operator AND and an
// unmatched parenthesis. it should be interpreted as ordinary text, not as
// query-language syntax. otherwise it will silently affect search results or
// fail with "invalid syntax" error
// TODO: pass "parsing_mode" arg to search() and explicitly disable syntax
// parsing in each engine, instead of rewriting queries
#[must_use]
fn strip_query_syntax(query: &str) -> String {
    query
        // parsers usually are case-sensitive, so lowercasing AND/OR/NOT/etc
        // operators disables them
        .to_lowercase()
        // strip special characters
        .replace(
            [
                '(', ')', '+', '-', '*', '?', '"', '\'', ':', '^', '~', '[',
                ']', '{', '}',
            ],
            "",
        )
}
