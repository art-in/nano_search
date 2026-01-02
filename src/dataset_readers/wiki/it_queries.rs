use anyhow::Result;

use crate::dataset_readers::WikiDatasetReader;
use crate::eval::model::{QueriesSource, Query};

impl QueriesSource for WikiDatasetReader {
    fn queries(&self) -> Result<Box<dyn Iterator<Item = Query>>> {
        unimplemented!()
    }
}
