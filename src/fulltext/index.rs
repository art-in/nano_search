use std::collections::{BTreeSet, HashMap};
use std::io::Read;
use std::path::Path;

pub fn build_index() -> HashMap<String, BTreeSet<u64>> {
    let mut index: HashMap<String, BTreeSet<u64>> = HashMap::new();

    let dir_path = Path::new("data/docs");
    println!("indexing documents in folder: {}", dir_path.display());

    crate::utils::visit_dir_files(dir_path, &mut |path| {
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
            let word = crate::utils::normalize_word(word);

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

pub fn search(
    text: &str,
    index: &HashMap<String, BTreeSet<u64>>,
) -> BTreeSet<u64> {
    let words: Vec<_> = text.split_whitespace().collect();

    let mut docids = BTreeSet::new();

    for word in words {
        let found_docids = index.get(word).unwrap_or(&BTreeSet::new()).clone();

        for docid in found_docids {
            docids.insert(docid);
        }
    }

    docids
}
