use anyhow::{Context, Result, bail};

use super::iterators::{
    ExcludingDocIdIterator, IntersectingDocIdIterator, PostingListIterator,
    ScoringDocIdIterator, UnionDocIdIterator,
};
use super::query::{QueryAst, QueryAstWord};
use crate::engines::nano::index::model::IndexSegment;

/// Builds document ID iterators tree out of query AST for one index segment.
///
/// It receives semantically validated and normalized query AST, so it should
/// not perform any structural rewrites and validations.
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
        QueryAst::Word(term) => plan_word(term, segment)?,
        QueryAst::And(operands) => plan_and(operands, segment)?,
        QueryAst::Or(operands) => plan_or(operands, segment)?,
        QueryAst::Not(_) => bail!("NOT should only be handled as part of AND"),
    })
}

fn plan_word<'a>(
    word: &QueryAstWord,
    segment: &'a dyn IndexSegment,
) -> Result<Box<dyn ScoringDocIdIterator + 'a>> {
    let it = PostingListIterator::create_for_segment(
        segment,
        &word.value,
        word.frequency,
    )?;
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
    let mut inputs = Vec::new();

    for operand in operands {
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
    use crate::engines::nano::search::query::test_utils::*;

    fn create_segment() -> Box<dyn IndexSegment> {
        let mut segment = MemoryIndex::default();
        for term in &["a", "b", "c", "d", "e"] {
            segment.terms.insert(term.to_string(), BTreeMap::new());
        }
        Box::new(segment)
    }

    fn plan_and_explain(query_ast: &QueryAst) -> Result<String> {
        let segment = create_segment();
        let it = plan_query_for_segment(query_ast, segment.as_ref())?;
        Ok(it.explain().to_string())
    }

    #[test]
    fn test_word() -> Result<()> {
        let query = word("a");
        assert_eq!(plan_and_explain(&query)?, "PostingList (term = a)\n");
        Ok(())
    }

    #[test]
    fn test_word_unknown() -> Result<()> {
        let query = word("x");
        assert_eq!(
            plan_and_explain(&query)?,
            "PostingList (term = x, unknown_term = true)\n"
        );
        Ok(())
    }

    #[test]
    fn test_and() -> Result<()> {
        let query = and([word("a"), word("b")]);
        assert_eq!(
            plan_and_explain(&query)?,
            indoc! {"
                Intersection
                ├── PostingList (term = a)
                └── PostingList (term = b)
            "}
        );
        Ok(())
    }

    #[test]
    fn test_and_single_include_single_exclude() -> Result<()> {
        let query = and([
            word("a"),      // include
            not(word("b")), // exclude
        ]);
        assert_eq!(
            plan_and_explain(&query)?,
            indoc! {"
                Exclusion
                ├── include = PostingList (term = a)
                └── exclude = PostingList (term = b)
            "}
        );
        Ok(())
    }

    #[test]
    fn test_and_single_include_nested_single_exclude() -> Result<()> {
        let query = and([
            or([word("a"), word("b")]), // include
            not(word("c")),             // exclude
        ]);
        assert_eq!(
            plan_and_explain(&query)?,
            indoc! {"
                Exclusion
                ├── include = Union
                │   ├── PostingList (term = a)
                │   └── PostingList (term = b)
                └── exclude = PostingList (term = c)
            "}
        );
        Ok(())
    }

    #[test]
    fn test_and_multiple_includes_single_exclude() -> Result<()> {
        let query = and([
            word("a"),      // include
            word("b"),      // include
            not(word("c")), // exclude
        ]);
        assert_eq!(
            plan_and_explain(&query)?,
            indoc! {"
                Exclusion
                ├── include = Intersection
                │   ├── PostingList (term = a)
                │   └── PostingList (term = b)
                └── exclude = PostingList (term = c)
            "}
        );
        Ok(())
    }

    #[test]
    fn test_and_single_include_multiple_excludes() -> Result<()> {
        let query = and([
            word("a"),      // include
            not(word("b")), // exclude
            not(word("c")), // exclude
        ]);
        assert_eq!(
            plan_and_explain(&query)?,
            indoc! {"
                Exclusion
                ├── include = PostingList (term = a)
                └── exclude = Union
                    ├── PostingList (term = b)
                    └── PostingList (term = c)
            "}
        );
        Ok(())
    }

    #[test]
    fn test_or() -> Result<()> {
        let query = or([word("a"), word("b")]);
        assert_eq!(
            plan_and_explain(&query)?,
            indoc! {"
                Union
                ├── PostingList (term = a)
                └── PostingList (term = b)
            "}
        );
        Ok(())
    }

    #[test]
    fn test_not_nested_or() -> Result<()> {
        let query = and([
            word("a"),
            not(or([
                word("b"), // nested
                word("c"),
            ])),
        ]);
        assert_eq!(
            plan_and_explain(&query)?,
            indoc! {"
                Exclusion
                ├── include = PostingList (term = a)
                └── exclude = Union
                    ├── PostingList (term = b)
                    └── PostingList (term = c)
            "}
        );
        Ok(())
    }

    #[test]
    fn test_not_nested_and() -> Result<()> {
        let query = and([
            word("a"),
            not(and([
                word("b"), // nested
                word("c"),
            ])),
        ]);
        assert_eq!(
            plan_and_explain(&query)?,
            indoc! {"
                Exclusion
                ├── include = PostingList (term = a)
                └── exclude = Intersection
                    ├── PostingList (term = b)
                    └── PostingList (term = c)
            "}
        );
        Ok(())
    }

    #[test]
    fn test_complex() -> Result<()> {
        let query = and([
            word("a"),
            or([
                and([word("b"), not(word("c"))]), // nested
                word("d"),
            ]),
            not(word("e")),
        ]);
        assert_eq!(
            plan_and_explain(&query)?,
            indoc! {"
                Exclusion
                ├── include = Intersection
                │   ├── PostingList (term = a)
                │   └── Union
                │       ├── Exclusion
                │       │   ├── include = PostingList (term = b)
                │       │   └── exclude = PostingList (term = c)
                │       └── PostingList (term = d)
                └── exclude = PostingList (term = e)
            "}
        );
        Ok(())
    }
}
