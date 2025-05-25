use crate::model::doc::Doc;
use std::{cell::RefCell, rc::Rc};

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
