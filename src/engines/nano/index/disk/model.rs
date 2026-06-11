use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use memmap2::Mmap;

use super::iterator::DiskDocPostingsIterator;
use crate::engines::nano::index::disk::serializer::deserialize_vec_item;
use crate::engines::nano::index::model::{
    DocPostingsForTerm, Index, IndexSegment, IndexSegmentStats, SegmentDocId,
    StoredDoc, Term,
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
    pub doc_term_counts_file: Mmap,
    pub docs_file: Mmap,
    pub stats: IndexSegmentStats,
}

#[derive(Copy, Clone)]
pub enum IndexFile {
    /// Maps terms to offsets of corresponding posting lists in Postings file
    Terms,

    /// Posting lists for terms from Terms file
    Postings,

    /// Count of terms in each document (a.k.a. document lengths)
    ///
    /// Note:
    /// For example, Tantivy/Lucene store document lengths in '.fieldnorm' file.
    /// They use log-scaled length approximations for better compression and
    /// search performance, trading some scoring precision for efficiency.
    /// See <https://github.com/quickwit-oss/tantivy/blob/5a2fe42c248a45635cbf4a37f1c85136ffe7bb16/src/fieldnorm/mod.rs>
    DocLen,

    /// Stored documents
    Docs,

    /// Statistics gathered while building index, which is used later by search
    /// routine (e.g. for candidates scoring) and debugging
    Stats,
}

impl IndexFile {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Terms => "terms",
            Self::Postings => "postings",
            Self::DocLen => "doclen",
            Self::Docs => "docs",
            Self::Stats => "stats",
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

        term_posting_list_addr.map_or_else(
            || Ok(None),
            |addr| {
                Ok(Some(DocPostingsForTerm {
                    count: addr.postings_count,
                    // TODO: avoid heap-allocation on hot path - try enum
                    iterator: Box::new(DiskDocPostingsIterator::new(
                        &self.postings_file,
                        addr,
                    )),
                }))
            },
        )
    }

    fn get_doc_terms_count(&self, docid: SegmentDocId) -> Result<Cow<'_, u16>> {
        deserialize_vec_item::<u16>(&self.doc_term_counts_file, docid as usize)
    }

    fn get_stored_doc(
        &self,
        docid: SegmentDocId,
    ) -> Result<Cow<'_, StoredDoc>> {
        deserialize_vec_item::<StoredDoc>(&self.docs_file, docid as usize)
    }

    fn get_stats(&self) -> &IndexSegmentStats {
        &self.stats
    }
}
