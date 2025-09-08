use std::time::Instant;

use anyhow::Result;
use nano_search::engines::nano::engine::NanoSearchEngine;
use nano_search::engines::tantivy::engine::TantivySearchEngine;
use nano_search::model::doc::DocId;
use nano_search::model::engine::SearchEngine;
use nano_search::utils::{GetPercentile, compare_ranked_arrays};

const TOP_K: u64 = 10;
const QUERIES: [&str; 10] = [
    "Solar eclipse",
    "Great Wall of China",
    "Albert Einstein",
    "Machine learning",
    "Leonardo da Vinci",
    "Photosynthesis",
    "Olympic Games",
    "Internet Protocol",
    "Mount Kilimanjaro",
    "Taj Mahal",
];

fn main() -> Result<()> {
    let engines = init_search_engines()?;

    let mut similarities = inc_stats::Percentiles::new();

    for query in QUERIES {
        let a = search(query, engines.0.as_ref())?;
        let b = search(query, engines.1.as_ref())?;

        let similarity = compare_ranked_arrays(&a, &b)?;
        println!("similarity: {:.2}%", similarity * 100.0);

        similarities.add(similarity);
    }

    println!("similarity (p50): {:.2}%", similarities.perc(0.5)? * 100.0);
    println!("similarity (p90): {:.2}%", similarities.perc(0.9)? * 100.0);

    Ok(())
}

fn init_search_engines()
-> Result<(Box<dyn SearchEngine>, Box<dyn SearchEngine>)> {
    println!("initializing search engines");
    Ok((
        Box::new(TantivySearchEngine::open_from_disk("index_tantivy")?),
        Box::new(NanoSearchEngine::open_from_disk("index_nano")?),
    ))
}

fn search(query: &str, engine: &dyn SearchEngine) -> Result<Vec<DocId>> {
    print!(
        "searching for query '{}' with {} search engine... ",
        query,
        engine.get_name()
    );

    let now = Instant::now();
    let found_docids = engine.search(query, TOP_K)?;
    println!("done in {}ms", now.elapsed().as_millis());

    println!(
        "found doc IDs: {}",
        found_docids
            .iter()
            .map(|doc_id| doc_id.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    );

    Ok(found_docids)
}
