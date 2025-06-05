use std::path::Path;

use anyhow::Result;

use super::doc::{Doc, DocId};

pub trait SearchEngine {
    /// Gets search engine name for debug logging purposes.
    fn name() -> &'static str
    where
        Self: Sized;
    fn get_name(&self) -> &'static str;

    /// Initializes search engine and creates new index in memory.
    fn create_in_memory() -> Result<Self>
    where
        Self: Sized;

    /// Initializes search engine and creates new index in target dir.
    fn create_on_disk(index_dir: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized;

    /// Initializes search engine and opens existing index from target dir.
    fn open_from_disk(index_dir: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized;

    /// Add documents to the index.
    fn index_docs(
        &mut self,
        // TODO: use iterator over doc reference
        docs: &mut dyn Iterator<Item = Doc>,
    ) -> Result<()>;

    /// Searches for documents relevant to passed query in the index.
    fn search(&self, query: &str, limit: u64) -> Result<Vec<DocId>>;
}

#[derive(Default, PartialEq, Debug, Clone)]
pub struct IndexStats {
    pub indexed_docs_count: u64,
    pub max_posting_list_size: u64,
    pub terms_count_per_doc_avg: f64,
}
