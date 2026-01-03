use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use memmap2::Mmap;

use super::iterator::DiskDocPostingsIterator;
use crate::engines::nano::index::model::{
    DocPostingsForTerm, Index, IndexSegment, IndexSegmentStats, Term,
};

#[derive(bon::Builder)]
pub struct DiskIndexOptions {
    /// Path to directory where index should be stored
    #[builder(into)]
    pub index_dir: PathBuf,

    /// Number of indexing threads.
    ///
    /// Each thread builds its own segments.
    ///
    /// Input documents are distributed randomly across threads and segments,
    /// introducing non-determinism to the indexing and search processes.
    /// Since a segment's specific document set determines its internal
    /// statistics - such as term IDF and average document length - this
    /// distribution directly impacts final relevance scoring.
    pub index_threads: Option<usize>,

    /// Maximum number of documents new segment is allowed to collect in memory
    /// before dumping to disk and starting next segment. Higher the number -
    /// higher the memory consumption by indexer
    #[builder(default = 25_000)]
    pub max_segment_docs: usize,
}

pub struct DiskIndex {
    pub segments: Vec<DiskIndexSegment>,
}

pub struct DiskIndexSegment {
    pub terms: HashMap<Term, TermPostingListFileAddress>,
    // use file mmap instead of open/seek/read to avoid "Too many opened files"
    // OS error on big indices with lots of segments
    pub postings_file: Mmap,
    pub stats: IndexSegmentStats,
}

pub enum IndexFile {
    // Maps terms to offsets of corresponding posting lists in Postings file
    Terms,

    // Posting lists for terms from Terms file
    Postings,

    // Statistics gathered while building index, which is used later by search
    // routine (e.g. for candidates scoring) and debugging
    Stats,
}

impl IndexFile {
    pub fn name(&self) -> &'static str {
        match self {
            IndexFile::Terms => "terms",
            IndexFile::Postings => "postings",
            IndexFile::Stats => "stats",
        }
    }
}

#[derive(Clone)]
pub struct TermPostingListFileAddress {
    pub postings_count: usize,
    pub start_byte: usize,
    pub end_byte: usize,
}

impl Index for DiskIndex {
    fn get_segments(&self) -> Vec<&dyn IndexSegment> {
        let mut res: Vec<&dyn IndexSegment> = Vec::new();
        for segment in &self.segments {
            res.push(segment);
        }
        res
    }
}

impl IndexSegment for DiskIndexSegment {
    fn get_doc_postings_for_term<'a>(
        &'a self,
        term: &Term,
    ) -> Result<Option<DocPostingsForTerm<'a>>> {
        let term_posting_list_addr = self.terms.get(term);

        if let Some(term_posting_list_addr) = term_posting_list_addr {
            Ok(Some(DocPostingsForTerm {
                count: term_posting_list_addr.postings_count,
                iterator: Box::new(DiskDocPostingsIterator::new(
                    &self.postings_file,
                    term_posting_list_addr,
                )?),
            }))
        } else {
            Ok(None)
        }
    }
    fn get_stats(&self) -> &IndexSegmentStats {
        &self.stats
    }
}
