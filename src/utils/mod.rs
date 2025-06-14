pub mod wikidump;

mod compare_arrays;
pub use compare_arrays::*;

mod normalize;
pub use normalize::*;

mod panic_on_error;
pub use panic_on_error::*;

#[cfg(test)]
pub mod test_docs_iterator;

#[cfg(test)]
pub mod test_docs;
