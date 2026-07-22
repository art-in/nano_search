/// Abstract syntax tree (AST) node of a search query.
///
/// This is intented to be pure object data structure directly reflecting
/// contents for search query string, without any additional logic.
#[derive(Debug, PartialEq, Eq)]
pub enum QueryAst<'a> {
    Word(&'a str),

    // operators
    And(Vec<Self>),
    Or(Vec<Self>),
    Not(Box<Self>),
}
