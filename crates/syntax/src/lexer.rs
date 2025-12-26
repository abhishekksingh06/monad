use std::fmt;

use crate::span::{SourceId, Span, Spanned};
use internment::Intern;
use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Clone, Default, PartialEq, Error, Diagnostic)]
pub enum LexError {
    #[error("invalid integer literal: {0}")]
    #[diagnostic(
        code(lex::invalid_int),
        help("ensure the integer is within valid range (0 to {})", usize::MAX)
    )]
    InvalidInt(String),

    #[error("invalid float literal: {0}")]
    #[diagnostic(
        code(lex::invalid_float),
        help("check the float format (e.g., 1.0, 1e10, .5)")
    )]
    InvalidFloat(String),

    #[error("character literal cannot be empty")]
    #[diagnostic(
        code(lex::empty_char),
        help("character literals must contain exactly one character, e.g., 'a'")
    )]
    EmptyChar,

    #[error("character literal must contain exactly one character")]
    #[diagnostic(
        code(lex::multi_char),
        help("use a string literal for multiple characters, or keep only one character")
    )]
    MultiChar,

    #[error("unknown escape sequence: '\\{0}'")]
    #[diagnostic(
        code(lex::unknown_escape),
        help("valid escape sequences are: \\', \\\", \\\\, \\n, \\r, \\t, \\0")
    )]
    UnknownEscape(char),

    #[error("unterminated character literal")]
    #[diagnostic(
        code(lex::unterminated_char),
        help("character literals must end with a closing single quote '")
    )]
    UnterminatedChar,

    #[error("invalid character in number literal")]
    #[diagnostic(
        code(lex::invalid_number_char),
        help("numbers can only contain digits, dots, and exponent notation (e/E)")
    )]
    InvalidNumberChar,

    #[default]
    #[error("invalid token")]
    #[diagnostic(
        code(lex::invalid_token),
        help("this character or sequence is not recognized")
    )]
    InvalidToken,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Token {
    // Keywords
    KwFun,
    KwInt,
    KwBool,
    KwReal,
    KwChar,
    KwUnit,
    KwVal,
    KwLet,
    KwIn,
    KwEnd,
    KwIf,
    KwThen,
    KwElse,
    KwNot,
    KwMut,
    KwWhile,
    KwDo,
    KwMod,
    KwDiv,

    Comma,
    Cons, // ::
    Eq,
    NotEq,
    Colon,
    ColonEq,
    LParen,
    RParen,
    Gt,
    GtEq,
    Less,
    LessEq,
    AndAnd,
    Or,
    And,
    Tilde,
    Plus,
    Minus,
    Star,

    Real(f64),
    Int(usize),
    Bool(bool),
    Char(char),

    Ident(Intern<String>),

    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::KwFun => write!(f, "fun"),
            Token::KwInt => write!(f, "int"),
            Token::KwBool => write!(f, "bool"),
            Token::KwReal => write!(f, "real"),
            Token::KwChar => write!(f, "char"),
            Token::KwUnit => write!(f, "unit"),
            Token::KwVal => write!(f, "val"),
            Token::KwLet => write!(f, "let"),
            Token::KwIn => write!(f, "in"),
            Token::KwEnd => write!(f, "end"),
            Token::KwIf => write!(f, "if"),
            Token::KwThen => write!(f, "then"),
            Token::KwElse => write!(f, "else"),
            Token::KwNot => write!(f, "not"),
            Token::KwMut => write!(f, "mut"),
            Token::KwWhile => write!(f, "while"),
            Token::KwDo => write!(f, "do"),
            Token::KwMod => write!(f, "mod"),
            Token::KwDiv => write!(f, "div"),
            Token::Comma => write!(f, ","),
            Token::Cons => write!(f, "::"),
            Token::Eq => write!(f, "="),
            Token::NotEq => write!(f, "<>"),
            Token::Colon => write!(f, ":"),
            Token::ColonEq => write!(f, ":="),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Gt => write!(f, ">"),
            Token::GtEq => write!(f, ">="),
            Token::Less => write!(f, "<"),
            Token::LessEq => write!(f, "<="),
            Token::AndAnd => write!(f, "&&"),
            Token::Or => write!(f, "or"),
            Token::And => write!(f, "and"),
            Token::Tilde => write!(f, "~"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Real(v) => write!(f, "{v}"),
            Token::Int(v) => write!(f, "{v}"),
            Token::Bool(v) => write!(f, "{v}"),
            Token::Char(c) => write!(f, "'{c}'"),
            Token::Ident(id) => write!(f, "{id}"),
            Token::Eof => write!(f, "<eof>"),
        }
    }
}

