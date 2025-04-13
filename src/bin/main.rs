use nano_search::{
    docs,
    model::{doc::DocsSource, engine::SearchEngine},
    search_engines::fulltext::engine::FulltextSearchEngine,
};

fn main() {
    let mut engine = create_search_engine();
    index(&mut engine, create_docs_source());
    search("psychology", &engine);
}

fn create_docs_source() -> impl DocsSource {
    println!("creating docs source");
    // docs::cisi::parse("data/cisi/CISI.ALL".into())
    docs::simplewiki::parse("data/simplewiki/simplewiki.xml".into())
}

fn create_search_engine() -> impl SearchEngine {
    println!("creating search engine");
    FulltextSearchEngine::default()
}

fn index(engine: &mut impl SearchEngine, docs_source: impl DocsSource) {
    println!("indexing docs");
    let index_stats = engine.index_docs(&mut docs_source.into_iter());

    println!(
        "index stats: indexed docs count = {}, \
        posting lists count = {}, \
        max posting list size = {}",
        index_stats.indexed_docs_count,
        index_stats.posting_lists_count,
        index_stats.max_posting_list_size
    );
}

fn search(query: &str, engine: &impl SearchEngine) {
    println!("searching for query: {}", query);

    let found_docids = engine.search(query);

    print!("found doc IDs: ");
    for docid in found_docids {
        print!("{} ", docid);
    }
    println!();
}
