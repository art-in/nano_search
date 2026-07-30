mod ast;
#[cfg(test)]
pub use ast::test_utils;
pub use ast::{QueryAst, QueryAstWord};

mod lexer;
#[allow(unused_imports)] // TODO: remove unused code silencer
pub use lexer::Lexer;

mod normalize;
#[allow(unused_imports)] // TODO: remove unused code silencer
pub use normalize::normalize;

mod parser;
#[allow(unused_imports)] // TODO: remove unused code silencer
pub use parser::Parser;
