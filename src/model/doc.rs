use anyhow::Result;

pub type DocId = u64;

#[derive(Clone)]
pub struct Doc {
    pub id: DocId,
    pub text: String,
}

pub trait DocsSource {
    fn docs(&self) -> Result<Box<dyn Iterator<Item = Doc>>>;
    fn docs_count(&self) -> Result<Option<usize>>;
}
