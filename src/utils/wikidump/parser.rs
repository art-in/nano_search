use parse_wiki_text::{Configuration, ConfigurationSource, Node};

pub struct WikiTextParser {
    config: parse_wiki_text::Configuration,
}

impl WikiTextParser {
    pub fn new(config_source: ConfigurationSource) -> Self {
        WikiTextParser {
            config: Configuration::new(&config_source),
        }
    }

    pub fn parse(&self, text: &str) -> String {
        let parsed_output = self.config.parse(text);
        let mut text = get_text_from_nodes(&parsed_output.nodes);

        text = text.replace("\t", " ");
        text = text.replace("\n", " ");
        text = text.replace("\r", " ");

        text.trim().to_string()
    }
}

fn get_text_from_nodes(nodes: &[Node]) -> String {
    let mut text = String::with_capacity(64 + 64 * nodes.len());

    nodes.iter().for_each(|node| {
        match node {
            Node::Text { value, .. } => text.push_str(value),
            Node::ParagraphBreak { .. } => text.push('\n'),
            Node::CharacterEntity { character, .. } => {
                text.push_str(character.to_string().as_str())
            }
            Node::Link { text: nodes, .. } => {
                text.push_str(get_text_from_nodes(nodes).as_str())
            }
            Node::ExternalLink { nodes, .. } => {
                text.push_str(get_text_from_nodes(nodes).as_str())
            }
            Node::Heading { nodes, .. } => {
                text.push('\n');
                text.push_str(get_text_from_nodes(nodes).as_str());
                text.push('\n');
            }
            Node::Image { .. } => {
                // TODO: Allow image text.
                // Currently not allowed because it's a bit difficult to figure
                // out what is normal text and what isn't.
            }
            Node::OrderedList { items, .. }
            | Node::UnorderedList { items, .. } => {
                items.iter().for_each(|i| {
                    text.push_str(get_text_from_nodes(&i.nodes).as_str());
                    text.push(' ');
                });
            }
            Node::DefinitionList { items, .. } => {
                items.iter().for_each(|i| {
                    text.push_str(get_text_from_nodes(&i.nodes).as_str());
                });
            }
            Node::Preformatted { nodes, .. } => {
                text.push_str(get_text_from_nodes(nodes).as_str())
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
    });

    text
}
