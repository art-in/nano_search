use nano_search::{
    docs::{self, cisi},
    model::{doc::DocsSource, engine::SearchEngine},
    search_engines::{
        nano::engine::NanoSearchEngine, tantivy::engine::TantivySearchEngine,
    },
};
use std::time::Instant;

fn main() {
    let mut engines = create_search_engines();
    for engine in &mut engines {
        index(engine.as_mut(), create_docs_source());
    }

    for engine in &engines {
        println!("searching queries with {} search engine", engine.get_name());

        let quality = cisi::search_quality::get_search_quality(engine.as_ref());

        println!("precision avg = {}", quality.precision);
        println!("recall avg = {}", quality.recall);
    }
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
fn search(query: &str, engine: &dyn SearchEngine) {
    print!(
        "searching for query '{}' with {} search engine... ",
        query,
        engine.get_name()
    );

    let now = Instant::now();
    let found_docids = engine.search(query, 10);
    println!("done in {}ms", now.elapsed().as_millis());

    print!("found doc IDs: ");
    for docid in found_docids {
        print!("{} ", docid);
    }
    println!();
}
