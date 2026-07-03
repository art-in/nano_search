use std::iter::Peekable;

use anyhow::{Result, bail, ensure};

use crate::engines::nano::search::query::lexer::Token;

#[derive(Debug, PartialEq, Eq)]
pub enum QueryAst<'a> {
    Word(&'a str),

    // operators
    And(Vec<Self>),
    Or(Vec<Self>),
    Not(Box<Self>),
}

// Search query parser that turns token stream into abstract syntax tree (AST).
//
// Implemented as simple recursive descent, which should be enough for search
// query grammar, even if it grows to more complex features like fields, ranges,
// etc.
//
// Grammar (EBNF):
//
//   ```
//   expression ::= or
//
//   or ::= and ("OR" and )*
//
//   and ::= unary ("AND" unary )*
//
//   unary ::= "NOT" unary
//           | primary
//
//   primary ::= WORD
//             | "(" expression ")"
//   ```
//
// Notes:
//
// Implicit OR rule: adjacent primary expressions are treated as OR.
// Examples:
// - "a b"     => "a OR b"
// - "a (b c)" => "a OR (b OR c)"
//
// NOT operator is allowed to be root AST node and be part of OR expressions.
// Forcing NOT to only be part of AND expression is not syntax parser's job,
// and should happen on later semantic analysis stage.
// Examples:
// - "NOT a"      - syntax OK
// - "a OR NOT b" - syntax OK
struct Parser<I: Iterator> {
    tokens: Peekable<I>,
}

