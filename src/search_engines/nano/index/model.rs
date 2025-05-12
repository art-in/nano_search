use crate::model::{doc::DocId, engine::IndexStats};

#[derive(Copy, Clone, PartialEq)]
pub enum IndexType {
    MemoryIndex,
    FsIndex,
}

pub trait Index {
    fn get_doc_postings_iterator(
        &self,
        term: &Term,
    ) -> Option<DocPostingsIterator>;
    fn get_index_stats(&self) -> IndexStats;
}

pub type Term = String;

// TODO: TF (term frequency) can be calculated as term_count/total_terms_count
// and stored to posting on indexing stage, instead of search stage
#[derive(Clone)]
pub struct DocPosting {
    pub docid: DocId,

    // number of times this term appears in the doc
    pub term_count: u64,

    // total number of terms in this doc
    pub total_terms_count: u64,
}

pub struct DocPostingsIterator {
    postings: Vec<DocPosting>,
    position: usize,
}

impl DocPostingsIterator {
    pub fn new(postings: Vec<DocPosting>) -> Self {
        Self {
            postings,
            position: 0,
        }
    }

    pub fn postings_len(&self) -> usize {
        self.postings.len()
    }
}

impl Iterator for DocPostingsIterator {
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
