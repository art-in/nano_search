use std::collections::{BTreeMap, HashMap};

use anyhow::Result;

use super::model::{DocPosting, DocPostingsForTerm, Index, Term};
use crate::model::doc::{Doc, DocId};
use crate::model::engine::IndexStats;

type TermPostingList = BTreeMap<DocId, DocPosting>;

#[derive(Default)]
pub struct MemoryIndex {
    pub terms: HashMap<Term, TermPostingList>,
    pub stats: IndexStats,
}

pub struct MemoryDocPostingsIterator {
    postings: Vec<DocPosting>,
    position: usize,
}

impl MemoryDocPostingsIterator {
    pub fn new(postings: Vec<DocPosting>) -> Self {
        Self {
            postings,
            position: 0,
        }
    }
}

impl Iterator for MemoryDocPostingsIterator {
    // TODO: iterate over references to avoid cloning
    type Item = DocPosting;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.postings.len() {
            let posting = self.postings[self.position].clone();
            self.position += 1;
            Some(posting)
        } else {
            None
        }
    }
}

impl Index for MemoryIndex {
    fn get_doc_postings_for_term(
        &self,
        term: &Term,
    ) -> Result<Option<DocPostingsForTerm>> {
        let term_posting_list = self.terms.get(term);

        if let Some(term_posting_list) = term_posting_list {
            Ok(Some(DocPostingsForTerm {
                count: term_posting_list.len(),
                iterator: Box::new(MemoryDocPostingsIterator::new(
                    term_posting_list.values().cloned().collect(),
                ))
                    as Box<dyn Iterator<Item = DocPosting>>,
            }))
        } else {
            Ok(None)
        }
    }

    fn get_stats(&self) -> &IndexStats {
        &self.stats
    }
}

pub fn build_memory_index(docs: &mut dyn Iterator<Item = Doc>) -> MemoryIndex {
    let mut index = MemoryIndex::default();

    let mut docs_terms_count_sum: u64 = 0;

    for doc in docs {
        let words: Vec<&str> = doc.text.split(' ').collect();

        let terms: Vec<String> = words
            .iter()
            .filter_map(|word| {
                let term = crate::utils::normalize_word(word);
                if term.is_empty() { None } else { Some(term) }
            })
            .collect();

        docs_terms_count_sum += terms.len() as u64;

        for term in &terms {
            match index.terms.get_mut(term) {
                Some(posting_list) => match posting_list.get_mut(&doc.id) {
                    Some(posting) => {
                        posting.term_count += 1;
                    }
                    None => {
                        let posting = DocPosting {
                            docid: doc.id,
                            term_count: 1,
                            total_terms_count: terms.len() as u64,
                        };
                        posting_list.insert(doc.id, posting);
                        index.stats.max_posting_list_size = (posting_list.len()
                            as u64)
                            .max(index.stats.max_posting_list_size);
                    }
                },
                None => {
                    let mut posting_list = TermPostingList::new();
                    let posting = DocPosting {
                        docid: doc.id,
                        term_count: 1,
                        total_terms_count: terms.len() as u64,
                    };
                    posting_list.insert(doc.id, posting);
                    index.terms.insert(term.clone(), posting_list);
                    index.stats.posting_lists_count += 1;
                }
            }
        }

        index.stats.indexed_docs_count += 1;
    }

    index.stats.terms_count_per_doc_avg =
        docs_terms_count_sum as f64 / index.stats.indexed_docs_count as f64;

    index
}
