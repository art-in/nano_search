use crate::model::doc::Doc;

use super::{
    memory_index::build_memory_index,
    model::{Index, IndexType},
};

pub fn build_index(
    index_type: IndexType,
    docs: &mut dyn Iterator<Item = Doc>,
) -> Box<dyn Index> {
    match index_type {
        IndexType::MemoryIndex => build_memory_index(docs),
        IndexType::FsIndex => panic!("FsIndex not implemented yet"),
    }
}
