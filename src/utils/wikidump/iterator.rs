use anyhow::{Result, bail};
use quick_xml::events::Event;

use super::dump::XmlReader;
use super::model::{WikiPage, WikiPageRevision};
use super::parser::WikiTextParser;

pub struct WikiPagesIterator {
    pub xml_reader: XmlReader,
    pub text_parser: WikiTextParser,
}

impl Iterator for WikiPagesIterator {
    type Item = WikiPage;

    fn next(&mut self) -> Option<Self::Item> {
        get_next_page(&mut self.xml_reader, &self.text_parser)
    }
}

fn get_next_page(
    xml_reader: &mut XmlReader,
    text_parser: &WikiTextParser,
) -> Option<WikiPage> {
    let mut event_buf = Vec::new();
    let mut text_buf = Vec::new();

    let mut current_page = WikiPage::default();
    let mut current_revision = WikiPageRevision::default();
    let mut skip_current_page = false;

    loop {
        match xml_reader.read_event_into(&mut event_buf) {
            Ok(Event::Start(ref e)) => {
                if skip_current_page {
                    continue;
                }

                match e.name().as_ref() {
                    b"title" => {
                        current_page.title =
                            get_text_from_event(xml_reader, &mut text_buf)
                                .expect("should get text from 'title' node")
                    }
                    b"text" => {
                        current_revision.text =
                            get_text_from_event(xml_reader, &mut text_buf)
                                .expect("should get text from 'text' node")
                    }
                    b"timestamp" => {
                        current_revision.timestamp =
                            get_text_from_event(xml_reader, &mut text_buf)
                                .expect("should get text from 'timestamp' node")
                    }
                    b"ns" => {
                        let ns = get_text_from_event(xml_reader, &mut text_buf)
                            .expect("should get text from 'ns' node");

                        // namespace "0" corresponds to articles.
                        // see. https://en.wikipedia.org/wiki/Wikipedia:Namespace
                        if ns != "0" {
                            skip_current_page = true;
                            continue;
                        }
                    }
                    _ => {}
                };
            }
            Ok(Event::End(ref e)) => {
                match e.name().as_ref() {
                    b"page" => {
                        if skip_current_page {
                            current_page.reset();
                            skip_current_page = false;
                        } else if !current_page.revisions.is_empty() {
                            current_page
                                .revisions
                                .sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
                            break Some(current_page);
                        }
                    }
                    b"revision" => {
                        let mut revision = current_revision.clone();
                        current_revision.reset();

                        revision.text = text_parser.parse(&revision.text);

                        // make sure to skip revisions with empty text,
                        // which may happen if raw text contains layout tags
                        // only (e.g. "redirect"), and was completely wiped
                        // out when parsing
                        if !revision.text.is_empty() {
                            current_page.revisions.push(revision);
                        }
                    }
                    _ => {}
                };
            }
            Ok(Event::Eof) => {
                break None;
            }
            Err(e) => panic!(
                "Error at position {}: {:?}",
                xml_reader.buffer_position(),
                e
            ),
            _ => (),
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to
        // keep memory usage low
        event_buf.clear();
        text_buf.clear();
    }
}

fn get_text_from_event(
    reader: &mut XmlReader,
    text_buf: &mut Vec<u8>,
) -> Result<String> {
    let event = reader.read_event_into(text_buf)?;
    match event {
        Event::Text(e) => Ok(e.unescape()?.into_owned()),
        Event::End(_) => Ok(" ".to_string()),
        _ => bail!(format!(
            "unexpected event type (expected text event, got {event:?})"
        )),
    }
}
