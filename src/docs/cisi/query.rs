use std::{
    collections::{BTreeMap, HashSet},
    io::{BufRead, Read},
};

#[derive(Default, Clone)]
pub struct Query {
    pub id: u64,
    pub text: String,
    pub expected_docids: HashSet<u64>,
}

pub fn get_queries() -> Vec<Query> {
    let queries_file =
        std::fs::File::open("data/source/CISI.QRY").expect("file should exist");
    let query_docids_file =
        std::fs::File::open("data/source/CISI.REL").expect("file should exist");

    let mut queries = parse_queries(queries_file);
    parse_query_docids(query_docids_file, &mut queries);

    let mut res = Vec::new();
    for (_, query) in queries {
        res.push(query);
    }

    res
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

// parses query contents from CISI.QRY file
//
// each query in CISI.QRY is defined in sequence of text lines, grouped by sections.
//
// each section has following format:
//  required section header :  .SECTION_TYPE <data>
//  optional section content:  <data>
//  optional section content:  <data>
//  etc.
fn parse_queries<R: Read>(input: R) -> BTreeMap<u64, Query> {
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
                            .expect("query id should present on .I line")
                            .parse::<u64>()
                            .expect("query id should be integer");

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
                            .expect("query should be initialized");

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

    queries
}

// parses query docids from CISI.REL file
fn parse_query_docids<R: Read>(input: R, queries: &mut BTreeMap<u64, Query>) {
    let lines = std::io::BufReader::new(input).lines();

    for line in lines.map_while(Result::ok) {
        let parts: Vec<_> = line.split_whitespace().collect();

        let query_id = parts
            .first()
            .expect("first part should contain query id")
            .parse::<u64>()
            .expect("query id should be valid integer");

        let docid = parts
            .get(1)
            .expect("first part should contain docid")
            .parse::<u64>()
            .expect("docid should be valid integer");

        queries
            .get_mut(&query_id)
            .unwrap_or_else(|| {
                panic!("query with id {} should exist", query_id)
            })
            .expected_docids
            .insert(docid);
    }
}
