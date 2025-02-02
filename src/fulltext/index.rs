use std::collections::{BTreeMap, HashMap};
use std::io::Read;
use std::path::Path;

pub type Term = String;

pub type DocId = u64;

pub struct DocPosting {
    pub docid: DocId,

    // number of times this term appears in the doc
    pub term_count: u64,

    // total number of terms in this doc
    pub total_terms_count: u64,
}

pub type TermPostingList = BTreeMap<DocId, DocPosting>;

#[derive(Default)]
pub struct Index {
    pub total_docs_count: u64,
    pub terms: HashMap<Term, TermPostingList>,
}

pub fn build_index() -> Index {
    let mut index = Index::default();

    let dir_path = Path::new("data/docs");
    println!("indexing documents in folder: {}", dir_path.display());

    crate::utils::visit_dir_files(dir_path, &mut |path| {
        let mut file =
            std::fs::File::open(path.clone()).expect("file should exist");

        let mut doc_content = String::new();
        file.read_to_string(&mut doc_content)
            .expect("file should contain valid unicode");

        let docid = path
            .file_name()
            .expect("filename should be extracted from path")
            .to_str()
            .expect("filename should be converted to string")
            .parse::<u64>()
            .expect("filename should be parsed to integer");

        index.total_docs_count += 1;

        let words: Vec<&str> = doc_content.split(' ').collect();

        let terms: Vec<String> = words
            .iter()
            .filter_map(|word| {
                let term = crate::utils::normalize_word(word);
                if term.is_empty() {
                    None
                } else {
                    Some(term)
                }
            })
            .collect();

        for term in &terms {
            match index.terms.get_mut(term) {
                Some(posting_list) => match posting_list.get_mut(&docid) {
                    Some(posting) => {
                        posting.term_count += 1;
                    }
                    None => {
                        let posting = DocPosting {
                            docid,
                            term_count: 1,
                            total_terms_count: terms.len() as u64,
                        };
                        posting_list.insert(docid, posting);
                    }
                },
                None => {
                    let mut posting_list = TermPostingList::new();
                    let posting = DocPosting {
                        docid,
                        term_count: 1,
                        total_terms_count: terms.len() as u64,
                    };
                    posting_list.insert(docid, posting);
                    index.terms.insert(term.clone(), posting_list);
                }
            }
        }
    });

    index
}
