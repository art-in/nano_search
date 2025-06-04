use std::collections::HashMap;
use std::io::{Read, Write};

use anyhow::Result;

use super::model::TermPostingListFileAddress;
use crate::engines::nano::index::model::DocPosting;
use crate::model::engine::IndexStats;

pub trait BinarySerializable: Sized {
    fn serialize(&self, write: &mut dyn Write) -> Result<()>;
    fn deserialize(read: &mut dyn Read) -> Result<Self>;
}

impl BinarySerializable for u32 {
    fn serialize(&self, write: &mut dyn Write) -> Result<()> {
        write.write_all(&self.to_le_bytes())?;
        Ok(())
    }
    fn deserialize(read: &mut dyn Read) -> Result<Self> {
        let mut buf: [u8; 4] = [0; 4];
        read.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }
}

impl BinarySerializable for usize {
    fn serialize(&self, write: &mut dyn Write) -> Result<()> {
        write.write_all(&self.to_le_bytes())?;
        Ok(())
    }
    fn deserialize(read: &mut dyn Read) -> Result<Self> {
        let mut buf: [u8; 8] = [0; 8];
        read.read_exact(&mut buf)?;
        Ok(usize::from_le_bytes(buf))
    }
}

impl BinarySerializable for u64 {
    fn serialize(&self, write: &mut dyn Write) -> Result<()> {
        write.write_all(&self.to_le_bytes())?;
        Ok(())
    }
    fn deserialize(read: &mut dyn Read) -> Result<Self> {
        let mut buf: [u8; 8] = [0; 8];
        read.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }
}

impl BinarySerializable for f64 {
    fn serialize(&self, write: &mut dyn Write) -> Result<()> {
        write.write_all(&self.to_le_bytes())?;
        Ok(())
    }
    fn deserialize(read: &mut dyn Read) -> Result<Self> {
        let mut buf: [u8; 8] = [0; 8];
        read.read_exact(&mut buf)?;
        Ok(f64::from_le_bytes(buf))
    }
}

impl BinarySerializable for String {
    fn serialize(&self, write: &mut dyn Write) -> Result<()> {
        self.len().serialize(write)?;
        write.write_all(self.as_bytes())?;
        Ok(())
    }
    fn deserialize(read: &mut dyn Read) -> Result<Self> {
        let string_length = usize::deserialize(read)?;
        let mut string = String::with_capacity(string_length);
        read.take(string_length as u64)
            .read_to_string(&mut string)?;
        Ok(string)
    }
}

impl BinarySerializable for IndexStats {
    fn serialize(&self, write: &mut dyn Write) -> Result<()> {
        self.indexed_docs_count.serialize(write)?;
        self.posting_lists_count.serialize(write)?;
        self.max_posting_list_size.serialize(write)?;
        self.terms_count_per_doc_avg.serialize(write)?;
        Ok(())
    }
    fn deserialize(read: &mut dyn Read) -> Result<Self> {
        Ok(IndexStats {
            indexed_docs_count: u64::deserialize(read)?,
            posting_lists_count: u64::deserialize(read)?,
            max_posting_list_size: u64::deserialize(read)?,
            terms_count_per_doc_avg: f64::deserialize(read)?,
        })
    }
}

impl BinarySerializable for DocPosting {
    fn serialize(&self, write: &mut dyn Write) -> Result<()> {
        self.docid.serialize(write)?;
        self.term_count.serialize(write)?;
        self.total_terms_count.serialize(write)?;
        Ok(())
    }
    fn deserialize(read: &mut dyn Read) -> Result<Self> {
        Ok(DocPosting {
            docid: u64::deserialize(read)?,
            term_count: u64::deserialize(read)?,
            total_terms_count: u64::deserialize(read)?,
        })
    }
}

impl BinarySerializable for TermPostingListFileAddress {
    fn serialize(&self, write: &mut dyn Write) -> Result<()> {
        self.postings_count.serialize(write)?;
        self.start_byte.serialize(write)?;
        self.end_byte.serialize(write)?;
        Ok(())
    }
    fn deserialize(read: &mut dyn Read) -> Result<Self> {
        Ok(TermPostingListFileAddress {
            postings_count: usize::deserialize(read)?,
            start_byte: u64::deserialize(read)?,
            end_byte: u64::deserialize(read)?,
        })
    }
}

impl BinarySerializable for HashMap<String, TermPostingListFileAddress> {
    fn serialize(&self, write: &mut dyn Write) -> Result<()> {
        self.len().serialize(write)?;
        for (term, address) in self {
            term.serialize(write)?;
            address.serialize(write)?;
        }
        Ok(())
    }
    fn deserialize(read: &mut dyn Read) -> Result<Self> {
        let terms_len = usize::deserialize(read)?;
        let mut terms = HashMap::with_capacity(terms_len);
        for _ in 0..terms_len {
            let term = String::deserialize(read)?;
            let address = TermPostingListFileAddress::deserialize(read)?;
            terms.insert(term, address);
        }
        Ok(terms)
    }
}
