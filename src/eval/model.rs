use std::collections::HashMap;

use anyhow::Result;

use crate::model::doc::ExternalDocId;

pub type QueryId = u64;
pub type Relevance = f64;

#[derive(Default, Clone)]
pub struct Query {
    pub id: QueryId,
    pub text: String,

    // IDs of documents considered relevant to this query, and thus expected to
    // be in search results
    pub relevant_docs: HashMap<ExternalDocId, Relevance>,
}

pub trait QueriesSource {
    fn queries(&self) -> Result<Box<dyn Iterator<Item = Result<Query>>>>;
}

// Search quality evaluation results summarized for entire query set
pub struct SearchQuality {
    pub queries_count: u64,
    pub search_limit: u64,
    pub precision_avg: f64,
    pub recall_avg: f64,
    pub ndcg_avg: f64,
    pub precisions: inc_stats::Percentiles<f64>,
    pub recalls: inc_stats::Percentiles<f64>,
}

// Search quality evaluation results for particular query
pub struct QuerySearchQuality {
    pub precision: f64,
    pub recall: f64,
    pub ndcg: f64,
}
