use std::borrow::Cow;
use std::collections::HashMap;

use anyhow::{Result, bail};

use super::{QueryAst, QueryAstWord};
use crate::engines::nano::search::stop_words::STOP_WORDS;
use crate::utils::normalize_word;

/// Semantically validates and rewrites input query AST and thus prepares it for
/// later execution.
///
/// All query-language semantic validation and rewrites should happen here, so
/// syntax parsing on previous stage can directly mirror input query string, and
/// iterators tree on next stage can directly mirror normalized and valid query
/// tree.
///
/// Normalizes words into terms.
///
/// Rewrites (optimizes execution by omitting unnecessary iterators):
/// - removes stop-words, e.g. "the AND term" => "term"
/// - removes double NOT-s, e.g. "NOT NOT a" => "a"
/// - removes AND and OR operators without operands
/// - flattens AND and OR operands, e.g. "a AND (b AND c)" => "a AND b AND c"
/// - deduplicates words, e.g. "a AND a AND b" => "a^2 AND b"
///
/// Validations:
/// - NOT is only allowed to be part of AND expression like in "a AND NOT b", so
///   root-level "NOT a" and "a OR NOT b" are not allowed
/// - AND should contain at least one including operand, so "NOT a AND NOT b" is
///   not allowed
///
/// Validation happens on original un-normalized tree, because normalization may
/// return disallowed states, in which case we just delete bad parts instead of
/// returning error. E.g. "the AND NOT term" => empty, since "the" stop word is
/// removed and NOT bubbled up to root node, which is disallowed structure, so
/// it is silently removed leaving query empty.
///
/// Implemented with simple post-order recursion, so each iteration can assume
/// that children are already normalized.
pub fn normalize(query: &QueryAst) -> Result<Option<QueryAst<'static>>> {
    if is_not(query) {
        bail!("NOT should not be root expression");
    }

    let normalized = normalize_node(query)?;

    if let Some(op) = &normalized
        && (is_not(op) || is_and_with_all_nots(op))
    {
        return Ok(None);
    }

    Ok(normalized)
}

fn normalize_node(query: &QueryAst) -> Result<Option<QueryAst<'static>>> {
    match query {
        QueryAst::Word(word) => normalize_word_node(word),
        QueryAst::And(operands) => normalize_and(operands),
        QueryAst::Or(operands) => normalize_or(operands),
        QueryAst::Not(operand) => normalize_not(operand),
    }
}

fn normalize_word_node(
    word: &QueryAstWord,
) -> Result<Option<QueryAst<'static>>> {
    let term = normalize_word(&word.value);

    term.map_or_else(
        || Ok(None),
        |term| {
            if STOP_WORDS.contains(&term) {
                Ok(None)
            } else {
                Ok(Some(QueryAst::Word(QueryAstWord {
                    value: Cow::Owned(term),
                    frequency: 1,
                })))
            }
        },
    )
}

fn normalize_and(operands: &[QueryAst]) -> Result<Option<QueryAst<'static>>> {
    if is_all_nots(operands) {
        bail!("AND should not contain NOT operands only");
    }

    // normalize children
    let mut children = Vec::new();
    for op in operands {
        let normalized = normalize_node(op)?;
        if let Some(op) = normalized {
            children.push(op);
        }
    }

    // flatten children
    let children = children
        .into_iter()
        .flat_map(|child| match child {
            QueryAst::And(operands) => operands,
            child => vec![child],
        })
        .collect::<Vec<_>>();

    let mut children = deduplicate_words(children);

    if children.is_empty() {
        Ok(None)
    } else if children.len() == 1 {
        Ok(children.pop())
    } else {
        Ok(Some(QueryAst::And(children)))
    }
}

fn normalize_or(operands: &[QueryAst]) -> Result<Option<QueryAst<'static>>> {
    // validate
    for child in operands {
        if is_not(child) {
            bail!("NOT should not be part of OR expression");
        }
    }

    // normalize children
    let mut children = Vec::new();
    for op in operands {
        let normalized = normalize_node(op)?;
        if let Some(op) = normalized {
            children.push(op);
        }
    }

    // flatten children
    let children = children.into_iter().flat_map(|child| match child {
        QueryAst::Or(operands) => operands,
        child => vec![child],
    });

    // remove bad children that may bubbled up during normalization
    let children = children
        .into_iter()
        .filter(|op| !is_not(op) && !is_and_with_all_nots(op))
        .collect::<Vec<_>>();

    let mut children = deduplicate_words(children);

    if children.is_empty() {
        Ok(None)
    } else if children.len() == 1 {
        Ok(children.pop())
    } else {
        Ok(Some(QueryAst::Or(children)))
    }
}

