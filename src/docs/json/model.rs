use std::path::{Path, PathBuf};

use crate::model::doc::DocsSource;

#[derive(Clone)]
pub struct JsonDocs {
    pub file_path: PathBuf,
}

impl JsonDocs {
    pub fn new(file_path: impl AsRef<Path>) -> Self {
        JsonDocs {
            file_path: file_path.as_ref().to_path_buf(),
        }
    }
}

impl DocsSource for JsonDocs {}
