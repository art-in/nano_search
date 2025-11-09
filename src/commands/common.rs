use anyhow::Result;

use crate::dataset_readers::BeirDatasetReader;
use crate::engines::nano::engine::NanoSearchEngine;
#[cfg(feature = "qdrant")]
use crate::engines::qdrant::engine::QdrantSearchEngine;
use crate::engines::tantivy::engine::TantivySearchEngine;
use crate::engines::vector::engine::VectorSearchEngine;
use crate::eval::model::QueriesSource;
use crate::model::doc::DocsSource;
use crate::model::engine::SearchEngine;

const NANO_INDEX_DIR: &str = "index_nano";
const TANTIVY_INDEX_DIR: &str = "index_tantivy";
const VECTOR_INDEX_DIR: &str = "index_vector";

pub fn init_search_engines_create() -> Result<Vec<Box<dyn SearchEngine>>> {
    println!("initializing search engines");
    Ok(vec![
        Box::new(NanoSearchEngine::create_on_disk(NANO_INDEX_DIR)?),
        Box::new(TantivySearchEngine::create_on_disk(TANTIVY_INDEX_DIR)?),
        Box::new(VectorSearchEngine::create_on_disk(VECTOR_INDEX_DIR)?),
        #[cfg(feature = "qdrant")]
        Box::new(QdrantSearchEngine::create_on_disk("")?),
    ])
}

pub fn init_search_engines_open() -> Result<Vec<Box<dyn SearchEngine>>> {
    println!("initializing search engines");
    Ok(vec![
        Box::new(NanoSearchEngine::open_from_disk(NANO_INDEX_DIR)?),
        Box::new(TantivySearchEngine::open_from_disk(TANTIVY_INDEX_DIR)?),
        Box::new(VectorSearchEngine::open_from_disk(VECTOR_INDEX_DIR)?),
        #[cfg(feature = "qdrant")]
        Box::new(QdrantSearchEngine::open_from_disk("")?),
    ])
}

pub fn init_dataset() -> impl DocsSource + QueriesSource {
    BeirDatasetReader::new("datasets/nfcorpus")
}
