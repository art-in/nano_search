use std::borrow::Cow;

use anyhow::Result;

use super::disk::DiskIndexOptions;
use crate::model::doc::ExternalDocId;

pub type Term = String;

/// Unique identifier of a document inside an index segment.
///
/// This is a simple sequential index, in the order doc was supplied to the
/// segment indexer.
///
/// Index posting lists contain this ID and not the external document ID,
/// because a client document can, in theory, have no ID assigned at all.
/// And a sequence of small, sorted numbers can be compressed much better.
pub type SegmentDocId = u32;

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

/// A segment is a self-contained immutable part of the index.
/// Segments can be built and searched independently from each other.
///
/// This trait is abstraction for in-memory and on-disk index implementations.
// TODO: fix cross-segment relevance skew for better ranking.
// - the problem: each segment maintains unique statistics that impact relevance
//   scoring. e.g., searching a single 10-document segment may yield different
//   results than searching ten 1-document segments. while document balancing
//   typically prevents significant skew, relevance remains inconsistent in
//   smaller, final segments
// - use approach from lucene/tantivy: maintain local segment stats in memory,
//   compute global stats for query terms on the fly, and pass it along with the
//   query to segments, so all segments score their local results using global
//   term statistics
pub trait IndexSegment {
    fn get_doc_postings_for_term<'a>(
        &'a self,
        term: &str,
    ) -> Result<Option<DocPostingsForTerm<'a>>>;

    fn get_doc_terms_count(&self, docid: SegmentDocId) -> Result<Cow<'_, u16>>;

    fn get_stored_doc(&self, docid: SegmentDocId)
    -> Result<Cow<'_, StoredDoc>>;

    fn get_stats(&self) -> &IndexSegmentStats;
}

/// Doc fields stored inside index.
///
/// Currently just bare minimum is stored - external doc IDs to answer
/// search queries with. Do not need to store doc text, since we do not support
/// snippets or any other feature that requires source doc text yet.
#[derive(Clone, Debug)]
pub struct StoredDoc {
    pub docid: ExternalDocId,
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
/// This struct is abstraction for iterator in in-memory and on-disk index
/// implementations.
pub struct DocPostingsForTerm<'a> {
    /// Total number of postings, that can be read through the iterator
    pub count: usize,

    /// Iterator over postings
    pub iterator: DocPostingsIterator<'a>,
}

type DocPostingsIterator<'a> =
    Box<dyn Iterator<Item = Result<Cow<'a, DocPosting>>> + 'a>;

/// Reference to document containing specific term.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DocPosting {
    /// Unique document identifier
    pub docid: SegmentDocId,

    /// Number of occurrences of the term in the document
    pub term_freq: u32,
}
