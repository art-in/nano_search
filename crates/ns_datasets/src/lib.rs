mod beir;
pub use beir::BeirDatasetReader;

mod wiki;
pub use wiki::WikiDatasetReader;

mod cisi;
pub use cisi::CisiDatasetReader;

mod json;
pub use json::JsonDatasetReader;

mod model;
pub use model::{Dataset, DocsSource, QueriesSource};

mod utils;
pub use utils::init_dataset_by_name;
