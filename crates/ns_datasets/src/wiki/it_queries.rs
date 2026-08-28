use anyhow::Result;
use ns_eval::Query;

use super::model::WikiDatasetReader;
use crate::model::QueriesSource;

impl QueriesSource for WikiDatasetReader {
    fn queries(&self) -> Result<Box<dyn Iterator<Item = Result<Query>>>> {
        unimplemented!()
    }
}
