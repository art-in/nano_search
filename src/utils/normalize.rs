pub fn normalize_word(word: &str) -> String {
    // lowercase
    let word = word.to_lowercase();

    // remove non-alphabetic characters
    word.replace(|c: char| !c.is_alphabetic(), "")
}
