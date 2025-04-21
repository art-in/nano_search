use super::index::Index;
use crate::model::doc::DocId;
use std::collections::{HashMap, HashSet};

struct DocCandidate {
    id: DocId,
    relevance: f64,
}

pub fn search(
    query: &str,
    index: &Index,
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

        if let Some(term_posting_list) = index.terms.get(&term) {
            for (&docid, posting) in term_posting_list {
                let tfidf = calc_tfidf(
                    posting.term_count,
                    posting.total_terms_count,
                    term_posting_list.len() as u64,
                    index.stats.indexed_docs_count,
                );

                if let Some(doc_candidate) = doc_candidates.get_mut(&docid) {
                    doc_candidate.relevance += tfidf;
                } else {
                    doc_candidates.insert(
                        docid,
                        DocCandidate {
                            id: docid,
                            relevance: tfidf,
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

// calculates term frequency – inverse document frequency.
// using count-idf weighting scheme - tf*log(N/n)
// https://en.wikipedia.org/wiki/Tf-idf
fn calc_tfidf(
    doc_term_count: u64,        // number of the term in the doc
    doc_total_terms_count: u64, // total number of terms in the doc
    docs_with_term_count: u64,  // number of docs containing this term
    docs_total_count: u64,      // total number of docs in the index
) -> f64 {
    let term_frequency = doc_term_count as f64 / doc_total_terms_count as f64;

    let inverted_doc_frequency =
        f64::ln(docs_total_count as f64 / docs_with_term_count as f64);

    term_frequency * inverted_doc_frequency
}