pub struct Lexer<'src> {
    src_id: SourceId,
    chars: std::iter::Peekable<std::str::CharIndices<'src>>,
    source: &'src str,
    current_pos: usize,
}

impl<'src> Lexer<'src> {
    pub fn new(src_id: SourceId, input: &'src str) -> Self {
        Self {
            src_id,
            chars: input.char_indices().peekable(),
            source: input,
            current_pos: 0,
        }
    }

    fn next_char(&mut self) -> Option<(usize, char)> {
        let result = self.chars.next();
        if let Some((pos, _)) = result {
            self.current_pos = pos;
        }
        result
    }

    fn peek(&mut self) -> Option<(usize, char)> {
        self.chars.peek().copied()
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, c)| *c)
    }

    fn skip_whitespace(&mut self) {
        while let Some((_, c)) = self.peek() {
            if c.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Spanned<Token>>, Vec<Spanned<LexError>>> {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();

        loop {
            self.skip_whitespace();

            let Some((start, c)) = self.peek() else {
                break;
            };

            let result = match c {
                ',' => {
                    self.next_char();
                    Ok(Token::Comma)
                }
                '~' => {
                    self.next_char();
                    Ok(Token::Tilde)
                }
                '(' => {
                    self.next_char();
                    Ok(Token::LParen)
                }
                ')' => {
                    self.next_char();
                    Ok(Token::RParen)
                }
                '=' => {
                    self.next_char();
                    Ok(Token::Eq)
                }
                '+' => {
                    self.next_char();
                    Ok(Token::Plus)
                }
                '-' => {
                    self.next_char();
                    Ok(Token::Minus)
                }
                '*' => {
                    self.next_char();
                    Ok(Token::Star)
                }
                ':' => self.lex_colon(),
                '<' => self.lex_less(),
                '>' => self.lex_gt(),
                '&' => self.lex_and(),
                '|' => self.lex_or(),
                '\'' => self.lex_char_literal(),
                '0'..='9' => self.lex_number(),
                '.' => {
                    // Check if this is a float starting with a dot
                    if matches!(self.chars.clone().nth(1), Some((_, '0'..='9'))) {
                        self.lex_number()
                    } else {
                        self.next_char();
                        Err(LexError::InvalidToken)
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => self.lex_ident(),
                _ => {
                    self.next_char();
                    Err(LexError::InvalidToken)
                }
            };

            let end = self.current_pos + 1;
            let span = Span::new(self.src_id, start..end);

            match result {
                Ok(token) => tokens.push((token, span)),
                Err(error) => errors.push((error, span)),
            }
        }

        // Add EOF token at the end
        let eof_pos = self.source.len();
        tokens.push((Token::Eof, Span::new(self.src_id, eof_pos..eof_pos)));

        if errors.is_empty() {
            Ok(tokens)
        } else {
            Err(errors)
        }
    }

    fn lex_colon(&mut self) -> Result<Token, LexError> {
        self.next_char(); // consume ':'
        match self.peek_char() {
            Some(':') => {
                self.next_char(); // consume second ':'
                Ok(Token::Cons)
            }
            Some('=') => {
                self.next_char(); // consume '='
                Ok(Token::ColonEq)
            }
            _ => Ok(Token::Colon),
        }
    }

    fn lex_less(&mut self) -> Result<Token, LexError> {
        self.next_char(); // consume '<'
        match self.peek_char() {
            Some('>') => {
                self.next_char(); // consume '>'
                Ok(Token::NotEq)
            }
            Some('=') => {
                self.next_char(); // consume '='
                Ok(Token::LessEq)
            }
            _ => Ok(Token::Less),
        }
    }

    fn lex_gt(&mut self) -> Result<Token, LexError> {
        self.next_char(); // consume '>'
        if self.peek_char() == Some('=') {
            self.next_char(); // consume '='
            Ok(Token::GtEq)
        } else {
            Ok(Token::Gt)
        }
    }

    fn lex_and(&mut self) -> Result<Token, LexError> {
        self.next_char(); // consume '&'
        if self.peek_char() == Some('&') {
            self.next_char(); // consume second '&'
            Ok(Token::AndAnd)
        } else {
            Ok(Token::And)
        }
    }

    fn lex_or(&mut self) -> Result<Token, LexError> {
        self.next_char(); // consume '|' 
        if self.peek_char() == Some('|') {
            self.next_char(); // consume second '|'
            Ok(Token::Or)
        } else {
            Err(LexError::InvalidToken)
        }
    }

    fn lex_char_literal(&mut self) -> Result<Token, LexError> {
        self.next_char(); // consume opening '

        let Some((_, c)) = self.next_char() else {
            return Err(LexError::UnterminatedChar);
        };

        let result = if c == '\\' {
            // Handle escape sequence
            let Some((_, esc)) = self.next_char() else {
                return Err(LexError::UnterminatedChar);
            };
            match esc {
                '\'' => '\'',
                '\"' => '\"',
                '\\' => '\\',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '0' => '\0',
                _ => return Err(LexError::UnknownEscape(esc)),
            }
        } else if c == '\'' {
            return Err(LexError::EmptyChar);
        } else {
            c
        };

        match self.next_char() {
            Some((_, '\'')) => Ok(Token::Char(result)),
            Some(_) => Err(LexError::MultiChar),
            None => Err(LexError::UnterminatedChar),
        }
    }

    fn lex_number(&mut self) -> Result<Token, LexError> {
        let start_pos = self.current_pos;
        let mut has_dot = false;
        let mut has_exponent = false;

        // Consume all valid number characters
        while let Some((_, c)) = self.peek() {
            match c {
                '.' if !has_dot && !has_exponent => {
                    has_dot = true;
                    self.next_char();
                }
                '0'..='9' => {
                    self.next_char();
                }
                'e' | 'E' if !has_exponent => {
                    has_dot = true;
                    has_exponent = true;
                    self.next_char();

                    if matches!(self.peek_char(), Some('+' | '-')) {
                        self.next_char();
                    }
                }
                _ => break,
            }
        }

        let end_pos = self.current_pos + 1;
        let num_str = &self.source[start_pos..end_pos];

        if has_dot || has_exponent {
            num_str
                .parse::<f64>()
                .map(Token::Real)
                .map_err(|_| LexError::InvalidFloat(num_str.to_string()))
        } else {
            num_str
                .parse::<usize>()
                .map(Token::Int)
                .map_err(|_| LexError::InvalidInt(num_str.to_string()))
        }
    }

    fn lex_ident(&mut self) -> Result<Token, LexError> {
        let start_pos = self.current_pos;

        while let Some((_, c)) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                self.next_char();
            } else {
                break;
            }
        }

        let end_pos = self.current_pos + 1;
        let ident = &self.source[start_pos..end_pos];

        Ok(self.classify_ident(ident))
    }

    fn classify_ident(&self, ident: &str) -> Token {
        match ident {
            "fun" => Token::KwFun,
            "int" => Token::KwInt,
            "bool" => Token::KwBool,
            "real" => Token::KwReal,
            "char" => Token::KwChar,
            "unit" => Token::KwUnit,
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            "val" => Token::KwVal,
            "let" => Token::KwLet,
            "in" => Token::KwIn,
            "end" => Token::KwEnd,
            "if" => Token::KwIf,
            "then" => Token::KwThen,
            "else" => Token::KwElse,
            "not" => Token::KwNot,
            "mut" => Token::KwMut,
            "do" => Token::KwDo,
            "while" => Token::KwWhile,
            "mod" => Token::KwMod,
            "div" => Token::KwDiv,
            _ => Token::Ident(Intern::new(ident.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let src_id = SourceId::default();
        let lexer = Lexer::new(src_id, "( ) , = ::");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 6); // 5 tokens + EOF
        assert_eq!(tokens[0].0, Token::LParen);
        assert_eq!(tokens[1].0, Token::RParen);
        assert_eq!(tokens[2].0, Token::Comma);
        assert_eq!(tokens[3].0, Token::Eq);
        assert_eq!(tokens[4].0, Token::Cons);
        assert_eq!(tokens[5].0, Token::Eof);
    }

    #[test]
    fn test_numbers() {
        let src_id = SourceId::default();
        let lexer = Lexer::new(src_id, "42 3.14 .5 1e10 2.5e-3");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].0, Token::Int(42));
        assert_eq!(tokens[2].0, Token::Real(0.5));
    }

    #[test]
    fn test_keywords() {
        let src_id = SourceId::default();
        let lexer = Lexer::new(src_id, "fun if then else true false");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].0, Token::KwFun);
        assert_eq!(tokens[1].0, Token::KwIf);
        assert_eq!(tokens[2].0, Token::KwThen);
        assert_eq!(tokens[3].0, Token::KwElse);
        assert_eq!(tokens[4].0, Token::Bool(true));
        assert_eq!(tokens[5].0, Token::Bool(false));
    }

    #[test]
    fn test_char_literals() {
        let src_id = SourceId::default();
        let lexer = Lexer::new(src_id, r"'a' '\n' '\'' '\\'");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].0, Token::Char('a'));
        assert_eq!(tokens[1].0, Token::Char('\n'));
        assert_eq!(tokens[2].0, Token::Char('\''));
        assert_eq!(tokens[3].0, Token::Char('\\'));
    }
}
