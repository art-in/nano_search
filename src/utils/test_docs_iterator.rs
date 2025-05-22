use crate::model::doc::Doc;

pub struct TestDocsIterator {
    docs: Vec<Doc>,
    position: usize,
}

impl TestDocsIterator {
    pub fn from_docs(docs: Vec<Doc>) -> Self {
        TestDocsIterator { docs, position: 0 }
    }

    pub fn from_enumerated_texts(texts: &Vec<(u64, &str)>) -> Self {
        TestDocsIterator {
            docs: texts
                .iter()
                .map(|(id, text)| Doc {
                    id: *id,
                    text: text.to_string(),
                })
                .collect(),
            position: 0,
        }
    }

    pub fn from_texts(texts: &Vec<&str>) -> Self {
        Self::from_enumerated_texts(
            &texts
                .iter()
                .enumerate()
                .map(|(id, text)| (id as u64, *text))
                .collect(),
        )
    }
}

impl Iterator for TestDocsIterator {
    type Item = Doc;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.docs.len() {
            let doc = self.docs[self.position].clone();
            self.position += 1;
            Some(doc)
        } else {
            None
        }
    }
}
