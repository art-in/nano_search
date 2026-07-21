use std::fmt::{self, Debug, Display, Formatter};

/// A generic tree node for representing hierarchical structures, useful for
/// debugging, logging, visualization and unit tests.
///
/// Easy to construct with minimal boilerplate, thanks to the chaining.
/// Pretty-prints to a string that reads better than e.g. JSON.
#[derive(PartialEq, Eq)]
#[must_use]
pub struct TreeNode {
    name: String,
    attrs: Option<Vec<(String, String)>>,
    children: Option<Vec<(Option<String>, Self)>>,
}

impl TreeNode {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            attrs: None,
            children: None,
        }
    }

    pub fn add_child(&mut self, child: Self) {
        self.children.get_or_insert_default().push((None, child));
    }

    pub fn with_child(mut self, child: Self) -> Self {
        self.add_child(child);
        self
    }

    pub fn add_keyed_child(&mut self, key: impl Into<String>, child: Self) {
        self.children
            .get_or_insert_default()
            .push((Some(key.into()), child));
    }

    pub fn with_keyed_child(
        mut self,
        key: impl Into<String>,
        child: Self,
    ) -> Self {
        self.add_keyed_child(key, child);
        self
    }

    pub fn add_attr(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) {
        self.attrs
            .get_or_insert_default()
            .push((key.into(), value.into()));
    }

    pub fn with_attr(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.add_attr(key, value);
        self
    }
}

// how many attributes to print on the same line before splitting them by lines
const INLINE_ATTR_THRESHOLD: usize = 5;

impl Display for TreeNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_root(f)
    }
}

impl Debug for TreeNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl TreeNode {
    fn fmt_root(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_header(f, None)?;
        self.fmt_children(f, "")
    }

    fn fmt_header(
        &self,
        f: &mut Formatter<'_>,
        key: Option<&str>,
    ) -> fmt::Result {
        if let Some(key) = key {
            write!(f, "{key} = ")?;
        }

        write!(f, "{}", self.name)?;

        let Some(attrs) = &self.attrs else {
            return writeln!(f);
        };

        let is_inline = attrs.len() <= INLINE_ATTR_THRESHOLD;

        let (open, indent, sep) = if is_inline {
            (" (", "", ", ")
        } else {
            (" (\n", "    ", ",\n")
        };

        write!(f, "{open}")?;

        let mut iter = attrs.iter().peekable();
        while let Some((k, v)) = iter.next() {
            write!(f, "{indent}{k} = {v}")?;
            if iter.peek().is_some() {
                write!(f, "{sep}")?;
            }
        }

        if !is_inline {
            writeln!(f)?;
        }

        writeln!(f, ")")
    }

    fn fmt_children(&self, f: &mut Formatter<'_>, prefix: &str) -> fmt::Result {
        let total = self.children.as_ref().map_or(0, Vec::len);

        for (index, (key, child)) in self.children.iter().flatten().enumerate()
        {
            child.fmt_node(f, prefix, index + 1 == total, key.as_deref())?;
        }

        Ok(())
    }

