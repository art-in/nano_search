use std::{
    collections::{BTreeSet, HashMap},
    io::{Read, Write},
    path::Path,
    time::Instant,
};

use nano_search::utils;

fn main() {
    let index = build_index();

    print!("type search word: ");
    std::io::stdout()
        .flush()
        .expect("prompt should be fully flushed to stdout");

    let mut search_word = String::new();
    std::io::stdin()
        .read_line(&mut search_word)
        .expect("search word should be read from stdin");

    let search_word = search_word.trim();
    let search_word = utils::normalize_word(search_word);

    let start = Instant::now();
    let docids = search(&search_word, &index);
    let search_duration = start.elapsed();

    print!("found docids: ");
    for docid in docids {
        print!("{docid} ");
    }
    println!();

    println!("time spent: {}us", search_duration.as_micros());
}

fn build_index() -> HashMap<String, BTreeSet<u64>> {
    let mut index: HashMap<String, BTreeSet<u64>> = HashMap::new();

    let dir_path = Path::new("data/docs");
    println!("indexing documents in folder: {}", dir_path.display());

    nano_search::utils::visit_dir_files(dir_path, &mut |path| {
        let mut file =
            std::fs::File::open(path.clone()).expect("file should exist");

        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .expect("file should contain valid unicode");

        let docid = path
            .file_name()
            .expect("filename should be extracted from path")
            .to_str()
            .expect("filename should be converted to string")
            .parse::<u64>()
            .expect("filename should be parsed to integer");

        let words: Vec<&str> = buffer.split(' ').collect();

        for word in words {
            let word = utils::normalize_word(word);

            if word.is_empty() {
                continue;
            }

            match index.get_mut(&word) {
                Some(docids) => {
                    docids.insert(docid);
                }
                None => {
                    index.insert(word, BTreeSet::from([docid]));
                }
            }
        }
    });

    index
}

fn search(word: &str, index: &HashMap<String, BTreeSet<u64>>) -> BTreeSet<u64> {
    index.get(word).unwrap_or(&BTreeSet::new()).clone()
}
