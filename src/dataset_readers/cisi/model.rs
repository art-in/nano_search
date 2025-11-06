use std::cell::RefCell;
use std::rc::Rc;

use crate::model::doc::Doc;

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
