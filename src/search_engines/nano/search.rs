use super::{index::model::Index, scoring};
use crate::model::doc::DocId;
use std::collections::{HashMap, HashSet};

struct DocCandidate {
    id: DocId,
    relevance: f64,
}

pub fn search(
    query: &str,
    index: &dyn Index,
    limit: u64,
    stop_words: &HashSet<String>,
) -> Vec<u64> {
    let words: Vec<_> = query.split_whitespace().collect();

    let mut doc_candidates: HashMap<DocId, DocCandidate> = HashMap::new();

    for word in words {
        let term = crate::utils::normalize_word(word);

        if stop_words.contains(&term) {
            continue;
        }

        if let Some(postings_iterator) = index.get_doc_postings_iterator(&term)
        {
            let postings_len = postings_iterator.postings_len() as u64;

            for posting in postings_iterator {
                let doc_term_relevance = scoring::calc_bm25(
                    posting.term_count,
                    posting.total_terms_count,
                    postings_len,
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

    // sort candidates in descending order of relevance
    candidates.sort_by(|a, b| b.relevance.total_cmp(&a.relevance));

    let mut res = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        res.push(candidate.id);
        if res.len() as u64 >= limit {
            break;
        }
    }
    res
}
