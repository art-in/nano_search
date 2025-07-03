use std::path::PathBuf;

use anyhow::Result;

use crate::model::doc::DocId;

pub type Term = String;

#[derive(Clone, PartialEq)]
pub enum IndexType {
    // Index which is build fully in memory.
    // Can be used to hold small, non-persistent and fast index.
    MemoryIndex,

    // Index which is persisted to file system on disk.
    // It holds dictionary part in memory, but main part with postings is read
    // from disk.
    FsIndex(PathBuf),
}

pub trait Index {
    fn get_segments(&self) -> Vec<&dyn IndexSegment>;
}

/// Segment is an independent, self sufficient part of index,
/// meaning that it can be build and searched upone separately
/// from other parts
pub trait IndexSegment {
    fn get_doc_postings_for_term(
        &self,
        term: &Term,
    ) -> Result<Option<DocPostingsForTerm>>;
    fn get_stats(&self) -> &IndexSegmentStats;
}

/// Some useful statistics for search candidates scoring and debugging
#[derive(Default, PartialEq, Debug, Clone)]
pub struct IndexSegmentStats {
    pub indexed_docs_count: u64,
    pub max_posting_list_size: u64,
    pub terms_count_per_doc_avg: f64,
}

/// Represents so called posting list (or inverted list) - list of references to
/// documents, which contain specific term.
///
/// This iterator abstracts the way reading is implemented for memory and fs
/// index types. I.e. memory index just reads postings from in-memory structure,
/// while fs index reads them from disk
pub struct DocPostingsForTerm {
    pub count: usize,
    pub iterator: Box<dyn Iterator<Item = DocPosting>>,
}

/// Represents a document posting entry in the index for a specific term
#[derive(Clone, PartialEq, Debug)]
pub struct DocPosting {
    /// Unique identifier for the document
    pub docid: DocId,

    /// Number of times this term appears in the document, i.e. term frequency
    pub term_count: u64,

    /// Total number of terms in this document, i.e. document length.
    /// Used in scoring functions like Tf-Idf/BM25 to normalize term
    /// frequencies
    ///
    /// Implementation Note:
    /// This value is currently duplicated across all postings for the same
    /// document in different term posting lists. A potential optimization
    /// would be to store it separately, in some per-document structure
    ///
    /// For example, Tantivy uses a separate '.fieldnorm' file to store
    /// document lengths. They use log-scaled approximations for better
    /// compression and search performance, trading some precision for
    /// efficiency. See: https://github.com/quickwit-oss/tantivy/blob/5a2fe42c248a45635cbf4a37f1c85136ffe7bb16/src/fieldnorm/mod.rs
    pub total_terms_count: u64,
}
