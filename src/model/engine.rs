use super::doc::{Doc, DocId};

pub trait SearchEngine {
    fn index_docs(
        &mut self,
        // TODO: use iterator over doc reference
        docs: &mut dyn Iterator<Item = Doc>,
    ) -> IndexStats;
    fn search(&self, query: &str) -> Vec<DocId>;
}

#[derive(Default, Clone)]
pub struct IndexStats {
    pub indexed_docs_count: u64,
    pub posting_lists_count: u64,
    pub max_posting_list_size: u64,
}
