use std::borrow::Cow;
use std::collections::btree_map;

use anyhow::Result;

use crate::engines::nano::index::model::DocPosting;

pub struct MemoryDocPostingsIterator<'a> {
    postings: btree_map::Values<'a, u64, DocPosting>,
}

impl<'a> MemoryDocPostingsIterator<'a> {
    pub const fn new(postings: btree_map::Values<'a, u64, DocPosting>) -> Self {
        Self { postings }
    }
}

impl<'a> Iterator for MemoryDocPostingsIterator<'a> {
    type Item = Result<Cow<'a, DocPosting>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.postings
            .next()
            .map(|posting| Ok(Cow::Borrowed(posting)))
    }
}
