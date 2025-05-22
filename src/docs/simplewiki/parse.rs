use super::docs::WikiDocs;
use anyhow::Result;
use std::path::PathBuf;

pub fn parse(file_path: PathBuf) -> Result<WikiDocs> {
    let parser = wikidump::Parser::new()
        .use_config(wikidump::config::wikipedia::simple_english());

    let site = parser
        .parse_file(file_path)
        .expect("wikipedia dump should be parsed");

    Ok(WikiDocs { site })
}