    fn fmt_node(
        &self,
        f: &mut Formatter<'_>,
        prefix: &str,
        is_last: bool,
        key: Option<&str>,
    ) -> fmt::Result {
        let branch = if is_last { "└── " } else { "├── " };

        write!(f, "{prefix}{branch}")?;
        self.fmt_header(f, key)?;

        let next_prefix = if is_last {
            format!("{prefix}    ")
        } else {
            format!("{prefix}│   ")
        };

        self.fmt_children(f, &next_prefix)
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_chaining() {
        // construct tree with chaining
        let tree_a = TreeNode::new("root")
            .with_attr("key", "value")
            .with_child(TreeNode::new("a"))
            .with_keyed_child("b", TreeNode::new("b"));

        // construct tree without chaining
        let mut tree_b = TreeNode::new("root");
        tree_b.add_attr("key", "value");
        tree_b.add_child(TreeNode::new("a"));
        tree_b.add_keyed_child("b", TreeNode::new("b"));

        assert_eq!(tree_a, tree_b);
    }

    #[test]
    fn test_display_empty_node() {
        assert_eq!(TreeNode::new("Root").to_string(), "Root\n");
    }

    #[test]
    fn test_display_single_attr() {
        let tree = TreeNode::new("Term").with_attr("term", "apple");

        assert_eq!(tree.to_string(), "Term (term = apple)\n");
    }

    #[test]
    fn test_display_multiple_inline_attrs() {
        let tree = TreeNode::new("PostingList")
            .with_attr("term", "apple")
            .with_attr("df", "123")
            .with_attr("blocks", "42")
            .with_attr("compression", "Delta")
            .with_attr("skip_blocks", "8");

        assert_eq!(
            tree.to_string(),
            "PostingList (term = apple, df = 123, blocks = 42, compression = \
             Delta, skip_blocks = 8)\n"
        );
    }

    #[test]
    fn test_display_multiline_attrs() {
        let tree = TreeNode::new("PostingList")
            .with_attr("term", "apple")
            .with_attr("df", "123")
            .with_attr("blocks", "42")
            .with_attr("compression", "Delta")
            .with_attr("skip_blocks", "8")
            .with_attr("postings", "1000");

        assert_eq!(
            tree.to_string(),
            indoc! {"
                PostingList (
                    term = apple,
                    df = 123,
                    blocks = 42,
                    compression = Delta,
                    skip_blocks = 8,
                    postings = 1000
                )
            "},
        );
    }

    #[test]
    fn test_display_children() {
        let tree = TreeNode::new("Intersection")
            .with_child(TreeNode::new("Term").with_attr("term", "a"))
            .with_child(TreeNode::new("Term").with_attr("term", "b"));

        assert_eq!(
            tree.to_string(),
            indoc! {"
                Intersection
                ├── Term (term = a)
                └── Term (term = b)
            "},
        );
    }

    #[test]
    fn test_display_keyed_children() {
        let tree = TreeNode::new("Exclusion")
            .with_child(
                TreeNode::new("Intersection")
                    .with_child(TreeNode::new("Term").with_attr("term", "a"))
                    .with_child(TreeNode::new("Term").with_attr("term", "b")),
            )
            .with_keyed_child(
                "exclude",
                TreeNode::new("Term").with_attr("term", "c"),
            );

        assert_eq!(
            tree.to_string(),
            indoc! {"
                Exclusion
                ├── Intersection
                │   ├── Term (term = a)
                │   └── Term (term = b)
                └── exclude = Term (term = c)
            "}
        );
    }

    #[test]
    fn test_display_mixed_children() {
        let tree = TreeNode::new("Root")
            .with_keyed_child("one", TreeNode::new("A"))
            .with_child(TreeNode::new("B"))
            .with_keyed_child("two", TreeNode::new("C"))
            .with_child(TreeNode::new("D"));

        assert_eq!(
            tree.to_string(),
            indoc! {"
                Root
                ├── one = A
                ├── B
                ├── two = C
                └── D
            "},
        );
    }

    #[test]
    fn test_display_deep_tree() {
        let tree = TreeNode::new("A").with_child(
            TreeNode::new("B").with_child(
                TreeNode::new("C")
                    .with_child(TreeNode::new("D").with_attr("value", "42")),
            ),
        );

        assert_eq!(
            tree.to_string(),
            indoc! {"
                A
                └── B
                    └── C
                        └── D (value = 42)
            "},
        );
    }

    #[test]
    fn test_display_multiline_attrs_with_children() {
        let tree = TreeNode::new("PostingList")
            .with_attr("term", "apple")
            .with_attr("df", "123")
            .with_attr("blocks", "42")
            .with_attr("compression", "Delta")
            .with_attr("skip_blocks", "8")
            .with_attr("postings", "1000")
            .with_child(TreeNode::new("Leaf"));

        assert_eq!(
            tree.to_string(),
            indoc! {"
                PostingList (
                    term = apple,
                    df = 123,
                    blocks = 42,
                    compression = Delta,
                    skip_blocks = 8,
                    postings = 1000
                )
                └── Leaf
            "},
        );
    }
}
