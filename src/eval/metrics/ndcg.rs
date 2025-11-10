use std::collections::HashMap;

use anyhow::{Result, bail};
use itertools::Itertools;

use crate::eval::model::Relevance;
use crate::model::doc::DocId;

/// Calculates Normalized Discounted Cumulative Gain (NDCG), which is a measure
/// of search result quality.
///
/// Unlike search precision, NDCG takes position of document in search results
/// into account, i.e. placing document is penalized more its position is closer
/// to the end.
///
/// See https://en.wikipedia.org/wiki/Discounted_cumulative_gain
pub fn ndcg(
    found_docids: &[DocId],
    relevant_docs: &HashMap<DocId, Relevance>,
    search_limit: u64,
) -> Result<f64> {
    let retrieved_relevances =
        extract_relevances(found_docids, relevant_docs, search_limit);
    let ideal_relevances =
        compute_ideal_relevances(relevant_docs, search_limit);

    compute_ndcg(
        &retrieved_relevances,
        &ideal_relevances,
        WeightingScheme::Burges,
    )
}

fn extract_relevances(
    found_docids: &[DocId],
    relevant_docs: &HashMap<DocId, Relevance>,
    search_limit: u64,
) -> Vec<Relevance> {
    found_docids
        .iter()
        .take(search_limit as usize)
        .map(|docid| relevant_docs.get(docid).copied().unwrap_or(0.0))
        .collect()
}

fn compute_ideal_relevances(
    relevant_docs: &HashMap<DocId, Relevance>,
    search_limit: u64,
) -> Vec<Relevance> {
    relevant_docs
        .values()
        .copied()
        .sorted_by(f64::total_cmp)
        .rev()
        .take(search_limit as usize)
        .collect()
}

fn compute_ndcg(
    retrieved_relevances: &[Relevance],
    ideal_relevances: &[Relevance],
    weighting_scheme: WeightingScheme,
) -> Result<f64> {
    let retrieved_dcg = compute_dcg(retrieved_relevances, weighting_scheme);
    let ideal_dcg = compute_dcg(ideal_relevances, weighting_scheme);

    if ideal_dcg == 0.0 {
        bail!(
            "Ideal DCG should not equal zero. This happens if there are no \
             relevant documents for a query."
        );
    }

    Ok(retrieved_dcg / ideal_dcg)
}

fn compute_dcg(
    relevances: &[Relevance],
    weighting_scheme: WeightingScheme,
) -> f64 {
    relevances
        .iter()
        .enumerate()
        .map(|(position, &relevance)| {
            let weighted_relevance = weighting_scheme.weight(relevance);
            let position_rank = (position + 2) as f64;
            weighted_relevance / f64::log2(position_rank)
        })
        .sum()
}

#[derive(Clone, Copy, Debug)]
pub enum WeightingScheme {
    /// Original version of NDCG proposed by Järvelin and Kekäläinen.
    /// Uses relevance scores directly without transformation.
    Jarvelin,

    /// Later evolution of NDCG metric, which places stronger emphasis on
    /// retrieving relevant documents. It is commonly used in industrial
    /// applications including major web search companies and data science
    /// competition platforms such as Kaggle.
    Burges,
}

impl WeightingScheme {
    fn weight(self, relevance: Relevance) -> f64 {
        match self {
            WeightingScheme::Jarvelin => relevance,
            WeightingScheme::Burges => 2.0_f64.powf(relevance) - 1.0,
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
