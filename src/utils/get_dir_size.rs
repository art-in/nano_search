use std::fs;
use std::path::Path;

use anyhow::Result;

// Gets byte size of file system directory.
pub fn get_dir_size(dir: impl AsRef<Path>) -> Result<u64> {
    let mut total = 0;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            total += get_dir_size(entry.path())?;
        } else {
            total += metadata.len();
        }
    }

    Ok(total)
}
