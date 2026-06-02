use std::collections::HashMap;

use crate::eval::model::Relevance;
use crate::model::doc::ExternalDocId;

/// Calculates number of relevant documents in search result
pub fn hits(
    found_docids: &[ExternalDocId],
    relevant_docs: &HashMap<ExternalDocId, Relevance>,
) -> usize {
    found_docids
        .iter()
        .filter(|docid| relevant_docs.contains_key(docid))
        .count()
}
