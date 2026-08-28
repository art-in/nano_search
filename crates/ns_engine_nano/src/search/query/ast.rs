use std::borrow::Cow;

use crate::utils::TreeNode;

/// Abstract syntax tree (AST) node of a search query.
///
/// This is intented to be pure object tree structure reflecting contents of
/// search query string, without any additional logic.
#[derive(Debug, PartialEq, Eq)]
pub enum QueryAst<'a> {
    Word(QueryAstWord<'a>),

    // operators
    And(Vec<Self>),
    Or(Vec<Self>),
    Not(Box<Self>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct QueryAstWord<'a> {
    /// Value that can either be original or normalized word a.k.a. "term"
    pub value: Cow<'a, str>,
    /// Number of times this word occurs among siblings
    pub frequency: usize,
}

impl QueryAst<'_> {
    #[allow(unused)]
    pub fn explain(&self) -> TreeNode {
        match self {
            QueryAst::Word(word) => {
                let mut tree = TreeNode::new("Word")
                    .with_attr("value", word.value.as_ref());

                if word.frequency > 1 {
                    tree.add_attr("freq", word.frequency.to_string());
                }

                tree
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
        QueryAst::Word(QueryAstWord {
            value: Cow::Borrowed(s),
            frequency: 1,
        })
    }

    pub fn word_with_freq(
        s: &'static str,
        frequency: usize,
    ) -> QueryAst<'static> {
        QueryAst::Word(QueryAstWord {
            value: Cow::Borrowed(s),
            frequency,
        })
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
