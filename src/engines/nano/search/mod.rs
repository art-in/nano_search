mod model;
mod scoring;
mod stop_words;

#[allow(dead_code)] // TODO: remove unused code silencer
mod query;

#[expect(clippy::module_inception)]
mod search;
pub use search::search;
