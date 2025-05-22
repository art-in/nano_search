use super::serialize::BinarySerializable;
use crate::{
    engines::nano::index::{
        memory::MemoryIndex,
        model::{DocPosting, DocPostingsForTerm, Index, Term},
    },
    model::engine::IndexStats,
};
use anyhow::{Context, Result};
use std::{collections::HashMap, fs::File, io::Seek, path::Path};

pub struct FsIndex {
    terms: HashMap<Term, TermPostingListFileAddress>,
    postings_file: File,
    stats: IndexStats,
}

#[derive(Clone)]
pub struct TermPostingListFileAddress {
    pub postings_count: usize,
    pub start: u64,
    pub end: u64,
}

impl Index for FsIndex {
    fn get_doc_postings_for_term(
        &self,
        term: &Term,
    ) -> Result<Option<DocPostingsForTerm>> {
        let term_posting_list_addr = self.terms.get(term);

        if let Some(term_posting_list_addr) = term_posting_list_addr {
            Ok(Some(DocPostingsForTerm {
                count: term_posting_list_addr.postings_count,
                iterator: Box::new(FsDocPostingsIterator::new(
                    self.postings_file.try_clone().context(
                        "posting file handle should be clonned for iterator",
                    )?,
                    term_posting_list_addr.clone(),
                )?)
                    as Box<dyn Iterator<Item = DocPosting>>,
            }))
        } else {
            Ok(None)
        }
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
    ) -> Result<Self> {
        postings_file
            .seek(std::io::SeekFrom::Start(address.start))
            .context("postings file position should be moved")?;

        Ok(FsDocPostingsIterator {
            postings_file,
            end_position: address.end,
        })
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
    index_dir: impl AsRef<Path>,
) -> Result<FsIndex> {
    let mut terms_file = File::create(index_dir.as_ref().join("terms"))
        .context("terms file should be created")?;
    let mut postings_file = File::options()
        .create(true)
        .append(true)
        .read(true)
        .open(index_dir.as_ref().join("postings"))
        .context("postings file should be created")?;
    let mut index_stats_file = File::create(index_dir.as_ref().join("stats"))
        .context("stats file should be created")?;

    let mut terms = HashMap::new();

    for (term, posting_list) in &memory_index.terms {
        let mut address = TermPostingListFileAddress {
            postings_count: posting_list.len(),
            start: 0,
            end: 0,
        };

        address.postings_count = posting_list.len();
        address.start = postings_file.stream_position()?;
        for posting in posting_list.values() {
            posting.serialize(&mut postings_file)?;
        }
        address.end = postings_file.stream_position()?;

        terms.insert(term.clone(), address.clone());
    }

    terms
        .serialize(&mut terms_file)
        .context("terms should be serialized to file")?;

    memory_index
        .stats
        .serialize(&mut index_stats_file)
        .context("stats should be serialized to file")?;

    Ok(FsIndex {
        terms: terms.into_iter().collect(),
        postings_file,
        stats: memory_index.stats.clone(),
    })
}

pub fn open_fs_index(index_dir: &Path) -> Result<FsIndex> {
    let mut terms_file = File::open(index_dir.join("terms"))
        .context("terms file should be created")?;
    let postings_file = File::open(index_dir.join("postings"))
        .context("postings file should be created")?;
    let mut index_stats_file = File::open(index_dir.join("stats"))
        .context("stats file should be created")?;

    Ok(FsIndex {
        terms: HashMap::<String, TermPostingListFileAddress>::deserialize(
            &mut terms_file,
        )?,
        postings_file,
        stats: IndexStats::deserialize(&mut index_stats_file)?,
    })
}