fn normalize_not(operand: &QueryAst) -> Result<Option<QueryAst<'static>>> {
    let child = normalize_node(operand)?;

    // remove bad child that may bubbled up during normalization
    if let Some(child) = &child
        && is_and_with_all_nots(child)
    {
        return Ok(None);
    }

    Ok(child.map(|op| {
        if let QueryAst::Not(op) = op {
            // remove double NOT
            *op
        } else {
            QueryAst::Not(Box::new(op))
        }
    }))
}

// deduplicates direct words only. subtrees like in "(a AND b) OR (a AND b)"
// are not deduplicated, since detecting them is costly and not worth it
fn deduplicate_words(operands: Vec<QueryAst>) -> Vec<QueryAst> {
    let mut word_freqs: HashMap<Cow<'_, str>, usize> = HashMap::new();

    for op in &operands {
        if let QueryAst::Word(word) = &op {
            let freq = word_freqs.entry(word.value.clone()).or_insert(0);

            // accumulate frequency from deeper levels in case the word bubbled
            // up from nested expressions, instead of just counting this level.
            // e.g. in "a AND (a OR a)" frequency of "a" is 3, not 2.
            // this way frequency can be used as a scoring multiplier, and final
            // score will be the same as if deduplication never happened
            *freq += word.frequency;
        }
    }

    let mut res = Vec::new();

    // do second pass to preserve order of operands
    for op in operands {
        if let QueryAst::Word(word) = op {
            let freq = word_freqs.get(&word.value).copied();
            if let Some(count) = freq {
                word_freqs.remove(&word.value);
                res.push(QueryAst::Word(QueryAstWord {
                    value: word.value,
                    frequency: count,
                }));
            }
        } else {
            res.push(op);
        }
    }

    res
}

const fn is_not(operator: &QueryAst) -> bool {
    matches!(operator, QueryAst::Not(_))
}

fn is_and_with_all_nots(operator: &QueryAst) -> bool {
    matches!(operator, QueryAst::And(ops) if is_all_nots(ops))
}

