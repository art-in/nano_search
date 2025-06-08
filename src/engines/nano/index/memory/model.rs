use std::collections::{BTreeMap, HashMap};

use anyhow::Result;

use super::iterator::MemoryDocPostingsIterator;
use crate::engines::nano::index::model::{
    DocPosting, DocPostingsForTerm, Index, IndexStats, Term,
};
use crate::model::doc::DocId;

#[derive(Default)]
pub struct MemoryIndex {
    pub terms: HashMap<Term, TermPostingList>,
    pub stats: IndexStats,
}

pub type TermPostingList = BTreeMap<DocId, DocPosting>;

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
