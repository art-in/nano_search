use std::collections::HashMap;

use crate::model::doc::DocId;

#[derive(Default, Clone)]
pub struct Query {
    pub id: u64,
    pub text: String,

    // IDs of documents considered relevant to this query, and thus expected to
    // be in search results
    pub relevant_docs: HashMap<DocId, Relevance>,
}

pub type Relevance = f64;

pub trait QueriesSource {
    type Iter: Iterator<Item = Query>;
    fn queries(&self) -> anyhow::Result<Self::Iter>;
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
