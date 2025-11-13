use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::{Context, Result};
use flate2::read::GzDecoder;

/// Gets lines iterator over target file.
/// Decompresses the file if it's gzip-compressed.
pub fn get_file_lines(
    file_path: &Path,
) -> Result<Box<dyn Iterator<Item = std::io::Result<String>>>> {
    let file = File::open(file_path)?;

    // more reliable way is to check magic number at file start, but checking
    // extension is faster and code-simpler, so stick with it for now
    let extension = file_path
        .extension()
        .context("file should have an extension")?;
    let is_gzipped = extension == "gz" || extension == "gzip";

    if is_gzipped {
        let decoder = GzDecoder::new(file);
        let reader = BufReader::new(decoder);
        Ok(Box::new(reader.lines()))
    } else {
        let reader = BufReader::new(file);
        Ok(Box::new(reader.lines()))
    }
}
