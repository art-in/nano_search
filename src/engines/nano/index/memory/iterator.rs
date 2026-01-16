use crate::engines::nano::index::model::DocPosting;

pub struct MemoryDocPostingsIterator {
    postings: Vec<DocPosting>,
    position: usize,
}

impl MemoryDocPostingsIterator {
    pub const fn new(postings: Vec<DocPosting>) -> Self {
        Self {
            postings,
            position: 0,
        }
    }
}

impl Iterator for MemoryDocPostingsIterator {
    // TODO: iterate over references to avoid cloning
    type Item = DocPosting;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.postings.len() {
            let posting = self.postings[self.position].clone();
            self.position += 1;
            Some(posting)
        } else {
            None
        }
    }
}
