use crate::model::doc::{Doc, DocsSource};

use super::docs::WikiDocs;

pub struct WikiDocsIterator {
    site: wikidump::Site,
    doc_index: usize,
}

impl IntoIterator for WikiDocs {
    type Item = Doc;
    type IntoIter = WikiDocsIterator;

    fn into_iter(self) -> Self::IntoIter {
        WikiDocsIterator {
            site: self.site,
            doc_index: 0,
        }
    }
}

impl Iterator for WikiDocsIterator {
    type Item = Doc;

    fn next(&mut self) -> Option<Self::Item> {
        if self.doc_index >= self.site.pages.len() {
            None
        } else {
            let page = &self.site.pages[self.doc_index];
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
