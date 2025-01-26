use std::fs::File;
use std::io::{BufRead, Write};

enum ESectionType {
    DocId,
    Title,
    Author,
    Text,
    Refs,
}

enum ELineType {
    Unknown,
    SectionHeader(ESectionType),
    SectionContent(ESectionType),
}

// splits CISI.ALL file to separate doc files
pub fn split_docs() {
    let source_file =
        File::open("data/source/CISI.ALL").expect("failed to open file");
    let source_file_reader = std::io::BufReader::new(source_file);

    let mut file_writer: Option<std::io::BufWriter<File>> = None;

    let mut current_line_type = ELineType::Unknown;

    for line in source_file_reader.lines().map_while(Result::ok) {
        if line.starts_with(".I") {
            current_line_type = ELineType::SectionHeader(ESectionType::DocId);
        } else if line.starts_with(".T") {
            current_line_type = ELineType::SectionHeader(ESectionType::Title);
        } else if line.starts_with(".A") {
            current_line_type = ELineType::SectionHeader(ESectionType::Author);
        } else if line.starts_with(".W") {
            current_line_type = ELineType::SectionHeader(ESectionType::Text);
        } else if line.starts_with(".X") {
            current_line_type = ELineType::SectionHeader(ESectionType::Refs);
        } else if let ELineType::SectionHeader(t) = current_line_type {
            current_line_type = ELineType::SectionContent(t);
        }

        match current_line_type {
            ELineType::Unknown => {
                panic!("unknown line type")
            }
            ELineType::SectionHeader(ref section_type) => {
                match section_type {
                    ESectionType::DocId => {
                        let parts: Vec<_> = line.split(' ').collect();
                        let docid = parts[1];

                        if let Some(file_writer) = &mut file_writer {
                            // flush previous file content, before creating new one
                            file_writer
                                .flush()
                                .expect("file buffer should be fully flushed");
                        }

                        let file_path =
                            std::path::PathBuf::from("data/docs/").join(docid);

                        std::fs::create_dir_all(
                            file_path
                                .parent()
                                .expect("file should have valid parent dir"),
                        )
                        .expect("failed to create data directory");

                        let file = File::create(file_path)
                            .expect("failed to create file");

                        file_writer = Some(std::io::BufWriter::new(file));
                    }
                    ESectionType::Author | ESectionType::Text => {
                        file_writer
                            .as_mut()
                            .expect("file writer should be mutable")
                            .write_all(b"\n")
                            .expect("failed to write");
                    }
                    _default => {
                        // skip
                    }
                }
            }
            ELineType::SectionContent(ref section_type) => {
                match section_type {
                    ESectionType::Refs => {
                        // skip
                    }
                    _default => {
                        file_writer
                            .as_mut()
                            .expect("file writer should be mutable")
                            .write_all(line.as_bytes())
                            .expect("failed to write");

                        file_writer
                            .as_mut()
                            .expect("file writer should be mutable")
                            .write_all(b"\n")
                            .expect("failed to write");
                    }
                }
            }
        }
    }
}
