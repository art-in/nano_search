use std::path::Path;

use anyhow::{Context, Result};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, NumericOptions, Schema, TEXT, Value};
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument};

use crate::model::doc::{Doc, DocId};
use crate::model::engine::SearchEngine;

pub struct TantivySearchEngine {
    index: Index,
    index_writer: IndexWriter,
    index_reader: IndexReader,

    id_field: Field,
    text_field: Field,
}

impl TantivySearchEngine {
    fn new(index: Index) -> Result<Self> {
        let index_writer: IndexWriter = index
            .writer(50_000_000)
            .context("index writer should be created")?;

        let index_reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
            .context("should get index reader")?;

        let id_field = index
            .schema()
            .get_field("id")
            .context("id field should be created")?;
        let text_field = index
            .schema()
            .get_field("text")
            .context("text field should be created")?;

        Ok(TantivySearchEngine {
            index,
            index_writer,
            index_reader,
            id_field,
            text_field,
        })
    }
}

fn create_schema() -> Schema {
    let mut schema_builder = Schema::builder();
    schema_builder.add_u64_field("id", NumericOptions::default().set_stored());
    schema_builder.add_text_field("text", TEXT);
    schema_builder.build()
}

impl SearchEngine for TantivySearchEngine {
    fn name() -> &'static str {
        "tantivy"
    }

    fn get_name(&self) -> &'static str {
        Self::name()
    }

    fn create_in_memory() -> Result<Self>
    where
        Self: Sized,
    {
        let index = Index::create_in_ram(create_schema());
        TantivySearchEngine::new(index)
    }

    fn create_on_disk(index_dir: impl AsRef<Path>) -> Result<Self> {
        if index_dir.as_ref().exists() {
            std::fs::remove_dir_all(index_dir.as_ref())
                .context("existing index dir should be removed")?;
        }
        std::fs::create_dir(index_dir.as_ref())
            .context("index dir should be created")?;

        let index = Index::create_in_dir(index_dir, create_schema())
            .context("index should be created in dir")?;

        TantivySearchEngine::new(index)
    }

    fn open_from_disk(index_dir: impl AsRef<Path>) -> Result<Self> {
        let index = Index::open_in_dir(index_dir)
            .context("index should be opened from dir")?;

        TantivySearchEngine::new(index)
    }

    fn index_docs(
        &mut self,
        docs: &mut dyn Iterator<Item = Doc>,
    ) -> Result<()> {
        for doc in docs {
            let mut tantivy_doc = TantivyDocument::default();
            tantivy_doc.add_u64(self.id_field, doc.id);
            tantivy_doc.add_text(self.text_field, doc.text);

            self.index_writer
                .add_document(tantivy_doc)
                .context("doc should be added to index")?;
        }

        self.index_writer
            .commit()
            .context("indexer_writer should commit documents to index")?;

        self.index_reader
            .reload()
            .context("index reader should be reloaded after writer commit")?;

        Ok(())
    }

    fn search(&self, query: &str, limit: u64) -> Result<Vec<DocId>> {
        let searcher = self.index_reader.searcher();

        let query_parser =
            QueryParser::for_index(&self.index, vec![self.text_field]);

        let (query, _) = query_parser.parse_query_lenient(query);

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(limit as usize))
            .context("should search")?;

        let mut result = Vec::new();

        for (_score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument =
                searcher.doc(doc_address).context("document")?;
            let id = retrieved_doc
                .get_first(self.id_field)
                .context("should get id from found document")?;

            result.push(id.as_u64().context("id should be integer")?);
        }

        Ok(result)
    }
}
