//! Scoring module implements various document scoring algorithms for search
//! relevance.

/// Common parameters used across scoring algorithms
#[derive(Debug, Clone, Copy)]
pub struct ScoringParams {
    /// Number of occurrences of the term in the document
    pub doc_term_count: u64,
    /// Total count of terms in the document (i.e. document length)
    pub doc_total_terms_count: u16,
    /// Count of documents containing this term in the index
    pub docs_with_term_count: u64,
    /// Total number of documents in the index
    pub docs_total_count: u64,
}

/// Calculates TF-IDF score for single document term.
///
/// TF-IDF stands for Term Frequency-Inverse Document Frequency
///
/// Uses count-idf weighting scheme: tf * log(N/n)
/// where:
/// - tf = term frequency in document
/// - N = total number of documents
/// - n = number of documents containing the term
///
/// # References
/// - [TF-IDF on Wikipedia](https://en.wikipedia.org/wiki/Tf-idf)
#[allow(dead_code)]
pub fn calc_tfidf(p: ScoringParams) -> f64 {
    let term_frequency =
        p.doc_term_count as f64 / p.doc_total_terms_count as f64;

    let inverted_doc_frequency =
        f64::ln(p.docs_total_count as f64 / p.docs_with_term_count as f64);

    term_frequency * inverted_doc_frequency
}

/// Calculates Okapi BM25 score for a single document term.
///
/// BM25 improves upon basic TF-IDF by normalizing term frequency saturation
/// and document length.
///
/// # Arguments
/// * `params` - Common scoring parameters
/// * `terms_count_per_doc_avg` - Average number of terms across all documents
///   in the index
///
/// # References
/// - [Okapi BM25 on Wikipedia](https://en.wikipedia.org/wiki/Okapi_BM25)
pub fn calc_bm25(p: ScoringParams, terms_count_per_doc_avg: f64) -> f64 {
    const K: f64 = 1.2; // usually between [1.2, 2.0]
    const B: f64 = 0.75;

    let term_frequency = {
        let doc_length_normalized =
            p.doc_total_terms_count as f64 / terms_count_per_doc_avg;
        let doc_length_normalization_factor =
            1.0 - B + (B * doc_length_normalized);

        let tf = p.doc_term_count as f64;

        let num = tf * (K + 1.0);
        let den = tf + (K * doc_length_normalization_factor);

        num / den
    };

    let inverted_doc_frequency = {
        let total_docs = p.docs_total_count as f64;
        let matching_docs = p.docs_with_term_count as f64;

        let num = total_docs - matching_docs + 0.5;
        let den = matching_docs + 0.5;

        f64::ln((num / den) + 1.0)
    };

    term_frequency * inverted_doc_frequency
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tfidf() {
        let params = ScoringParams {
            doc_term_count: 2,
            doc_total_terms_count: 100,
            docs_with_term_count: 5,
            docs_total_count: 1000,
        };

        let score = calc_tfidf(params);
        assert_eq!(score, 0.10596634733096073);
    }

    #[test]
    fn test_bm25() {
        let params = ScoringParams {
            doc_term_count: 2,
            doc_total_terms_count: 100,
            docs_with_term_count: 5,
            docs_total_count: 1000,
        };
        let avg_terms = 150.0;

        let score = calc_bm25(params, avg_terms);
        assert_eq!(score, 7.895734283840656);
    }
}
