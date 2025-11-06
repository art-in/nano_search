use std::cell::RefCell;
use std::fs::File;
use std::io::BufRead;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::{Context, Result};

use super::model::CisiDocs;
use crate::model::doc::Doc;

enum ESectionType {
    DocId,
    Title,
    Author,
    Text,
    Refs,
}

enum ELineType {
    Unknown,
    SectionHeader(ESectionType),
    SectionContent(ESectionType),
}

pub fn load_docs() -> Result<CisiDocs> {
    load_docs_from("datasets/cisi/CISI.ALL".into())
}

fn load_docs_from(file_path: PathBuf) -> Result<CisiDocs> {
    let source_file =
        File::open(file_path).context("cisi file should exist")?;
    let source_file_reader = std::io::BufReader::new(source_file);

    let mut docs = Vec::new();

    let mut current_line_type = ELineType::Unknown;
    let mut current_doc: Option<Doc> = None;

    for line in source_file_reader.lines().map_while(Result::ok) {
        if line.starts_with(".I") {
            current_line_type = ELineType::SectionHeader(ESectionType::DocId);
        } else if line.starts_with(".T") {
            current_line_type = ELineType::SectionHeader(ESectionType::Title);
        } else if line.starts_with(".A") {
            current_line_type = ELineType::SectionHeader(ESectionType::Author);
        } else if line.starts_with(".W") {
            current_line_type = ELineType::SectionHeader(ESectionType::Text);
        } else if line.starts_with(".X") {
            current_line_type = ELineType::SectionHeader(ESectionType::Refs);
        } else if let ELineType::SectionHeader(t) = current_line_type {
            current_line_type = ELineType::SectionContent(t);
        }

        match current_line_type {
            ELineType::Unknown => {
                panic!("unknown line type")
            }
            ELineType::SectionHeader(ref section_type) => {
                match section_type {
                    ESectionType::DocId => {
                        let parts: Vec<_> = line.split(' ').collect();
                        let docid = parts[1];
                        let docid = docid
                            .parse::<u64>()
                            .context("docid in line should be valid integer")?;

                        if let Some(doc) = current_doc.as_ref() {
                            docs.push(doc.clone());
                        }

                        current_doc = Some(Doc {
                            id: docid,
                            text: String::new(),
                        });
                    }
                    _default => {
                        // skip
                    }
                }
            }
            ELineType::SectionContent(ref section_type) => {
                match section_type {
                    ESectionType::Refs => {
                        // skip
                    }
                    _default => {
                        current_doc
                            .as_mut()
                            .context("doc should be initialized")?
                            .text += &line;
                    }
                }
            }
        }
    }

    Ok(CisiDocs {
        docs: Rc::new(RefCell::new(docs)),
    })
}
