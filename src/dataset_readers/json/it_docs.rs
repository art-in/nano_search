use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::{Map, Value};

use super::model::JsonDatasetReader;
use crate::model::doc::{Doc, DocsSource, ExternalDocId};

pub struct JsonDocsIterator {
    lines: Lines<BufReader<File>>,
    docid: ExternalDocId,
}

impl DocsSource for JsonDatasetReader {
    fn docs(&self) -> Result<Box<dyn Iterator<Item = Result<Doc>>>> {
        Ok(Box::new(JsonDocsIterator {
            lines: get_doc_lines(&self.file_path)?,
            docid: 0,
        }))
    }

    fn docs_count(&self) -> Result<Option<usize>> {
        Ok(Some(get_doc_lines(&self.file_path)?.count()))
    }
}

fn get_doc_lines(file_path: &Path) -> Result<Lines<BufReader<File>>> {
    let file = File::open(file_path).context("file should exist")?;
    let reader = BufReader::new(file);
    Ok(reader.lines())
}

impl Iterator for JsonDocsIterator {
    type Item = Result<Doc>;

    fn next(&mut self) -> Option<Self::Item> {
        let doc = self
            .lines
            .next()
            .map(|line| line.context("line should be read"))
            .map(|line| {
                parse_doc(&line?, self.docid).context("doc should be parsed")
            });

        self.docid += 1;

        doc
    }
}

fn parse_doc(json: &str, docid: ExternalDocId) -> Option<Doc> {
    let json_obj: Map<String, Value> = serde_json::from_str(json).ok()?;

    let body = json_obj.get("body")?.as_str()?.to_string();

    Some(Doc {
        id: docid,
        text: body,
    })
}
