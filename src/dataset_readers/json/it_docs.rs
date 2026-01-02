use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::Path;

use anyhow::Result;
use serde_json::{Map, Value};

use super::model::JsonDatasetReader;
use crate::model::doc::{Doc, DocId, DocsSource};

pub struct JsonDocsIterator {
    lines: Lines<BufReader<File>>,
    docid: DocId,
}

impl DocsSource for JsonDatasetReader {
    fn docs(&self) -> Result<Box<dyn Iterator<Item = Doc>>> {
        Ok(Box::new(JsonDocsIterator {
            lines: get_doc_lines(&self.file_path),
            docid: 0,
        }))
    }

    fn docs_count(&self) -> Result<Option<usize>> {
        Ok(Some(get_doc_lines(&self.file_path).count()))
    }
}

fn get_doc_lines(file_path: &Path) -> Lines<BufReader<File>> {
    let file = File::open(file_path).expect("file should exist");
    let reader = BufReader::new(file);
    reader.lines()
}

impl Iterator for JsonDocsIterator {
    type Item = Doc;

    fn next(&mut self) -> Option<Self::Item> {
        let doc = self
            .lines
            .next()
            .map(|line| line.expect("should read line"))
            .map(|line| {
                parse_doc_from_json(&line, self.docid)
                    .expect("doc should be parsed from json string")
            });

        self.docid += 1;

        doc
    }
}

fn parse_doc_from_json(json: &str, docid: DocId) -> Option<Doc> {
    let json_obj: Map<String, Value> = serde_json::from_str(json).ok()?;

    let body = json_obj.get("body")?.as_str()?.to_string();

    Some(Doc {
        id: docid,
        text: body,
    })
}
