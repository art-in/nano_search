use nano_search::utils;

use std::io::{Read, Write};
use std::path::Path;
use std::time::Instant;

fn main() {
    print!("type search word: ");
    std::io::stdout()
        .flush()
        .expect("prompt should be fully flushed to stdout");

    let mut search_word = String::new();
    std::io::stdin()
        .read_line(&mut search_word)
        .expect("search word should be read from stdin");

    let search_word = search_word.trim();

    let start = Instant::now();
    let docids = search(search_word);
    let search_duration = start.elapsed();

    print!("found docids: ");
    for docid in docids {
        print!("{docid} ");
    }
    println!();

    println!("search duration: {}us", search_duration.as_micros());
}

fn search(search_word: &str) -> Vec<u64> {
    let dir_path = Path::new("data/docs");
    let mut docids = std::collections::BTreeSet::new();

    utils::visit_dir_files(dir_path, &mut |path| {
        let mut file =
            std::fs::File::open(path.clone()).expect("doc file should exist");

        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .expect("file should contain valid unicode");

        if buffer.contains(search_word) {
            docids.insert(
                path.file_name()
                    .expect("filename should be present")
                    .to_str()
                    .expect("filename should be a valid unicode")
                    .parse::<u64>()
                    .expect("filename should be integer"),
            );
        }
    });

    let mut res = Vec::new();

    for docid in docids {
        res.push(docid);
    }

    res
}
