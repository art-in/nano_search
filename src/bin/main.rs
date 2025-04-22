use nano_search::{
    docs::{self, cisi},
    model::{
        doc::{DocId, DocsSource},
        engine::SearchEngine,
    },
    search_engines::{
        nano::engine::NanoSearchEngine, tantivy::engine::TantivySearchEngine,
    },
    utils::compare_arrays,
};
use std::time::Instant;

fn main() {
    let mut engines = create_search_engines();
    for engine in &mut engines {
        index(engine.as_mut(), create_docs_source());
    }

    let mut search_results = Vec::new();
    for engine in &engines {
        let found_docids = search("psychology", engine.as_ref());
        search_results.push(found_docids);
    }

    compare_search_results(&search_results);
}

fn create_docs_source() -> impl DocsSource {
    print!("creating docs source... ");
    let now = Instant::now();
    let res = docs::cisi::parse("data/cisi/CISI.ALL".into());
    // let res = docs::simplewiki::parse("data/simplewiki/simplewiki.xml".into());
    println!("done in {}ms", now.elapsed().as_millis());
    res
}

fn create_search_engines() -> Vec<Box<dyn SearchEngine>> {
    println!("creating search engines");
    vec![
        Box::new(TantivySearchEngine::default()),
        Box::new(NanoSearchEngine::default()),
    ]
}

fn index(engine: &mut dyn SearchEngine, docs_source: impl DocsSource) {
    print!("indexing docs with {} search engine... ", engine.get_name());
    let now = Instant::now();
    engine.index_docs(&mut docs_source.into_iter());
    println!("done in {}ms", now.elapsed().as_millis());
}

#[allow(dead_code)]
fn search_and_calc_quality(engine: &dyn SearchEngine) {
    println!("searching queries with {} search engine", engine.get_name());

    let quality = cisi::search_quality::search_and_calc_quality(engine);

    println!("precision avg = {}", quality.precision);
    println!("recall avg = {}", quality.recall);
}

fn search(query: &str, engine: &dyn SearchEngine) -> Vec<DocId> {
    print!(
        "searching for query '{}' with {} search engine... ",
        query,
        engine.get_name()
    );

    let now = Instant::now();
    let found_docids = engine.search(query, 20);
    println!("done in {}ms", now.elapsed().as_millis());

    print!("found doc IDs: ");
    for docid in &found_docids {
        print!("{} ", docid);
    }
    println!();

    found_docids
}

fn compare_search_results(search_results: &[Vec<DocId>]) {
    if search_results.len() >= 2 {
        let compare_result =
            compare_arrays(&search_results[0], &search_results[1]);
        println!("comparing search results: {}", compare_result);
    }
}
