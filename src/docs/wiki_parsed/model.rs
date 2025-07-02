use std::path::{Path, PathBuf};

use crate::model::doc::DocsSource;

#[derive(Clone)]
pub struct WikiDocs {
    pub file_path: PathBuf,
}

impl WikiDocs {
    pub fn new(file_path: impl AsRef<Path>) -> Self {
        WikiDocs {
            file_path: file_path.as_ref().to_path_buf(),
        }
    }
}

impl DocsSource for WikiDocs {}
