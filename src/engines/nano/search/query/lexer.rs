use std::iter::Peekable;
use std::str::CharIndices;

pub struct Lexer<'a> {
    input: &'a str,
    chars: Peekable<CharIndices<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input,
            chars: input.char_indices().peekable(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    Word(&'a str),

    // operators
    And,
    Or,
    Not,

    // scope delimiters
    LParen,
    RParen,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // skip all leading whitespaces
        while let Some(&(_, ch)) = self.chars.peek() {
            if ch.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }

        // get the structural starting point
        let (start_idx, first_ch) = self.chars.next()?;

        match first_ch {
            '(' => return Some(Token::LParen),
            ')' => return Some(Token::RParen),
            _ => {}
        }

        // scan a continuous text block
        let mut end_idx = start_idx + first_ch.len_utf8();

        while let Some(&(idx, ch)) = self.chars.peek() {
            if ch.is_whitespace() || matches!(ch, '(' | ')') {
                break;
            }
            self.chars.next();
            end_idx = idx + ch.len_utf8();
        }

        // slice out the exact string segment unicode-safely
        let slice = &self.input[start_idx..end_idx];

        match slice {
            "AND" => Some(Token::And),
            "OR" => Some(Token::Or),
            "NOT" => Some(Token::Not),
            _ => Some(Token::Word(slice)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_and_whitespace() {
        let mut lex = Lexer::new("");
        assert_eq!(lex.next(), None);

        let mut lex = Lexer::new("   \n\t  ");
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_single_word() {
        let mut lex = Lexer::new("word");

        assert_eq!(lex.next(), Some(Token::Word("word")));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_multiple_words_and_whitespace() {
        let mut lex = Lexer::new("  word1   word2\tword3  ");

        assert_eq!(lex.next(), Some(Token::Word("word1")));
        assert_eq!(lex.next(), Some(Token::Word("word2")));
        assert_eq!(lex.next(), Some(Token::Word("word3")));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_operators_basic() {
        let mut lex = Lexer::new("word1 AND word2 OR word3 NOT word4");

        assert_eq!(lex.next(), Some(Token::Word("word1")));
        assert_eq!(lex.next(), Some(Token::And));
        assert_eq!(lex.next(), Some(Token::Word("word2")));
        assert_eq!(lex.next(), Some(Token::Or));
        assert_eq!(lex.next(), Some(Token::Word("word3")));
        assert_eq!(lex.next(), Some(Token::Not));
        assert_eq!(lex.next(), Some(Token::Word("word4")));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_parentheses_basic() {
        let mut lex = Lexer::new("(word1)");

        assert_eq!(lex.next(), Some(Token::LParen));
        assert_eq!(lex.next(), Some(Token::Word("word1")));
        assert_eq!(lex.next(), Some(Token::RParen));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_nested_parentheses() {
        let mut lex = Lexer::new("word1 AND ((word2) OR (word3))");

        assert_eq!(lex.next(), Some(Token::Word("word1")));
        assert_eq!(lex.next(), Some(Token::And));

        assert_eq!(lex.next(), Some(Token::LParen));
        assert_eq!(lex.next(), Some(Token::LParen));

        assert_eq!(lex.next(), Some(Token::Word("word2")));
        assert_eq!(lex.next(), Some(Token::RParen));

        assert_eq!(lex.next(), Some(Token::Or));

        assert_eq!(lex.next(), Some(Token::LParen));
        assert_eq!(lex.next(), Some(Token::Word("word3")));
        assert_eq!(lex.next(), Some(Token::RParen));

        assert_eq!(lex.next(), Some(Token::RParen));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_operator_should_not_match_prefix_or_suffix() {
        let mut lex = Lexer::new("ANDAND ORORword NOTword wordAND wordOR");

        assert_eq!(lex.next(), Some(Token::Word("ANDAND")));
        assert_eq!(lex.next(), Some(Token::Word("ORORword")));
        assert_eq!(lex.next(), Some(Token::Word("NOTword")));
        assert_eq!(lex.next(), Some(Token::Word("wordAND")));
        assert_eq!(lex.next(), Some(Token::Word("wordOR")));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_mixed_structure_without_spaces() {
        let mut lex = Lexer::new("(word1)(word2)");

        assert_eq!(lex.next(), Some(Token::LParen));
        assert_eq!(lex.next(), Some(Token::Word("word1")));
        assert_eq!(lex.next(), Some(Token::RParen));
        assert_eq!(lex.next(), Some(Token::LParen));
        assert_eq!(lex.next(), Some(Token::Word("word2")));
        assert_eq!(lex.next(), Some(Token::RParen));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_mixed_structure_with_operators() {
        let mut lex = Lexer::new("word1 AND(word2 OR(word3))");

        assert_eq!(lex.next(), Some(Token::Word("word1")));
        assert_eq!(lex.next(), Some(Token::And));

        assert_eq!(lex.next(), Some(Token::LParen));
        assert_eq!(lex.next(), Some(Token::Word("word2")));
        assert_eq!(lex.next(), Some(Token::Or));

        assert_eq!(lex.next(), Some(Token::LParen));
        assert_eq!(lex.next(), Some(Token::Word("word3")));
        assert_eq!(lex.next(), Some(Token::RParen));
        assert_eq!(lex.next(), Some(Token::RParen));

        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_unicode_safety() {
        let mut lex = Lexer::new("🦀 AND ⚙️");

        assert_eq!(lex.next(), Some(Token::Word("🦀")));
        assert_eq!(lex.next(), Some(Token::And));
        assert_eq!(lex.next(), Some(Token::Word("⚙️")));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_complex_query() {
        let mut lex = Lexer::new("(rust OR c++) AND NOT (java OR python)");

        assert_eq!(lex.next(), Some(Token::LParen));
        assert_eq!(lex.next(), Some(Token::Word("rust")));
        assert_eq!(lex.next(), Some(Token::Or));
        assert_eq!(lex.next(), Some(Token::Word("c++")));
        assert_eq!(lex.next(), Some(Token::RParen));

        assert_eq!(lex.next(), Some(Token::And));
        assert_eq!(lex.next(), Some(Token::Not));

        assert_eq!(lex.next(), Some(Token::LParen));
        assert_eq!(lex.next(), Some(Token::Word("java")));
        assert_eq!(lex.next(), Some(Token::Or));
        assert_eq!(lex.next(), Some(Token::Word("python")));
        assert_eq!(lex.next(), Some(Token::RParen));

        assert_eq!(lex.next(), None);
    }
}
