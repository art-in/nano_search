use anyhow::{Result, bail};

use super::disk::{build_disk_index, open_disk_index};
use super::memory::build_memory_index;
use super::model::{Index, IndexMedium};
use crate::model::doc::Doc;

pub fn build_index(
    index_medium: &IndexMedium,
    docs: &mut dyn Iterator<Item = Doc>,
) -> Result<Box<dyn Index>> {
    match index_medium {
        IndexMedium::Memory => Ok(Box::new(build_memory_index(docs))),
        IndexMedium::Disk(options) => {
            Ok(Box::new(build_disk_index(docs, options)?))
        }
    }
}

pub fn open_index(index_medium: &IndexMedium) -> Result<Box<dyn Index>> {
    match index_medium {
        IndexMedium::Memory => bail!("memory index cannot be opened"),
        IndexMedium::Disk(options) => Ok(Box::new(open_disk_index(options)?)),
    }
}
