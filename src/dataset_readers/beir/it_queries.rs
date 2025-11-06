use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::{Map, Value};

use crate::dataset_readers::beir::BeirDatasetReader;
use crate::dataset_readers::beir::utils::parse_id;
use crate::eval::model::{QueriesSource, Query};
use crate::model::doc::DocId;

pub struct BeirQueriesIterator {
    lines: Lines<BufReader<File>>,
    qrels: HashMap<u64, HashSet<DocId>>,
}

impl QueriesSource for BeirDatasetReader {
    type Iter = BeirQueriesIterator;

    fn queries(&self) -> Self::Iter {
        let qrels_path = self.dir.join("qrels/test.tsv");
        let qrels = load_qrels(&qrels_path).expect("qrels should be loaded");

        let file = File::open(self.dir.join("queries.jsonl"))
            .expect("file should exist");
        let reader = BufReader::new(file);
        let lines = reader.lines();

        BeirQueriesIterator { lines, qrels }
    }
}

impl Iterator for BeirQueriesIterator {
    type Item = Query;

    fn next(&mut self) -> Option<Self::Item> {
        // skip queries lacking qrels to ensure evaluation has expected docids.
        // reduced qrels, like test.tsv, may not have lines for each query
        for line in self.lines.by_ref() {
            let line = line.expect("line should be read");
            let query = parse_query_from_json(&line, &mut self.qrels)
                .expect("query should be parsed");
            if !query.relevant_docids.is_empty() {
                return Some(query);
            }
        }

        None
    }
}

fn parse_query_from_json(
    line: &str,
    qrels: &mut HashMap<u64, HashSet<DocId>>,
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

    let expected_docids = qrels.remove(&query_id).unwrap_or_default();

    Ok(Query {
        id: query_id,
        text,
        relevant_docids: expected_docids,
    })
}

fn load_qrels(
    file_path: impl AsRef<Path>,
) -> Result<HashMap<u64, HashSet<DocId>>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut map: HashMap<u64, HashSet<DocId>> = HashMap::new();

    let lines = reader.lines().skip(1); // skip header

    for line in lines {
        let line = line?;
        let mut parts = line.split_whitespace();

        let query_id = parse_id(parts.next().context("should read query ID")?)?;
        let doc_id = parse_id(parts.next().context("should read doc ID")?)?;

        map.entry(query_id).or_default().insert(doc_id);
    }

    Ok(map)
}
