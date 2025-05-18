use super::serialize::BinarySerializable;
use crate::{
    model::engine::IndexStats,
    search_engines::nano::index::{
        memory::MemoryIndex,
        model::{DocPosting, Index, Term},
    },
};
use anyhow::{Context, Result};
use std::{collections::HashMap, fs::File, io::Seek, path::PathBuf};

pub struct FsIndex {
    terms: HashMap<Term, TermPostingListFileAddress>,
    postings_file: File,
    stats: IndexStats,
}

#[derive(Clone)]
pub struct TermPostingListFileAddress {
    pub postings_count: u64,
    pub start: u64,
    pub end: u64,
}

impl Index for FsIndex {
    fn get_doc_postings_for_term(
        &self,
        term: &Term,
    ) -> Option<(u64, Box<dyn Iterator<Item = DocPosting>>)> {
        self.terms.get(term).map(|term_posting_list_addr| {
            (
                term_posting_list_addr.postings_count,
                Box::new(FsDocPostingsIterator::new(
                    self.postings_file
                        .try_clone()
                        .expect("posting file handle should be clonned"),
                    term_posting_list_addr.clone(),
                )) as Box<dyn Iterator<Item = DocPosting>>,
            )
        })
    }
    fn get_index_stats(&self) -> &IndexStats {
        &self.stats
    }
}

pub struct FsDocPostingsIterator {
    postings_file: File,
    end_position: u64,
}

impl FsDocPostingsIterator {
    fn new(
        mut postings_file: File,
        address: TermPostingListFileAddress,
    ) -> Self {
        postings_file
            .seek(std::io::SeekFrom::Start(address.start))
            .expect("postings file position should be moved");

        FsDocPostingsIterator {
            postings_file,
            end_position: address.end,
        }
    }
}

impl Iterator for FsDocPostingsIterator {
    type Item = DocPosting;

    fn next(&mut self) -> Option<Self::Item> {
        let current_position = self
            .postings_file
            .stream_position()
            .expect("current position on postings file should be read");

        if current_position < self.end_position {
            let posting = DocPosting::deserialize(&mut self.postings_file)
                .expect("posting should be deserialized from file");
            Some(posting)
        } else {
            None
        }
    }
}

// fs index is built basically by serializing memory index into files,
// since it's currently easier to reuse index building logic this way.
// it works only for small index, which can fit entirely into memory, while
// big index should be split into segments.
// so index building logic will be reorganized in future:
// - either by abstracting and reusing for both memory and fs index,
// - or simply by removing memory index in favor of fs index
pub fn build_fs_index(
    memory_index: &MemoryIndex,
    index_dir: PathBuf,
) -> Result<FsIndex> {
    std::fs::remove_dir_all(&index_dir)
        .context("existing index dir should be removed")?;
    std::fs::create_dir(&index_dir).context("index dir should be created")?;

    let mut terms_file = File::create(index_dir.join("terms"))
        .context("terms file should be created")?;
    let mut postings_file = File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .read(true)
        .open(index_dir.join("postings"))
        .context("postings file should be created")?;

    memory_index.terms.len().serialize(&mut terms_file)?;

    let mut terms_posting_lists = HashMap::new();

    for (term, posting_list) in &memory_index.terms {
        let mut posting_list_file_address = TermPostingListFileAddress {
            postings_count: posting_list.len() as u64,
            start: 0,
            end: 0,
        };

        posting_list_file_address.postings_count = posting_list.len() as u64;
        posting_list_file_address.start = postings_file.stream_position()?;
        for doc_posting in posting_list.values() {
            doc_posting.serialize(&mut postings_file)?;
        }
        posting_list_file_address.end = postings_file.stream_position()?;

        terms_posting_lists
            .insert(term.clone(), posting_list_file_address.clone());

        term.serialize(&mut terms_file)?;
        posting_list_file_address.serialize(&mut terms_file)?;
    }

    let mut index_stats_file = File::create(index_dir.join("stats"))
        .context("stats file should be created")?;

    memory_index
        .stats
        .serialize(&mut index_stats_file)
        .context("stats should be serialized to file")?;

    open_fs_index(index_dir)
}

pub fn open_fs_index(index_dir: PathBuf) -> Result<FsIndex> {
    let mut terms_file = File::open(index_dir.join("terms"))
        .context("terms file should be created")?;
    let postings_file = File::open(index_dir.join("postings"))
        .context("postings file should be created")?;
    let mut index_stats_file = File::open(index_dir.join("stats"))
        .context("stats file should be created")?;

    let terms_len = usize::deserialize(&mut terms_file)?;

    let mut terms = HashMap::new();

    for _ in 0..terms_len {
        let term = String::deserialize(&mut terms_file)?;
        let term_posting_list_address =
            TermPostingListFileAddress::deserialize(&mut terms_file)?;
        terms.insert(term, term_posting_list_address);
    }

    Ok(FsIndex {
        terms,
        postings_file,
        stats: IndexStats::deserialize(&mut index_stats_file)?,
    })
}
