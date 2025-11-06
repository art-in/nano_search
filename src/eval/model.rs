use std::collections::HashSet;

use crate::model::doc::DocId;

#[derive(Default, Clone)]
pub struct Query {
    pub id: u64,
    pub text: String,

    // IDs of documents considered relevant to this query, and thus expected to
    // be in search results
    pub relevant_docids: HashSet<DocId>,
}

pub trait QueriesSource {
    type Iter: Iterator<Item = Query>;
    fn queries(&self) -> Self::Iter;
}

// Search quality evaluation results summarized for entire query set
pub struct SearchQuality {
    pub queries_count: u64,
    pub precision_avg: f64,
    pub recall_avg: f64,
    pub precisions: inc_stats::Percentiles<f64>,
    pub recalls: inc_stats::Percentiles<f64>,
}

// Search quality evaluation results for particular query
pub struct QuerySearchQuality {
    pub precision: f64,
    pub recall: f64,
}
