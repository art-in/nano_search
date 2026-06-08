mod binary;
mod compression;
mod postings;

pub use binary::{BinarySerializable, deserialize_vec_item};
pub use postings::{PostingsDeserializer, PostingsSerializer};
