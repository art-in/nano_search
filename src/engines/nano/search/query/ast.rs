use std::borrow::Cow;

use crate::utils::TreeNode;

/// Abstract syntax tree (AST) node of a search query.
///
/// This is intented to be pure object tree structure reflecting contents of
/// search query string, without any additional logic.
#[derive(Debug, PartialEq, Eq)]
pub enum QueryAst<'a> {
    Word(Cow<'a, str>),

    // operators
    And(Vec<Self>),
    Or(Vec<Self>),
    Not(Box<Self>),
}

impl QueryAst<'_> {
    pub fn explain(&self) -> TreeNode {
        match self {
            QueryAst::Word(word) => {
                TreeNode::new("Word").with_attr("word", word.as_ref())
            }
            QueryAst::And(operands) => {
                let mut tree = TreeNode::new("AND");
                for op in operands {
                    tree.add_child(op.explain());
                }
                tree
            }
            QueryAst::Or(operands) => {
                let mut tree = TreeNode::new("OR");
                for op in operands {
                    tree.add_child(op.explain());
                }
                tree
            }
            QueryAst::Not(operand) => {
                TreeNode::new("NOT").with_child(operand.explain())
            }
        }
    }
}

#[cfg(test)]
pub mod test_utils {
    use super::*;

    pub fn word(s: &'static str) -> QueryAst<'static> {
        QueryAst::Word(Cow::Borrowed(s))
    }

    pub fn and(
        children: impl IntoIterator<Item = QueryAst<'static>>,
    ) -> QueryAst<'static> {
        QueryAst::And(children.into_iter().collect())
    }

    pub fn or(
        children: impl IntoIterator<Item = QueryAst<'static>>,
    ) -> QueryAst<'static> {
        QueryAst::Or(children.into_iter().collect())
    }

    pub fn not(child: QueryAst<'static>) -> QueryAst<'static> {
        QueryAst::Not(Box::new(child))
    }
}
