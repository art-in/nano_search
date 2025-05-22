use crate::model::{doc::DocId, engine::IndexStats};
use anyhow::Result;
use std::path::PathBuf;

#[derive(Clone, PartialEq)]
pub enum IndexType {
    MemoryIndex,
    FsIndex(PathBuf),
}

pub struct DocPostingsForTerm {
    pub count: usize,
    pub iterator: Box<dyn Iterator<Item = DocPosting>>,
}

pub trait Index {
    fn get_doc_postings_for_term(
        &self,
        term: &Term,
    ) -> Result<Option<DocPostingsForTerm>>;
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
