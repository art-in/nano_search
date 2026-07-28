// TODO: add stemming/lemmatization
#[must_use]
pub fn normalize_word(word: &str) -> Option<String> {
    // remove non-alphabetic characters
    let word = word.replace(|c: char| !c.is_alphabetic(), "");

    // make lowercase
    let word = word.to_lowercase();

    if word.is_empty() { None } else { Some(word) }
}
