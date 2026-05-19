use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_json::{Map, Value};

use super::qrels::load_qrels;
use super::utils::{extract_string_from_json, parse_id};
use crate::eval::model::{QueriesSource, Query, QueryId, Relevance};
use crate::model::doc::DocId;
use crate::utils::get_file_lines;

pub struct BeirQueriesJsonReader {
    queries_file: PathBuf,
    qrels_file: PathBuf,
}

impl BeirQueriesJsonReader {
    pub const fn new(queries_file: PathBuf, qrels_file: PathBuf) -> Self {
        Self {
            queries_file,
            qrels_file,
        }
    }
}

impl QueriesSource for BeirQueriesJsonReader {
    fn queries(&self) -> Result<Box<dyn Iterator<Item = Result<Query>>>> {
        let lines = get_file_lines(&self.queries_file)?;
        let qrels = load_qrels(&self.qrels_file)?;
        Ok(Box::new(BeirQueriesJsonIterator { lines, qrels }))
    }
}

struct BeirQueriesJsonIterator {
    lines: Box<dyn Iterator<Item = std::io::Result<String>>>,
    qrels: HashMap<QueryId, HashMap<DocId, Relevance>>,
}

impl Iterator for BeirQueriesJsonIterator {
    type Item = Result<Query>;

    fn next(&mut self) -> Option<Self::Item> {
        get_next_query(&mut self.lines, &mut self.qrels).transpose()
    }
}

fn get_next_query(
    lines: &mut dyn Iterator<Item = std::io::Result<String>>,
    qrels: &mut HashMap<QueryId, HashMap<DocId, Relevance>>,
) -> Result<Option<Query>> {
    // skip queries lacking relevant docs to ensure evaluation is possible.
    // since reduced qrels, like test.tsv, may not have lines for each query
    for line in lines {
        let line = line.context("line should be read")?;
        let query = parse_query_from_json(&line, qrels)
            .context("query should be parsed")?;
        if !query.relevant_docs.is_empty() {
            return Ok(Some(query));
        }
    }

    Ok(None)
}

fn parse_query_from_json(
    line: &str,
    qrels: &mut HashMap<u64, HashMap<DocId, Relevance>>,
) -> Result<Query> {
    let json: Map<String, Value> = serde_json::from_str(line)?;

    let id = parse_id(extract_string_from_json(&json, "_id")?)?;
    let text = extract_string_from_json(&json, "text")?.to_string();
    let relevant_docs = qrels.remove(&id).unwrap_or_default();

    Ok(Query {
        id,
        text,
        relevant_docs,
    })
}
