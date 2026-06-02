use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};

use anyhow::{Context, Result};

use super::iterator::MemoryDocPostingsIterator;
use crate::engines::nano::index::model::{
    DocPosting, DocPostingsForTerm, Index, IndexSegment, IndexSegmentStats,
    SegmentDocId, StoredDoc, Term,
};

#[derive(Default)]
pub struct MemoryIndex {
    pub terms: HashMap<Term, TermPostingList>,
    /// Count of terms for each document, in form of vector that can be indexed
    /// with [`SegmentDocId`].
    pub doc_term_counts: Vec<u16>,
    /// Stored documents, in form of vector that can be indexed with
    /// [`SegmentDocId`].
    pub docs: Vec<StoredDoc>,
    pub stats: IndexSegmentStats,
}

pub type TermPostingList = BTreeMap<SegmentDocId, DocPosting>;

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

    fn get_doc_terms_count(&self, docid: SegmentDocId) -> Result<Cow<'_, u16>> {
        let count = self
            .doc_term_counts
            .get(docid as usize)
            .context("doc with such ID should exist in segment")?;
        Ok(Cow::Borrowed(count))
    }

    fn get_stored_doc(
        &self,
        docid: SegmentDocId,
    ) -> Result<Cow<'_, StoredDoc>> {
        let doc = self
            .docs
            .get(docid as usize)
            .context("doc with such ID should exist in segment")?;
        Ok(Cow::Borrowed(doc))
    }

    fn get_stats(&self) -> &IndexSegmentStats {
        &self.stats
    }
}
