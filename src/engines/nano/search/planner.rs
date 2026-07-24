use anyhow::{Context, Result, bail, ensure};

use super::iterators::{
    ExcludingDocIdIterator, IntersectingDocIdIterator, PostingListIterator,
    ScoringDocIdIterator, UnionDocIdIterator,
};
use super::query::QueryAst;
use crate::engines::nano::index::model::IndexSegment;
use crate::utils::normalize_word;

/// Builds document ID iterators tree out of query AST for one index segment.
///
/// This is the place for constructing optimal execution plan, e.g. reordering
/// iterators, choosing right iterators, etc.
///
/// Invariants for input query AST:
/// - `NOT` may only appear as operand of `AND` expression (e.g. "x AND NOT y")
/// - `AND` expression should contain at least one including operand and at
///   least two operands total (e.g. "x AND y" or "x AND NOT y")
/// - `OR` should contain at least two operands (e.g. "x OR y")
///
/// Implementation note:
/// Lucene and Tantivy distribute this planning logic across their `Query`
/// implementations (`TermQuery`, `BooleanQuery`, `PhraseQuery`, etc.), where
/// each query type constructs its own execution objects.
/// Nano centralizes this logic in a dedicated planning stage.
pub fn plan_query_for_segment<'a>(
    query_ast: &QueryAst,
    segment: &'a dyn IndexSegment,
) -> Result<Box<dyn ScoringDocIdIterator + 'a>> {
    Ok(match query_ast {
        QueryAst::Word(word) => plan_word(word, segment)?,
        QueryAst::And(operands) => plan_and(operands, segment)?,
        QueryAst::Or(operands) => plan_or(operands, segment)?,
        QueryAst::Not(_) => {
            bail!("NOT should only appear as direct operand of AND")
        }
    })
}

fn plan_word<'a>(
    word: &str,
    segment: &'a dyn IndexSegment,
) -> Result<Box<dyn ScoringDocIdIterator + 'a>> {
    let term = normalize_word(word);
    let it = PostingListIterator::create_for_segment(segment, &term)?;
    Ok(Box::new(it))
}

fn plan_and<'a>(
    operands: &[QueryAst],
    segment: &'a dyn IndexSegment,
) -> Result<Box<dyn ScoringDocIdIterator + 'a>> {
    let mut includes = Vec::new();
    let mut excludes = Vec::new();

    for operand in operands {
        if let QueryAst::Not(inner) = operand {
            excludes.push(plan_query_for_segment(inner, segment)?);
        } else {
            includes.push(plan_query_for_segment(operand, segment)?);
        }
    }

    ensure!(
        !includes.is_empty(),
        "AND should contain at least one including operand"
    );

    ensure!(
        operands.len() >= 2,
        "AND should contain at least two operands"
    );

    let includes_it: Box<dyn ScoringDocIdIterator + 'a> = if includes.len() == 1
    {
        includes.pop().context("should exist")?
    } else {
        Box::new(IntersectingDocIdIterator::new(includes))
    };

    let result = if excludes.is_empty() {
        includes_it
    } else if excludes.len() == 1 {
        let excludes_it = excludes.pop().context("should exist")?;
        Box::new(ExcludingDocIdIterator::new(includes_it, excludes_it))
    } else {
        let excludes_it = Box::new(UnionDocIdIterator::new(excludes));
        Box::new(ExcludingDocIdIterator::new(includes_it, excludes_it))
    };

    Ok(result)
}

