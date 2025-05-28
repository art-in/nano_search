mod model;
mod scoring;
mod stop_words;

#[allow(clippy::module_inception)]
mod search;
pub use search::search;
