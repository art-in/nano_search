use anyhow::{Result, bail};

use crate::engines::nano::engine::NanoSearchEngine;
use crate::engines::tantivy::engine::TantivySearchEngine;
use crate::engines::vector::engine::VectorSearchEngine;
use crate::model::engine::{CreateOnDiskOptions, SearchEngine};

const NANO_INDEX_DIR: &str = "index_nano";
const TANTIVY_INDEX_DIR: &str = "index_tantivy";
const VECTOR_INDEX_DIR: &str = "index_vector";

pub fn get_all_engine_names() -> Vec<String> {
    Vec::from([
        NanoSearchEngine::name().to_string(),
        TantivySearchEngine::name().to_string(),
        VectorSearchEngine::name().to_string(),
    ])
}

pub fn engine_create_on_disk_by_name(
    engine_name: &str,
    threads: Option<usize>,
) -> Result<Box<dyn SearchEngine>> {
    let engine: Box<dyn SearchEngine> = match engine_name {
        n if n == NanoSearchEngine::name() => {
            Box::new(NanoSearchEngine::create_on_disk(
                CreateOnDiskOptions::builder()
                    .index_dir(NANO_INDEX_DIR)
                    .maybe_index_threads(threads)
                    .build(),
            )?)
        }
        n if n == TantivySearchEngine::name() => {
            Box::new(TantivySearchEngine::create_on_disk(
                CreateOnDiskOptions::builder()
                    .index_dir(TANTIVY_INDEX_DIR)
                    .maybe_index_threads(threads)
                    .build(),
            )?)
        }
        n if n == VectorSearchEngine::name() => {
            Box::new(VectorSearchEngine::create_on_disk(
                CreateOnDiskOptions::builder()
                    .index_dir(VECTOR_INDEX_DIR)
                    .build(),
            )?)
        }
        _ => bail!("unknown engine '{engine_name}'"),
    };
    Ok(engine)
}

pub fn engine_open_from_disk_by_name(
    engine_name: &str,
) -> Result<Box<dyn SearchEngine>> {
    let engine: Box<dyn SearchEngine> = match engine_name {
        n if n == NanoSearchEngine::name() => {
            Box::new(NanoSearchEngine::open_from_disk(NANO_INDEX_DIR)?)
        }
        n if n == TantivySearchEngine::name() => {
            Box::new(TantivySearchEngine::open_from_disk(TANTIVY_INDEX_DIR)?)
        }
        n if n == VectorSearchEngine::name() => {
            Box::new(VectorSearchEngine::open_from_disk(VECTOR_INDEX_DIR)?)
        }
        _ => bail!("unknown engine '{engine_name}'"),
    };
    Ok(engine)
}

pub fn engine_create_on_disk_by_names(
    engine_names: &[String],
    threads: Option<usize>,
) -> Result<Vec<Box<dyn SearchEngine>>> {
    engine_names
        .iter()
        .map(|engine_name| engine_create_on_disk_by_name(engine_name, threads))
        .collect::<Result<Vec<_>>>()
}

pub fn engine_open_from_disk_by_names(
    engine_names: &[String],
) -> Result<Vec<Box<dyn SearchEngine>>> {
    engine_names
        .iter()
        .map(|engine_name| engine_open_from_disk_by_name(engine_name))
        .collect::<Result<Vec<_>>>()
}
