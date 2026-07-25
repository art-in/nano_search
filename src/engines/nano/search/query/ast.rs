use crate::utils::TreeNode;

/// Abstract syntax tree (AST) node of a search query.
///
/// This is intented to be pure object tree structure reflecting contents of
/// search query string, without any additional logic.
#[derive(Debug, PartialEq, Eq)]
pub enum QueryAst<'a> {
    Word(&'a str),

    // operators
    And(Vec<Self>),
    Or(Vec<Self>),
    Not(Box<Self>),
}

impl QueryAst<'_> {
    pub fn explain(&self) -> TreeNode {
        match self {
            QueryAst::Word(word) => {
                TreeNode::new("Word").with_attr("word", *word)
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
