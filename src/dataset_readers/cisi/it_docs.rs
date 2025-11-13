use std::cell::RefCell;
use std::rc::Rc;

use anyhow::Result;

use super::model::CisiDocs;
use crate::model::doc::{Doc, DocsSource};

pub struct CisiDocsIterator {
    docs: Rc<RefCell<Vec<Doc>>>,
    doc_index: usize,
}

impl DocsSource for CisiDocs {
    type Iter = CisiDocsIterator;

    fn docs(&self) -> Result<Self::Iter> {
        Ok(CisiDocsIterator {
            docs: Rc::clone(&self.docs),
            doc_index: 0,
        })
    }

    fn docs_count(&self) -> Result<Option<usize>> {
        Ok(Some(self.docs.borrow().len()))
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
