use super::doc::{Doc, DocId};
use std::path::Path;

pub trait SearchEngine {
    fn get_name(&self) -> &'static str;

    fn create_index(index_dir: impl AsRef<Path>) -> Self
    where
        Self: Sized;
    fn open_index(index_dir: impl AsRef<Path>) -> Self
    where
        Self: Sized;

    fn index_docs(
        &mut self,
        // TODO: use iterator over doc reference
        docs: &mut dyn Iterator<Item = Doc>,
    );
    fn search(&self, query: &str, limit: u64) -> Vec<DocId>;
}

#[derive(Default, PartialEq, Debug, Clone)]
pub struct IndexStats {
    pub indexed_docs_count: u64,
    pub posting_lists_count: u64,
    pub max_posting_list_size: u64,
    pub terms_count_per_doc_avg: f64,
}
