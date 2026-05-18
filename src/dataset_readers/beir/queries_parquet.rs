use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use parquet::record::{Field, Row};

use super::qrels::load_qrels;
use super::utils::parse_id;
use crate::eval::model::{QueriesSource, Query, Relevance};
use crate::model::doc::DocId;
use crate::utils::get_parquet_rows;

pub struct BeirQueriesParquetReader {
    queries_files: Vec<PathBuf>,
    qrels_file: PathBuf,
}

impl BeirQueriesParquetReader {
    pub const fn new(queries_files: Vec<PathBuf>, qrels_file: PathBuf) -> Self {
        Self {
            queries_files,
            qrels_file,
        }
    }
}

impl QueriesSource for BeirQueriesParquetReader {
    fn queries(&self) -> Result<Box<dyn Iterator<Item = Query>>> {
        let rows = get_parquet_rows(self.queries_files.clone())?;
        let qrels = load_qrels(&self.qrels_file)?;
        Ok(Box::new(BeirQueriesParquetIterator { rows, qrels }))
    }
}

struct BeirQueriesParquetIterator {
    rows: Box<dyn Iterator<Item = Result<Row>>>,
    qrels: HashMap<u64, HashMap<DocId, Relevance>>,
}

impl Iterator for BeirQueriesParquetIterator {
    type Item = Query;

    fn next(&mut self) -> Option<Self::Item> {
        // skip queries lacking relevant docs to ensure evaluation is possible.
        // since reduced qrels, like test.tsv, may not have lines for each query
        for row in self.rows.by_ref() {
            let line = row.expect("line should be read");
            let query = parse_query_from_row(&line, &mut self.qrels)
                .expect("query should be parsed");
            if !query.relevant_docs.is_empty() {
                return Some(query);
            }
        }

        None
    }
}

pub fn parse_query_from_row(
    row: &Row,
    qrels: &mut HashMap<u64, HashMap<DocId, Relevance>>,
) -> Result<Query> {
    let mut id = None;
    let mut text = None;

    for (name, field) in row.get_column_iter() {
        match name.as_str() {
            "_id" => {
                if let Field::Str(val) = field {
                    id = Some(val);
                }
            }
            "title" => {
                // ignore empty column
            }
            "text" => {
                if let Field::Str(val) = field {
                    text = Some(val);
                }
            }
            _ => {
                bail!("unknown row column: {}", name)
            }
        }
    }

    let id = parse_id(id.context("id column should exist")?)?;
    let text = text.context("text column should exist")?.clone();
    let relevant_docs = qrels.remove(&id).unwrap_or_default();

    Ok(Query {
        id,
        text,
        relevant_docs,
    })
}
