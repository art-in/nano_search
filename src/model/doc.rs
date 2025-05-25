pub type DocId = u64;

#[derive(Clone)]
pub struct Doc {
    pub id: DocId,
    pub text: String,
}

pub trait DocsSource: IntoIterator<Item = Doc> + Clone {}
