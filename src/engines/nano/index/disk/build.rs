use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Seek};
use std::path::Path;

use anyhow::{Context, Result};
use itertools::Itertools;

use super::model::{
    DiskIndex, DiskIndexSegment, IndexFile, TermPostingListFileAddress,
};
use super::serialize::BinarySerializable;
use crate::engines::nano::index::memory::{MemoryIndex, build_memory_index};
use crate::engines::nano::index::model::IndexSegmentStats;
use crate::model::doc::Doc;

const SEGMENT_DIR_PREFIX: &str = "segment-";
const SEGMENT_MAX_DOCS: usize = 250000;

pub fn build_disk_index(
    docs: &mut dyn Iterator<Item = Doc>,
    index_dir: impl AsRef<Path>,
) -> Result<DiskIndex> {
    let mut segments = Vec::new();

    for docs_chunk in &docs.chunks(SEGMENT_MAX_DOCS) {
        let memory_index = build_memory_index(&mut docs_chunk.into_iter());
        let segment = build_disk_index_segment(&memory_index, &index_dir)?;
        segments.push(segment);
    }

    Ok(DiskIndex { segments })
}

fn build_disk_index_segment(
    memory_index: &MemoryIndex,
    index_dir: impl AsRef<Path>,
) -> Result<DiskIndexSegment> {
    let segment_id = uuid::Uuid::new_v4().as_simple().to_string();
    let segment_dir_name = SEGMENT_DIR_PREFIX.to_string() + &segment_id;
    let segment_dir = index_dir.as_ref().join(segment_dir_name);
    fs::create_dir(&segment_dir)?;

    let mut terms_file = create_writer(&segment_dir, IndexFile::Terms)?;
    let mut postings_file = create_writer(&segment_dir, IndexFile::Postings)?;
    let mut stats_file = create_writer(&segment_dir, IndexFile::Stats)?;

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

    Ok(DiskIndexSegment {
        terms,
        postings_file: File::open(
            segment_dir.join(IndexFile::Postings.name()),
        )?,
        stats: memory_index.stats.clone(),
    })
}

pub fn open_disk_index(index_dir: &Path) -> Result<DiskIndex> {
    let mut segments = Vec::new();

    for entry in fs::read_dir(index_dir)? {
        let entry = entry?;
        if entry.path().is_dir() {
            let segment = open_disk_index_segment(&entry.path())?;
            segments.push(segment);
        }
    }

    Ok(DiskIndex { segments })
}

fn open_disk_index_segment(segment_dir: &Path) -> Result<DiskIndexSegment> {
    let mut terms_file = open_reader(segment_dir, IndexFile::Terms)?;
    let mut stats_file = open_reader(segment_dir, IndexFile::Stats)?;

    let terms = HashMap::<String, TermPostingListFileAddress>::deserialize(
        &mut terms_file,
    )?;
    let stats = IndexSegmentStats::deserialize(&mut stats_file)?;

    Ok(DiskIndexSegment {
        terms,
        postings_file: File::open(
            segment_dir.join(IndexFile::Postings.name()),
        )?,
        stats,
    })
}

fn create_writer(
    dir: impl AsRef<Path>,
    file: IndexFile,
) -> Result<BufWriter<File>> {
    let filename = file.name();
    let file = File::create(dir.as_ref().join(filename))
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
