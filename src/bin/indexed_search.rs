use std::{io::Write, time::Instant};

fn main() {
    let index = nano_search::fulltext::index::build_index();

    print!("type search word: ");
    std::io::stdout()
        .flush()
        .expect("prompt should be fully flushed to stdout");

    let mut search_word = String::new();
    std::io::stdin()
        .read_line(&mut search_word)
        .expect("search word should be read from stdin");

    let search_word = search_word.trim();
    let search_word = nano_search::utils::normalize_word(search_word);

    let start = Instant::now();
    let docids = nano_search::fulltext::index::search(&search_word, &index);
    let search_duration = start.elapsed();

    print!("found docids: ");
    for docid in docids {
        print!("{docid} ");
    }
    println!();

    println!("time spent: {}us", search_duration.as_micros());
}
