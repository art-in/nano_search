use crate::model::doc::Doc;
use anyhow::Result;

use super::{
    fs::build_fs_index,
    memory::build_memory_index,
    model::{Index, IndexType},
};

pub fn build_index(
    index_type: IndexType,
    docs: &mut dyn Iterator<Item = Doc>,
) -> Result<Box<dyn Index>> {
    let memory_index = build_memory_index(docs);

    match index_type {
        IndexType::MemoryIndex => Ok(Box::new(memory_index)),
        IndexType::FsIndex(index_dir) => {
            Ok(Box::new(build_fs_index(&memory_index, index_dir)?))
        }
    }
}
