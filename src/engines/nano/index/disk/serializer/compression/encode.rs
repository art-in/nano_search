//! Integer compression procedures.
//!
//! For sorted arrays, it uses delta-encoding and bit-packing.
//! For unsorted arrays, it directly bit-packs input numbers.
//!
//! Technique explanation: <https://fulmicoton.com/posts/bitpacking/>
//!
//! The current implementation intentionally prioritizes simplicity and
//! readability over performance, so it does not use SIMD vectorization.

use std::io::Write;

use anyhow::{Context, Result, ensure};

use crate::engines::nano::index::disk::serializer::BinarySerializable;
use crate::engines::nano::index::disk::serializer::compression::bitpacker::{
    BitPacker, BitUnpacker,
};

pub fn encode_sorted(input: &[u32], output: &mut dyn Write) -> Result<()> {
    debug_assert!(!input.is_empty(), "input nums should not be empty");

    // calculate max delta and its bit width.
    // we do not store deltas anywhere, and later will calculate them again,
    // only because recalculation is faster than writing/reading from memory
    let mut max_delta = 0u32;
    for i in 1..input.len() {
        ensure!(input[i] >= input[i - 1], "input numbers should be sorted");
        let delta = input[i] - input[i - 1];
        max_delta = delta.max(max_delta);
    }
    let bit_width = max_delta.checked_ilog2().map_or(1, |log| log + 1) as u8;

    // serialize headers
    input[0].serialize(output)?;
    bit_width.serialize(output)?;

    // pack deltas
    let mut packer = BitPacker::new(bit_width, output);
    for i in 1..input.len() {
        let delta = input[i] - input[i - 1];
        packer.write_num(delta)?;
    }
    packer.flush()?;

    Ok(())
}

pub fn decode_sorted(input: &mut &[u8], output: &mut [u32]) -> Result<()> {
    // deserialize headers
    let initial = u32::deserialize_from_slice(input)?;
    let bit_width = u8::deserialize_from_slice(input)?;

    ensure!(
        (1..=32).contains(&bit_width),
        "bit width should be in bounds"
    );

    output[0] = initial;

    // unpack numbers
    let mut unpacker = BitUnpacker::new(bit_width, input);
    for i in 1..output.len() {
        let delta = unpacker.read_num()?.context("number should be read")?;
        output[i] = output[i - 1]
            .checked_add(delta)
            .context("adding delta should not overflow")?;
    }

    Ok(())
}

pub fn encode_unsorted(input: &[u32], output: &mut dyn Write) -> Result<()> {
    debug_assert!(!input.is_empty(), "input nums should not be empty");

    // find the max number and calculate its bit width
    let mut max_num = input[0];
    for &num in input.iter().skip(1) {
        max_num = num.max(max_num);
    }
    let bit_width = max_num.checked_ilog2().map_or(1, |log| log + 1) as u8;

    // serialize headers
    bit_width.serialize(output)?;

    // pack numbers
    let mut packer = BitPacker::new(bit_width, output);
    for &num in input {
        packer.write_num(num)?;
    }
    packer.flush()?;

    Ok(())
}

pub fn decode_unsorted(input: &mut &[u8], output: &mut [u32]) -> Result<()> {
    // deserialize headers
    let bit_width = u8::deserialize_from_slice(input)?;

    ensure!(
        (1..=32).contains(&bit_width),
        "bit width should be in bounds"
    );

    // unpack numbers
    let mut unpacker = BitUnpacker::new(bit_width, input);
    for num in output {
        *num = unpacker.read_num()?.context("number should be read")?;
    }

    Ok(())
}
