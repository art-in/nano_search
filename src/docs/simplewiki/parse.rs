use std::path::PathBuf;

use super::docs::WikiDocs;

pub fn parse(file_path: PathBuf) -> WikiDocs {
    let parser = wikidump::Parser::new()
        .use_config(wikidump::config::wikipedia::simple_english());

    let site = parser
        .parse_file(file_path)
        .expect("Could not parse wikipedia dump file.");

    WikiDocs { site }
}
