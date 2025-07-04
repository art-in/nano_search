use std::collections::HashMap;
use std::fs::File;

use anyhow::Result;

use super::iterator::FsDocPostingsIterator;
use crate::engines::nano::index::model::{
    DocPosting, DocPostingsForTerm, Index, IndexSegment, IndexSegmentStats,
    Term,
};

pub struct FsIndex {
    pub segments: Vec<FsIndexSegment>,
}

pub struct FsIndexSegment {
    pub terms: HashMap<Term, TermPostingListFileAddress>,
    pub postings_file: File,
    pub stats: IndexSegmentStats,
}

pub enum IndexFile {
    Terms,
    Postings,
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
    pub start_byte: u64,
    pub end_byte: u64,
}

impl Index for FsIndex {
    fn get_segments(&self) -> Vec<&dyn IndexSegment> {
        let mut res: Vec<&dyn IndexSegment> = Vec::new();

        for segment in &self.segments {
            res.push(segment);
        }

        res
    }
}

impl IndexSegment for FsIndexSegment {
    fn get_doc_postings_for_term(
        &self,
        term: &Term,
    ) -> Result<Option<DocPostingsForTerm>> {
        let term_posting_list_addr = self.terms.get(term);

        if let Some(term_posting_list_addr) = term_posting_list_addr {
            Ok(Some(DocPostingsForTerm {
                count: term_posting_list_addr.postings_count,
                iterator: Box::new(FsDocPostingsIterator::new(
                    self.postings_file.try_clone()?,
                    term_posting_list_addr.clone(),
                )?)
                    as Box<dyn Iterator<Item = DocPosting>>,
            }))
        } else {
            Ok(None)
        }
    }
    fn get_stats(&self) -> &IndexSegmentStats {
        &self.stats
    }
}
