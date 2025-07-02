use std::fs::File;
use std::io::{BufRead, BufReader, Lines};

use serde_json::{Map, Value};

use super::model::WikiDocs;
use crate::model::doc::{Doc, DocId};

pub struct WikiDocsIterator {
    lines: Lines<BufReader<File>>,
    docid: DocId,
}

impl IntoIterator for WikiDocs {
    type Item = Doc;
    type IntoIter = WikiDocsIterator;

    fn into_iter(self) -> Self::IntoIter {
        let file = File::open(self.file_path).expect("file should exist");
        let reader = BufReader::new(file);

        WikiDocsIterator {
            lines: reader.lines(),
            docid: 0,
        }
    }
}

impl Iterator for WikiDocsIterator {
    type Item = Doc;

    fn next(&mut self) -> Option<Self::Item> {
        let doc = self
            .lines
            .next()
            .map(|line| line.expect("should read line"))
            .map(|line| {
                parse_json_doc(&line, self.docid)
                    .expect("doc should be parsed from json string")
            });

        self.docid += 1;

        doc
    }
}

fn parse_json_doc(json_string: &str, docid: DocId) -> Option<Doc> {
    let json_obj: Map<String, Value> =
        serde_json::from_str(json_string).ok()?;

    let body = json_obj.get("body")?.as_str()?.to_string();

    Some(Doc {
        id: docid,
        text: body,
    })
}
