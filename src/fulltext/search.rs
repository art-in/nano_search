use std::collections::{BTreeSet, HashMap, HashSet};

pub fn search(
    text: &str,
    index: &HashMap<String, BTreeSet<u64>>,
    stop_words: &HashSet<String>,
) -> BTreeSet<u64> {
    let words: Vec<_> = text.split_whitespace().collect();

    let mut docids = BTreeSet::new();

    for word in words {
        let word = crate::utils::normalize_word(word);

        if stop_words.contains(&word) {
            continue;
        }

        let found_docids = index.get(&word).unwrap_or(&BTreeSet::new()).clone();

        for docid in found_docids {
            docids.insert(docid);
        }
    }

    docids
}
