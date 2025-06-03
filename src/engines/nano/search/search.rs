use std::collections::HashMap;

use anyhow::Result;

use super::model::DocCandidate;
use super::scoring;
use super::stop_words::STOP_WORDS;
use crate::engines::nano::index::model::Index;
use crate::model::doc::DocId;

pub fn search(
    query: &str,
    index: &dyn Index,
    limit: u64,
) -> Result<Vec<DocId>> {
    let words: Vec<&str> = query.split_whitespace().collect();

    let mut candidates: HashMap<DocId, DocCandidate> = HashMap::new();

    for word in words {
        let term = crate::utils::normalize_word(word);

        if STOP_WORDS.contains(&term) {
            continue;
        }

        if let Some(postings) = index.get_doc_postings_for_term(&term)? {
            for posting in postings.iterator {
                let relevance = scoring::calc_bm25(
                    scoring::ScoringParams {
                        doc_term_count: posting.term_count,
                        doc_total_terms_count: posting.total_terms_count,
                        docs_with_term_count: postings.count as u64,
                        docs_total_count: index.get_stats().indexed_docs_count,
                    },
                    index.get_stats().terms_count_per_doc_avg,
                );

                candidates
                    .entry(posting.docid)
                    .and_modify(|c| c.relevance += relevance)
                    .or_insert(DocCandidate {
                        id: posting.docid,
                        relevance,
                    });
            }
        }
    }

    let mut candidates: Vec<&DocCandidate> = candidates.values().collect();

    if candidates.len() > limit as usize {
        candidates.select_nth_unstable(limit as usize);
        candidates.truncate(limit as usize);
    }

    candidates.sort();

    let docids = candidates.iter().map(|c| c.id).collect();

    Ok(docids)
}
