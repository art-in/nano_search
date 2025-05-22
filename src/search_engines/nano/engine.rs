use super::{
    index::{
        build_index,
        model::{Index, IndexType},
        open_index,
    },
    search::search,
};
use crate::model::{
    doc::{Doc, DocId},
    engine::SearchEngine,
};
use std::collections::HashSet;

pub struct NanoSearchEngine {
    index_type: IndexType,
    index: Option<Box<dyn Index>>,
    stop_words: Option<HashSet<String>>,
}

impl SearchEngine for NanoSearchEngine {
    fn get_name(&self) -> &'static str {
        "nano"
    }

    fn create_index(index_dir: &str) -> Self {
        std::fs::remove_dir_all(index_dir)
            .expect("existing index dir should be removed");
        std::fs::create_dir(index_dir).expect("index dir should be created");

        NanoSearchEngine {
            index_type: IndexType::FsIndex(index_dir.into()),
            index: None,
            stop_words: Some(crate::stop_words::parse_stop_words()),
        }
    }

    fn open_index(index_dir: &str) -> Self {
        let index_type = IndexType::FsIndex(index_dir.into());
        let index = open_index(&index_type).expect("index should be opened");

        NanoSearchEngine {
            index_type,
            index: Some(index),
            stop_words: Some(crate::stop_words::parse_stop_words()),
        }
    }

    fn index_docs(&mut self, docs: &mut dyn Iterator<Item = Doc>) {
        self.index = Some(
            build_index(&self.index_type, docs).expect("index should be built"),
        );
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
