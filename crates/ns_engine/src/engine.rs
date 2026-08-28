use std::path::{Path, PathBuf};

use anyhow::Result;

use super::doc::{Doc, ExternalDocId};

/// Main search engine interface.
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

    /// Initializes search engine and creates new index on disk.
    fn create_on_disk(options: CreateOnDiskOptions) -> Result<Self>
    where
        Self: Sized;

    /// Initializes search engine and opens existing index from disk.
    fn open_from_disk(index_dir: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized;

    /// Adds documents to the index.
    fn index_docs(
        &mut self,
        docs: &mut dyn Iterator<Item = Result<Doc>>,
    ) -> Result<()>;

    /// Searches for documents relevant to passed query.
    fn search(&self, query: &str, limit: u64) -> Result<Vec<ExternalDocId>>;
}

#[derive(bon::Builder)]
pub struct CreateOnDiskOptions {
    /// Path to directory where index should be stored.
    #[builder(into)]
    pub index_dir: PathBuf,

    /// Number of indexing threads to start.
    pub index_threads: Option<usize>,
}
