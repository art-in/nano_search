use std::cell::RefCell;
use std::rc::Rc;

use super::model::CisiDocs;
use crate::model::doc::{Doc, DocsSource};

pub struct CisiDocsIterator {
    docs: Rc<RefCell<Vec<Doc>>>,
    doc_index: usize,
}

impl IntoIterator for CisiDocs {
    type Item = Doc;
    type IntoIter = CisiDocsIterator;

    fn into_iter(self) -> Self::IntoIter {
        CisiDocsIterator {
            docs: Rc::clone(&self.docs),
            doc_index: 0,
        }
    }
}

impl Iterator for CisiDocsIterator {
    type Item = Doc;

    fn next(&mut self) -> Option<Self::Item> {
        if self.doc_index >= self.docs.borrow().len() {
            None
        } else {
            let doc = Some(self.docs.borrow()[self.doc_index].clone());
            self.doc_index += 1;
            doc
        }
    }
}

impl DocsSource for CisiDocs {}
