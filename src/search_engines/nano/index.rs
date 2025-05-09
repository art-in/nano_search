use std::collections::{BTreeMap, HashMap};

use crate::model::{
    doc::{Doc, DocId},
    engine::IndexStats,
};

pub type Term = String;

// TODO: TF (term frequency) can be calculated as term_count/total_terms_count
// and stored to posting on indexing stage, instead of search stage
pub struct DocPosting {
    pub docid: DocId,

    // number of times this term appears in the doc
    pub term_count: u64,

    // total number of terms in this doc
    pub total_terms_count: u64,
}

pub type TermPostingList = BTreeMap<DocId, DocPosting>;

#[derive(Default)]
pub struct Index {
    pub terms: HashMap<Term, TermPostingList>,
    pub stats: IndexStats,
}

pub fn build_index(docs: &mut dyn Iterator<Item = Doc>) -> Index {
    let mut index = Index::default();

    let mut docs_terms_count_sum: u64 = 0;

    for doc in docs {
        let words: Vec<&str> = doc.text.split(' ').collect();

        let terms: Vec<String> = words
            .iter()
            .filter_map(|word| {
                let term = crate::utils::normalize_word(word);
                if term.is_empty() {
                    None
                } else {
                    Some(term)
                }
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
