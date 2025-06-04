use std::fs::File;
use std::io::{BufReader, Seek};

use anyhow::{Context, Result};

use super::model::TermPostingListFileAddress;
use super::serialize::BinarySerializable;
use crate::engines::nano::index::model::DocPosting;

pub struct FsDocPostingsIterator {
    postings_file_reader: BufReader<File>,
    end_position: u64,
}

impl FsDocPostingsIterator {
    pub fn new(
        postings_file: File,
        address: TermPostingListFileAddress,
    ) -> Result<Self> {
        let mut postings_file_reader = BufReader::new(postings_file);

        postings_file_reader
            .seek(std::io::SeekFrom::Start(address.start_byte))
            .context("postings file position should be moved")?;

        Ok(FsDocPostingsIterator {
            postings_file_reader,
            end_position: address.end_byte,
        })
    }
}

impl Iterator for FsDocPostingsIterator {
    type Item = DocPosting;

    fn next(&mut self) -> Option<Self::Item> {
        let current_position = self
            .postings_file_reader
            .stream_position()
            .expect("current position on postings file should be read");

        if current_position < self.end_position {
            let posting =
                DocPosting::deserialize(&mut self.postings_file_reader)
                    .expect("posting should be deserialized from file");
            Some(posting)
        } else {
            None
        }
    }
}
