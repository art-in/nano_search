use anyhow::Result;
use ns_eval::Query;

use super::JsonDatasetReader;
use crate::model::QueriesSource;

impl QueriesSource for JsonDatasetReader {
    fn queries(&self) -> Result<Box<dyn Iterator<Item = Result<Query>>>> {
        unimplemented!()
    }
}
