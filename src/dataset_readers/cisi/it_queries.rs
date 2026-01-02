use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

use anyhow::{Context, Result};

use crate::dataset_readers::cisi::model::CisiDatasetReader;
use crate::eval::model::{QueriesSource, Query};

impl QueriesSource for CisiDatasetReader {
    fn queries(&self) -> Result<Box<dyn Iterator<Item = Query>>> {
        let queries_file = BufReader::new(File::open(&self.queries_file)?);
        let qrels_file = BufReader::new(File::open(&self.qrels_file)?);

        let mut queries = read_queries(queries_file)?;
        read_qrels(qrels_file, &mut queries)?;

        let queries = queries.into_values().collect::<Vec<Query>>();
        Ok(Box::new(queries.into_iter()))
    }
}

enum ELineType {
    SectionHeader(ESectionType),
    SectionContent(ESectionType),
}

#[derive(Clone, Copy)]
enum ESectionType {
    Unknown,
    QueryId,
    QueryText,
}

/// Parses query contents from CISI.QRY file
///
/// Each query in CISI.QRY is defined in sequence of text lines, grouped by
/// sections.
///
/// Each section has following format:
///  required section header :  .SECTION_TYPE <data>
///  optional section content:  <data>
///  optional section content:  <data>
///  etc.
fn read_queries<R: Read>(input: R) -> Result<BTreeMap<u64, Query>> {
    let lines = std::io::BufReader::new(input).lines();

    let mut queries: BTreeMap<u64, Query> = BTreeMap::new();

    let mut current_query: Option<Query> = None;
    let mut current_line_type: Option<ELineType> = None;

    for line in lines.map_while(Result::ok) {
        current_line_type = if line.starts_with('.') {
            // this line is section header
            if line.starts_with(".I") {
                Some(ELineType::SectionHeader(ESectionType::QueryId))
            } else if line.starts_with(".W") {
                Some(ELineType::SectionHeader(ESectionType::QueryText))
            } else {
                Some(ELineType::SectionHeader(ESectionType::Unknown))
            }
        } else {
            // this line is section content
            match current_line_type {
                Some(ELineType::SectionHeader(section_type)) => {
                    // first content line in section after header
                    Some(ELineType::SectionContent(section_type))
                }
                Some(ELineType::SectionContent(_)) => {
                    // another content line of same section
                    current_line_type
                }
                _default => {
                    panic!("content line should go after section header");
                }
            }
        };

        match current_line_type {
            Some(ELineType::SectionHeader(section_type)) => {
                match section_type {
                    ESectionType::QueryId => {
                        if let Some(current_query) = current_query {
                            // met id header of another query.
                            // consider current query finalized, dump it to
                            // result vector and proceed to next query
                            queries.insert(current_query.id, current_query);
                        }

                        let mut query = Query::default();

                        let parts: Vec<_> = line.split(' ').collect();
                        query.id = parts
                            .get(1)
                            .context("query id should present on .I line")?
                            .parse::<u64>()
                            .context("query id should be integer")?;

                        current_query = Some(query);
                    }
                    _default => {
                        // skip headers of other sections
                    }
                }
            }
            Some(ELineType::SectionContent(section_type)) => {
                match section_type {
                    ESectionType::QueryText => {
                        let current_query = current_query
                            .as_mut()
                            .context("query should be initialized")?;

                        current_query.text += &line;
                        current_query.text += " ";
                    }
                    _default => {
                        // skip contents of other sections
                    }
                }
            }
            _default => {
                panic!("unknown line type");
            }
        }
    }

    if let Some(current_query) = current_query {
        queries.insert(current_query.id, current_query);
    }

    Ok(queries)
}

/// Parses query docids from CISI.REL file
fn read_qrels<R: Read>(
    input: R,
    queries: &mut BTreeMap<u64, Query>,
) -> Result<()> {
    let lines = std::io::BufReader::new(input).lines();

    for line in lines.map_while(Result::ok) {
        let parts: Vec<_> = line.split_whitespace().collect();

        let query_id = parts
            .first()
            .context("first part should contain query id")?
            .parse::<u64>()
            .context("query id should be valid integer")?;

        let docid = parts
            .get(1)
            .context("first part should contain docid")?
            .parse::<u64>()
            .context("docid should be valid integer")?;

        queries
            .get_mut(&query_id)
            .context("query should exist")?
            .relevant_docs
            // relevance is not provided in this dataset, so default to 1.0
            .insert(docid, 1.0);
    }

    Ok(())
}
