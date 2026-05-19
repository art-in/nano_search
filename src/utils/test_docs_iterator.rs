use std::vec::IntoIter;

use anyhow::Result;

use crate::model::doc::Doc;

pub struct TestDocsIterator {
    docs: IntoIter<Result<Doc>>,
}

impl TestDocsIterator {
    #[must_use]
    pub fn from_enumerated_texts(texts: &Vec<(u64, &str)>) -> Self {
        Self {
            docs: texts
                .iter()
                .map(|(id, text)| {
                    Ok(Doc {
                        id: *id,
                        text: text.to_string(),
                    })
                })
                .collect::<Vec<Result<Doc>>>()
                .into_iter(),
        }
    }
}

impl Iterator for TestDocsIterator {
    type Item = Result<Doc>;

    fn next(&mut self) -> Option<Self::Item> {
        self.docs.next()
    }
}
