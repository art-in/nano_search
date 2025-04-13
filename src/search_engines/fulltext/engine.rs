use std::collections::HashSet;

use crate::model::{
    doc::{Doc, DocId},
    engine::{IndexStats, SearchEngine},
};

use super::{
    index::{build_index, Index},
    search::search,
};

#[derive(Default)]
pub struct FulltextSearchEngine {
    index: Option<Index>,
    stop_words: Option<HashSet<String>>,
}

impl SearchEngine for FulltextSearchEngine {
    fn index_docs(
        &mut self,
        docs: &mut dyn Iterator<Item = Doc>,
    ) -> IndexStats {
        self.index = Some(build_index(docs));
        self.stop_words = Some(crate::stop_words::parse_stop_words());

        self.index
            .as_ref()
            .expect("index should be initialized")
            .stats
            .clone()
    }

    fn search(&self, query: &str) -> Vec<DocId> {
        let index = self
            .index
            .as_ref()
            .expect("index should be initialized before search");

        let stop_words = self
            .stop_words
            .as_ref()
            .expect("stop words should be initialized before search");

        search(query, index, stop_words)
    }
}
