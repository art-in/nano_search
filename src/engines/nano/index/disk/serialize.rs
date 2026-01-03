use std::collections::HashMap;
use std::io::{Read, Write};

use anyhow::{Context, Result};

use super::model::TermPostingListFileAddress;
use crate::engines::nano::index::model::{DocPosting, IndexSegmentStats};

pub trait BinarySerializable: Sized {
    fn serialize(&self, write: &mut dyn Write) -> Result<()>;
    fn deserialize(read: &mut dyn Read) -> Result<Self>;
    fn deserialize_from_slice(data: &mut &[u8]) -> Result<Self>;
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
    fn deserialize_from_slice(data: &mut &[u8]) -> Result<Self> {
        let (bytes, rest) = data
            .split_first_chunk::<4>()
            .context("should read u32 from slice")?;
        *data = rest;
        Ok(u32::from_le_bytes(*bytes))
    }
}

impl BinarySerializable for usize {
    fn serialize(&self, write: &mut dyn Write) -> Result<()> {
        write.write_all(&self.to_le_bytes())?;
        Ok(())
    }
    fn deserialize(read: &mut dyn Read) -> Result<Self> {
        const SIZE: usize = std::mem::size_of::<usize>();
        let mut buf: [u8; SIZE] = [0; SIZE];
        read.read_exact(&mut buf)?;
        Ok(usize::from_le_bytes(buf))
    }
    fn deserialize_from_slice(data: &mut &[u8]) -> Result<Self> {
        const SIZE: usize = std::mem::size_of::<usize>();
        let (bytes, rest) = data
            .split_first_chunk::<SIZE>()
            .context("should read usize from slice")?;
        *data = rest;
        Ok(usize::from_le_bytes(*bytes))
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
    fn deserialize_from_slice(data: &mut &[u8]) -> Result<Self> {
        let (bytes, rest) = data
            .split_first_chunk::<8>()
            .context("should read u64 from slice")?;
        *data = rest;
        Ok(u64::from_le_bytes(*bytes))
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
    fn deserialize_from_slice(data: &mut &[u8]) -> Result<Self> {
        let (bytes, rest) = data
            .split_first_chunk::<8>()
            .context("should read f64 from slice")?;
        *data = rest;
        Ok(f64::from_le_bytes(*bytes))
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
    fn deserialize_from_slice(data: &mut &[u8]) -> Result<Self> {
        let string_length = usize::deserialize_from_slice(data)?;
        let (mut bytes, rest) = data.split_at(string_length);
        *data = rest;
        let mut string = String::with_capacity(string_length);
        bytes.read_to_string(&mut string)?;
        Ok(string)
    }
}

impl BinarySerializable for IndexSegmentStats {
    fn serialize(&self, write: &mut dyn Write) -> Result<()> {
        self.indexed_docs_count.serialize(write)?;
        self.max_posting_list_size.serialize(write)?;
        self.terms_count_per_doc_avg.serialize(write)?;
        Ok(())
    }
    fn deserialize(read: &mut dyn Read) -> Result<Self> {
        Ok(IndexSegmentStats {
            indexed_docs_count: u64::deserialize(read)?,
            max_posting_list_size: u64::deserialize(read)?,
            terms_count_per_doc_avg: f64::deserialize(read)?,
        })
    }
    fn deserialize_from_slice(data: &mut &[u8]) -> Result<Self> {
        Ok(IndexSegmentStats {
            indexed_docs_count: u64::deserialize_from_slice(data)?,
            max_posting_list_size: u64::deserialize_from_slice(data)?,
            terms_count_per_doc_avg: f64::deserialize_from_slice(data)?,
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
    fn deserialize_from_slice(data: &mut &[u8]) -> Result<Self> {
        Ok(DocPosting {
            docid: u64::deserialize_from_slice(data)?,
            term_count: u64::deserialize_from_slice(data)?,
            total_terms_count: u64::deserialize_from_slice(data)?,
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
            start_byte: usize::deserialize(read)?,
            end_byte: usize::deserialize(read)?,
        })
    }
    fn deserialize_from_slice(data: &mut &[u8]) -> Result<Self> {
        Ok(TermPostingListFileAddress {
            postings_count: usize::deserialize_from_slice(data)?,
            start_byte: usize::deserialize_from_slice(data)?,
            end_byte: usize::deserialize_from_slice(data)?,
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
        let len = usize::deserialize(read)?;
        let mut map = HashMap::with_capacity(len);
        for _ in 0..len {
            let term = String::deserialize(read)?;
            let address = TermPostingListFileAddress::deserialize(read)?;
            map.insert(term, address);
        }
        Ok(map)
    }
    fn deserialize_from_slice(data: &mut &[u8]) -> Result<Self> {
        let len = usize::deserialize_from_slice(data)?;
        let mut map = HashMap::with_capacity(len);
        for _ in 0..len {
            let term = String::deserialize_from_slice(data)?;
            let address =
                TermPostingListFileAddress::deserialize_from_slice(data)?;
            map.insert(term, address);
        }
        Ok(map)
    }
}
