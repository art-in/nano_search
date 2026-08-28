use std::path::{Path, PathBuf};

pub struct CisiDatasetReader {
    pub(super) docs_file: PathBuf,
    pub(super) queries_file: PathBuf,
    pub(super) qrels_file: PathBuf,
}

impl CisiDatasetReader {
    pub fn new(dir: impl AsRef<Path>) -> Self {
        Self {
            docs_file: dir.as_ref().join("CISI.ALL"),
            queries_file: dir.as_ref().join("CISI.QRY"),
            qrels_file: dir.as_ref().join("CISI.REL"),
        }
    }
}
