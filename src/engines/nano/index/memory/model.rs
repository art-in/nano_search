use std::collections::{BTreeMap, HashMap};

use anyhow::{Context, Result};

use super::iterator::MemoryDocPostingsIterator;
use crate::engines::nano::index::model::{
    DocPosting, DocPostingsForTerm, Index, IndexSegment, IndexSegmentStats,
    Term,
};
use crate::model::doc::DocId;

#[derive(Default)]
pub struct MemoryIndex {
    pub terms: HashMap<Term, TermPostingList>,
    pub doc_terms_count: HashMap<DocId, u16>,
    pub stats: IndexSegmentStats,
}

pub type TermPostingList = BTreeMap<DocId, DocPosting>;

impl Index for MemoryIndex {
    fn get_segments(&self) -> Vec<&dyn IndexSegment> {
        vec![self]
    }
}

impl IndexSegment for MemoryIndex {
    fn get_doc_postings_for_term<'a>(
        &'a self,
        term: &Term,
    ) -> Result<Option<DocPostingsForTerm<'a>>> {
        let term_posting_list = self.terms.get(term);

        term_posting_list.map_or_else(
            || Ok(None),
            |list| {
                Ok(Some(DocPostingsForTerm {
                    count: list.len(),
                    iterator: Box::new(MemoryDocPostingsIterator::new(
                        list.values(),
                    )),
                }))
            },
        )
    }

    fn get_doc_terms_count(&self, docid: DocId) -> Result<u16> {
        self.doc_terms_count
            .get(&docid)
            .copied()
            .context("document should exist")
    }

    fn get_stats(&self) -> &IndexSegmentStats {
        &self.stats
    }
}
