use std::time::Instant;

use anyhow::Result;

use super::common::{init_dataset, init_search_engines_create};
use crate::model::doc::{Doc, DocsSource};
use crate::model::engine::SearchEngine;

pub fn index_command() -> Result<()> {
    let mut engines = init_search_engines_create()?;
    let dataset = init_dataset();

    for engine in &mut engines {
        index(engine.as_mut(), &dataset)?;
    }

    Ok(())
}

fn index(engine: &mut dyn SearchEngine, docs: &impl DocsSource) -> Result<()> {
    println!("indexing docs with {} search engine... ", engine.get_name());
    let now = Instant::now();
    engine.index_docs(&mut log_progress(docs.docs(), docs.docs_count()))?;
    println!(
        "indexing docs with {} search engine... done in {:.1} seconds",
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
                format_number(docs_processed as f64, "docs"),
                format_number(docs_per_second, "docs/sec"),
                format_number(bytes_total as f64, "B"),
                format_number(bytes_per_second, "B/sec")
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

/// Formats a number with human-readable units (KB, MB, etc.)
pub fn format_number(value: f64, units: &str) -> String {
    human_format::Formatter::new()
        .with_units(units)
        .format(value)
}
