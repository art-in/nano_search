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
        // TODO: not implemented yet. it's bit more complex with wiki dump,
        // since it is big compressed XML file, so we need to decompress it
        // fully in order to get number of doc elements. skip it for now
        Ok(None)
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
