use parse_wiki_text::{Configuration, ConfigurationSource, Node};

pub struct WikiTextParser {
    config: parse_wiki_text::Configuration,
}

impl WikiTextParser {
    pub fn new(config_source: &ConfigurationSource) -> Self {
        Self {
            config: Configuration::new(config_source),
        }
    }

    pub fn parse(&self, text: &str) -> String {
        let parsed_output = self.config.parse(text);
        let mut text = get_text_from_nodes(&parsed_output.nodes);

        text = text.replace('\t', " ");
        text = text.replace('\n', " ");
        text = text.replace('\r', " ");

        text.trim().to_string()
    }
}

fn get_text_from_nodes(nodes: &[Node]) -> String {
    let mut text = String::with_capacity(64 + 64 * nodes.len());

    for node in nodes {
        match node {
            Node::Text { value, .. } => text.push_str(value),
            Node::ParagraphBreak { .. } => text.push('\n'),
            Node::CharacterEntity { character, .. } => {
                text.push_str(character.to_string().as_str());
            }
            Node::Link { text: nodes, .. }
            | Node::ExternalLink { nodes, .. }
            | Node::Preformatted { nodes, .. } => {
                text.push_str(get_text_from_nodes(nodes).as_str());
            }
            Node::Heading { nodes, .. } => {
                text.push('\n');
                text.push_str(get_text_from_nodes(nodes).as_str());
                text.push('\n');
            }
            Node::OrderedList { items, .. }
            | Node::UnorderedList { items, .. } => {
                for item in items {
                    text.push_str(get_text_from_nodes(&item.nodes).as_str());
                    text.push(' ');
                }
            }
            Node::DefinitionList { items, .. } => {
                for item in items {
                    text.push_str(get_text_from_nodes(&item.nodes).as_str());
                }
            }
            #[expect(clippy::match_same_arms)]
            Node::Image { .. } => {
                // Currently not allowed because it's a bit difficult to figure
                // out what is normal text and what isn't.
            }

            Node::Template { .. }
            | Node::Bold { .. }
            | Node::BoldItalic { .. }
            | Node::HorizontalDivider { .. }
            | Node::MagicWord { .. }
            | Node::Italic { .. }
            | Node::Redirect { .. }
            | Node::Comment { .. }
            | Node::Tag { .. }
            | Node::StartTag { .. }
            | Node::EndTag { .. }
            | Node::Parameter { .. }
            | Node::Category { .. }
            | Node::Table { .. } => {}
        }
    }

    text
}
