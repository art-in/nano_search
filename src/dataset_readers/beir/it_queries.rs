use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::{Map, Value};

use super::BeirDatasetReader;
use super::utils::parse_id;
use crate::eval::model::{QueriesSource, Query, Relevance};
use crate::model::doc::DocId;
use crate::utils::get_file_lines;

pub struct BeirQueriesIterator {
    lines: Box<dyn Iterator<Item = std::io::Result<String>>>,
    qrels: HashMap<u64, HashMap<DocId, Relevance>>,
}

impl QueriesSource for BeirDatasetReader {
    type Iter = BeirQueriesIterator;

    fn queries(&self) -> Result<Self::Iter> {
        let qrels = load_qrels(&self.qrels_file)?;
        let lines = get_file_lines(&self.queries_file)?;
        Ok(BeirQueriesIterator { lines, qrels })
    }
}

impl Iterator for BeirQueriesIterator {
    type Item = Query;

    fn next(&mut self) -> Option<Self::Item> {
        // skip queries lacking relevant docs to ensure evaluation is possible.
        // since reduced qrels, like test.tsv, may not have lines for each query
        for line in self.lines.by_ref() {
            let line = line.expect("line should be read");
            let query = parse_query_from_json(&line, &mut self.qrels)
                .expect("query should be parsed");
            if !query.relevant_docs.is_empty() {
                return Some(query);
            }
        }

        None
    }
}

fn parse_query_from_json(
    line: &str,
    qrels: &mut HashMap<u64, HashMap<DocId, Relevance>>,
) -> Result<Query> {
    let json: Map<String, Value> = serde_json::from_str(line)?;

    let query_id = parse_id(
        json.get("_id")
            .context("json should have _id field")?
            .as_str()
            .context("ID should be a string")?,
    )?;

    let text = json
        .get("text")
        .context("json should have text field")?
        .as_str()
        .context("text should be a string")?
        .to_string();

    let relevant_docs = qrels.remove(&query_id).unwrap_or_default();

    Ok(Query {
        id: query_id,
        text,
        relevant_docs,
    })
}

fn load_qrels(
    file_path: &Path,
) -> Result<HashMap<u64, HashMap<DocId, Relevance>>> {
    let lines = get_file_lines(file_path)?;
    let lines = lines.skip(1); // skip header line

    let mut map: HashMap<u64, HashMap<DocId, Relevance>> = HashMap::new();

    for line in lines {
        let line = line?;
        let mut parts = line.split_whitespace();

        let query_id = parse_id(parts.next().context("should read query ID")?)?;
        let doc_id = parse_id(parts.next().context("should read doc ID")?)?;
        let relevance: Relevance =
            parts.next().context("should read relevance")?.parse()?;

        // skip not relevant docs. they may appear in qrels/train.tsv for
        // contrast model training, and don't matter for evaluation
        if relevance <= 0.0 {
            continue;
        }

        map.entry(query_id).or_default().insert(doc_id, relevance);
    }

    Ok(map)
}
