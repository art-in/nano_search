pub fn normalize_word(word: &str) -> String {
    // remove non-alphabetic characters
    let word = word.replace(|c: char| !c.is_alphabetic(), "");

    // make lowercase
    word.to_lowercase()
}
