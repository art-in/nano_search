use anyhow::{Context, Result};

use super::index::model::{Index, IndexMedium};
use super::index::{DiskIndexOptions, build_index, open_index};
use super::search::search;
use crate::model::doc::{Doc, DocId};
use crate::model::engine::{CreateOnDiskOptions, SearchEngine};

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

    fn create_on_disk(opts: CreateOnDiskOptions) -> Result<Self> {
        if opts.index_dir.exists() {
            std::fs::remove_dir_all(&opts.index_dir)
                .context("existing index dir should be removed")?;
        }
        std::fs::create_dir(&opts.index_dir)
            .context("index dir should be created")?;

        let index_medium = IndexMedium::Disk(
            DiskIndexOptions::builder()
                .index_dir(opts.index_dir.clone())
                .maybe_index_threads(opts.index_threads)
                .build(),
        );

        Ok(NanoSearchEngine {
            index_medium,
            index: None,
        })
    }

    fn open_from_disk(index_dir: impl AsRef<std::path::Path>) -> Result<Self> {
        let index_medium = IndexMedium::Disk(
            DiskIndexOptions::builder()
                .index_dir(index_dir.as_ref())
                .build(),
        );
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
