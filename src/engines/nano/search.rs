use super::{index::model::Index, scoring};
use crate::model::doc::DocId;
use anyhow::Result;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
struct DocCandidate {
    id: DocId,
    relevance: f64,
}

pub fn search(
    query: &str,
    index: &dyn Index,
    limit: u64,
    stop_words: &HashSet<String>,
) -> Result<Vec<u64>> {
    let words: Vec<_> = query.split_whitespace().collect();

    let mut doc_candidates: HashMap<DocId, DocCandidate> = HashMap::new();

    for word in words {
        let term = crate::utils::normalize_word(word);

        if stop_words.contains(&term) {
            continue;
        }

        if let Some(doc_postings_for_term) =
            index.get_doc_postings_for_term(&term)?
        {
            for posting in doc_postings_for_term.iterator {
                let doc_term_relevance = scoring::calc_bm25(
                    posting.term_count,
                    posting.total_terms_count,
                    doc_postings_for_term.count as u64,
                    index.get_index_stats().indexed_docs_count,
                    index.get_index_stats().terms_count_per_doc_avg,
                );

                if let Some(doc_candidate) =
                    doc_candidates.get_mut(&posting.docid)
                {
                    doc_candidate.relevance += doc_term_relevance;
                } else {
                    doc_candidates.insert(
                        posting.docid,
                        DocCandidate {
                            id: posting.docid,
                            relevance: doc_term_relevance,
                        },
                    );
                }
            }
        }
    }

    let mut candidates = Vec::with_capacity(doc_candidates.len());
    for (_, doc_candidate) in doc_candidates {
        candidates.push(doc_candidate);
    }

    // sort candidates
    candidates.sort_by(|a, b| {
        // 1. in descending order of relevance
        let res = b.relevance.total_cmp(&a.relevance);
        if res != std::cmp::Ordering::Equal {
            return res;
        }

        // 2. in ascending order of document ID
        // (to stabilize search results for unit tests)
        a.id.cmp(&b.id)
    });

    let mut res = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        res.push(candidate.id);
        if res.len() as u64 >= limit {
            break;
        }
    }
    Ok(res)
}
