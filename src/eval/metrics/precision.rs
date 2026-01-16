use std::collections::HashMap;

use crate::eval::metrics::hits::hits;
use crate::eval::model::Relevance;
use crate::model::doc::DocId;

#[must_use]
pub fn precision(
    found_docids: &[DocId],
    relevant_docs: &HashMap<DocId, Relevance>,
) -> f64 {
    let found_relevant_docids_count = hits(found_docids, relevant_docs);

    if found_docids.is_empty() {
        if relevant_docs.is_empty() { 1.0 } else { 0.0 }
    } else {
        found_relevant_docids_count as f64 / found_docids.len() as f64
    }
}
