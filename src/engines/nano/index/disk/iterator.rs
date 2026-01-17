use anyhow::Result;
use memmap2::Mmap;

use super::model::TermPostingListFileAddress;
use super::serialize::BinarySerializable;
use crate::engines::nano::index::model::DocPosting;

pub struct DiskDocPostingsIterator<'a> {
    posting_list: &'a [u8],
}

impl<'a> DiskDocPostingsIterator<'a> {
    pub fn new(
        postings_file: &'a Mmap,
        address: &TermPostingListFileAddress,
    ) -> Self {
        Self {
            posting_list: &postings_file[address.start_byte..address.end_byte],
        }
    }
}

impl Iterator for DiskDocPostingsIterator<'_> {
    type Item = Result<DocPosting>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.posting_list.is_empty() {
            None
        } else {
            let posting =
                DocPosting::deserialize_from_slice(&mut self.posting_list);
            Some(posting)
        }
    }
}
