use std::fs::File;
use std::io::{BufRead, BufReader, Lines};

use serde_json::{Map, Value};

use super::model::JsonDatasetReader;
use crate::model::doc::{Doc, DocId, DocsSource};

pub struct JsonDocsIterator {
    lines: Lines<BufReader<File>>,
    docid: DocId,
}

impl DocsSource for JsonDatasetReader {
    type Iter = JsonDocsIterator;

    fn docs(&self) -> Self::Iter {
        let file = File::open(&self.file_path).expect("file should exist");
        let reader = BufReader::new(file);

        JsonDocsIterator {
            lines: reader.lines(),
            docid: 0,
        }
    }
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
