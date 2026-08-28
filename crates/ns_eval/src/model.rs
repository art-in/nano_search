use std::collections::HashMap;

use ns_engine::{ExternalDocId, Score};

pub type QueryId = u64;

/// Search query with expected docs info for search quality evaluation.
#[derive(Default, Clone)]
pub struct Query {
    pub id: QueryId,
    pub text: String,

    /// IDs of documents considered relevant to this query, and thus expected
    /// to be in search results.
    pub relevant_docs: HashMap<ExternalDocId, Score>,
}

/// Search quality evaluation results summarized for entire query set.
#[derive(Debug, PartialEq)]
pub struct SearchQuality {
    pub queries_count: u64,
    pub search_limit: u64,

    pub ndcg_avg: f64,

    pub precision_avg: f64,
    pub precision_p50: f64,
    pub precision_p90: f64,
    pub precision_p100: f64,

    pub recall_avg: f64,
    pub recall_p50: f64,
    pub recall_p90: f64,
    pub recall_p100: f64,
}

/// Search quality evaluation results for one query.
pub struct QuerySearchQuality {
    pub ndcg: f64,
    pub precision: f64,
    pub recall: f64,
}
