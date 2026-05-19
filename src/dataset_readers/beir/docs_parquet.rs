use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use parquet::record::Row;

use super::utils::{extract_string_from_parquet, parse_id};
use crate::model::doc::{Doc, DocsSource};
use crate::utils::get_parquet_rows;

pub struct BeirDocsParquetReader {
    docs_files: Vec<PathBuf>,
}

impl BeirDocsParquetReader {
    pub const fn new(docs_files: Vec<PathBuf>) -> Self {
        Self { docs_files }
    }
}

impl DocsSource for BeirDocsParquetReader {
    fn docs(&self) -> Result<Box<dyn Iterator<Item = Result<Doc>>>> {
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
    type Item = Result<Doc>;

    fn next(&mut self) -> Option<Self::Item> {
        self.rows
            .next()
            .map(|row| row.context("row should be read"))
            .map(|row| parse_doc(&row?).context("doc should be parsed"))
    }
}

pub fn parse_doc(row: &Row) -> Result<Doc> {
    let mut id = None;
    let mut title = None;
    let mut text = None;

    for (name, field) in row.get_column_iter() {
        match name.as_str() {
            "_id" => id = Some(extract_string_from_parquet(field)?),
            "title" => title = Some(extract_string_from_parquet(field)?),
            "text" => text = Some(extract_string_from_parquet(field)?),
            _ => bail!("unknown row column: {}", name),
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
