use super::test_docs_iterator::TestDocsIterator;

static CAT: (u64, &str) = (0, "cat");
static DOG: (u64, &str) = (1, "dog");
static MOUSE: (u64, &str) = (2, "mouse");
static CAT_DOG: (u64, &str) = (3, "cat dog");
static DOG_MOUSE: (u64, &str) = (4, "dog mouse");
static CAT_MOUSE: (u64, &str) = (5, "cat mouse");
static CAT_MOUSE_CAT: (u64, &str) = (6, "cat mouse cat");

pub struct DocIds {
    pub cat: u64,
    pub dog: u64,
    pub mouse: u64,
    pub cat_dog: u64,
    pub dog_mouse: u64,
    pub cat_mouse: u64,
    pub cat_mouse_cat: u64,
}

pub static ID: DocIds = DocIds {
    cat: CAT.0,
    dog: DOG.0,
    mouse: MOUSE.0,
    cat_dog: CAT_DOG.0,
    dog_mouse: DOG_MOUSE.0,
    cat_mouse: CAT_MOUSE.0,
    cat_mouse_cat: CAT_MOUSE_CAT.0,
};

#[must_use]
pub fn create_cat_mouse_docs_iterator() -> TestDocsIterator {
    TestDocsIterator::from_enumerated_texts(&Vec::from([
        CAT,
        DOG,
        MOUSE,
        CAT_DOG,
        DOG_MOUSE,
        CAT_MOUSE,
        CAT_MOUSE_CAT,
    ]))
}
