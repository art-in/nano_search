use super::model::WikiDocs;
use crate::model::doc::Doc;
use crate::utils::wikidump::WikiPagesIterator;

pub struct WikiDocsIterator {
    it: WikiPagesIterator,
    docid: u64,
}

impl IntoIterator for WikiDocs {
    type Item = Doc;
    type IntoIter = WikiDocsIterator;

    fn into_iter(self) -> Self::IntoIter {
        WikiDocsIterator {
            it: self.wikidump.into_iter(),
            docid: 0,
        }
    }
}

impl Iterator for WikiDocsIterator {
    type Item = Doc;

    fn next(&mut self) -> Option<Self::Item> {
        let doc = self.it.next().map(|page| Doc {
            id: self.docid,
            text: page
                .revisions
                // last revision here means latest revision by timestamp
                .last()
                .expect("should get last revision")
                .text
                .clone(),
        });

        self.docid += 1;

        doc
    }
}
