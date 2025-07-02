use std::time::Instant;

use anyhow::Result;
use nano_search::docs;
use nano_search::engines::nano::engine::NanoSearchEngine;
use nano_search::engines::tantivy::engine::TantivySearchEngine;
use nano_search::model::doc::{Doc, DocsSource};
use nano_search::model::engine::SearchEngine;

fn main() -> Result<()> {
    let docs_source = create_docs_source()?;
    let mut engines = init_search_engines()?;

    for engine in &mut engines {
        index(engine.as_mut(), docs_source.clone())?;
    }

    Ok(())
}

fn init_search_engines() -> Result<Vec<Box<dyn SearchEngine>>> {
    println!("initializing search engines");
    Ok(vec![
        Box::new(TantivySearchEngine::create_on_disk("index_tantivy")?),
        Box::new(NanoSearchEngine::create_on_disk("index_nano")?),
    ])
}

fn create_docs_source() -> Result<impl DocsSource> {
    print!("creating docs source... ");
    let now = Instant::now();
    // let res = docs::cisi::load_docs()?;
    // let res = docs::wiki::WikiDocs::new("data/simplewiki/dump.xml.bz2")?;
    // let res = docs::wiki::WikiDocs::new("data/enwiki/dump.xml.bz2")?;
    let res = docs::wiki_parsed::WikiDocs::new(
        "data/enwiki_parsed/wiki-articles.json",
    );
    println!("done in {}ms", now.elapsed().as_millis());
    Ok(res)
}

fn index(
    engine: &mut dyn SearchEngine,
    docs_source: impl DocsSource,
) -> Result<()> {
    println!("indexing docs with {} search engine... ", engine.get_name());
    let now = Instant::now();
    engine.index_docs(&mut log_progress(docs_source.into_iter()))?;
    println!("done in {}ms", now.elapsed().as_millis());

    Ok(())
}

fn log_progress(it: impl Iterator<Item = Doc>) -> impl Iterator<Item = Doc> {
    let mut docs_total = 0;
    let mut docs_in_period = 0;
    let mut bytes_total = 0;
    let mut bytes_in_period = 0;
    let mut period_start = Instant::now();

    it.inspect(move |doc| {
        docs_total += 1;
        docs_in_period += 1;

        bytes_total += doc.text.len();
        bytes_in_period += doc.text.len();

        let elapsed_seconds = period_start.elapsed().as_secs_f64();

        if elapsed_seconds >= 10.0 {
            let docs_per_second = docs_in_period as f64 / elapsed_seconds;
            let bytes_per_second = bytes_in_period as f64 / elapsed_seconds;

            println!(
                "{} ({}) {} ({})",
                format_number(docs_total as f64, "docs"),
                format_number(docs_per_second, "docs/sec"),
                format_number(bytes_total as f64, "B"),
                format_number(bytes_per_second, "B/sec"),
            );

            docs_in_period = 0;
            bytes_in_period = 0;

            period_start = Instant::now();
        }
    })
}

fn format_number(value: f64, units: &str) -> String {
    human_format::Formatter::new()
        .with_units(units)
        .format(value)
}
