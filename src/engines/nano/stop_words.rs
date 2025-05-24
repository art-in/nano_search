use std::collections::HashSet;

pub fn get_stop_words() -> HashSet<String> {
    // using same stop words as Apache Lucene in
    // https://github.com/apache/lucene/blob/41abd7ad3169fb54a2573341d2ab3fef815758ea/lucene/analysis/common/src/java/org/apache/lucene/analysis/en/EnglishAnalyzer.java#L47
    [
        "a", "an", "and", "are", "as", "at", "be", "but", "by", "for", "if",
        "in", "into", "is", "it", "no", "not", "of", "on", "or", "such",
        "that", "the", "their", "then", "there", "these", "they", "this", "to",
        "was", "will", "with",
    ]
    .map(|s| s.to_string())
    .into()
}