fn plan_or<'a>(
    operands: &[QueryAst],
    segment: &'a dyn IndexSegment,
) -> Result<Box<dyn ScoringDocIdIterator + 'a>> {
    ensure!(
        operands.len() >= 2,
        "OR should contain at least two operands"
    );

    let mut inputs = Vec::new();
    for operand in operands {
        ensure!(
            !matches!(operand, QueryAst::Not(_)),
            "NOT should not be a part of OR expression"
        );
        let it = plan_query_for_segment(operand, segment)?;
        inputs.push(it);
    }

    Ok(Box::new(UnionDocIdIterator::new(inputs)))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::engines::nano::index::MemoryIndex;
    use crate::engines::nano::search::query::{Lexer, Parser};

    fn create_segment() -> Box<dyn IndexSegment> {
        let mut segment = MemoryIndex::default();
        for term in &["a", "b", "c", "d", "e"] {
            segment.terms.insert(term.to_string(), BTreeMap::new());
        }
        Box::new(segment)
    }

    fn plan_query_and_explain(query: &str) -> Result<String> {
        let tokens = Lexer::new(query);
        let query_ast = Parser::new(tokens).parse()?;
        let segment = create_segment();
        let it = plan_query_for_segment(&query_ast, segment.as_ref())?;
        Ok(it.explain().to_string())
    }

    fn plan_query_ast_and_explain(query_ast: &QueryAst) -> Result<String> {
        let segment = create_segment();
        let it = plan_query_for_segment(query_ast, segment.as_ref())?;
        Ok(it.explain().to_string())
    }

    fn err<T>(input: Result<T>) -> Result<String> {
        match input {
            Ok(_) => bail!("should return error"),
            Err(message) => Ok(message.to_string()),
        }
    }

    #[test]
    fn test_word() -> Result<()> {
        assert_eq!(plan_query_and_explain("a")?, "Term (term = a)\n");
        Ok(())
    }

    #[test]
    fn test_word_unknown() -> Result<()> {
        assert_eq!(
            plan_query_and_explain("x")?,
            "Term (term = x, unknown_term = true)\n"
        );
        Ok(())
    }

    #[test]
    fn test_and() -> Result<()> {
        assert_eq!(
            plan_query_and_explain("a AND b")?,
            indoc! {"
                Intersection
                ├── Term (term = a)
                └── Term (term = b)
            "}
        );
        Ok(())
    }

    #[test]
    fn test_and_single_include_single_exclude() -> Result<()> {
        assert_eq!(
            plan_query_and_explain("a AND NOT b")?,
            indoc! {"
                Exclusion
                ├── include = Term (term = a)
                └── exclude = Term (term = b)
            "}
        );
        Ok(())
    }

    #[test]
    fn test_and_single_include_nested_single_exclude() -> Result<()> {
        assert_eq!(
            plan_query_and_explain("(a OR b) AND NOT c")?,
            indoc! {"
                Exclusion
                ├── include = Union
                │   ├── Term (term = a)
                │   └── Term (term = b)
                └── exclude = Term (term = c)
            "}
        );
        Ok(())
    }

    #[test]
    fn test_and_multiple_includes_single_exclude() -> Result<()> {
        assert_eq!(
            plan_query_and_explain("a AND b AND NOT c")?,
            indoc! {"
                Exclusion
                ├── include = Intersection
                │   ├── Term (term = a)
                │   └── Term (term = b)
                └── exclude = Term (term = c)
            "}
        );
        Ok(())
    }

    #[test]
    fn test_and_single_include_multiple_excludes() -> Result<()> {
        assert_eq!(
            plan_query_and_explain("a AND NOT b AND NOT c")?,
            indoc! {"
                Exclusion
                ├── include = Term (term = a)
                └── exclude = Union
                    ├── Term (term = b)
                    └── Term (term = c)
            "}
        );
        Ok(())
    }

    #[test]
    fn test_and_all_excluding() -> Result<()> {
        assert_eq!(
            err(plan_query_and_explain("NOT a AND NOT b"))?,
            "AND should contain at least one including operand"
        );
        Ok(())
    }

    #[test]
    fn test_and_all_excluding_nested() -> Result<()> {
        // TODO: support this by removing nesting in some AST normalization
        // layer between parser and planner
        assert_eq!(
            err(plan_query_and_explain("c AND (NOT a AND NOT b)"))?,
            "AND should contain at least one including operand"
        );
        Ok(())
    }

    #[test]
    fn test_and_no_operands() -> Result<()> {
        assert_eq!(
            err(plan_query_ast_and_explain(&QueryAst::And(vec![])))?,
            "AND should contain at least one including operand"
        );
        Ok(())
    }

    #[test]
    fn test_and_single_operand() -> Result<()> {
        assert_eq!(
            err(plan_query_ast_and_explain(&QueryAst::And(vec![
                QueryAst::Word("a")
            ])))?,
            "AND should contain at least two operands"
        );
        Ok(())
    }

    #[test]
    fn test_or() -> Result<()> {
        assert_eq!(
            plan_query_and_explain("a OR b")?,
            indoc! {"
                Union
                ├── Term (term = a)
                └── Term (term = b)
            "}
        );
        Ok(())
    }

    #[test]
    fn test_or_no_operands() -> Result<()> {
        assert_eq!(
            err(plan_query_ast_and_explain(&QueryAst::Or(vec![])))?,
            "OR should contain at least two operands"
        );
        Ok(())
    }

    #[test]
    fn test_or_single_operand() -> Result<()> {
        assert_eq!(
            err(plan_query_ast_and_explain(&QueryAst::Or(vec![
                QueryAst::Word("a")
            ])))?,
            "OR should contain at least two operands"
        );
        Ok(())
    }

    #[test]
    fn test_or_not() -> Result<()> {
        assert_eq!(
            err(plan_query_and_explain("a OR NOT b"))?,
            "NOT should not be a part of OR expression"
        );
        Ok(())
    }

    #[test]
    fn test_not_nested_or() -> Result<()> {
        assert_eq!(
            plan_query_and_explain("a AND NOT (b OR c)")?,
            indoc! {"
                Exclusion
                ├── include = Term (term = a)
                └── exclude = Union
                    ├── Term (term = b)
                    └── Term (term = c)
            "}
        );
        Ok(())
    }

    #[test]
    fn test_not_nested_and() -> Result<()> {
        assert_eq!(
            plan_query_and_explain("a AND NOT (b AND c)")?,
            indoc! {"
                Exclusion
                ├── include = Term (term = a)
                └── exclude = Intersection
                    ├── Term (term = b)
                    └── Term (term = c)
            "}
        );
        Ok(())
    }

    #[test]
    fn test_not_not() -> Result<()> {
        assert_eq!(
            err(plan_query_and_explain("a AND NOT NOT b"))?,
            "NOT should only appear as direct operand of AND"
        );
        Ok(())
    }

    #[test]
    fn test_not_root() -> Result<()> {
        assert_eq!(
            err(plan_query_and_explain("NOT a"))?,
            "NOT should only appear as direct operand of AND"
        );
        Ok(())
    }

    #[test]
    fn test_complex() -> Result<()> {
        assert_eq!(
            plan_query_and_explain("a AND ((b AND NOT c) OR d) AND NOT e")?,
            indoc! {"
                Exclusion
                ├── include = Intersection
                │   ├── Term (term = a)
                │   └── Union
                │       ├── Exclusion
                │       │   ├── include = Term (term = b)
                │       │   └── exclude = Term (term = c)
                │       └── Term (term = d)
                └── exclude = Term (term = e)
            "}
        );
        Ok(())
    }
}
