use std::io::Write;

use anyhow::{Context, Result, ensure};

use crate::engines::nano::index::disk::serializer::BinarySerializable;
use crate::engines::nano::index::disk::serializer::compression::bitpacker::{
    BitPacker, BitUnpacker,
};

// Encoding currently implemented as simple and understandable as possible, so
// it doesn't leverage SIMD vectorization, which is standard for such task. It
// allows us to use simple stateless routines, otherwise it would require to add
// intermediate buffer wrapped into some Encoder struct, which would be nice not
// to reallocated for each encode/decode operation, so it'll have to be reused
// and passed along inside serializer

pub fn encode_sorted(
    nums: &[u32],
    len: usize,
    output: &mut dyn Write,
) -> Result<()> {
    ensure!(len > 0, "len should be greater than zero");
    ensure!(nums.len() >= len, "len should be in bounds of input");

    // calculate max delta and its bit width.
    // we do not store deltas anywhere, and later will calculate them again,
    // only because recalculation is faster than writing/reading from memory
    let mut max_delta = 0u32;
    for i in 1..len {
        ensure!(nums[i] >= nums[i - 1], "input numbers should be sorted");
        let delta = nums[i] - nums[i - 1];
        max_delta = delta.max(max_delta);
    }
    let bit_width = max_delta.checked_ilog2().map_or(1, |log| log + 1) as u8;

    // serialize headers
    nums[0].serialize(output)?;
    bit_width.serialize(output)?;

    // pack deltas
    let mut packer = BitPacker::new(output, bit_width);
    for i in 1..len {
        let delta = nums[i] - nums[i - 1];
        packer.write_num(delta)?;
    }
    packer.flush()?;

    Ok(())
}

pub fn decode_sorted(
    len: usize,
    input: &mut &[u8],
    output: &mut [u32],
) -> Result<()> {
    ensure!(len > 0, "len should be greater than zero");
    ensure!(output.len() >= len, "len should be in bounds of output");

    // deserialize headers
    let initial = u32::deserialize_from_slice(input)?;
    let bit_width = u8::deserialize_from_slice(input)?;

    ensure!(
        (1..=32).contains(&bit_width),
        "bit width should be in bounds"
    );

    output[0] = initial;

    // unpack numbers
    let mut unpacker = BitUnpacker::new(input, bit_width);
    for i in 1..len {
        let delta = unpacker.read_num()?.context("number should be read")?;
        output[i] = output[i - 1]
            .checked_add(delta)
            .context("adding delta should not overflow")?;
    }

    Ok(())
}

pub fn encode_unsorted(
    nums: &[u32],
    len: usize,
    output: &mut dyn Write,
) -> Result<()> {
    ensure!(len > 0, "len should be greater than zero");
    ensure!(nums.len() >= len, "len should be in bounds of input");

    // find the max number and calculate its bit width
    let mut max_num = nums[0];
    for &num in nums.iter().take(len).skip(1) {
        max_num = num.max(max_num);
    }
    let bit_width = max_num.checked_ilog2().map_or(1, |log| log + 1) as u8;

    // serialize headers
    bit_width.serialize(output)?;

    // pack numbers
    let mut packer = BitPacker::new(output, bit_width);
    for &num in nums.iter().take(len) {
        packer.write_num(num)?;
    }
    packer.flush()?;

    Ok(())
}

pub fn decode_unsorted(
    len: usize,
    input: &mut &[u8],
    output: &mut [u32],
) -> Result<()> {
    ensure!(len > 0, "len should be greater than zero");
    ensure!(output.len() >= len, "len should be in bounds of output");

    // deserialize headers
    let bit_width = u8::deserialize_from_slice(input)?;

    ensure!(
        (1..=32).contains(&bit_width),
        "bit width should be in bounds"
    );

    // unpack numbers
    let mut unpacker = BitUnpacker::new(input, bit_width);
    for num in output.iter_mut().take(len) {
        *num = unpacker.read_num()?.context("number should be read")?;
    }

    Ok(())
}
