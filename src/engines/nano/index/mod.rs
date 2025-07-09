mod build;
mod disk;
mod memory;

pub mod model;
pub use build::{build_index, open_index};
pub use disk::DiskIndexOptions;

#[cfg(test)]
mod tests;
