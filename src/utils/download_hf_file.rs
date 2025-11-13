use std::path::PathBuf;

use anyhow::Result;
use hf_hub::api::sync::ApiBuilder;

pub const HF_CACHE_DIR: &str = ".hf_cache";

/// Downloads a file from huggingface.co repo
///
/// Downloaded file is cached on disk, so future calls for the same file will
/// not require download. No network calls happen in case file is in cache, so
/// it will not download latest version of the file if it was updated in repo
pub fn download_hf_file(repo_id: &str, file_name: &str) -> Result<PathBuf> {
    let api = ApiBuilder::new()
        .with_cache_dir(HF_CACHE_DIR.into())
        .build()?;

    let dataset = api.dataset(repo_id.into());
    let file_path = dataset.get(file_name)?;

    Ok(file_path)
}
