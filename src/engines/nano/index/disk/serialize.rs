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

impl BinarySerializable for u16 {
    fn serialize(&self, write: &mut dyn Write) -> Result<()> {
        write.write_all(&self.to_le_bytes())?;
        Ok(())
    }
    fn deserialize(read: &mut dyn Read) -> Result<Self> {
        let mut buf: [u8; 2] = [0; 2];
        read.read_exact(&mut buf)?;
        Ok(Self::from_le_bytes(buf))
    }
    fn deserialize_from_slice(data: &mut &[u8]) -> Result<Self> {
        let (bytes, rest) = data
            .split_first_chunk::<2>()
            .context("should read u16 from slice")?;
        *data = rest;
        Ok(Self::from_le_bytes(*bytes))
    }
}

impl BinarySerializable for u32 {
    fn serialize(&self, write: &mut dyn Write) -> Result<()> {
        write.write_all(&self.to_le_bytes())?;
        Ok(())
    }
    fn deserialize(read: &mut dyn Read) -> Result<Self> {
        let mut buf: [u8; 4] = [0; 4];
        read.read_exact(&mut buf)?;
        Ok(Self::from_le_bytes(buf))
    }
    fn deserialize_from_slice(data: &mut &[u8]) -> Result<Self> {
        let (bytes, rest) = data
            .split_first_chunk::<4>()
            .context("should read u32 from slice")?;
        *data = rest;
        Ok(Self::from_le_bytes(*bytes))
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
        Ok(Self::from_le_bytes(buf))
    }
    fn deserialize_from_slice(data: &mut &[u8]) -> Result<Self> {
        const SIZE: usize = std::mem::size_of::<usize>();
        let (bytes, rest) = data
            .split_first_chunk::<SIZE>()
            .context("should read usize from slice")?;
        *data = rest;
        Ok(Self::from_le_bytes(*bytes))
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
        Ok(Self::from_le_bytes(buf))
    }
    fn deserialize_from_slice(data: &mut &[u8]) -> Result<Self> {
        let (bytes, rest) = data
            .split_first_chunk::<8>()
            .context("should read u64 from slice")?;
        *data = rest;
        Ok(Self::from_le_bytes(*bytes))
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
        Ok(Self::from_le_bytes(buf))
    }
    fn deserialize_from_slice(data: &mut &[u8]) -> Result<Self> {
        let (bytes, rest) = data
            .split_first_chunk::<8>()
            .context("should read f64 from slice")?;
        *data = rest;
        Ok(Self::from_le_bytes(*bytes))
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
        let mut string = Self::with_capacity(string_length);
        read.take(string_length as u64)
            .read_to_string(&mut string)?;
        Ok(string)
    }
    fn deserialize_from_slice(data: &mut &[u8]) -> Result<Self> {
        let string_length = usize::deserialize_from_slice(data)?;
        let (mut bytes, rest) = data.split_at(string_length);
        *data = rest;
        let mut string = Self::with_capacity(string_length);
        bytes.read_to_string(&mut string)?;
        Ok(string)
    }
}

impl<K, V> BinarySerializable for HashMap<K, V>
where
    K: BinarySerializable + std::cmp::Eq + std::hash::Hash,
    V: BinarySerializable,
{
    fn serialize(&self, write: &mut dyn Write) -> Result<()> {
        self.len().serialize(write)?;
        for (key, value) in self {
            key.serialize(write)?;
            value.serialize(write)?;
        }
        Ok(())
    }
    fn deserialize(read: &mut dyn Read) -> Result<Self> {
        let len = usize::deserialize(read)?;
        let mut map = Self::with_capacity(len);
        for _ in 0..len {
            let key = K::deserialize(read)?;
            let value = V::deserialize(read)?;
            map.insert(key, value);
        }
        Ok(map)
    }
    fn deserialize_from_slice(data: &mut &[u8]) -> Result<Self> {
        let len = usize::deserialize_from_slice(data)?;
        let mut map = Self::with_capacity(len);
        for _ in 0..len {
            let key = K::deserialize_from_slice(data)?;
            let value = V::deserialize_from_slice(data)?;
            map.insert(key, value);
        }
        Ok(map)
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
        Ok(Self {
            indexed_docs_count: u64::deserialize(read)?,
            max_posting_list_size: u64::deserialize(read)?,
            terms_count_per_doc_avg: f64::deserialize(read)?,
        })
    }
    fn deserialize_from_slice(data: &mut &[u8]) -> Result<Self> {
        Ok(Self {
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
        Ok(())
    }
    fn deserialize(read: &mut dyn Read) -> Result<Self> {
        Ok(Self {
            docid: u64::deserialize(read)?,
            term_count: u64::deserialize(read)?,
        })
    }
    fn deserialize_from_slice(data: &mut &[u8]) -> Result<Self> {
        Ok(Self {
            docid: u64::deserialize_from_slice(data)?,
            term_count: u64::deserialize_from_slice(data)?,
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
        Ok(Self {
            postings_count: usize::deserialize(read)?,
            start_byte: usize::deserialize(read)?,
            end_byte: usize::deserialize(read)?,
        })
    }
    fn deserialize_from_slice(data: &mut &[u8]) -> Result<Self> {
        Ok(Self {
            postings_count: usize::deserialize_from_slice(data)?,
            start_byte: usize::deserialize_from_slice(data)?,
            end_byte: usize::deserialize_from_slice(data)?,
        })
    }
}
