mod model;

mod scoring;
mod stop_words;

#[allow(dead_code)] // TODO: remove unused code silencer
mod collectors;

#[allow(dead_code)] // TODO: remove unused code silencer
mod iterators;

#[allow(dead_code)] // TODO: remove unused code silencer
mod query;

#[allow(dead_code)] // TODO: remove unused code silencer
mod planner;

#[expect(clippy::module_inception)]
mod search;
pub use search::search;
