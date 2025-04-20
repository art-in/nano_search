use crate::model::doc::DocId;

use super::index::Index;

use std::collections::{BTreeSet, HashSet};

pub fn search(
    query: &str,
    index: &Index,
    stop_words: &HashSet<String>,
) -> Vec<u64> {
    let words: Vec<_> = query.split_whitespace().collect();

    let mut docids: BTreeSet<DocId> = BTreeSet::new();

    for word in words {
        let term = crate::utils::normalize_word(word);

        if stop_words.contains(&term) {
            continue;
        }

        if let Some(term_posting_list) = index.terms.get(&term) {
            let docs_with_term = term_posting_list.len();

            let doc_frequency =
                docs_with_term as f64 / index.stats.indexed_docs_count as f64;

            for (&docid, posting) in term_posting_list {
                let term_frequency = posting.term_count as f64
                    / posting.total_terms_count as f64;

                let inverted_doc_frequency = f64::ln(1.0 / doc_frequency);

                let tfidf = term_frequency * inverted_doc_frequency;

                if tfidf < 0.001 {
                    continue;
                }

                docids.insert(docid);
            }
        }
    }

    let mut res = Vec::with_capacity(docids.len());
    for docid in docids {
        res.push(docid);
    }
    res
}
