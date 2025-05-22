mod compare_arrays;
mod normalize;
mod stop_words;

pub use compare_arrays::*;
pub use normalize::*;
pub use stop_words::*;

#[cfg(test)]
pub mod test_docs_iterator;

#[cfg(test)]
pub mod test_docs;
