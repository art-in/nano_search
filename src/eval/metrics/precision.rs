use std::collections::HashMap;

use crate::eval::model::Relevance;
use crate::model::doc::DocId;

pub fn precision(
    found_docids: &[DocId],
    relevant_docs: &HashMap<DocId, Relevance>,
) -> f64 {
    let found_relevant_docids_count: usize = found_docids
        .iter()
        .filter(|docid| relevant_docs.contains_key(*docid))
        .count();

    if found_docids.is_empty() {
        if relevant_docs.is_empty() { 1.0 } else { 0.0 }
    } else {
        found_relevant_docids_count as f64 / found_docids.len() as f64
    }
}
