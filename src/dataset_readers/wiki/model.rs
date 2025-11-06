use std::path::Path;

use anyhow::Result;

use crate::utils::wikidump::WikiDump;

#[derive(Clone)]
pub struct WikiDatasetReader {
    pub wikidump: WikiDump,
}

impl WikiDatasetReader {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        Ok(WikiDatasetReader {
            wikidump: WikiDump::new(file_path)?,
        })
    }
}
