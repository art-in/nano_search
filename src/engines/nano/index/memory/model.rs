use std::collections::{BTreeMap, HashMap};

use anyhow::Result;

use super::iterator::MemoryDocPostingsIterator;
use crate::engines::nano::index::model::{
    DocPosting, DocPostingsForTerm, Index, IndexSegment, IndexSegmentStats,
    Term,
};
use crate::model::doc::DocId;

#[derive(Default)]
pub struct MemoryIndex {
    pub terms: HashMap<Term, TermPostingList>,
    pub stats: IndexSegmentStats,
}

pub type TermPostingList = BTreeMap<DocId, DocPosting>;

impl Index for MemoryIndex {
    fn get_segments(&self) -> Vec<&dyn IndexSegment> {
        vec![self]
    }
}

impl IndexSegment for MemoryIndex {
    fn get_doc_postings_for_term(
        &self,
        term: &Term,
    ) -> Result<Option<DocPostingsForTerm<'_>>> {
        let term_posting_list = self.terms.get(term);

        if let Some(term_posting_list) = term_posting_list {
            Ok(Some(DocPostingsForTerm {
                count: term_posting_list.len(),
                iterator: Box::new(MemoryDocPostingsIterator::new(
                    term_posting_list.values().cloned().collect(),
                )),
            }))
        } else {
            Ok(None)
        }
    }

    fn get_stats(&self) -> &IndexSegmentStats {
        &self.stats
    }
}