impl<'a, I> Parser<I>
where
    I: Iterator<Item = Token<'a>>,
{
    pub fn new(tokens: I) -> Self {
        Self {
            tokens: tokens.peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<QueryAst<'a>> {
        let ast = self.parse_expression()?;
        ensure!(self.tokens.next().is_none(), "should consume all tokens");
        Ok(ast)
    }

    fn consume_token(&mut self, token: &Token<'a>) -> bool {
        if self.tokens.peek() == Some(token) {
            self.tokens.next();
            true
        } else {
            false
        }
    }

    fn expect_token(
        &mut self,
        token: &Token<'a>,
        message: &'static str,
    ) -> Result<()> {
        ensure!(self.consume_token(token), message);
        Ok(())
    }

    fn starts_operand(&mut self) -> bool {
        matches!(
            self.tokens.peek(),
            Some(Token::Word(_) | Token::LParen | Token::Not)
        )
    }

    fn parse_expression(&mut self) -> Result<QueryAst<'a>> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<QueryAst<'a>> {
        let ast = self.parse_and()?;

        // starts_operand() check allows implicit OR, e.g. "a b" == "a OR b"
        if self.tokens.peek() == Some(&Token::Or) || self.starts_operand() {
            let mut chain = vec![ast];

            while self.consume_token(&Token::Or) || self.starts_operand() {
                chain.push(self.parse_and()?);
            }

            Ok(QueryAst::Or(chain))
        } else {
            Ok(ast)
        }
    }

    fn parse_and(&mut self) -> Result<QueryAst<'a>> {
        let ast = self.parse_unary()?;

        if self.tokens.peek() == Some(&Token::And) {
            let mut chain = vec![ast];

            while self.consume_token(&Token::And) {
                chain.push(self.parse_unary()?);
            }

            Ok(QueryAst::And(chain))
        } else {
            Ok(ast)
        }
    }

    fn parse_unary(&mut self) -> Result<QueryAst<'a>> {
        if self.consume_token(&Token::Not) {
            Ok(QueryAst::Not(Box::new(self.parse_unary()?)))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<QueryAst<'a>> {
        match self.tokens.next() {
            Some(Token::Word(word)) => Ok(QueryAst::Word(word)),
            Some(Token::LParen) => {
                let ast = self.parse_expression()?;
                self.expect_token(&Token::RParen, "should receive ')'")?;
                Ok(ast)
            }
            // TODO: print offset position of problem in all errors
            _ => bail!("should receive word or '('"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::nano::search::query::lexer::Lexer;

    fn parse(input: &str) -> Result<QueryAst<'_>> {
        Parser::new(Lexer::new(input)).parse()
    }

    fn parse_err(input: &str) -> Result<String> {
        match Parser::new(Lexer::new(input)).parse() {
            Ok(_) => bail!("should return error"),
            Err(message) => Ok(message.to_string()),
        }
    }

    // --- BASIC & STRUCTURAL ---

    #[test]
    fn test_single_word() -> Result<()> {
        assert_eq!(parse("word")?, QueryAst::Word("word"));
        Ok(())
    }

    #[test]
    fn test_nested_parentheses() -> Result<()> {
        assert_eq!(parse("(((word)))")?, QueryAst::Word("word"));
        Ok(())
    }

    // --- UNARY OPERATORS ---

    #[test]
    fn test_double_not() -> Result<()> {
        assert_eq!(
            parse("NOT NOT word")?,
            QueryAst::Not(Box::new(QueryAst::Not(Box::new(QueryAst::Word(
                "word"
            )))))
        );
        Ok(())
    }

    #[test]
    fn test_not_over_parentheses() -> Result<()> {
        assert_eq!(
            parse("NOT (a OR b)")?,
            QueryAst::Not(Box::new(QueryAst::Or(vec![
                QueryAst::Word("a"),
                QueryAst::Word("b"),
            ])))
        );
        Ok(())
    }

    #[test]
    fn test_not_ouside_and() -> Result<()> {
        assert_eq!(
            parse("a NOT b")?,
            QueryAst::Or(vec![
                QueryAst::Word("a"),
                QueryAst::Not(Box::new(QueryAst::Word("b")))
            ])
        );
        assert_eq!(
            parse("NOT a b")?,
            QueryAst::Or(vec![
                QueryAst::Not(Box::new(QueryAst::Word("a"))),
                QueryAst::Word("b"),
            ])
        );
        Ok(())
    }

    // --- EXPLICIT CHAINS ---

    #[test]
    fn test_and_chain() -> Result<()> {
        assert_eq!(
            parse("a AND b AND c")?,
            QueryAst::And(vec![
                QueryAst::Word("a"),
                QueryAst::Word("b"),
                QueryAst::Word("c"),
            ])
        );
        Ok(())
    }

    #[test]
    fn test_or_chain() -> Result<()> {
        assert_eq!(
            parse("a OR b OR c")?,
            QueryAst::Or(vec![
                QueryAst::Word("a"),
                QueryAst::Word("b"),
                QueryAst::Word("c"),
            ])
        );
        Ok(())
    }

    // --- IMPLICIT OPERATORS ---

    #[test]
    fn test_implicit_or_words() -> Result<()> {
        assert_eq!(
            parse("a b c")?,
            QueryAst::Or(vec![
                QueryAst::Word("a"),
                QueryAst::Word("b"),
                QueryAst::Word("c"),
            ])
        );
        Ok(())
    }

    #[test]
    fn test_implicit_or_parenthesized() -> Result<()> {
        assert_eq!(
            parse("a (b AND c)")?,
            QueryAst::Or(vec![
                QueryAst::Word("a"),
                QueryAst::And(vec![QueryAst::Word("b"), QueryAst::Word("c")]),
            ])
        );
        Ok(())
    }

    #[test]
    fn test_implicit_or_after_parentheses() -> Result<()> {
        assert_eq!(
            parse("(a AND b) c d")?,
            QueryAst::Or(vec![
                QueryAst::And(vec![QueryAst::Word("a"), QueryAst::Word("b")]),
                QueryAst::Word("c"),
                QueryAst::Word("d"),
            ])
        );
        Ok(())
    }

    // --- PRECEDENCE ---

    #[test]
    fn test_and_before_or() -> Result<()> {
        assert_eq!(
            parse("a OR b AND c")?,
            QueryAst::Or(vec![
                QueryAst::Word("a"),
                QueryAst::And(vec![QueryAst::Word("b"), QueryAst::Word("c")]),
            ])
        );
        Ok(())
    }

    #[test]
    fn test_or_after_and() -> Result<()> {
        assert_eq!(
            parse("a AND b OR c")?,
            QueryAst::Or(vec![
                QueryAst::And(vec![QueryAst::Word("a"), QueryAst::Word("b")]),
                QueryAst::Word("c"),
            ])
        );
        Ok(())
    }

    #[test]
    fn test_not_before_and() -> Result<()> {
        assert_eq!(
            parse("NOT a AND b")?,
            QueryAst::And(vec![
                QueryAst::Not(Box::new(QueryAst::Word("a"))),
                QueryAst::Word("b"),
            ])
        );
        Ok(())
    }

    #[test]
    fn test_parentheses_override_precedence() -> Result<()> {
        assert_eq!(
            parse("(a OR b) AND c")?,
            QueryAst::And(vec![
                QueryAst::Or(vec![QueryAst::Word("a"), QueryAst::Word("b")]),
                QueryAst::Word("c"),
            ])
        );
        Ok(())
    }

    #[test]
    fn test_mixed_explicit_and_implicit_or() -> Result<()> {
        assert_eq!(
            parse("a AND b c AND d")?,
            QueryAst::Or(vec![
                QueryAst::And(vec![QueryAst::Word("a"), QueryAst::Word("b")]),
                QueryAst::And(vec![QueryAst::Word("c"), QueryAst::Word("d")]),
            ])
        );
        Ok(())
    }

    #[test]
    fn test_implicit_or_yields_to_explicit_and() -> Result<()> {
        assert_eq!(
            parse("a b AND c")?,
            QueryAst::Or(vec![
                QueryAst::Word("a"),
                QueryAst::And(vec![QueryAst::Word("b"), QueryAst::Word("c")]),
            ])
        );
        Ok(())
    }

    // --- ERRORS ---

    #[test]
    fn test_empty_input() -> Result<()> {
        assert_eq!(parse_err("")?, "should receive word or '('");
        Ok(())
    }

    #[test]
    fn test_missing_rhs_binary_operators() -> Result<()> {
        assert_eq!(parse_err("a AND")?, "should receive word or '('");
        assert_eq!(parse_err("a OR")?, "should receive word or '('");
        Ok(())
    }

    #[test]
    fn test_missing_rhs_unary_operator() -> Result<()> {
        assert_eq!(parse_err("NOT")?, "should receive word or '('");
        assert_eq!(parse_err("a AND NOT")?, "should receive word or '('");
        Ok(())
    }

    #[test]
    fn test_parentheses_mismatch_errors() -> Result<()> {
        assert_eq!(parse_err("(a")?, "should receive ')'");
        assert_eq!(parse_err("((a AND b)")?, "should receive ')'");
        assert_eq!(parse_err("a)")?, "should consume all tokens");
        assert_eq!(parse_err("()")?, "should receive word or '('");
        Ok(())
    }

    #[test]
    fn test_unexpected_operators_at_start() -> Result<()> {
        assert_eq!(parse_err("AND a")?, "should receive word or '('");
        assert_eq!(parse_err("OR a")?, "should receive word or '('");
        Ok(())
    }

    #[test]
    fn test_consecutive_binary_operators() -> Result<()> {
        assert_eq!(parse_err("a AND AND b")?, "should receive word or '('");
        assert_eq!(parse_err("a OR OR b")?, "should receive word or '('");
        assert_eq!(parse_err("a AND OR b")?, "should receive word or '('");
        Ok(())
    }
}
