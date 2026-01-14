use std::time::Instant;

use anyhow::Result;
use colored::Colorize;

use crate::dataset_readers::utils::init_dataset_by_name;
use crate::engines::utils::engine_create_on_disk_by_names;
use crate::model::doc::{Doc, DocsSource};
use crate::model::engine::SearchEngine;
use crate::utils::format_number_si;

pub fn index(
    engines: &[String],
    dataset: &str,
    threads: Option<usize>,
) -> Result<()> {
    println!("initializing search engines: {}", engines.join(","));
    println!("initializing dataset '{dataset}'");

    let mut engines = engine_create_on_disk_by_names(engines, threads)?;
    let dataset = init_dataset_by_name(dataset)?;

    for engine in &mut engines {
        index_with_engine(engine.as_mut(), dataset.as_ref())?;
    }

    Ok(())
}

fn index_with_engine(
    engine: &mut dyn SearchEngine,
    docs: &dyn DocsSource,
) -> Result<()> {
    println!("indexing docs with {} engine... ", engine.get_name().red());
    let now = Instant::now();

    engine.index_docs(&mut log_progress(docs.docs()?, docs.docs_count()?))?;

    println!(
        "indexing docs with {} engine... done in {:.1} seconds",
        engine.get_name(),
        now.elapsed().as_secs_f32()
    );

    Ok(())
}

/// Creates an iterator that logs progress while iterating over documents
pub fn log_progress(
    it: impl Iterator<Item = Doc>,
    docs_count: Option<usize>,
) -> impl Iterator<Item = Doc> {
    let mut docs_processed = 0;
    let mut docs_processed_in_period = 0;
    let mut bytes_total = 0;
    let mut bytes_in_period = 0;
    let mut period_start = Instant::now();
    let start = Instant::now();

    it.inspect(move |doc| {
        docs_processed += 1;
        docs_processed_in_period += 1;

        bytes_total += doc.text.len();
        bytes_in_period += doc.text.len();

        let period_seconds = period_start.elapsed().as_secs_f64();

        if period_seconds >= 10.0 {
            let docs_per_second =
                docs_processed_in_period as f64 / period_seconds;
            let bytes_per_second = bytes_in_period as f64 / period_seconds;

            print!(
                "{} ({}) {} ({}) ",
                format_number_si(docs_processed, "docs"),
                format_number_si(docs_per_second, "docs/sec"),
                format_number_si(bytes_total, "B"),
                format_number_si(bytes_per_second, "B/sec")
            );

            if let Some(docs_count) = docs_count {
                let docs_per_second =
                    docs_processed as f64 / start.elapsed().as_secs_f64();
                let docs_remaining = docs_count - docs_processed;
                let eta_seconds = docs_remaining as f64 / docs_per_second;
                let eta_minutes = eta_seconds / 60.0;

                print!("ETA ~{} minutes", eta_minutes as usize)
            }

            println!();

            docs_processed_in_period = 0;
            bytes_in_period = 0;

            period_start = Instant::now();
        }
    })
}
