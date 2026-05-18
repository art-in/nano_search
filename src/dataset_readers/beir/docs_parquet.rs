use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use parquet::record::{Field, Row};

use super::utils::parse_id;
use crate::model::doc::{Doc, DocsSource};
use crate::utils::get_parquet_rows;

pub struct BeirDocsParquetReader {
    pub docs_files: Vec<PathBuf>,
}

impl BeirDocsParquetReader {
    pub const fn new(docs_files: Vec<PathBuf>) -> Self {
        Self { docs_files }
    }
}

impl DocsSource for BeirDocsParquetReader {
    fn docs(&self) -> Result<Box<dyn Iterator<Item = Doc>>> {
        Ok(Box::new(BeirDocsParquetIterator {
            rows: get_parquet_rows(self.docs_files.clone())?,
        }))
    }
    fn docs_count(&self) -> Result<Option<usize>> {
        Ok(Some(get_parquet_rows(self.docs_files.clone())?.count()))
    }
}

struct BeirDocsParquetIterator {
    rows: Box<dyn Iterator<Item = Result<Row>>>,
}

impl Iterator for BeirDocsParquetIterator {
    type Item = Doc;

    fn next(&mut self) -> Option<Self::Item> {
        self.rows
            .next()
            .map(|row| row.expect("row should be read"))
            .map(|row| parse_doc_from_row(&row).expect("doc should be parsed"))
    }
}

pub fn parse_doc_from_row(row: &Row) -> Result<Doc> {
    let mut id = None;
    let mut title = None;
    let mut text = None;

    for (name, field) in row.get_column_iter() {
        match name.as_str() {
            "_id" => {
                if let Field::Str(val) = field {
                    id = Some(val);
                }
            }
            "title" => {
                if let Field::Str(val) = field {
                    title = Some(val);
                }
            }
            "text" => {
                if let Field::Str(val) = field {
                    text = Some(val);
                }
            }
            _ => {
                bail!("unknown row column: {}", name)
            }
        }
    }

    let id = parse_id(id.context("id column should exist")?)?;
    let title = title.context("title column should exist")?;
    let text = text.context("text column should exist")?;

    Ok(Doc {
        id,
        text: format!("{title} {text}"),
    })
}
