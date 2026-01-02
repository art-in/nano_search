use anyhow::Result;

use crate::dataset_readers::json::JsonDatasetReader;
use crate::eval::model::{QueriesSource, Query};

impl QueriesSource for JsonDatasetReader {
    fn queries(&self) -> Result<Box<dyn Iterator<Item = Query>>> {
        unimplemented!()
    }
}
