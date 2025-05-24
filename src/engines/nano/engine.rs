use super::{
    index::{
        build_index,
        model::{Index, IndexType},
        open_index,
    },
    search::search,
    stop_words::get_stop_words,
};
use crate::model::{
    doc::{Doc, DocId},
    engine::SearchEngine,
};
use anyhow::{Context, Result};
use std::{collections::HashSet, path::Path};

pub struct NanoSearchEngine {
    index_type: IndexType,
    index: Option<Box<dyn Index>>,
    stop_words: HashSet<String>,
}

impl SearchEngine for NanoSearchEngine {
    fn get_name(&self) -> &'static str {
        "nano"
    }

    fn create_index(index_dir: impl AsRef<Path>) -> Result<Self> {
        if index_dir.as_ref().exists() {
            std::fs::remove_dir_all(index_dir.as_ref())
                .context("existing index dir should be removed")?;
        }
        std::fs::create_dir(index_dir.as_ref())
            .context("index dir should be created")?;

        Ok(NanoSearchEngine {
            index_type: IndexType::FsIndex(index_dir.as_ref().to_path_buf()),
            index: None,
            stop_words: get_stop_words(),
        })
    }

    fn open_index(index_dir: impl AsRef<Path>) -> Result<Self> {
        let index_type = IndexType::FsIndex(index_dir.as_ref().to_path_buf());
        let index =
            open_index(&index_type).context("index should be opened")?;

        Ok(NanoSearchEngine {
            index_type,
            index: Some(index),
            stop_words: get_stop_words(),
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

        search(query, index.as_ref(), limit, &self.stop_words)
    }
}
