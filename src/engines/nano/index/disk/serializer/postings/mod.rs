mod block;
mod serializer;

pub use serializer::{PostingsDeserializer, PostingsSerializer};

#[cfg(test)]
mod tests;
