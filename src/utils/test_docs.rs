use super::test_docs_iterator::TestDocsIterator;
use crate::model::doc::ExternalDocId;

pub struct TestDoc {
    /// Sequential index this doc will have when emitted from iterator
    pub index: u32,
    pub id: ExternalDocId,
    pub text: &'static str,
}

pub mod docs {
    use super::TestDoc;

    pub static CAT: TestDoc = TestDoc {
        index: 0,
        id: 0,
        text: "cat",
    };
    pub static DOG: TestDoc = TestDoc {
        index: 1,
        id: 1,
        text: "dog",
    };
    pub static MOUSE: TestDoc = TestDoc {
        index: 2,
        id: 2,
        text: "mouse",
    };
    pub static CAT_DOG: TestDoc = TestDoc {
        index: 3,
        id: 3,
        text: "cat dog",
    };
    pub static DOG_MOUSE: TestDoc = TestDoc {
        index: 4,
        id: 4,
        text: "dog mouse",
    };
    pub static CAT_MOUSE: TestDoc = TestDoc {
        index: 5,
        id: 5,
        text: "cat mouse",
    };
    pub static CAT_MOUSE_CAT: TestDoc = TestDoc {
        index: 6,
        id: 6,
        text: "cat mouse cat",
    };
}

#[must_use]
pub fn create_cat_mouse_docs_iterator() -> TestDocsIterator {
    TestDocsIterator::from_enumerated_texts(&Vec::from([
        &docs::CAT,
        &docs::DOG,
        &docs::MOUSE,
        &docs::CAT_DOG,
        &docs::DOG_MOUSE,
        &docs::CAT_MOUSE,
        &docs::CAT_MOUSE_CAT,
    ]))
}
