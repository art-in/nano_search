use crate::model::{
    doc::{Doc, DocId},
    engine::SearchEngine,
};
use tantivy::{
    collector::TopDocs,
    query::QueryParser,
    schema::{Field, NumericOptions, Schema, Value, TEXT},
    Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument,
};
use tempfile::TempDir;

pub struct TantivySearchEngine {
    #[allow(dead_code)] // do not remove temp dir with index
    index_dir: TempDir,

    index: Index,
    index_writer: IndexWriter,
    index_reader: IndexReader,

    id_field: Field,
    text_field: Field,
}

impl Default for TantivySearchEngine {
    fn default() -> Self {
        let index_dir = TempDir::new().expect("temp dir should be created");

        let mut schema_builder = Schema::builder();
        schema_builder
            .add_u64_field("id", NumericOptions::default().set_stored());
        schema_builder.add_text_field("text", TEXT);
        let schema = schema_builder.build();

        let index = Index::create_in_dir(&index_dir, schema.clone())
            .expect("index in dir should be created");

        let index_writer: IndexWriter = index
            .writer(50_000_000)
            .expect("index writer should be created");

        let index_reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
            .expect("should get index reader");

        let id_field =
            schema.get_field("id").expect("id field should be created");
        let text_field = schema
            .get_field("text")
            .expect("text field should be created");

        TantivySearchEngine {
            index_dir,
            index,
            index_writer,
            index_reader,
            id_field,
            text_field,
        }
    }
}

impl SearchEngine for TantivySearchEngine {
    fn get_name(&self) -> &'static str {
        "tantivy"
    }

    fn index_docs(&mut self, docs: &mut dyn Iterator<Item = Doc>) {
        for doc in docs {
            let mut tantivy_doc = TantivyDocument::default();
            tantivy_doc.add_u64(self.id_field, doc.id);
            tantivy_doc.add_text(self.text_field, doc.text);

            self.index_writer
                .add_document(tantivy_doc)
                .expect("doc should be added to index");
        }

        self.index_writer
            .commit()
            .expect("indexer_writer should commit documents to index");

        self.index_reader
            .reload()
            .expect("index reader should be reloaded after writer commit");
    }

    fn search(&self, query: &str, limit: u64) -> Vec<DocId> {
        let searcher = self.index_reader.searcher();

        let query_parser =
            QueryParser::for_index(&self.index, vec![self.text_field]);

        let (query, _) = query_parser.parse_query_lenient(query);

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(limit as usize))
            .expect("should search");

        let mut result = Vec::new();

        for (_score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument =
                searcher.doc(doc_address).expect("document");
            let id = retrieved_doc
                .get_first(self.id_field)
                .expect("document should have id field");

            result.push(id.as_u64().expect("id should be integer"));
        }

        result
    }
}
