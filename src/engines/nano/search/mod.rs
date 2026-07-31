mod collectors;
mod iterators;
mod model;
mod plan;
mod query;
mod scoring;
mod stop_words;

#[expect(clippy::module_inception)]
mod search;
pub use search::search;
