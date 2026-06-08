use std::io::Write;

use anyhow::{Result, bail};

use crate::engines::nano::index::disk::serializer::binary::BinarySerializable;
use crate::engines::nano::index::disk::serializer::compression::{
    decode_sorted, decode_unsorted, encode_sorted, encode_unsorted,
};
use crate::engines::nano::index::model::{DocPosting, SegmentDocId};

const BLOCK_CAPACITY: usize = 128;

/// Group of [`DocPosting`]-s serialized together as single block.
///
/// It acts both as internal buffer while serializing/deserializing, and as
/// basic unit of compression - posting lists are block compressed in encoder.
///
/// Static size is used for block in order to make encoding/deconding
/// SIMD-friendly (SIMD is not used yet though).
pub struct DocPostingsBlock {
    docids: [SegmentDocId; BLOCK_CAPACITY],
    term_freqs: [u32; BLOCK_CAPACITY],
    len: usize,
}

impl DocPostingsBlock {
    pub const fn new() -> Self {
        Self {
            docids: [0; BLOCK_CAPACITY],
            term_freqs: [0; BLOCK_CAPACITY],
            len: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_full(&self) -> bool {
        self.len == BLOCK_CAPACITY
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn clean(&mut self) {
        self.len = 0;
    }

    pub fn add_posting(&mut self, posting: &DocPosting) -> Result<()> {
        if self.is_full() {
            bail!("postings should not exceed block capacity");
        }

        self.docids[self.len] = posting.docid;
        self.term_freqs[self.len] = posting.term_freq;

        self.len += 1;

        Ok(())
    }

    pub fn get_posting(&self, idx: usize) -> Result<DocPosting> {
        if idx >= self.len {
            bail!("index of posting should be in bounds of the block");
        }

        Ok(DocPosting {
            docid: self.docids[idx],
            term_freq: self.term_freqs[idx],
        })
    }

    pub fn serialize(&self, output: &mut dyn Write) -> Result<()> {
        (self.len as u16).serialize(output)?;

        encode_sorted(&self.docids, self.len, output)?;
        encode_unsorted(&self.term_freqs, self.len, output)?;

        Ok(())
    }

    pub fn deserialize_from_slice(&mut self, input: &mut &[u8]) -> Result<()> {
        self.len = u16::deserialize_from_slice(input)? as usize;
        if self.len > BLOCK_CAPACITY {
            bail!("block length should be in bounds");
        }

        decode_sorted(self.len, input, &mut self.docids)?;
        decode_unsorted(self.len, input, &mut self.term_freqs)?;

        Ok(())
    }
}
