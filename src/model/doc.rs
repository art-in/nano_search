use anyhow::Result;

/// Unique identifier of a document in the input dataset and the whole index.
///
/// It is called "external" as opposed to the internal ID which is assigned to
/// each input document and stored inside the index.
pub type ExternalDocId = u64;

/// Input document supplied by a client to the search engine for indexing.
///
/// Note: Lucene/Tantivy does not have any static contract for an input
/// document. It uses a dynamic, custom multi-field schema approach, where each
/// field can have different properties, like "field should be indexed" or
/// "field should be stored inside the index". Such a custom document is not
/// required to have any ID field, unless the client wants to update or delete
/// documents using an ID later.
#[derive(Clone)]
pub struct Doc {
    pub id: ExternalDocId,
    pub text: String,
}

pub trait DocsSource {
    fn docs(&self) -> Result<Box<dyn Iterator<Item = Result<Doc>>>>;
    fn docs_count(&self) -> Result<Option<usize>>;
}
