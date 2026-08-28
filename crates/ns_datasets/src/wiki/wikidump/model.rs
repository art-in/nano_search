#[derive(Default)]
pub struct WikiPage {
    pub title: String,
    pub revisions: Vec<WikiPageRevision>,
}

impl WikiPage {
    pub fn reset(&mut self) -> &Self {
        self.title.clear();
        self.revisions.clear();
        self
    }
}

#[derive(Default, Clone)]
pub struct WikiPageRevision {
    pub text: String,
    pub timestamp: String,
}

impl WikiPageRevision {
    pub fn reset(&mut self) -> &mut Self {
        self.text.clear();
        self.timestamp.clear();
        self
    }
}
