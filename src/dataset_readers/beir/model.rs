use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::utils::download_hf_file;

pub struct BeirDatasetReader {
    pub docs_file: PathBuf,
    pub queries_file: PathBuf,
    pub qrels_file: PathBuf,
}

impl BeirDatasetReader {
    pub fn from_dir(dir: impl AsRef<Path>) -> Result<Self> {
        let docs_file = dir.as_ref().join("corpus.jsonl");
        let queries_file = dir.as_ref().join("queries.jsonl");
        let qrels_file = dir.as_ref().join("qrels/test.tsv");

        Ok(Self {
            docs_file,
            queries_file,
            qrels_file,
        })
    }

    pub fn from_hf(dataset_name: &str) -> Result<Self> {
        let main_repo_id = format!("BeIR/{dataset_name}");
        let qrels_repo_id = format!("BeIR/{dataset_name}-qrels");

        let docs_file = download_hf_file(&main_repo_id, "corpus.jsonl.gz")?;
        let queries_file = download_hf_file(&main_repo_id, "queries.jsonl.gz")?;
        let qrels_file = download_hf_file(&qrels_repo_id, "test.tsv")?;

        Ok(Self {
            docs_file,
            queries_file,
            qrels_file,
        })
    }
}
