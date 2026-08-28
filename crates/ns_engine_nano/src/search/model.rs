use ns_engine::Score;

/// Reference to document that matches search query, but still needs to go
/// through some triage before including into search results.
#[derive(Debug, Clone)]
pub struct DocCandidate<TID> {
    pub id: TID,
    pub score: Score,
}

impl<TID> DocCandidate<TID> {
    #[cfg(test)]
    pub const fn new(id: TID, score: Score) -> Self {
        Self { id, score }
    }
}

impl<TID: Ord> Ord for DocCandidate<TID> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // 1. in descending order of score
        let res = other.score.total_cmp(&self.score);
        if res != std::cmp::Ordering::Equal {
            return res;
        }

        // 2. in ascending order of document ID
        // (to stabilize search results in case candidates have equal score)
        self.id.cmp(&other.id)
    }
}

impl<TID: Ord> PartialOrd for DocCandidate<TID> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<TID: Ord> PartialEq for DocCandidate<TID> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl<TID: Ord> Eq for DocCandidate<TID> {}
