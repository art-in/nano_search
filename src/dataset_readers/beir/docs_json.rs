use std::path::PathBuf;

use anyhow::Result;
use serde_json::{Map, Value};

use super::utils::{extract_string_from_json, parse_id};
use crate::model::doc::{Doc, DocsSource};
use crate::utils::get_file_lines;

pub struct BeirDocsJsonReader {
    docs_file: PathBuf,
}

impl BeirDocsJsonReader {
    pub const fn new(docs_file: PathBuf) -> Self {
        Self { docs_file }
    }
}

impl DocsSource for BeirDocsJsonReader {
    fn docs(&self) -> Result<Box<dyn Iterator<Item = Doc>>> {
        Ok(Box::new(BeirDocsJsonIterator {
            lines: get_file_lines(&self.docs_file)?,
        }))
    }
    fn docs_count(&self) -> Result<Option<usize>> {
        Ok(Some(get_file_lines(&self.docs_file)?.count()))
    }
}

struct BeirDocsJsonIterator {
    lines: Box<dyn Iterator<Item = std::io::Result<String>>>,
}

impl Iterator for BeirDocsJsonIterator {
    type Item = Doc;

    fn next(&mut self) -> Option<Self::Item> {
        self.lines
            .next()
            // TODO: avoid expect-panics in all dataset readers,
            // use Iterator<Item = Result<Doc/Query>> instead
            .map(|line| line.expect("line should be read"))
            .map(|line| {
                parse_doc_from_json(&line).expect("doc should be parsed")
            })
    }
}

fn parse_doc_from_json(line: &str) -> Result<Doc> {
    let json: Map<String, Value> = serde_json::from_str(line)?;

    let id = parse_id(extract_string_from_json(&json, "_id")?)?;
    let title = extract_string_from_json(&json, "title")?;
    let text = extract_string_from_json(&json, "text")?;

    Ok(Doc {
        id,
        text: format!("{title} {text}"),
    })
}
