use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use parquet::record::Row;

use super::qrels::load_qrels;
use super::utils::{extract_string_from_parquet, parse_id};
use crate::eval::model::{QueriesSource, Query, QueryId, Relevance};
use crate::model::doc::ExternalDocId;
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
    fn queries(&self) -> Result<Box<dyn Iterator<Item = Result<Query>>>> {
        let rows = get_parquet_rows(self.queries_files.clone())?;
        let qrels = load_qrels(&self.qrels_file)?;
        Ok(Box::new(BeirQueriesParquetIterator { rows, qrels }))
    }
}

struct BeirQueriesParquetIterator {
    rows: Box<dyn Iterator<Item = Result<Row>>>,
    qrels: HashMap<QueryId, HashMap<ExternalDocId, Relevance>>,
}

impl Iterator for BeirQueriesParquetIterator {
    type Item = Result<Query>;

    fn next(&mut self) -> Option<Self::Item> {
        get_next_query(&mut self.rows, &mut self.qrels).transpose()
    }
}

fn get_next_query(
    rows: &mut dyn Iterator<Item = Result<Row>>,
    qrels: &mut HashMap<QueryId, HashMap<ExternalDocId, Relevance>>,
) -> Result<Option<Query>> {
    // skip queries lacking relevant docs to ensure evaluation is possible.
    // since reduced qrels, like test.tsv, may not have lines for each query
    for row in rows {
        let line = row.context("row should be read")?;
        let query = parse_query_from_row(&line, qrels)
            .context("query should be parsed")?;
        if !query.relevant_docs.is_empty() {
            return Ok(Some(query));
        }
    }

    Ok(None)
}

fn parse_query_from_row(
    row: &Row,
    qrels: &mut HashMap<u64, HashMap<ExternalDocId, Relevance>>,
) -> Result<Query> {
    let mut id = None;
    let mut text = None;

    for (name, field) in row.get_column_iter() {
        match name.as_str() {
            "_id" => id = Some(extract_string_from_parquet(field)?),
            "title" => { /* ignore empty column */ }
            "text" => text = Some(extract_string_from_parquet(field)?),
            _ => bail!("unknown row column: {}", name),
        }
    }

    let id = parse_id(id.context("id column should exist")?)?;
    let text = text.context("text column should exist")?.to_owned();
    let relevant_docs = qrels.remove(&id).unwrap_or_default();

    Ok(Query {
        id,
        text,
        relevant_docs,
    })
}
