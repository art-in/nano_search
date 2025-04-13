use crate::model::doc::{Doc, DocsSource};

use super::docs::CisiDocs;

pub struct CisiDocsIterator {
    docs: Vec<Doc>,
    doc_index: usize,
}

impl IntoIterator for CisiDocs {
    type Item = Doc;
    type IntoIter = CisiDocsIterator;

    fn into_iter(self) -> Self::IntoIter {
        CisiDocsIterator {
            docs: self.docs,
            doc_index: 0,
        }
    }
}

impl Iterator for CisiDocsIterator {
    type Item = Doc;

    fn next(&mut self) -> Option<Self::Item> {
        if self.doc_index >= self.docs.len() {
            None
        } else {
            let doc = Some(self.docs[self.doc_index].clone());
            self.doc_index += 1;
            doc
        }
    }
}

impl DocsSource for CisiDocs {}
