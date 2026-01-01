use anyhow::{Context, Result};

use super::model::CisiDatasetReader;
use crate::model::doc::{Doc, DocsSource};
use crate::utils::get_file_lines;

pub struct CisiDocsIterator {
    lines: Box<dyn Iterator<Item = std::io::Result<String>>>,
    current_line_type: ELineType,
    current_doc: Option<Doc>,
}

impl DocsSource for CisiDatasetReader {
    type Iter = CisiDocsIterator;

    fn docs(&self) -> Result<Self::Iter> {
        Ok(CisiDocsIterator {
            lines: get_file_lines(&self.docs_file)?,
            current_line_type: ELineType::Unknown,
            current_doc: None,
        })
    }

    fn docs_count(&self) -> Result<Option<usize>> {
        Ok(Some(get_file_lines(&self.docs_file)?.count()))
    }
}

impl Iterator for CisiDocsIterator {
    type Item = Doc;

    fn next(&mut self) -> Option<Self::Item> {
        read_next_doc(self).expect("next doc should be read")
    }
}

#[derive(Clone, Copy)]
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

fn read_next_doc(it: &mut CisiDocsIterator) -> Result<Option<Doc>> {
    for line in it.lines.by_ref().map_while(Result::ok) {
        if line.starts_with(".I") {
            it.current_line_type =
                ELineType::SectionHeader(ESectionType::DocId);
        } else if line.starts_with(".T") {
            it.current_line_type =
                ELineType::SectionHeader(ESectionType::Title);
        } else if line.starts_with(".A") {
            it.current_line_type =
                ELineType::SectionHeader(ESectionType::Author);
        } else if line.starts_with(".W") {
            it.current_line_type = ELineType::SectionHeader(ESectionType::Text);
        } else if line.starts_with(".X") {
            it.current_line_type = ELineType::SectionHeader(ESectionType::Refs);
        } else if let ELineType::SectionHeader(t) = &it.current_line_type {
            it.current_line_type = ELineType::SectionContent(*t);
        }

        match it.current_line_type {
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

                        let prev_doc = it.current_doc.take();

                        it.current_doc = Some(Doc {
                            id: docid,
                            text: String::new(),
                        });

                        if prev_doc.is_some() {
                            return Ok(prev_doc);
                        }
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
                        it.current_doc
                            .as_mut()
                            .context("doc should be initialized")?
                            .text += &line;
                    }
                }
            }
        }
    }

    Ok(None)
}
