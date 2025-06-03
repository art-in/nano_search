use std::cell::RefCell;
use std::rc::Rc;

pub struct WikiDocs {
    pub site: Rc<RefCell<wikidump::Site>>,
}

impl Clone for WikiDocs {
    fn clone(&self) -> Self {
        Self {
            site: Rc::clone(&self.site),
        }
    }
}
