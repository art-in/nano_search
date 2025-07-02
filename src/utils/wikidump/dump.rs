use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use anyhow::Result;
use bzip2::read::MultiBzDecoder;
use quick_xml::reader::Reader;

use super::iterator::WikiPagesIterator;
use super::model::WikiPage;
use super::parser::WikiTextParser;
use super::parser_config;

pub type XmlReader = quick_xml::Reader<
    std::io::BufReader<bzip2::read::MultiBzDecoder<std::fs::File>>,
>;

#[derive(Clone)]
pub struct WikiDump {
    file_path: PathBuf,
}

impl WikiDump {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        let file_path = file_path.as_ref().to_path_buf();
        Ok(Self { file_path })
    }

    fn create_xml_reader(&self) -> Result<XmlReader> {
        let file = File::open(&self.file_path)?;
        let decoder = MultiBzDecoder::new(file);
        let reader = BufReader::new(decoder);
        let mut xml_reader = Reader::from_reader(reader);

        let config = xml_reader.config_mut();
        config.trim_markup_names_in_closing_tags = false;

        Ok(xml_reader)
    }
}

impl IntoIterator for WikiDump {
    type Item = WikiPage;
    type IntoIter = WikiPagesIterator;

    fn into_iter(self) -> Self::IntoIter {
        let xml_reader = self
            .create_xml_reader()
            .expect("XML reader should be created");
        let text_parser =
            WikiTextParser::new(parser_config::english_wikipedia());

        WikiPagesIterator {
            xml_reader,
            text_parser,
        }
    }
}
