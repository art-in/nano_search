mod exclude;
pub use exclude::ExcludingDocIdIterator;

mod intersect;
pub use intersect::IntersectingDocIdIterator;

mod model;
pub use model::ScoringDocIdIterator;

mod posting_list;
pub use posting_list::PostingListIterator;

mod union;
pub use union::UnionDocIdIterator;

#[cfg(test)]
pub mod test_utils;
