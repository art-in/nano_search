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
    ) -> Result<Self> {
        Ok(DiskDocPostingsIterator {
            posting_list: &postings_file[address.start_byte..address.end_byte],
        })
    }
}

impl<'a> Iterator for DiskDocPostingsIterator<'a> {
    // TODO: use Result<DocPosting> to avoid .expect() in next()
    type Item = DocPosting;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.posting_list.is_empty() {
            let posting =
                DocPosting::deserialize_from_slice(&mut self.posting_list)
                    .expect("posting should be deserialized from file");
            Some(posting)
        } else {
            None
        }
    }
}
