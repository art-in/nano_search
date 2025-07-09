use std::path::PathBuf;

use anyhow::Result;

use crate::model::doc::DocId;

pub type Term = String;

#[derive(Clone, PartialEq)]
pub enum IndexMedium {
    /// Index built and used entirely in RAM.
    /// - Suitable for small indices that fit in memory
    /// - Fast indexing and searching (no file I/O)
    /// - Non-persistent: lost on shutdown
    /// - Used for building disk segments and in unit tests
    Memory,

    /// Index persisted to the file system on the disk.
    /// - Supports large indices by splitting into segments
    /// - Total size limited mainly by disk space
    /// - Most data is on disk; term dictionary is memory-resident
    Disk(PathBuf),
}

/// An index is composed of one or more segments.
/// Each segment can be built and searched independently.
pub trait Index {
    fn get_segments(&self) -> Vec<&dyn IndexSegment>;
}

/// A segment is a self-contained part of the index.
/// Segments can be built and searched independently from each other.
pub trait IndexSegment {
    fn get_doc_postings_for_term(
        &self,
        term: &Term,
    ) -> Result<Option<DocPostingsForTerm>>;
    fn get_stats(&self) -> &IndexSegmentStats;
}

/// Useful statistics for search results scoring and debugging.
#[derive(Default, PartialEq, Debug, Clone)]
pub struct IndexSegmentStats {
    /// Number of documents indexed in this segment
    pub indexed_docs_count: u64,

    /// Largest posting list size in this segment
    pub max_posting_list_size: u64,
    
    /// Average number of terms per document
    pub terms_count_per_doc_avg: f64,
}

/// Posting list (inverted list): references to documents containing a term.
///
/// Abstracts over memory and disk index implementations:
/// - Memory: reads from in-memory structures
/// - Disk: reads from on-disk segment files
pub struct DocPostingsForTerm {
    /// Total number of postings, that can be read through the iterator
    pub count: usize,
    
    /// Iterator over postings
    pub iterator: Box<dyn Iterator<Item = DocPosting>>,
}

/// Reference to document containing specific term.
#[derive(Clone, PartialEq, Debug)]
pub struct DocPosting {
    /// Unique document identifier
    pub docid: DocId,

    /// Number of times the term appears in the document (term frequency)
    pub term_count: u64,

    /// Total number of terms in the document (document length).
    /// Used for scoring (e.g., Tf-Idf, BM25) to normalize term frequencies.
    ///
    /// Implementation Note:
    /// This value is currently duplicated across all postings for the same
    /// document in different term posting lists. A potential optimization
    /// would be to store it separately, in some per-document structure
    ///
    /// For example, Tantivy uses a separate '.fieldnorm' file to store
    /// document lengths. They use log-scaled approximations for better
    /// compression and search performance, trading some precision for
    /// efficiency.
    /// See: https://github.com/quickwit-oss/tantivy/blob/5a2fe42c248a45635cbf4a37f1c85136ffe7bb16/src/fieldnorm/mod.rs
    pub total_terms_count: u64,
}
