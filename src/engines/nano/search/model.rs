use crate::model::doc::DocId;

#[derive(Debug, PartialEq)]
pub struct DocCandidate {
    pub id: DocId,
    pub relevance: f64,
}

impl Ord for DocCandidate {
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

impl PartialOrd for DocCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for DocCandidate {}
