use anyhow::Result;

use super::encode::{
    decode_sorted, decode_unsorted, encode_sorted, encode_unsorted,
};

fn generate_sorted_numbers(len: usize) -> Vec<u32> {
    let mut nums = Vec::with_capacity(len);
    for i in 0..len {
        nums.push((i as u32) * 10);
    }
    nums
}

fn generate_unsorted_numbers(len: usize) -> Vec<u32> {
    let mut nums = Vec::with_capacity(len);
    for i in 0..len {
        // fill with pseudo-random numbers
        nums.push(((i as u32).wrapping_mul(17) + 5) % 100);
    }
    nums
}

#[test]
fn test_encode_sorted() -> Result<()> {
    let test_lengths = vec![1, 2, 7, 32, 63, 127, 128];

    for len in test_lengths {
        let mut storage = Vec::<u8>::new();

        let original_nums = generate_sorted_numbers(len);

        encode_sorted(&original_nums, &mut storage)?;

        let mut input: &[u8] = &storage[..];
        let mut decoded_nums = [0u32; 128];

        decode_sorted(&mut input, &mut decoded_nums[..len])?;

        assert_eq!(original_nums[..], decoded_nums[..len]);
        assert!(input.is_empty());
    }

    Ok(())
}

#[test]
fn test_encode_unsorted() -> Result<()> {
    let test_lengths = vec![1, 2, 7, 32, 63, 127, 128];

    for len in test_lengths {
        let mut storage = Vec::<u8>::new();

        let original_nums = generate_unsorted_numbers(len);

        encode_unsorted(&original_nums, &mut storage)?;

        let mut input: &[u8] = &storage[..];
        let mut decoded_nums = [0u32; 128];

        decode_unsorted(&mut input, &mut decoded_nums[..len])?;

        assert_eq!(original_nums[..], decoded_nums[..len],);
        assert!(input.is_empty());
    }

    Ok(())
}

#[test]
fn test_encode_multiblock() -> Result<()> {
    let mut storage = Vec::<u8>::new();

    let block_1_len = 10;
    let block_2_len = 50;

    let block_1_nums = generate_sorted_numbers(block_1_len);
    let block_2_nums = generate_unsorted_numbers(block_2_len);

    encode_sorted(&block_1_nums, &mut storage)?;
    encode_unsorted(&block_2_nums, &mut storage)?;

    let mut input: &[u8] = &storage[..];

    let mut block_1_decoded = [0u32; 128];
    decode_sorted(&mut input, &mut block_1_decoded[..block_1_len])?;
    assert_eq!(block_1_nums[..], block_1_decoded[..block_1_len]);

    let mut block_2_decoded = [0u32; 128];
    decode_unsorted(&mut input, &mut block_2_decoded[..block_2_len])?;

    assert_eq!(block_2_nums[..], block_2_decoded[..block_2_len]);
    assert!(input.is_empty());

    Ok(())
}

#[test]
fn test_encode_sorted_single_value_max_u32() -> Result<()> {
    let original_nums = [u32::MAX];

    let mut storage = Vec::new();
    encode_sorted(&original_nums, &mut storage)?;

    let mut input: &[u8] = &storage;
    let mut decoded_nums = [0u32; 1];

    decode_sorted(&mut input, &mut decoded_nums)?;

    assert_eq!(original_nums, decoded_nums);
    assert!(input.is_empty());

    Ok(())
}

#[test]
fn test_encode_sorted_max_delta_u32() -> Result<()> {
    let original_nums = [0, u32::MAX];

    let mut storage = Vec::new();
    encode_sorted(&original_nums, &mut storage)?;

    let mut input: &[u8] = &storage;
    let mut decoded_nums = [0u32; 2];

    decode_sorted(&mut input, &mut decoded_nums)?;

    assert_eq!(original_nums, decoded_nums);
    assert!(input.is_empty());

    Ok(())
}

#[test]
fn test_encode_unsorted_all_zeros() -> Result<()> {
    let original_nums = [0u32; 32];

    let mut storage = Vec::new();
    encode_unsorted(&original_nums, &mut storage)?;

    let mut input: &[u8] = &storage;
    let mut decoded_nums = [0u32; 32];

    decode_unsorted(&mut input, &mut decoded_nums[..original_nums.len()])?;

    assert_eq!(original_nums, decoded_nums);
    assert!(input.is_empty());

    Ok(())
}

#[test]
fn test_encode_unsorted_single_bit_values() -> Result<()> {
    let original_nums = [0, 1, 0, 1, 1, 0, 1, 0, 1];

    let mut storage = Vec::new();
    encode_unsorted(&original_nums, &mut storage)?;

    let mut input: &[u8] = &storage;
    let mut decoded_nums = [0u32; 16];

    decode_unsorted(&mut input, &mut decoded_nums[..original_nums.len()])?;

    assert_eq!(original_nums[..], decoded_nums[..original_nums.len()]);
    assert!(input.is_empty());

    Ok(())
}

#[test]
fn test_decode_unsorted_input_exhaust() -> Result<()> {
    let mut storage = Vec::<u8>::new();
    let original_nums = generate_unsorted_numbers(10);
    encode_unsorted(&original_nums, &mut storage)?;

    // intentionally truncate the buffer by hacking off the last byte
    let truncated_len = storage.len() - 1;
    let mut input: &[u8] = &storage[..truncated_len];
    let mut decoded_nums = [0u32; 128];

    let result = decode_unsorted(&mut input, &mut decoded_nums[..10]);

    assert!(result.is_err());

    if let Err(msg) = result {
        assert!(msg.to_string().contains("input stream shouldn't exhaust"));
    }

    Ok(())
}

#[test]
fn test_decode_unsorted_bit_width_zero() {
    let mut input: &[u8] = &[
        0, 0, 0, 0, // min value
        0, // bit width
    ];

    let mut output = [0u32; 1];

    let result = decode_unsorted(&mut input, &mut output);

    assert!(result.is_err());

    if let Err(msg) = result {
        assert!(msg.to_string().contains("bit width should be in bounds"));
    }
}

#[test]
fn test_encode_sorted_unsorted_input() {
    let nums = [10, 5];

    let mut storage = Vec::new();
    let result = encode_sorted(&nums, &mut storage);

    assert!(result.is_err());

    if let Err(msg) = result {
        assert!(msg.to_string().contains("input numbers should be sorted"));
    }
}
