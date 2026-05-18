use std::path::Path;

use anyhow::Result;

use super::docs_json::BeirDocsJsonReader;
use super::docs_parquet::BeirDocsParquetReader;
use crate::dataset_readers::beir::queries_json::BeirQueriesJsonReader;
use crate::dataset_readers::beir::queries_parquet::BeirQueriesParquetReader;
use crate::eval::model::QueriesSource;
use crate::model::doc::{Doc, DocsSource};
use crate::utils::{download_hf_dir, download_hf_file};

pub struct BeirDatasetReader {
    pub docs_reader: Box<dyn DocsSource>,
    pub queries_reader: Box<dyn QueriesSource>,
}

impl BeirDatasetReader {
    pub fn from_dir(dir: impl AsRef<Path>) -> Result<Self> {
        let docs_file = dir.as_ref().join("corpus.jsonl");
        let queries_file = dir.as_ref().join("queries.jsonl");
        let qrels_file = dir.as_ref().join("qrels/test.tsv");

        Ok(Self {
            docs_reader: Box::new(BeirDocsJsonReader::new(docs_file)),
            queries_reader: Box::new(BeirQueriesJsonReader::new(
                queries_file,
                qrels_file,
            )),
        })
    }

    pub fn from_hf(dataset_name: &str) -> Result<Self> {
        let main_repo_id = format!("BeIR/{dataset_name}");
        let qrels_repo_id = format!("BeIR/{dataset_name}-qrels");

        let docs_files = download_hf_dir(&main_repo_id, "corpus", ".parquet")?;
        let queries_files =
            download_hf_dir(&main_repo_id, "queries", ".parquet")?;
        let qrels_file = download_hf_file(&qrels_repo_id, "test.tsv")?;

        Ok(Self {
            docs_reader: Box::new(BeirDocsParquetReader::new(docs_files)),
            queries_reader: Box::new(BeirQueriesParquetReader::new(
                queries_files,
                qrels_file,
            )),
        })
    }
}

impl DocsSource for BeirDatasetReader {
    fn docs(&self) -> Result<Box<dyn Iterator<Item = Doc>>> {
        self.docs_reader.docs()
    }
    fn docs_count(&self) -> Result<Option<usize>> {
        self.docs_reader.docs_count()
    }
}

impl QueriesSource for BeirDatasetReader {
    fn queries(
        &self,
    ) -> Result<Box<dyn Iterator<Item = crate::eval::model::Query>>> {
        self.queries_reader.queries()
    }
}
