use std::path::PathBuf;

use anyhow::{Result, bail};
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

/// Downloads a directory from huggingface.co repo
///
/// E.g. this is useful if target dataset split into multiple files.
pub fn download_hf_dir(
    repo_id: &str,
    dir: &str,
    file_extentions: &str,
) -> Result<Vec<PathBuf>> {
    let api = ApiBuilder::new()
        .with_cache_dir(HF_CACHE_DIR.into())
        .build()?;

    let dataset = api.dataset(repo_id.into());
    let files = dataset.info()?.siblings;
    let files = files.iter().filter(|f| {
        f.rfilename.starts_with(dir) && f.rfilename.ends_with(file_extentions)
    });

    if files.clone().count() == 0 {
        bail!("hf dir should not be empty");
    }

    let mut file_paths = Vec::new();

    for file in files {
        let filename = &file.rfilename;
        let file_path = dataset.get(filename)?;
        file_paths.push(file_path);
    }

    file_paths.sort();

    Ok(file_paths)
}
