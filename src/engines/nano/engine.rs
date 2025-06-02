use std::path::Path;

use anyhow::{Context, Result};

use super::{
    index::{
        build_index,
        model::{Index, IndexType},
        open_index,
    },
    search::search,
};
use crate::model::{
    doc::{Doc, DocId},
    engine::SearchEngine,
};

pub struct NanoSearchEngine {
    index_type: IndexType,
    index: Option<Box<dyn Index>>,
}

impl SearchEngine for NanoSearchEngine {
    fn name() -> &'static str {
        "nano"
    }

    fn get_name(&self) -> &'static str {
        Self::name()
    }

    fn create_in_memory() -> Result<Self>
    where
        Self: Sized,
    {
        Ok(NanoSearchEngine {
            index_type: IndexType::MemoryIndex,
            index: None,
        })
    }

    fn create_on_disk(index_dir: impl AsRef<Path>) -> Result<Self> {
        if index_dir.as_ref().exists() {
            std::fs::remove_dir_all(index_dir.as_ref())
                .context("existing index dir should be removed")?;
        }
        std::fs::create_dir(index_dir.as_ref())
            .context("index dir should be created")?;

        Ok(NanoSearchEngine {
            index_type: IndexType::FsIndex(index_dir.as_ref().to_path_buf()),
            index: None,
        })
    }

    fn open_from_disk(index_dir: impl AsRef<Path>) -> Result<Self> {
        let index_type = IndexType::FsIndex(index_dir.as_ref().to_path_buf());
        let index =
            open_index(&index_type).context("index should be opened")?;

        Ok(NanoSearchEngine {
            index_type,
            index: Some(index),
        })
    }

    fn index_docs(
        &mut self,
        docs: &mut dyn Iterator<Item = Doc>,
    ) -> Result<()> {
        self.index = Some(
            build_index(&self.index_type, docs)
                .context("index should be built")?,
        );
        Ok(())
    }

    fn search(&self, query: &str, limit: u64) -> Result<Vec<DocId>> {
        let index = self
            .index
            .as_ref()
            .context("index should be initialized before search")?;

        search(query, index.as_ref(), limit)
    }
}
