use anyhow::{Context, Result};
use std::{collections::HashSet, io::BufRead};

pub fn parse_stop_words() -> Result<HashSet<String>> {
    let source_file = std::fs::File::open("data/stop_words/stop_words.txt")
        .context("file should exist")?;
    let source_file_reader = std::io::BufReader::new(source_file);

    let mut stop_words = HashSet::new();

    for line in source_file_reader.lines().map_while(Result::ok) {
        stop_words.insert(line);
    }

    Ok(stop_words)
}
