use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct JsonDatasetReader {
    pub file_path: PathBuf,
}

impl JsonDatasetReader {
    pub fn new(file_path: impl AsRef<Path>) -> Self {
        JsonDatasetReader {
            file_path: file_path.as_ref().to_path_buf(),
        }
    }
}
