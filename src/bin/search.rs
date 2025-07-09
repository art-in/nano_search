use std::time::Instant;

use anyhow::Result;
use itertools::Itertools;
use nano_search::engines::nano::engine::NanoSearchEngine;
use nano_search::engines::tantivy::engine::TantivySearchEngine;
use nano_search::model::doc::DocId;
use nano_search::model::engine::SearchEngine;
use nano_search::utils::compare_ranked_arrays;

fn main() -> Result<()> {
    let engines = init_search_engines()?;

    let mut results = Vec::new();

    for engine in &engines {
        let res = search("needle", engine.as_ref())?;
        results.push(res);
    }

    compare_search_results(&results)?;

    Ok(())
}

fn init_search_engines() -> Result<Vec<Box<dyn SearchEngine>>> {
    println!("initializing search engines");
    Ok(vec![
        Box::new(TantivySearchEngine::open_from_disk("index_tantivy")?),
        Box::new(NanoSearchEngine::open_from_disk("index_nano")?),
    ])
}

fn search(query: &str, engine: &dyn SearchEngine) -> Result<Vec<DocId>> {
    print!(
        "searching for query '{}' with {} search engine... ",
        query,
        engine.get_name()
    );

    let now = Instant::now();
    let found_docids = engine.search(query, 10)?;
    println!("done in {}ms", now.elapsed().as_millis());

    print!("found doc IDs: ");
    for docid in &found_docids {
        print!("{docid} ");
    }
    println!();

    Ok(found_docids)
}

fn compare_search_results(results: &[Vec<DocId>]) -> Result<()> {
    for (idx_a, idx_b) in (0..results.len()).tuple_combinations() {
        let a = &results[idx_a];
        let b = &results[idx_b];

        let similarity = compare_ranked_arrays(a, b)?;

        println!(
            "results similarity ({idx_a}-{idx_b}): {:.2}%",
            similarity * 100.0
        );
    }
    Ok(())
}
