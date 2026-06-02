use std::vec::IntoIter;

use anyhow::Result;
use itertools::Itertools;

use crate::model::doc::Doc;
use crate::utils::test_docs::TestDoc;

pub struct TestDocsIterator {
    docs: IntoIter<Result<Doc>>,
}

impl TestDocsIterator {
    #[must_use]
    pub fn from_enumerated_texts(texts: &Vec<&TestDoc>) -> Self {
        Self {
            docs: texts
                .iter()
                .sorted_by_key(|doc| doc.index)
                .map(|doc| {
                    Ok(Doc {
                        id: doc.id,
                        text: doc.text.to_owned(),
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
