use std::collections::HashSet;

// compares two arrays with distinct numbers by comparing set of windows on both arrays.
// where window is a slice of elements starting from the begging of array.
// each window equality is calculated by counting same elements in both arrays.
// this approach naturally prioritize elements at the begging.
pub fn compare_arrays(first_array: &[u64], second_array: &[u64]) -> f64 {
    if first_array.is_empty() ^ second_array.is_empty() {
        return 0.0;
    }

    if first_array.is_empty() && second_array.is_empty() {
        return 1.0;
    }

    let mut windows_scores_sum = 0.0;
    let max_array_len = first_array.len().max(second_array.len());

    for window_size in 1..=max_array_len {
        let mut first_window_set = HashSet::new();
        for item in first_array.iter().take(window_size) {
            assert!(
                !first_window_set.contains(item),
                "distinct numbers expected"
            );
            first_window_set.insert(item);
        }

        let mut second_window_set = HashSet::new();
        for item in second_array.iter().take(window_size) {
            assert!(
                !second_window_set.contains(item),
                "distinct numbers expected"
            );
            second_window_set.insert(item);
        }

        let matched_items_count = first_window_set.len()
            - first_window_set.difference(&second_window_set).count();

        let window_score = matched_items_count as f64 / window_size as f64;
        windows_scores_sum += window_score;
    }

    let windows_count = max_array_len;
    windows_scores_sum / windows_count as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_arrays() {
        // trivial
        assert_eq!(compare_arrays(&[], &[]), 1.0);
        assert_eq!(compare_arrays(&[1], &[]), 0.0);
        assert_eq!(compare_arrays(&[], &[1]), 0.0);

        // basic
        assert_eq!(compare_arrays(&[1], &[1]), 1.0);
        assert_eq!(compare_arrays(&[1], &[2]), 0.0);
        assert_eq!(compare_arrays(&[1, 2], &[1, 2]), 1.0);

        // first window:  [1]    -> [1]    = score 1
        // second window: [1, 2] -> [1, 3] = score 0.5
        // avg score: 1.5 score sum / 2 windows = 0.75
        assert_eq!(compare_arrays(&[1, 2], &[1, 3]), 0.75);

        // first window:  [2]    -> [3]    = score 0
        // second window: [2, 1] -> [3, 1] = score 0.5
        // avg score: 0.5 score sum / 2 windows = 0.25
        assert_eq!(compare_arrays(&[2, 1], &[3, 1]), 0.25);

        // different sizes
        assert_eq!(compare_arrays(&[1, 2], &[1]), 0.75);
        assert_eq!(compare_arrays(&[1], &[1, 2]), 0.75);

        // complex
        assert_eq!(compare_arrays(&[1, 2, 3, 4], &[1, 2, 3, 4]), 1.0);
        assert_eq!(compare_arrays(&[1, 2, 3, 4], &[2, 1, 3, 4]), 0.75);
        assert_eq!(compare_arrays(&[1, 2, 3, 4], &[1, 3, 2, 4]), 0.875);
        assert_eq!(
            compare_arrays(&[1, 2, 3, 4], &[1, 2, 4, 3]),
            0.9166666666666666
        );
    }

    #[test]
    #[should_panic]
    fn test_compare_arrays_panics_on_not_distinct_numbers() {
        compare_arrays(&[1, 1], &[1, 2]);
        compare_arrays(&[1, 2], &[2, 2]);
    }
}
