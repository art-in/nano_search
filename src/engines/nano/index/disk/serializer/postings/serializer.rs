use std::io::Write;

use anyhow::Result;

use super::block::DocPostingsBlock;
use crate::engines::nano::index::model::DocPosting;
use crate::utils::CountingWriter;

/// Serializer for [`DocPosting`]-lists.
///
/// Data Layout:
///
/// [Block 1 [length][docids][freqs]] [Block 2 [length][docids][freqs]] ...
///
/// - `[length]` - The number of docids/freqs in this block.
/// - `[docids]` - Delta-encoded and bit-packed series of sorted document IDs.
/// - `[freqs]`  - Bit-packed series of unsorted term frequencies.
///
/// Document IDs and frequencies are interleaved block-by-block using a
/// Structure of Arrays (SOA) layout. This maximizes spatial locality because
/// posting lists are currently always read sequentially from start to finish,
/// and IDs are always processed alongside with corresponding frequencies.
///
/// Future Evolution:
///
/// When implementing complex search queries that require intersecting posting
/// lists (i.e. `AND` queries) this interleaved layout remains optimal. But it
/// can be improved by adding skip lists, to support fast jumping and avoid
/// reading unnecessary blocks during intersections.
///
/// Lucene and Tantivy use same IDs+freqs interleaved block layout + skip lists.
pub struct PostingsSerializer<'a, W: Write> {
    block: DocPostingsBlock,
    output: &'a mut CountingWriter<W>,
}

impl<'a, W: Write> PostingsSerializer<'a, W> {
    pub const fn new(output: &'a mut CountingWriter<W>) -> Self {
        Self {
            block: DocPostingsBlock::new(),
            output,
        }
    }

    pub fn write_posting(&mut self, posting: &DocPosting) -> Result<()> {
        self.block.add_posting(posting);

        if self.block.is_full() {
            self.block.serialize(self.output)?;
            self.block.clear();
        }

        Ok(())
    }

    pub const fn get_written_bytes(&self) -> usize {
        self.output.get_written_bytes()
    }

    /// Flushes internal buffer to output.
    ///
    /// It is required to flush changes explicitly before [`drop()`], otherwise
    /// serializer will panic. This allows to propogate errors if something goes
    /// wrong while flushing and not to forget some postings inside buffer.
    pub fn flush(&mut self) -> Result<()> {
        if !self.block.is_empty() {
            self.block.serialize(self.output)?;
            self.block.clear();
        }
        Ok(())
    }
}

impl<W: Write> Drop for PostingsSerializer<'_, W> {
    fn drop(&mut self) {
        assert!(
            self.block.is_empty(),
            "serializer should be explicitly flushed before drop"
        );
    }
}

pub struct PostingsDeserializer<'a> {
    block: DocPostingsBlock,
    pos: usize,
    input: &'a [u8],
}

impl<'a> PostingsDeserializer<'a> {
    pub const fn new(input: &'a [u8]) -> Self {
        Self {
            block: DocPostingsBlock::new(),
            pos: 0,
            input,
        }
    }

    fn read_next_block(&mut self) -> Result<()> {
        self.pos = 0;

        if self.input.is_empty() {
            self.block.clear();
        } else {
            self.block.deserialize_from_slice(&mut self.input)?;
        }

        Ok(())
    }
}

impl Iterator for PostingsDeserializer<'_> {
    type Item = Result<DocPosting>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == self.block.len() {
            match self.read_next_block() {
                Ok(()) => {}
                Err(e) => {
                    return Some(Err(e));
                }
            }
        }

        if self.block.is_empty() {
            return None;
        }

        let item = self.block.get_posting(self.pos);
        self.pos += 1;
        Some(Ok(item))
    }
}
