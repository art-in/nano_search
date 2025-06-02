use std::{cell::RefCell, rc::Rc};

use super::model::WikiDocs;
use crate::model::doc::{Doc, DocsSource};

// TODO: stream docs from data file instead of loading all of them into memory
pub struct WikiDocsIterator {
    site: Rc<RefCell<wikidump::Site>>,
    doc_index: usize,
}

impl IntoIterator for WikiDocs {
    type Item = Doc;
    type IntoIter = WikiDocsIterator;

    fn into_iter(self) -> Self::IntoIter {
        WikiDocsIterator {
            site: Rc::clone(&self.site),
            doc_index: 0,
        }
    }
}

impl Iterator for WikiDocsIterator {
    type Item = Doc;

    fn next(&mut self) -> Option<Self::Item> {
        if self.doc_index >= self.site.borrow().pages.len() {
            None
        } else {
            let page = &self.site.borrow().pages[self.doc_index];
            assert!(!page.revisions.is_empty());
            let res = Some(Doc {
                id: self.doc_index as u64,
                text: page.revisions[0].text.clone(),
            });
            self.doc_index += 1;
            res
        }
    }
}

impl DocsSource for WikiDocs {}
