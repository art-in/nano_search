use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use ns_engine::{ExternalDocId, Score};
use ns_eval::QueryId;

use super::utils::parse_id;
use crate::utils::get_file_lines;

pub fn load_qrels(
    file_path: &Path,
) -> Result<HashMap<QueryId, HashMap<ExternalDocId, Score>>> {
    let lines = get_file_lines(file_path)?;
    let lines = lines.skip(1); // skip header line

    let mut map: HashMap<u64, HashMap<ExternalDocId, Score>> = HashMap::new();

    for line in lines {
        let line = line?;
        let mut parts = line.split_whitespace();

        let query_id = parse_id(parts.next().context("should read query ID")?)?;
        let doc_id = parse_id(parts.next().context("should read doc ID")?)?;
        let score = parts.next().context("should read score")?.parse()?;

        // skip not relevant docs. they may appear in qrels/train.tsv for
        // contrast model training, and don't matter for evaluation
        if score <= 0.0 {
            continue;
        }

        map.entry(query_id).or_default().insert(doc_id, score);
    }

    Ok(map)
}
