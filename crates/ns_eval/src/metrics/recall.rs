use std::collections::HashMap;

use ns_engine::{ExternalDocId, Score};

use super::hits::hits;

#[must_use]
pub fn recall(
    found_docids: &[ExternalDocId],
    relevant_docs: &HashMap<ExternalDocId, Score>,
) -> f64 {
    let found_relevant_docids_count = hits(found_docids, relevant_docs);

    if relevant_docs.is_empty() {
        1.0
    } else {
        found_relevant_docids_count as f64 / relevant_docs.len() as f64
    }
}
