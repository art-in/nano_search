use crate::eval::model::QueriesSource;
use crate::model::doc::DocsSource;

// utility super-trait to use with "dyn", e.g.:
// "Box<dyn DocsSource + QueriesSource>" is not allowed
// "Box<dyn Dataset>" is ok
pub trait Dataset: DocsSource + QueriesSource {}

// auto implement this super-trait for all structs that already implement all
// child traits
impl<T> Dataset for T where T: DocsSource + QueriesSource {}
