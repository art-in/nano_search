use std::path::PathBuf;

use anyhow::{Result, bail};

use crate::engines::nano::engine::NanoSearchEngine;
use crate::engines::tantivy::engine::TantivySearchEngine;
use crate::engines::vector::engine::VectorSearchEngine;
use crate::model::engine::{CreateOnDiskOptions, SearchEngine};

#[must_use]
pub fn get_all_engine_names() -> Vec<String> {
    Vec::from([
        NanoSearchEngine::name().to_string(),
        TantivySearchEngine::name().to_string(),
        VectorSearchEngine::name().to_string(),
    ])
}

#[must_use]
pub fn get_engine_index_dir(
    parent_index_dir: &str,
    engine_name: &str,
) -> PathBuf {
    PathBuf::from(parent_index_dir).join("index_".to_string() + engine_name)
}

pub fn engine_create_on_disk_by_name(
    engine_name: &str,
    threads: Option<usize>,
    parent_index_dir: &str,
) -> Result<Box<dyn SearchEngine>> {
    let index_dir = get_engine_index_dir(parent_index_dir, engine_name);

    let engine: Box<dyn SearchEngine> = match engine_name {
        n if n == NanoSearchEngine::name() => {
            Box::new(NanoSearchEngine::create_on_disk(
                CreateOnDiskOptions::builder()
                    .index_dir(index_dir)
                    .maybe_index_threads(threads)
                    .build(),
            )?)
        }
        n if n == TantivySearchEngine::name() => {
            Box::new(TantivySearchEngine::create_on_disk(
                CreateOnDiskOptions::builder()
                    .index_dir(index_dir)
                    .maybe_index_threads(threads)
                    .build(),
            )?)
        }
        n if n == VectorSearchEngine::name() => {
            Box::new(VectorSearchEngine::create_on_disk(
                CreateOnDiskOptions::builder().index_dir(index_dir).build(),
            )?)
        }
        _ => bail!("unknown engine '{engine_name}'"),
    };

    Ok(engine)
}

pub fn engine_open_from_disk_by_name(
    engine_name: &str,
    parent_index_dir: &str,
) -> Result<Box<dyn SearchEngine>> {
    let index_dir = get_engine_index_dir(parent_index_dir, engine_name);

    let engine: Box<dyn SearchEngine> = match engine_name {
        n if n == NanoSearchEngine::name() => {
            Box::new(NanoSearchEngine::open_from_disk(index_dir)?)
        }
        n if n == TantivySearchEngine::name() => {
            Box::new(TantivySearchEngine::open_from_disk(index_dir)?)
        }
        n if n == VectorSearchEngine::name() => {
            Box::new(VectorSearchEngine::open_from_disk(index_dir)?)
        }
        _ => bail!("unknown engine '{engine_name}'"),
    };
    Ok(engine)
}

pub fn engine_create_on_disk_by_names(
    engine_names: &[String],
    threads: Option<usize>,
    parent_index_dir: &str,
) -> Result<Vec<Box<dyn SearchEngine>>> {
    engine_names
        .iter()
        .map(|engine_name| {
            engine_create_on_disk_by_name(
                engine_name,
                threads,
                parent_index_dir,
            )
        })
        .collect::<Result<Vec<_>>>()
}

pub fn engine_open_from_disk_by_names(
    engine_names: &[String],
    parent_index_dir: &str,
) -> Result<Vec<Box<dyn SearchEngine>>> {
    engine_names
        .iter()
        .map(|engine_name| {
            engine_open_from_disk_by_name(engine_name, parent_index_dir)
        })
        .collect::<Result<Vec<_>>>()
}
