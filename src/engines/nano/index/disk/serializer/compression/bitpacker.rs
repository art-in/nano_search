use std::io::Write;

use anyhow::{Result, bail, ensure};

use crate::engines::nano::index::disk::serializer::BinarySerializable;

pub struct BitPacker<'a> {
    accumulator: u64,
    bits_count: usize,
    bit_width: usize,
    output: &'a mut dyn Write,
}

impl<'a> BitPacker<'a> {
    pub fn new(output: &'a mut dyn Write, bit_width: u8) -> Self {
        Self {
            accumulator: 0,
            bits_count: 0,
            bit_width: bit_width as usize,
            output,
        }
    }

    pub fn write_num(&mut self, num: u32) -> Result<()> {
        ensure!(
            self.bit_width == 32 || num < (1u32 << self.bit_width),
            "input number should fit into target bit width"
        );

        self.accumulator |= (num as u64) << self.bits_count;
        self.bits_count += self.bit_width;

        while self.bits_count >= 8 {
            let n = self.accumulator as u8;
            n.serialize(self.output)?;
            self.accumulator >>= 8;
            self.bits_count -= 8;
        }

        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        if self.bits_count > 0 {
            let n = self.accumulator as u8;
            n.serialize(self.output)?;
            self.bits_count = 0;
            self.accumulator = 0;
        }

        self.output.flush()?;

        Ok(())
    }
}

impl Drop for BitPacker<'_> {
    fn drop(&mut self) {
        assert!(
            self.bits_count == 0 && self.accumulator == 0,
            "bitpacker should be explicitly flushed before drop"
        );
    }
}

pub struct BitUnpacker<'a, 'b> {
    accumulator: u64,
    bits_count: usize,
    bit_width: usize,
    input: &'a mut &'b [u8],
}

impl<'a, 'b> BitUnpacker<'a, 'b> {
    pub const fn new(input: &'a mut &'b [u8], bit_width: u8) -> Self {
        Self {
            accumulator: 0,
            bits_count: 0,
            bit_width: bit_width as usize,
            input,
        }
    }

    pub fn read_num(&mut self) -> Result<Option<u32>> {
        while self.bits_count < self.bit_width {
            if let Some((&next_byte, rest)) = self.input.split_first() {
                *self.input = rest;

                self.accumulator |= (next_byte as u64) << self.bits_count;
                self.bits_count += 8;
            } else {
                if self.bits_count == 0 {
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

        let num = (self.accumulator as u32) & mask;
        self.accumulator >>= self.bit_width;
        self.bits_count -= self.bit_width;

        Ok(Some(num))
    }
}
