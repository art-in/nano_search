use std::vec::IntoIter;

use crate::model::doc::Doc;

pub struct TestDocsIterator {
    docs: IntoIter<Doc>,
}

impl TestDocsIterator {
    #[must_use]
    pub fn from_enumerated_texts(texts: &Vec<(u64, &str)>) -> Self {
        Self {
            docs: texts
                .iter()
                .map(|(id, text)| Doc {
                    id: *id,
                    text: text.to_string(),
                })
                .collect::<Vec<Doc>>()
                .into_iter(),
        }
    }
}

impl Iterator for TestDocsIterator {
    type Item = Doc;

    fn next(&mut self) -> Option<Self::Item> {
        self.docs.next()
    }
}
