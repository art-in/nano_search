use crate::model::doc::Doc;
use std::{cell::RefCell, collections::HashSet, rc::Rc};

pub struct CisiDocs {
    pub docs: Rc<RefCell<Vec<Doc>>>,
}

impl Clone for CisiDocs {
    fn clone(&self) -> Self {
        Self {
            docs: Rc::clone(&self.docs),
        }
    }
}

#[derive(Default, Clone)]
pub struct Query {
    pub id: u64,
    pub text: String,
    pub expected_docids: HashSet<u64>,
}
