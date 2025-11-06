use std::path::{Path, PathBuf};

pub struct BeirDatasetReader {
    pub dir: PathBuf,
}

impl BeirDatasetReader {
    pub fn new(dir: impl AsRef<Path>) -> Self {
        Self {
            dir: dir.as_ref().join("data").to_path_buf(),
        }
    }
}
