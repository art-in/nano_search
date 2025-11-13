pub type DocId = u64;

#[derive(Clone)]
pub struct Doc {
    pub id: DocId,
    pub text: String,
}

pub trait DocsSource {
    type Iter: Iterator<Item = Doc>;
    fn docs(&self) -> anyhow::Result<Self::Iter>;
    fn docs_count(&self) -> anyhow::Result<Option<usize>>;
}
