use super::docs::CisiDocs;
use crate::model::doc::Doc;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::BufRead;
use std::path::PathBuf;

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

// splits CISI.ALL file to separate doc files
pub fn parse(file_path: PathBuf) -> Result<CisiDocs> {
    let source_file =
        File::open(file_path).context("cisi file should exist")?;
    let source_file_reader = std::io::BufReader::new(source_file);

    let mut current_line_type = ELineType::Unknown;

    let mut cisi_docs: CisiDocs = CisiDocs { docs: Vec::new() };
    let mut doc: Option<Doc> = None;

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

                        if let Some(doc) = doc.as_ref() {
                            cisi_docs.docs.push(doc.clone());
                        }

                        doc = Some(Doc {
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
                        doc.as_mut()
                            .context("doc should be initialized")?
                            .text += &line;
                    }
                }
            }
        }
    }

    Ok(cisi_docs)
}
