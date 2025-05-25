use anyhow::{Context, Result};
use nano_search::{
    docs::{self, cisi},
    engines::{
        nano::engine::NanoSearchEngine, tantivy::engine::TantivySearchEngine,
    },
    model::{
        doc::{DocId, DocsSource},
        engine::SearchEngine,
    },
    utils::compare_arrays,
};
use std::time::Instant;

fn main() -> Result<()> {
    let docs_source = create_docs_source()?;

    let mut engines = init_search_engines()?;
    for engine in &mut engines {
        index(engine.as_mut(), docs_source.clone())?;
    }

    for engine in &engines {
        search_and_calc_quality(engine.as_ref())?;
    }

    Ok(())
}

fn create_docs_source() -> Result<impl DocsSource> {
    print!("creating docs source... ");
    let now = Instant::now();
    let res = docs::cisi::load_docs()?;
    // let res = docs::simplewiki::load_docs()?;
    println!("done in {}ms", now.elapsed().as_millis());
    Ok(res)
}

fn init_search_engines() -> Result<Vec<Box<dyn SearchEngine>>> {
    println!("initializing search engines");
    Ok(vec![
        Box::new(TantivySearchEngine::create_index("index_tantivy")?),
        Box::new(NanoSearchEngine::create_index("index_nano")?),
    ])
}

fn index(
    engine: &mut dyn SearchEngine,
    docs_source: impl DocsSource,
) -> Result<()> {
    print!("indexing docs with {} search engine... ", engine.get_name());
    let now = Instant::now();
    engine.index_docs(&mut docs_source.into_iter())?;
    println!("done in {}ms", now.elapsed().as_millis());

    Ok(())
}

#[allow(dead_code)]
fn search(query: &str, engine: &dyn SearchEngine) -> Result<Vec<DocId>> {
    print!(
        "searching for query '{}' with {} search engine... ",
        query,
        engine.get_name()
    );

    let now = Instant::now();
    let found_docids = engine.search(query, 20)?;
    println!("done in {}ms", now.elapsed().as_millis());

    print!("found doc IDs: ");
    for docid in &found_docids {
        print!("{} ", docid);
    }
    println!();

    Ok(found_docids)
}

#[allow(dead_code)]
fn search_and_calc_quality(engine: &dyn SearchEngine) -> Result<()> {
    print!("searching with {} search engine... ", engine.get_name());
    let now = Instant::now();
    let quality = cisi::search_quality::search_and_calc_quality(engine)?;
    println!("done in {}ms", now.elapsed().as_millis());

    println!("processed {} queries", quality.queries_count);

    let precision_percs = quality
        .precision_percs
        .percentiles([0.5, 0.9, 1.0])
        .context("percentiles should be calculated")?
        .context("percentiles should exist in result")?;

    println!(
        "precision: avg={:.1}%, p50={:.1}%, p90={:.1}%, p100={:.1}%",
        quality.precision_avg * 100.0,
        precision_percs[0] * 100.0,
        precision_percs[1] * 100.0,
        precision_percs[2] * 100.0
    );

    let recall_percs = quality
        .recall_percs
        .percentiles([0.5, 0.9, 1.0])
        .context("percentiles should be calculated")?
        .context("percentiles should exist in result")?;

    println!(
        "recall: avg={:.1}%, p50={:.1}%, p90={:.1}%, p100={:.1}%",
        quality.recall_avg * 100.0,
        recall_percs[0] * 100.0,
        recall_percs[1] * 100.0,
        recall_percs[2] * 100.0
    );

    Ok(())
}

#[allow(dead_code)]
fn compare_search_results(search_results: &[Vec<DocId>]) {
    if search_results.len() >= 2 {
        let compare_result =
            compare_arrays(&search_results[0], &search_results[1]);
        println!("comparing search results: {}", compare_result);
    }
}
