use super::model::{MemoryIndex, TermPostingList};
use crate::engines::nano::index::model::DocPosting;
use crate::model::doc::Doc;

pub fn build_memory_index(docs: &mut dyn Iterator<Item = Doc>) -> MemoryIndex {
    let mut index = MemoryIndex::default();

    let mut terms_total: u64 = 0;

    for doc in docs {
        let words = doc.text.split_whitespace();

        let terms = words.filter_map(|word| {
            let term = crate::utils::normalize_word(word);
            if term.is_empty() { None } else { Some(term) }
        });

        let mut doc_terms_count = 0_u16;

        for term in terms {
            let posting_list =
                index.terms.entry(term).or_insert_with(TermPostingList::new);

            let posting =
                posting_list.entry(doc.id).or_insert_with(|| DocPosting {
                    docid: doc.id,
                    term_count: 0,
                });

            posting.term_count += 1;
            doc_terms_count += 1;

            index.stats.max_posting_list_size = (posting_list.len() as u64)
                .max(index.stats.max_posting_list_size);
        }

        terms_total += doc_terms_count as u64;
        index.doc_terms_count.insert(doc.id, doc_terms_count);
        index.stats.indexed_docs_count += 1;
    }

    index.stats.terms_count_per_doc_avg =
        terms_total as f64 / index.stats.indexed_docs_count as f64;

    index
}
