use anyhow::Result;
use serde_json::{Map, Value};

use super::model::BeirDatasetReader;
use super::utils::{extract_string, parse_id};
use crate::model::doc::{Doc, DocsSource};
use crate::utils::get_file_lines;

pub struct BeirDocsIterator {
    lines: Box<dyn Iterator<Item = std::io::Result<String>>>,
}

impl DocsSource for BeirDatasetReader {
    fn docs(&self) -> Result<Box<dyn Iterator<Item = Doc>>> {
        Ok(Box::new(BeirDocsIterator {
            lines: get_file_lines(&self.docs_file)?,
        }))
    }

    fn docs_count(&self) -> Result<Option<usize>> {
        Ok(Some(get_file_lines(&self.docs_file)?.count()))
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

fn parse_doc_from_json(line: &str) -> Result<Doc> {
    let json: Map<String, Value> = serde_json::from_str(line)?;

    let id = parse_id(extract_string(&json, "_id")?)?;
    let title = extract_string(&json, "title")?;
    let text = extract_string(&json, "text")?;

    Ok(Doc {
        id,
        text: format!("{title} {text}"),
    })
}
