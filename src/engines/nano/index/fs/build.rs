use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Seek};
use std::path::Path;

use anyhow::{Context, Result};

use super::model::{FsIndex, IndexFile, TermPostingListFileAddress};
use super::serialize::BinarySerializable;
use crate::engines::nano::index::memory::MemoryIndex;
use crate::model::engine::IndexStats;

// fs index is built basically by serializing memory index into files,
// since it's currently easier to reuse index building logic this way.
pub fn build_fs_index(
    memory_index: &MemoryIndex,
    index_dir: impl AsRef<Path>,
) -> Result<FsIndex> {
    let mut terms_file = create_writer(&index_dir, IndexFile::Terms)?;
    let mut postings_file = create_writer(&index_dir, IndexFile::Postings)?;
    let mut stats_file = create_writer(&index_dir, IndexFile::Stats)?;

    let mut terms = HashMap::new();

    for (term, posting_list) in &memory_index.terms {
        let start_byte = postings_file.stream_position()?;
        for posting in posting_list.values() {
            posting.serialize(&mut postings_file)?;
        }
        let end_byte = postings_file.stream_position()?;

        let address = TermPostingListFileAddress {
            postings_count: posting_list.len(),
            start_byte,
            end_byte,
        };

        terms.insert(term.clone(), address);
    }

    terms
        .serialize(&mut terms_file)
        .context("terms should be serialized to file")?;

    memory_index
        .stats
        .serialize(&mut stats_file)
        .context("stats should be serialized to file")?;

    Ok(FsIndex {
        terms,
        postings_file: File::open(
            index_dir.as_ref().join(IndexFile::Postings.name()),
        )?,
        stats: memory_index.stats.clone(),
    })
}

pub fn open_fs_index(index_dir: &Path) -> Result<FsIndex> {
    let mut terms_file = open_reader(index_dir, IndexFile::Terms)?;
    let mut stats_file = open_reader(index_dir, IndexFile::Stats)?;

    let terms = HashMap::<String, TermPostingListFileAddress>::deserialize(
        &mut terms_file,
    )?;
    let stats = IndexStats::deserialize(&mut stats_file)?;

    Ok(FsIndex {
        terms,
        postings_file: File::open(index_dir.join(IndexFile::Postings.name()))?,
        stats,
    })
}

fn create_writer(
    dif: impl AsRef<Path>,
    file: IndexFile,
) -> Result<BufWriter<File>> {
    let filename = file.name();
    let file = File::create(dif.as_ref().join(filename))
        .with_context(|| format!("{filename} file should be created"))?;
    Ok(BufWriter::new(file))
}

fn open_reader(
    dir: impl AsRef<Path>,
    file: IndexFile,
) -> Result<BufReader<File>> {
    let filename = file.name();
    let file = File::open(dir.as_ref().join(filename))
        .with_context(|| format!("{filename} file should be opened"))?;
    Ok(BufReader::new(file))
}
