use anyhow::Result;

use super::model::WikiDatasetReader;
use crate::model::doc::{Doc, DocsSource};
use crate::utils::wikidump::WikiPagesIterator;

pub struct WikiDocsIterator {
    it: WikiPagesIterator,
    docid: u64,
}

impl DocsSource for WikiDatasetReader {
    fn docs(&self) -> Result<Box<dyn Iterator<Item = Doc>>> {
        Ok(Box::new(WikiDocsIterator {
            it: self.wikidump.clone().into_iter(),
            docid: 0,
        }))
    }

    fn docs_count(&self) -> Result<Option<usize>> {
        // do not provide docs count for wiki dump, since it requires to
        // iteratively decompress entire dump, which can take a lot of time
        Ok(None)
    }
}

impl Iterator for WikiDocsIterator {
    type Item = Doc;

    fn next(&mut self) -> Option<Self::Item> {
        let doc = self.it.next().map(|mut page| Doc {
            id: self.docid,
            text: page
                .revisions
                // last revision here means latest revision by timestamp
                .pop()
                .expect("should get latest revision")
                .text,
        });

        self.docid += 1;

        doc
    }
}
