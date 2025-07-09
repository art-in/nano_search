use std::collections::HashSet;

use anyhow::{Result, bail};

/// Compares two arrays by evaluating the similarity of their prefix windows.
///
/// How it works: for each window size from 1 up to the length of the longer
/// array, it compares the sets of elements in the prefixes of window size and
/// computes the fraction of overlap. The final score is the average of these
/// fractions.
///
/// This approach naturally prioritizes elements at the beginning of the arrays,
/// making it suitable for comparing ranked arrays.
pub fn compare_ranked_arrays(a: &[u64], b: &[u64]) -> Result<f64> {
    match (a.is_empty(), b.is_empty()) {
        (true, true) => return Ok(1.0),
        (true, false) | (false, true) => return Ok(0.0),
        _ => {}
    }

    if contains_duplicates(a) || contains_duplicates(b) {
        bail!("input arrays should not contain duplicate items");
    }

    let max_len = a.len().max(b.len());
    let mut total_score = 0.0;

    for window_size in 1..=max_len {
        let a_window: HashSet<_> = a.iter().take(window_size).collect();
        let b_window: HashSet<_> = b.iter().take(window_size).collect();
        let matched = a_window.intersection(&b_window).count();
        let score = matched as f64 / window_size as f64;
        total_score += score;
    }

    Ok(total_score / max_len as f64)
}

fn contains_duplicates(array: &[u64]) -> bool {
    let mut seen = HashSet::with_capacity(array.len());
    array.iter().any(|&item| !seen.insert(item))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trivial() -> Result<()> {
        assert_eq!(compare_ranked_arrays(&[], &[])?, 1.0);
        assert_eq!(compare_ranked_arrays(&[1], &[])?, 0.0);
        assert_eq!(compare_ranked_arrays(&[], &[1])?, 0.0);
        Ok(())
    }

    #[test]
    fn basic() -> Result<()> {
        assert_eq!(compare_ranked_arrays(&[1], &[1])?, 1.0);
        assert_eq!(compare_ranked_arrays(&[1], &[2])?, 0.0);
        assert_eq!(compare_ranked_arrays(&[1, 2], &[1, 2])?, 1.0);

        // first window:  [1]    -> [1]    = score 1
        // second window: [1, 2] -> [1, 3] = score 0.5
        // avg score: 1.5 total score / 2 windows = 0.75
        assert_eq!(compare_ranked_arrays(&[1, 2], &[1, 3])?, 0.75);

        // first window:  [2]    -> [3]    = score 0
        // second window: [2, 1] -> [3, 1] = score 0.5
        // avg score: 0.5 total score / 2 windows = 0.25
        assert_eq!(compare_ranked_arrays(&[2, 1], &[3, 1])?, 0.25);

        Ok(())
    }

    #[test]
    fn complex() -> Result<()> {
        assert_eq!(compare_ranked_arrays(&[1, 2, 3, 4], &[1, 2, 3, 4])?, 1.0);
        assert_eq!(compare_ranked_arrays(&[1, 2, 3, 4], &[2, 1, 3, 4])?, 0.75);
        assert_eq!(compare_ranked_arrays(&[1, 2, 3, 4], &[1, 3, 2, 4])?, 0.875);
        assert_eq!(
            compare_ranked_arrays(&[1, 2, 3, 4], &[1, 2, 4, 3])?,
            0.9166666666666666
        );
        Ok(())
    }

    #[test]
    fn different_size() -> Result<()> {
        assert_eq!(compare_ranked_arrays(&[1, 2], &[1])?, 0.75);
        assert_eq!(compare_ranked_arrays(&[1], &[1, 2])?, 0.75);
        Ok(())
    }

    #[test]
    fn duplicates() {
        assert!(compare_ranked_arrays(&[1, 1], &[1, 2]).is_err());
        assert!(compare_ranked_arrays(&[1, 2], &[2, 2]).is_err());
    }
}
