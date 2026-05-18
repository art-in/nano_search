use std::fs::File;
use std::path::PathBuf;

use anyhow::{Context, Result};
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::Row;

// Get rows iterator over parquet dataset.
// Dataset is represented by multiple files, because big dataset can be
// split-sharded into multiple files.
pub fn get_parquet_rows(
    file_paths: Vec<PathBuf>,
) -> Result<Box<dyn Iterator<Item = Result<Row>>>> {
    let iter = file_paths.into_iter().flat_map(|file_path| {
        let process_file = || -> Result<Vec<Result<Row>>> {
            let file = File::open(&file_path).with_context(|| {
                format!(
                    "parquet file should be opened: {}",
                    file_path.display()
                )
            })?;

            let reader = SerializedFileReader::new(file).with_context(
                || -> String {
                    format!(
                        "parquet reader should be created for file: {}",
                        file_path.display()
                    )
                },
            )?;

            let iter = reader.get_row_iter(None).with_context(|| {
                format!(
                    "iterator should be initialized for parquet file: {}",
                    file_path.display()
                )
            })?;

            let rows: Vec<Result<Row>> = iter
                .map(|res| res.context("error reading row from parquet stream"))
                .collect();

            Ok(rows)
        };

        process_file().map_or_else(
            |err| vec![Err(err)].into_iter(),
            std::iter::IntoIterator::into_iter,
        )
    });

    Ok(Box::new(iter))
}
