mod bitpacker;
mod encode;

pub use encode::{
    decode_sorted, decode_unsorted, encode_sorted, encode_unsorted,
};

#[cfg(test)]
mod tests;
