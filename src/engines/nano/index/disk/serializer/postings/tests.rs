use anyhow::{Context, Result};

use super::serializer::{PostingsDeserializer, PostingsSerializer};
use crate::engines::nano::index::model::DocPosting;
use crate::utils::CountingWriter;

#[test]
fn test_postings_serializer() -> Result<()> {
    for count in 0..1000 {
        assert_postings_serializer(count)?;
    }

    Ok(())
}

fn assert_postings_serializer(postings_count: u32) -> Result<()> {
    let storage = Vec::<u8>::new();
    let mut storage_writer = CountingWriter::new(storage);

    // serialize
    {
        let mut serializer = PostingsSerializer::new(&mut storage_writer);

        for idx in 0..postings_count {
            serializer.write_posting(&DocPosting {
                docid: idx * 2,
                term_freq: idx * 3,
            })?;
        }

        serializer.flush()?;
    }

    // deserialize
    {
        let storage = storage_writer.into_inner();
        let deserializer = PostingsDeserializer::new(&storage[..]);

        let mut actual_postings_count = 0;

        for (idx, posting) in deserializer.enumerate() {
            let posting = posting.context("posting should be valid")?;

            assert_eq!(posting.docid as usize, idx * 2);
            assert_eq!(posting.term_freq as usize, idx * 3);

            actual_postings_count += 1;
        }

        assert_eq!(postings_count, actual_postings_count);
    }

    Ok(())
}
