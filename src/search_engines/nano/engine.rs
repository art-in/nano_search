use std::collections::HashSet;

use crate::model::{
    doc::{Doc, DocId},
    engine::SearchEngine,
};

use super::{
    index::{
        build_index,
        model::{Index, IndexType},
    },
    search::search,
};

pub struct NanoSearchEngine {
    index_type: IndexType,
    index: Option<Box<dyn Index>>,
    stop_words: Option<HashSet<String>>,
}

impl NanoSearchEngine {
    pub fn new(index_type: IndexType) -> Self {
        NanoSearchEngine {
            index_type,
            index: None,
            stop_words: None,
        }
    }
}

impl SearchEngine for NanoSearchEngine {
    fn get_name(&self) -> &'static str {
        "nano"
    }

    fn index_docs(&mut self, docs: &mut dyn Iterator<Item = Doc>) {
        self.index = Some(build_index(self.index_type, docs));
        self.stop_words = Some(crate::stop_words::parse_stop_words());
    }

    fn search(&self, query: &str, limit: u64) -> Vec<DocId> {
        let index = self
            .index
            .as_ref()
            .expect("index should be initialized before search");

        let stop_words = self
            .stop_words
            .as_ref()
            .expect("stop words should be initialized before search");

        search(query, index.as_ref(), limit, stop_words)
    }
}
