use std::path::PathBuf;

use crate::model::{doc::DocId, engine::IndexStats};

#[derive(Clone, PartialEq)]
pub enum IndexType {
    MemoryIndex,
    FsIndex(PathBuf),
}

pub trait Index {
    fn get_doc_postings_for_term(
        &self,
        term: &Term,
    ) -> Option<(u64, Box<dyn Iterator<Item = DocPosting>>)>;
    fn get_index_stats(&self) -> &IndexStats;
}

pub type Term = String;

// TODO: TF (term frequency) can be calculated as term_count/total_terms_count
// and stored to posting on indexing stage, instead of search stage
#[derive(Clone, PartialEq, Debug)]
pub struct DocPosting {
    pub docid: DocId,

    // number of times this term appears in the doc
    pub term_count: u64,

    // total number of terms in this doc
    pub total_terms_count: u64,
}
