mod build_index;
mod fs;
mod memory;

pub mod model;
pub use build_index::build_index;

#[cfg(test)]
mod tests;
