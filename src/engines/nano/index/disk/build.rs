use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Seek};
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;

use anyhow::{Context, Result, anyhow};
use crossbeam_channel::Receiver;
use itertools::Itertools;
use memmap2::Mmap;

use super::model::{
    DiskIndex, DiskIndexSegment, IndexFile, TermPostingListFileAddress,
};
use super::serialize::BinarySerializable;
use crate::engines::nano::index::disk::DiskIndexOptions;
use crate::engines::nano::index::memory::{MemoryIndex, build_memory_index};
use crate::engines::nano::index::model::IndexSegmentStats;
use crate::model::doc::{Doc, DocId};

const SEGMENT_DIR_PREFIX: &str = "segment-";

// limit number of index threads to not create too much segments
const MAX_INDEX_THREADS: usize = 10;
const DOCS_CHANNEL_CAPACITY: usize = 10_000;

pub fn build_disk_index(
    docs: &mut dyn Iterator<Item = Doc>,
    opts: &DiskIndexOptions,
) -> Result<DiskIndex> {
    let (docs_sender, docs_receiver) =
        crossbeam_channel::bounded(DOCS_CHANNEL_CAPACITY);

    let mut thread_handles = Vec::new();

    let threads_count = opts
        .index_threads
        .unwrap_or(std::thread::available_parallelism()?.get())
        .min(MAX_INDEX_THREADS);

    for thread_idx in 0..threads_count {
        let handle = spawn_indexer_thread(
            thread_idx,
            docs_receiver.clone(),
            opts.max_segment_docs,
            opts.index_dir.clone(),
        )?;
        thread_handles.push(handle);
    }

    for doc in docs {
        docs_sender.send(doc)?;
    }

    // drop sender, so threads may exit from wait loop
    drop(docs_sender);

    let mut segments = Vec::new();

    for handle in thread_handles {
        let thread_segments = handle
            .join()
            .map_err(|_| anyhow!("index thread should be joined"))??;
        segments.extend(thread_segments);
    }

    Ok(DiskIndex { segments })
}

fn spawn_indexer_thread(
    thread_idx: usize,
    docs_receiver: Receiver<Doc>,
    max_segment_docs: usize,
    index_dir: PathBuf,
) -> Result<JoinHandle<Result<Vec<DiskIndexSegment>>>> {
    let handle = std::thread::Builder::new()
        .name(format!("indexer-{}", thread_idx))
        .spawn(move || -> Result<_> {
            let mut segments = Vec::new();

            let docs_chunks =
                docs_receiver.into_iter().chunks(max_segment_docs);

            for docs_chunk in &docs_chunks {
                let mem_idx = build_memory_index(&mut docs_chunk.into_iter());
                let segment = build_disk_index_segment(mem_idx, &index_dir)?;
                segments.push(segment);
            }

            Ok(segments)
        })?;

    Ok(handle)
}

fn build_disk_index_segment(
    memory_index: MemoryIndex,
    index_dir: impl AsRef<Path>,
) -> Result<DiskIndexSegment> {
    let segment_id = uuid::Uuid::new_v4().as_simple().to_string();
    let segment_dir_name = SEGMENT_DIR_PREFIX.to_string() + &segment_id;
    let segment_dir = index_dir.as_ref().join(segment_dir_name);
    fs::create_dir(&segment_dir)?;

    let mut terms_file = create_writer(&segment_dir, IndexFile::Terms)?;
    let mut postings_file = create_writer(&segment_dir, IndexFile::Postings)?;
    let mut doclen_file = create_writer(&segment_dir, IndexFile::DocLen)?;
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
            start_byte: start_byte as usize,
            end_byte: end_byte as usize,
        };

        terms.insert(term.clone(), address);
    }

    terms
        .serialize(&mut terms_file)
        .context("terms should be serialized to file")?;

    memory_index.doc_terms_count.serialize(&mut doclen_file)?;

    memory_index
        .stats
        .serialize(&mut stats_file)
        .context("stats should be serialized to file")?;

    let postings_file =
        mmap_file(segment_dir.join(IndexFile::Postings.name()))?;

    Ok(DiskIndexSegment {
        terms,
        postings_file,
        doc_terms_count: memory_index.doc_terms_count,
        stats: memory_index.stats,
    })
}

pub fn open_disk_index(options: &DiskIndexOptions) -> Result<DiskIndex> {
    let mut segments = Vec::new();

    for entry in fs::read_dir(&options.index_dir)? {
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
    let mut doclen_file = open_reader(segment_dir, IndexFile::DocLen)?;
    let mut stats_file = open_reader(segment_dir, IndexFile::Stats)?;
    let postings_file_name = segment_dir.join(IndexFile::Postings.name());

    let terms = HashMap::<String, TermPostingListFileAddress>::deserialize(
        &mut terms_file,
    )?;
    let doc_terms_count = HashMap::<DocId, u16>::deserialize(&mut doclen_file)?;
    let stats = IndexSegmentStats::deserialize(&mut stats_file)?;
    let postings_file = mmap_file(postings_file_name)?;

    Ok(DiskIndexSegment {
        terms,
        postings_file,
        doc_terms_count,
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

fn mmap_file(file_path: impl AsRef<Path>) -> Result<Mmap> {
    let file = File::open(file_path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    Ok(mmap)
}
