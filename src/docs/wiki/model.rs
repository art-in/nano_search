use std::path::Path;

use anyhow::Result;

use crate::utils::wikidump::WikiDump;

#[derive(Clone)]
pub struct WikiDocs {
    pub wikidump: WikiDump,
}

impl WikiDocs {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        Ok(WikiDocs {
            wikidump: WikiDump::new(file_path)?,
        })
    }
}
