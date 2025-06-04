mod build;
mod fs;
mod memory;

pub mod model;
pub use build::{build_index, open_index};

#[cfg(test)]
mod tests;
