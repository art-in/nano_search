use anyhow::Result;
use ns_engine::Doc;
use ns_eval::Query;

pub trait DocsSource {
    fn docs(&self) -> Result<Box<dyn Iterator<Item = Result<Doc>>>>;
    fn docs_count(&self) -> Result<Option<usize>>;
}

pub trait QueriesSource {
    fn queries(&self) -> Result<Box<dyn Iterator<Item = Result<Query>>>>;
}

// utility super-trait to use with "dyn", e.g.:
// "Box<dyn DocsSource + QueriesSource>" is not allowed
// "Box<dyn Dataset>" is ok
pub trait Dataset: DocsSource + QueriesSource {}

// auto implement this super-trait for all structs that already implement all
// child traits
impl<T> Dataset for T where T: DocsSource + QueriesSource {}
