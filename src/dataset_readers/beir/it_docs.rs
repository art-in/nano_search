use std::fs::File;
use std::io::{BufRead, BufReader, Lines};

use serde_json::{Map, Value};

use super::model::BeirDatasetReader;
use super::utils::parse_id;
use crate::model::doc::{Doc, DocsSource};

pub struct BeirDocsIterator {
    lines: Lines<BufReader<File>>,
}

impl DocsSource for BeirDatasetReader {
    type Iter = BeirDocsIterator;

    fn docs(&self) -> Self::Iter {
        let file = File::open(self.dir.join("corpus.jsonl"))
            .expect("file should exist");
        let reader = BufReader::new(file);
        let lines = reader.lines();
        BeirDocsIterator { lines }
    }
}

impl Iterator for BeirDocsIterator {
    type Item = Doc;

    fn next(&mut self) -> Option<Self::Item> {
        self.lines
            .next()
            .map(|line| line.expect("line should be read"))
            .map(|line| {
                parse_doc_from_json(&line).expect("doc should be parsed")
            })
    }
}

fn parse_doc_from_json(line: &str) -> Option<Doc> {
    let json: Map<String, Value> = serde_json::from_str(line).ok()?;

    let id = parse_id(json.get("_id")?.as_str()?).ok()?;
    let title = json.get("title")?.as_str()?;
    let text = json.get("text")?.as_str()?;

    Some(Doc {
        id,
        text: format!("{title} {text}"),
    })
}
