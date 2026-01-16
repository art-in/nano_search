use anyhow::Result;

use super::disk::DiskIndexOptions;
use crate::model::doc::DocId;

pub type Term = String;

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
    Disk(DiskIndexOptions),
}

/// An index is composed of one or more segments.
pub trait Index {
    fn get_segments(&self) -> Vec<&dyn IndexSegment>;
}

/// A segment is a self-contained part of the index.
/// Segments can be built and searched independently from each other.
///
/// Each segment maintains unique statistics that impact relevance scoring.
/// Consequently, searching a single 10-document segment may yield different
/// results than searching ten 1-document segments. While document balancing
/// typically prevents significant skew, relevance remains inconsistent in
/// smaller, final segments. To ensure accuracy, minimize small segments by
/// optimizing indexing thread counts and maximum segment document limits.
pub trait IndexSegment {
    fn get_doc_postings_for_term<'a>(
        &'a self,
        term: &Term,
    ) -> Result<Option<DocPostingsForTerm<'a>>>;

    fn get_doc_terms_count(&self, docid: DocId) -> Result<u16>;

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
pub struct DocPostingsForTerm<'a> {
    /// Total number of postings, that can be read through the iterator
    pub count: usize,

    /// Iterator over postings
    pub iterator: Box<dyn Iterator<Item = DocPosting> + 'a>,
}

/// Reference to document containing specific term.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DocPosting {
    /// Unique document identifier
    pub docid: DocId,

    /// Number of times the term appears in the document (term frequency)
    pub term_count: u64,
}
