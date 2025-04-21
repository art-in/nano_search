use super::doc::{Doc, DocId};

pub trait SearchEngine {
    fn get_name(&self) -> &'static str;
    fn index_docs(
        &mut self,
        // TODO: use iterator over doc reference
        docs: &mut dyn Iterator<Item = Doc>,
    );
    fn search(&self, query: &str, limit: u64) -> Vec<DocId>;
}

#[derive(Default, Clone)]
pub struct IndexStats {
    pub indexed_docs_count: u64,
    pub posting_lists_count: u64,
    pub max_posting_list_size: u64,
}
