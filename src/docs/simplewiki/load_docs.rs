use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::Result;

use super::model::WikiDocs;

pub fn load_docs() -> Result<WikiDocs> {
    load_docs_from("data/simplewiki/simplewiki.xml".into())
}

fn load_docs_from(file_path: PathBuf) -> Result<WikiDocs> {
    let parser = wikidump::Parser::new()
        .use_config(wikidump::config::wikipedia::simple_english());

    let site = parser
        .parse_file(file_path)
        .expect("wikipedia dump should be parsed");

    let site = Rc::new(RefCell::new(site));

    Ok(WikiDocs { site })
}
