use std::borrow::Cow;

use anyhow::Result;
use memmap2::Mmap;

use super::model::TermPostingListFileAddress;
use super::serializer::PostingsDeserializer;
use crate::engines::nano::index::model::DocPosting;

pub struct DiskDocPostingsIterator<'a> {
    deserializer: PostingsDeserializer<'a>,
}

impl<'a> DiskDocPostingsIterator<'a> {
    pub fn new(
        postings_file: &'a Mmap,
        address: &TermPostingListFileAddress,
    ) -> Self {
        Self {
            // TODO: find a way to reuse block buffer inside serializer to not
            // allocate it for each search query
            deserializer: PostingsDeserializer::new(
                &postings_file[address.start_byte..address.end_byte],
            ),
        }
    }
}

impl<'a> Iterator for DiskDocPostingsIterator<'a> {
    type Item = Result<Cow<'a, DocPosting>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.deserializer.next().map(|res| res.map(Cow::Owned))
    }
}
