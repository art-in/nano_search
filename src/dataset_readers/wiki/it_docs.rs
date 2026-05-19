use anyhow::{Context, Result};

use super::model::WikiDatasetReader;
use crate::model::doc::{Doc, DocsSource};
use crate::utils::wikidump::WikiPagesIterator;

pub struct WikiDocsIterator {
    it: WikiPagesIterator,
    docid: u64,
}

impl DocsSource for WikiDatasetReader {
    fn docs(&self) -> Result<Box<dyn Iterator<Item = Result<Doc>>>> {
        Ok(Box::new(WikiDocsIterator {
            it: self.wikidump.pages()?,
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
    type Item = Result<Doc>;

    fn next(&mut self) -> Option<Self::Item> {
        let doc = self.it.next().map(|page| {
            Ok(Doc {
                id: self.docid,
                text: page?
                    .revisions
                    // last revision here means latest revision by timestamp
                    .pop()
                    .context("should get latest revision")?
                    .text,
            })
        });

        self.docid += 1;

        doc
    }
}
