use std::collections::HashMap;

use anyhow::{Result, bail};
use itertools::Itertools;
use ns_engine::{ExternalDocId, Score};

/// Calculates Normalized Discounted Cumulative Gain (NDCG), which is a measure
/// of search result quality.
///
/// Unlike search precision, NDCG takes position of document in search results
/// into account, i.e. placing document is penalized more its position is closer
/// to the end.
///
/// See <https://en.wikipedia.org/wiki/Discounted_cumulative_gain>
pub fn ndcg(
    found_docids: &[ExternalDocId],
    relevant_docs: &HashMap<ExternalDocId, Score>,
    search_limit: u64,
) -> Result<f64> {
    if relevant_docs.is_empty() {
        return Ok(if found_docids.is_empty() { 1.0 } else { 0.0 });
    }

    let retrieved_scores =
        extract_scores(found_docids, relevant_docs, search_limit);
    let ideal_scores = compute_ideal_scores(relevant_docs, search_limit);

    compute_ndcg(&retrieved_scores, &ideal_scores, WeightingScheme::Burges)
}

fn extract_scores(
    found_docids: &[ExternalDocId],
    relevant_docs: &HashMap<ExternalDocId, Score>,
    search_limit: u64,
) -> Vec<Score> {
    found_docids
        .iter()
        .take(search_limit as usize)
        .map(|docid| relevant_docs.get(docid).copied().unwrap_or(0.0))
        .collect()
}

fn compute_ideal_scores(
    relevant_docs: &HashMap<ExternalDocId, Score>,
    search_limit: u64,
) -> Vec<Score> {
    relevant_docs
        .values()
        .copied()
        .sorted_by(f64::total_cmp)
        .rev()
        .take(search_limit as usize)
        .collect()
}

fn compute_ndcg(
    retrieved_scores: &[Score],
    ideal_scores: &[Score],
    weighting_scheme: WeightingScheme,
) -> Result<f64> {
    let retrieved_dcg = compute_dcg(retrieved_scores, weighting_scheme);
    let ideal_dcg = compute_dcg(ideal_scores, weighting_scheme);

    if ideal_dcg == 0.0 {
        bail!(
            "Ideal DCG should not equal zero. This happens if there are no \
             relevant documents for a query or search limit is zero."
        );
    }

    Ok(retrieved_dcg / ideal_dcg)
}

fn compute_dcg(scores: &[Score], weighting_scheme: WeightingScheme) -> f64 {
    scores
        .iter()
        .enumerate()
        .map(|(position, &score)| {
            let weighted_score = weighting_scheme.weight(score);
            let position_rank = (position + 2) as f64;
            weighted_score / f64::log2(position_rank)
        })
        .sum()
}

#[derive(Clone, Copy, Debug)]
pub enum WeightingScheme {
    /// Original version of NDCG proposed by Järvelin and Kekäläinen.
    /// Uses scores directly without transformation.
    #[allow(unused)]
    Jarvelin,

    /// Later evolution of NDCG metric, which places stronger emphasis on
    /// retrieving relevant documents. It is commonly used in industrial
    /// applications including major web search companies and data science
    /// competition platforms such as Kaggle.
    Burges,
}

impl WeightingScheme {
    fn weight(self, score: Score) -> f64 {
        match self {
            Self::Jarvelin => score,
            Self::Burges => score.exp2() - 1.0,
        }
    }
}

#[cfg(test)]
mod test {
    use assert_float_eq::assert_float_relative_eq;

    use super::*;

    #[test]
    fn test_ndcg() -> Result<()> {
        let found_docids = vec![1, 2, 3];
        let relevant_docs =
            HashMap::from([(2, 0.5), (3, 0.8), (4, 0.9), (5, 1.0)]);
        let search_limit = 3;

        assert_float_relative_eq!(
            ndcg(&found_docids, &relevant_docs, search_limit)?,
            0.33,
            0.01
        );

        Ok(())
    }

    #[test]
    fn test_ndcg_zero_found_docs_returns_zero() -> Result<()> {
        let found_docids: Vec<ExternalDocId> = vec![];
        let relevant_docs = HashMap::from([(1, 1.0)]);
        let search_limit = 10;

        let val = ndcg(&found_docids, &relevant_docs, search_limit)?;
        assert_eq!(val, 0.0);

        Ok(())
    }

    #[test]
    fn test_ndcg_no_found_and_no_relevant_docs_returns_one() -> Result<()> {
        let found_docids: Vec<ExternalDocId> = vec![];
        let relevant_docs = HashMap::new();
        let search_limit = 10;

        let val = ndcg(&found_docids, &relevant_docs, search_limit)?;
        assert_eq!(val, 1.0);

        Ok(())
    }

    #[test]
    fn test_ndcg_some_found_and_no_relevant_docs_returns_zero() -> Result<()> {
        let found_docids: Vec<ExternalDocId> = vec![1, 2, 3];
        let relevant_docs = HashMap::new();
        let search_limit = 10;

        let val = ndcg(&found_docids, &relevant_docs, search_limit)?;
        assert_eq!(val, 0.0);

        Ok(())
    }

    #[test]
    fn test_ndcg_zero_relevant_docs_is_error() {
        let found_docids = vec![1, 2, 3];
        let relevant_docs: HashMap<ExternalDocId, Score> =
            HashMap::from([(1, 0.0), (2, 0.0), (3, 0.0)]);
        let search_limit = 10;

        let res = ndcg(&found_docids, &relevant_docs, search_limit);
        assert!(res.is_err(), "expected error when all docs have zero score");
    }

    #[test]
    fn test_ndcg_zero_search_limit_is_error() {
        let found_docids = vec![1, 2];
        let relevant_docs = HashMap::from([(1, 1.0)]);
        let search_limit = 0;

        let res = ndcg(&found_docids, &relevant_docs, search_limit);
        assert!(res.is_err(), "expected error when search_limit is zero");
    }

    #[test]
    fn test_compute_dcg() {
        assert_float_relative_eq!(
            compute_dcg(
                &[3.0, 2.0, 3.0, 0.0, 1.0, 2.0],
                WeightingScheme::Jarvelin
            ),
            6.861,
            0.01
        );
        assert_float_relative_eq!(
            compute_dcg(
                &[3.0, 3.0, 3.0, 2.0, 2.0, 2.0],
                WeightingScheme::Jarvelin
            ),
            8.740,
            0.01
        );
    }

    #[test]
    fn test_compute_ndcg() -> Result<()> {
        assert_float_relative_eq!(
            compute_ndcg(
                &[3.0, 2.0, 3.0, 0.0, 1.0, 2.0],
                &[3.0, 3.0, 3.0, 2.0, 2.0, 2.0],
                WeightingScheme::Jarvelin
            )?,
            0.785,
            0.01
        );

        Ok(())
    }
}
