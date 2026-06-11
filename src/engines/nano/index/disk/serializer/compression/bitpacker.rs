use std::io::Write;

use anyhow::{Result, bail};

use crate::engines::nano::index::disk::serializer::BinarySerializable;

/// A streaming bit-packer that packs integers into a custom `bit_width`
/// and writes them tightly into an `output` byte stream.
pub struct BitPacker<'a> {
    buffer: u64,
    buffer_bits: usize,
    bit_width: usize,
    output: &'a mut dyn Write,
}

impl<'a> BitPacker<'a> {
    pub fn new(bit_width: u8, output: &'a mut dyn Write) -> Self {
        Self {
            buffer: 0,
            buffer_bits: 0,
            bit_width: bit_width as usize,
            output,
        }
    }

    pub fn write_num(&mut self, num: u32) -> Result<()> {
        debug_assert!(
            self.bit_width == 32 || num < (1u32 << self.bit_width),
            "input number should fit into target bit width"
        );

        self.buffer |= (num as u64) << self.buffer_bits;
        self.buffer_bits += self.bit_width;

        while self.buffer_bits >= 8 {
            let byte = self.buffer as u8;
            byte.serialize(self.output)?;
            self.buffer >>= 8;
            self.buffer_bits -= 8;
        }

        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        if self.buffer_bits > 0 {
            let byte = self.buffer as u8;
            byte.serialize(self.output)?;
            self.buffer_bits = 0;
            self.buffer = 0;
        }
        Ok(())
    }
}

impl Drop for BitPacker<'_> {
    fn drop(&mut self) {
        assert!(
            self.buffer_bits == 0 && self.buffer == 0,
            "bitpacker should be explicitly flushed before drop"
        );
    }
}

pub struct BitUnpacker<'a, 'b> {
    buffer: u64,
    buffer_bits: usize,
    bit_width: usize,
    input: &'a mut &'b [u8],
}

impl<'a, 'b> BitUnpacker<'a, 'b> {
    pub const fn new(bit_width: u8, input: &'a mut &'b [u8]) -> Self {
        Self {
            buffer: 0,
            buffer_bits: 0,
            bit_width: bit_width as usize,
            input,
        }
    }

    pub fn read_num(&mut self) -> Result<Option<u32>> {
        while self.buffer_bits < self.bit_width {
            if let Some((&next_byte, rest)) = self.input.split_first() {
                *self.input = rest;

                self.buffer |= (next_byte as u64) << self.buffer_bits;
                self.buffer_bits += 8;
            } else {
                if self.buffer_bits == 0 {
                    // input stream is exhausted
                    return Ok(None);
                }

                bail!(
                    "input stream shouldn't exhaust while some number is \
                     still partially read in the buffer"
                );
            }
        }

        let mask = if self.bit_width == 32 {
            !0u32
        } else {
            (1 << self.bit_width) - 1
        };

        let num = (self.buffer as u32) & mask;
        self.buffer >>= self.bit_width;
        self.buffer_bits -= self.bit_width;

        Ok(Some(num))
    }
}
