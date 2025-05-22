// calculates term frequency – inverse document frequency.
// using count-idf weighting scheme - tf*log(N/n)
// https://en.wikipedia.org/wiki/Tf-idf
pub fn calc_tfidf(
    doc_term_count: u64,        // number of the term in the doc
    doc_total_terms_count: u64, // total number of terms in the doc
    docs_with_term_count: u64,  // number of docs containing this term
    docs_total_count: u64,      // total number of docs in the index
) -> f64 {
    let term_frequency = doc_term_count as f64 / doc_total_terms_count as f64;

    let inverted_doc_frequency =
        f64::ln(docs_total_count as f64 / docs_with_term_count as f64);

    term_frequency * inverted_doc_frequency
}

// calculates BM25 score for single doc term
// https://en.wikipedia.org/wiki/Okapi_BM25
pub fn calc_bm25(
    doc_term_count: u64,          // number of the term in the doc
    doc_total_terms_count: u64,   // total number of terms in the doc
    docs_with_term_count: u64,    // number of docs containing the term
    docs_total_count: u64,        // total number of docs in the index
    terms_count_per_doc_avg: f64, // average number of terms in docs
) -> f64 {
    let term_frequency = {
        const K: f64 = 1.2; // [1.2, 2.0]
        const B: f64 = 0.75;

        let numerator = doc_term_count as f64 * (K + 1.0);
        let denominator = doc_term_count as f64
            + K * (1.0 - B
                + ((B * doc_total_terms_count as f64)
                    / terms_count_per_doc_avg));

        numerator / denominator
    };

    let inverted_doc_frequency = f64::ln(
        ((docs_total_count as f64 - docs_with_term_count as f64 + 0.5)
            / (docs_with_term_count as f64 + 0.5))
            + 1.0,
    );

    term_frequency * inverted_doc_frequency
}
