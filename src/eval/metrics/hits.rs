use std::collections::HashMap;

use crate::model::{ExternalDocId, Score};

/// Calculates number of relevant documents in search result
pub fn hits(
    found_docids: &[ExternalDocId],
    relevant_docs: &HashMap<ExternalDocId, Score>,
) -> usize {
    found_docids
        .iter()
        .filter(|docid| relevant_docs.contains_key(docid))
        .count()
}