fn is_all_nots(operands: &[QueryAst]) -> bool {
    !operands.is_empty() && operands.iter().all(|op| is_not(op))
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use pretty_assertions::assert_eq;

    use super::super::ast::test_utils::*;
    use super::*;

    fn err<T>(input: Result<T>) -> Result<String> {
        match input {
            Ok(_) => bail!("should return error"),
            Err(message) => Ok(message.to_string()),
        }
    }

    static STOP_WORD: LazyLock<&'static str> = LazyLock::new(|| {
        #[allow(clippy::expect_used)]
        STOP_WORDS
            .iter()
            .next()
            .expect("stop words should not be empty")
            .as_str()
    });

    #[test]
    fn test_word() -> Result<()> {
        let original = word(" First  ");
        let expected = word("first");
        assert_eq!(normalize(&original)?, Some(expected));
        Ok(())
    }

    #[test]
    fn test_word_non_alphabetical() -> Result<()> {
        let original = word("1!@#$%^&*");
        let expected = None;
        assert_eq!(normalize(&original)?, expected);
        Ok(())
    }

    #[test]
    fn test_and_empty() -> Result<()> {
        let original = and([]);
        let expected = None;
        assert_eq!(normalize(&original)?, expected);
        Ok(())
    }

    #[test]
    fn test_and_empty_nested() -> Result<()> {
        let original = or([
            and([]), // empty
            word("first"),
            word("second"),
        ]);
        let expected = or([word("first"), word("second")]);
        assert_eq!(normalize(&original)?, Some(expected));
        Ok(())
    }

    #[test]
    fn test_and_single_operand() -> Result<()> {
        let original = and([word("term")]);
        let expected = word("term");
        assert_eq!(normalize(&original)?, Some(expected));
        Ok(())
    }

    #[test]
    fn test_and_nested_and() -> Result<()> {
        let original = and([
            and([
                word("first"), // nested
                word("second"),
            ]),
            word("third"),
        ]);
        let expected = and([
            word("first"), // flattened
            word("second"),
            word("third"),
        ]);
        assert_eq!(normalize(&original)?, Some(expected));
        Ok(())
    }

    #[test]
    fn test_and_nested_and_not() -> Result<()> {
        // this could be supported by flattening nested AND, but it needs to
        // move validation from single place in AND into all other possible
        // parents (i.e. root, OR, NOT), which would complicate code and not
        // worth it
        let original = and([
            word("first"),
            and([
                not(word("second")), // nested
                not(word("third")),
            ]),
        ]);
        assert_eq!(
            err(normalize(&original))?,
            "AND should not contain NOT operands only"
        );
        Ok(())
    }

    #[test]
    fn test_and_duplicate_words() -> Result<()> {
        let original = and([
            word("first"),
            word("second"),
            word("third"),
            or([
                word("  SeCoNd "), // duplicate
                word(" SeCoNd  "), // duplicate
            ]),
            word("fourth"),
        ]);
        let expected = and([
            word("first"),
            word_with_freq("second", 3),
            word("third"),
            word("fourth"),
        ]);
        assert_eq!(normalize(&original)?, Some(expected));
        Ok(())
    }

    #[test]
    fn test_and_duplicate_subtrees() -> Result<()> {
        // only direct words are deduplicated, subtrees (even simple) are not
        let original = and([
            word("first"),
            not(word("second")),
            not(word("second")), // duplicate
        ]);
        let expected = and([
            word("first"),
            not(word("second")),
            not(word("second")), // duplicate
        ]);
        assert_eq!(normalize(&original)?, Some(expected));
        Ok(())
    }

    #[test]
    fn test_and_all_nots() -> Result<()> {
        let original = and([
            not(word("first")), // not
            not(word("second")),
        ]);
        assert_eq!(
            err(normalize(&original))?,
            "AND should not contain NOT operands only"
        );
        Ok(())
    }

    #[test]
    fn test_and_all_nots_nested() -> Result<()> {
        let original = or([
            word("first"),
            and([
                not(word("first")), // nested
                not(word("second")),
            ]),
        ]);
        assert_eq!(
            err(normalize(&original))?,
            "AND should not contain NOT operands only"
        );
        Ok(())
    }

    #[test]
    fn test_and_all_nots_with_double_not() -> Result<()> {
        // even though first double-NOT is negated, this is still an error
        let original = and([
            not(not(word("first"))), // double-NOT
            not(word("second")),
        ]);
        assert_eq!(
            err(normalize(&original))?,
            "AND should not contain NOT operands only"
        );
        Ok(())
    }

    #[test]
    fn test_or_empty() -> Result<()> {
        let original = or([]);
        let expected = None;
        assert_eq!(normalize(&original)?, expected);
        Ok(())
    }

    #[test]
    fn test_or_empty_nested() -> Result<()> {
        let original = and([
            or([]), // nested
            word("first"),
            word("second"),
        ]);
        let expected = and([word("first"), word("second")]);
        assert_eq!(normalize(&original)?, Some(expected));
        Ok(())
    }

    #[test]
    fn test_or_single_operand() -> Result<()> {
        let original = or([word("term")]);
        let expected = word("term");
        assert_eq!(normalize(&original)?, Some(expected));
        Ok(())
    }

    #[test]
    fn test_or_nested_or() -> Result<()> {
        let original = or([
            or([
                word("first"), // nested
                word("second"),
            ]),
            word("third"),
        ]);
        let expected = or([
            word("first"), // flattened
            word("second"),
            word("third"),
        ]);
        assert_eq!(normalize(&original)?, Some(expected));
        Ok(())
    }

    #[test]
    fn test_or_duplicate_words() -> Result<()> {
        let original = or([
            word("first"),
            word("second"),
            word("third"),
            and([
                word("  SeCoNd "), // duplicate
                word(" SeCoNd  "), // duplicate
            ]),
            word("fourth"),
        ]);
        let expected = or([
            word("first"),
            word_with_freq("second", 3),
            word("third"),
            word("fourth"),
        ]);
        assert_eq!(normalize(&original)?, Some(expected));
        Ok(())
    }

    #[test]
    fn test_not_double() -> Result<()> {
        let original = and([not(not(word("first"))), word("second")]);
        let expected = and([word("first"), word("second")]);
        assert_eq!(normalize(&original)?, Some(expected));
        Ok(())
    }

    #[test]
    fn test_not_tripple() -> Result<()> {
        let original = and([not(not(not(word("first")))), word("second")]);
        let expected = and([not(word("first")), word("second")]);
        assert_eq!(normalize(&original)?, Some(expected));
        Ok(())
    }

    #[test]
    fn test_not_inside_or() -> Result<()> {
        let original = or([not(word("first")), word("second")]);
        assert_eq!(
            err(normalize(&original))?,
            "NOT should not be part of OR expression"
        );
        Ok(())
    }

    #[test]
    fn test_not_inside_or_nested() -> Result<()> {
        let original = or([
            or([
                not(word("first")), // nested
                word("second"),
            ]),
            word("third"),
        ]);
        assert_eq!(
            err(normalize(&original))?,
            "NOT should not be part of OR expression"
        );
        Ok(())
    }

    #[test]
    fn test_not_inside_and() -> Result<()> {
        let original = and([word("first"), not(word("second"))]);
        let expected = and([word("first"), not(word("second"))]);
        assert_eq!(normalize(&original)?, Some(expected));
        Ok(())
    }

    #[test]
    fn test_not_root() -> Result<()> {
        let original = not(word("first"));
        assert_eq!(
            err(normalize(&original))?,
            "NOT should not be root expression"
        );
        Ok(())
    }

    #[test]
    fn test_not_root_double() -> Result<()> {
        // even though double-NOT negate itself, it is still an error
        let original = not(not(word("term")));
        assert_eq!(
            err(normalize(&original))?,
            "NOT should not be root expression"
        );
        Ok(())
    }

    #[test]
    fn test_not_flat_to_root() -> Result<()> {
        let original = and([and([
            not(word("term")), // will be removed after bubbling to root
            word(*STOP_WORD),  // will be removed in place as a stop word
        ])]);
        let expected = None;
        assert_eq!(normalize(&original)?, expected);
        Ok(())
    }

    #[test]
    fn test_not_flat_to_or() -> Result<()> {
        let original = or([
            and([and([
                not(word("first")), // will be removed after bubbling to OR
                word(*STOP_WORD),   // will be removed in place as a stop word
            ])]),
            word("second"),
            word("third"),
        ]);
        let expected = or([word("second"), word("third")]);
        assert_eq!(normalize(&original)?, Some(expected));
        Ok(())
    }

    #[test]
    fn test_not_root_with_stop_word() -> Result<()> {
        // even though stop word gets removed, this is still an error
        let original = not(word(*STOP_WORD));
        assert_eq!(
            err(normalize(&original))?,
            "NOT should not be root expression"
        );
        Ok(())
    }

    #[test]
    fn test_not_inside_or_with_stop_word() -> Result<()> {
        let original = or([not(word(*STOP_WORD)), word("term")]);
        assert_eq!(
            err(normalize(&original))?,
            "NOT should not be part of OR expression"
        );
        Ok(())
    }

    #[test]
    fn test_not_stop_word_inside_and() -> Result<()> {
        let original = and([word("term"), not(word(*STOP_WORD))]);
        let expected = word("term");
        assert_eq!(normalize(&original)?, Some(expected));
        Ok(())
    }

    #[test]
    fn test_not_and_all_nots() -> Result<()> {
        let original = and([
            word("first"),
            not(and([
                word(*STOP_WORD), // will be removed
                not(word("second")),
                not(word("third")),
            ])),
        ]);
        let expected = word("first");
        assert_eq!(normalize(&original)?, Some(expected));
        Ok(())
    }

    #[test]
    fn test_stop_words() -> Result<()> {
        let original = word(*STOP_WORD);
        let expected = None;
        assert_eq!(normalize(&original)?, expected);
        Ok(())
    }

    #[test]
    fn test_stop_words_and_all_nots() -> Result<()> {
        let original = and([
            word(*STOP_WORD), // will be removed
            not(word("first")),
            not(word("second")),
        ]);
        let expected = None;
        assert_eq!(normalize(&original)?, expected);
        Ok(())
    }

    #[test]
    fn test_stop_words_and_all_nots_nested_in_or() -> Result<()> {
        let original = or([
            and([
                word(*STOP_WORD), // will be removed
                not(word("first")),
                not(word("second")),
            ]),
            word("third"),
        ]);
        let expected = word("third");
        assert_eq!(normalize(&original)?, Some(expected));
        Ok(())
    }

    #[test]
    fn test_stop_words_and_all_nots_nested_in_and() -> Result<()> {
        let original = and([
            and([
                word(*STOP_WORD),    // will be removed
                not(word("first")),  // nested
                not(word("second")), // nested
            ]),
            word("third"),
        ]);
        let expected = and([
            not(word("first")),  // flattened
            not(word("second")), // flattened
            word("third"),
        ]);
        assert_eq!(normalize(&original)?, Some(expected));
        Ok(())
    }

    #[test]
    fn test_stop_words_nested() -> Result<()> {
        let original = or([word(*STOP_WORD), word("term")]);
        let expected = word("term");
        assert_eq!(normalize(&original)?, Some(expected));
        Ok(())
    }

    #[test]
    fn test_complex() -> Result<()> {
        // combination of the cases above
        let original = or([
            and([word(" First "), not(not(word(*STOP_WORD))), word("FIRST")]),
            or([word("second"), and([not(word("y")), word(*STOP_WORD)])]),
            and([]),
            and([word("third"), not(word("z"))]),
            word(*STOP_WORD),
            word(" FIRST "),
        ]);
        let expected = or([
            word_with_freq("first", 3),
            word("second"),
            and([word("third"), not(word("z"))]),
        ]);
        assert_eq!(normalize(&original)?, Some(expected));
        Ok(())
    }
}
