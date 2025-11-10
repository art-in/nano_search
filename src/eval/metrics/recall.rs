use std::collections::HashMap;

use crate::eval::model::Relevance;
use crate::model::doc::DocId;

pub fn recall(
    found_docids: &[DocId],
    relevant_docs: &HashMap<DocId, Relevance>,
) -> f64 {
    let found_relevant_docids_count: usize = found_docids
        .iter()
        .filter(|docid| relevant_docs.contains_key(*docid))
        .count();

    if relevant_docs.is_empty() {
        1.0
    } else {
        found_relevant_docids_count as f64 / relevant_docs.len() as f64
    }
}
