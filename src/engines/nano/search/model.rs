/// Reference to document that matches search query, but needs to go through
/// some triage before including into search results.
#[derive(Debug, PartialEq, Clone)]
pub struct DocCandidate<TID> {
    pub id: TID,
    // TODO: rename to score and use Score type
    pub relevance: f64,
}

impl<TID> DocCandidate<TID> {
    #[cfg(test)]
    pub const fn new(id: TID, relevance: f64) -> Self {
        Self { id, relevance }
    }
}

impl<TID: Ord> Ord for DocCandidate<TID> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // 1. in descending order of relevance
        let res = other.relevance.total_cmp(&self.relevance);
        if res != std::cmp::Ordering::Equal {
            return res;
        }

        // 2. in ascending order of document ID
        // (to stabilize search results in case candidates have equal relevance)
        self.id.cmp(&other.id)
    }
}

impl<TID: Ord> PartialOrd for DocCandidate<TID> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<TID: Eq> Eq for DocCandidate<TID> {}
