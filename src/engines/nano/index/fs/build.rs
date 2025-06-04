use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Seek};
use std::path::Path;

use anyhow::{Context, Result};

use super::model::{FsIndex, TermPostingListFileAddress};
use super::serialize::BinarySerializable;
use crate::engines::nano::index::memory::MemoryIndex;
use crate::model::engine::IndexStats;

// fs index is built basically by serializing memory index into files,
// since it's currently easier to reuse index building logic this way.
pub fn build_fs_index(
    memory_index: &MemoryIndex,
    index_dir: impl AsRef<Path>,
) -> Result<FsIndex> {
    let mut terms_file_writer = BufWriter::new(
        File::create(index_dir.as_ref().join("terms"))
            .context("terms file should be created")?,
    );
    let mut postings_file_writer = BufWriter::new(
        File::create(index_dir.as_ref().join("postings"))
            .context("postings file should be created")?,
    );
    let mut index_stats_file_writer = BufWriter::new(
        File::create(index_dir.as_ref().join("stats"))
            .context("stats file should be created")?,
    );

    let mut terms = HashMap::new();

    for (term, posting_list) in &memory_index.terms {
        let start_byte = postings_file_writer.stream_position()?;
        for posting in posting_list.values() {
            posting.serialize(&mut postings_file_writer)?;
        }
        let end_byte = postings_file_writer.stream_position()?;

        let address = TermPostingListFileAddress {
            postings_count: posting_list.len(),
            start_byte,
            end_byte,
        };

        terms.insert(term.clone(), address);
    }

    terms
        .serialize(&mut terms_file_writer)
        .context("terms should be serialized to file")?;

    memory_index
        .stats
        .serialize(&mut index_stats_file_writer)
        .context("stats should be serialized to file")?;

    Ok(FsIndex {
        terms: terms.into_iter().collect(),
        postings_file: File::open(index_dir.as_ref().join("postings"))?,
        stats: memory_index.stats.clone(),
    })
}

pub fn open_fs_index(index_dir: &Path) -> Result<FsIndex> {
    let mut terms_file_reader = BufReader::new(
        File::open(index_dir.join("terms"))
            .context("terms file should be opened")?,
    );
    let postings_file = File::open(index_dir.join("postings"))
        .context("postings file should be opened")?;
    let mut index_stats_file_reader = BufReader::new(
        File::open(index_dir.join("stats"))
            .context("stats file should be opened")?,
    );

    let terms = HashMap::<String, TermPostingListFileAddress>::deserialize(
        &mut terms_file_reader,
    )?;
    let stats = IndexStats::deserialize(&mut index_stats_file_reader)?;

    Ok(FsIndex {
        terms,
        postings_file,
        stats,
    })
}
