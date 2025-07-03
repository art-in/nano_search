use anyhow::{Result, bail};

use super::fs::{build_fs_index, open_fs_index};
use super::memory::build_memory_index;
use super::model::{Index, IndexType};
use crate::model::doc::Doc;

pub fn build_index(
    index_type: &IndexType,
    docs: &mut dyn Iterator<Item = Doc>,
) -> Result<Box<dyn Index>> {
    match index_type {
        IndexType::MemoryIndex => Ok(Box::new(build_memory_index(docs))),
        IndexType::FsIndex(index_dir) => {
            Ok(Box::new(build_fs_index(docs, index_dir)?))
        }
    }
}

pub fn open_index(index_type: &IndexType) -> Result<Box<dyn Index>> {
    match index_type {
        IndexType::MemoryIndex => bail!("memory index cannot be opened"),
        IndexType::FsIndex(index_dir) => {
            Ok(Box::new(open_fs_index(index_dir)?))
        }
    }
}
