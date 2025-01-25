use nano_search::utils;

use std::io::{Read, Write};
use std::path::Path;
use std::time::Instant;

fn main() {
    print!("type search word: ");
    std::io::stdout().flush().unwrap();

    let mut search_word = String::new();
    std::io::stdin().read_line(&mut search_word).unwrap();

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
        let mut file = std::fs::File::open(path.clone()).unwrap();

        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();

        if buffer.contains(search_word) {
            docids.insert(
                path.file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .parse::<u64>()
                    .unwrap(),
            );
        }
    });

    let mut res = Vec::new();

    for docid in docids {
        res.push(docid);
    }

    res
}
