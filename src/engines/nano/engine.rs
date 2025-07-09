use std::path::Path;

use anyhow::{Context, Result};

use super::index::model::{Index, IndexMedium};
use super::index::{DiskIndexOptions, build_index, open_index};
use super::search::search;
use crate::model::doc::{Doc, DocId};
use crate::model::engine::SearchEngine;

pub struct NanoSearchEngine {
    index_medium: IndexMedium,
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
            index_medium: IndexMedium::Memory,
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
            index_medium: IndexMedium::Disk(DiskIndexOptions::new(index_dir)),
            index: None,
        })
    }

    fn open_from_disk(index_dir: impl AsRef<Path>) -> Result<Self> {
        let index_medium = IndexMedium::Disk(DiskIndexOptions::new(index_dir));
        let index =
            open_index(&index_medium).context("index should be opened")?;

        Ok(NanoSearchEngine {
            index_medium,
            index: Some(index),
        })
    }

    fn index_docs(
        &mut self,
        docs: &mut dyn Iterator<Item = Doc>,
    ) -> Result<()> {
        self.index = Some(
            build_index(&self.index_medium, docs)
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
