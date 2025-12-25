use crate::span::{SourceId, Span, Spanned};
use internment::Intern;
use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Clone, Default, PartialEq, Error, Diagnostic)]
pub enum LexError {
    #[error("invalid integer literal")]
    #[diagnostic(
        code(lex::invalid_int),
        help("ensure the integer is within valid range")
    )]
    InvalidInt(#[from] std::num::ParseIntError),

    #[error("invalid float literal")]
    #[diagnostic(
        code(lex::invalid_float),
        help("check the float format (e.g., 1.0, 1e10, .5)")
    )]
    InvalidFloat(#[from] std::num::ParseFloatError),

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

    #[error("unknown escape sequence in character literal")]
    #[diagnostic(
        code(lex::unknown_escape),
        help("valid escape sequences are: \\', \\\\, \\n, \\r, \\t, \\0")
    )]
    UnknownEscape,

    #[error("invalid character literal")]
    #[diagnostic(code(lex::invalid_char))]
    InvalidChar,

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
    KwFun,
    KwInt,
    KwBool,
    KwFloat,
    KwChar,
    KwUnit,
    Comma,
    Cons,
    Eq,
    LParen,
    RParen,
    Float(f64),
    Int(usize),
    Bool(bool),
    Char(char),
    Ident(Intern<String>),
}

pub struct Lexer<'src> {
    src_id: SourceId,
    chars: std::iter::Peekable<std::str::Chars<'src>>,
    offset: usize,
}

impl<'src> Lexer<'src> {
    pub fn new(src_id: SourceId, input: &'src str) -> Self {
        Self {
            src_id,
            chars: input.chars().peekable(),
            offset: 0,
        }
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        self.offset += c.len_utf8();
        Some(c)
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn consume_while<F>(&mut self, mut test: F) -> String
    where
        F: FnMut(char) -> bool,
    {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if test(c) {
                s.push(self.next_char().unwrap());
            } else {
                break;
            }
        }
        s
    }

    pub fn tokenize(mut self) -> Result<Vec<Spanned<Token>>, Vec<Spanned<LexError>>> {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();

        while let Some(c) = self.peek() {
            let start = self.offset;

            match c {
                c if c.is_whitespace() => {
                    self.next_char();
                    continue;
                }

                ',' => {
                    self.next_char();
                    tokens.push((Token::Comma, Span::new(self.src_id, start..self.offset)));
                }
                '(' => {
                    self.next_char();
                    tokens.push((Token::LParen, Span::new(self.src_id, start..self.offset)));
                }
                ')' => {
                    self.next_char();
                    tokens.push((Token::RParen, Span::new(self.src_id, start..self.offset)));
                }
                '=' => {
                    self.next_char();
                    tokens.push((Token::Eq, Span::new(self.src_id, start..self.offset)));
                }
                ':' => {
                    self.next_char();
                    if self.peek() == Some(':') {
                        self.next_char();
                        tokens.push((Token::Cons, Span::new(self.src_id, start..self.offset)));
                    } else {
                        errors.push((
                            LexError::InvalidToken,
                            Span::new(self.src_id, start..self.offset),
                        ));
                    }
                }

                '\'' => match self.lex_char_literal() {
                    Ok(ch) => {
                        tokens.push((Token::Char(ch), Span::new(self.src_id, start..self.offset)))
                    }
                    Err(e) => errors.push((e, Span::new(self.src_id, start..self.offset))),
                },

                '0'..='9' | '.' => match self.lex_number() {
                    Ok(t) => tokens.push((t, Span::new(self.src_id, start..self.offset))),
                    Err(e) => errors.push((e, Span::new(self.src_id, start..self.offset))),
                },

                'a'..='z' | '_' => {
                    let ident = self.consume_while(|c| c.is_alphanumeric() || c == '_');
                    let token = match ident.as_str() {
                        "fun" => Token::KwFun,
                        "int" => Token::KwInt,
                        "bool" => Token::KwBool,
                        "float" => Token::KwFloat,
                        "char" => Token::KwChar,
                        "unit" => Token::KwUnit,
                        "true" => Token::Bool(true),
                        "false" => Token::Bool(false),
                        _ => Token::Ident(Intern::new(ident)),
                    };
                    tokens.push((token, Span::new(self.src_id, start..self.offset)));
                }

                _ => {
                    self.next_char();
                    errors.push((
                        LexError::InvalidToken,
                        Span::new(self.src_id, start..self.offset),
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(tokens)
        } else {
            Err(errors)
        }
    }

    fn lex_char_literal(&mut self) -> Result<char, LexError> {
        self.next_char();
        let c = self.next_char().ok_or(LexError::InvalidChar)?;

        let result = if c == '\\' {
            let esc = self.next_char().ok_or(LexError::UnknownEscape)?;
            match esc {
                '\'' => '\'',
                '\\' => '\\',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '0' => '\0',
                _ => return Err(LexError::UnknownEscape),
            }
        } else if c == '\'' {
            return Err(LexError::EmptyChar);
        } else {
            c
        };

        if self.next_char() != Some('\'') {
            return Err(LexError::MultiChar);
        }
        Ok(result)
    }

    fn lex_number(&mut self) -> Result<Token, LexError> {
        let mut has_dot = false;
        let mut s = String::new();

        while let Some(c) = self.peek() {
            match c {
                '.' if !has_dot => {
                    has_dot = true;
                    s.push(self.next_char().unwrap());
                }
                '0'..='9' => s.push(self.next_char().unwrap()),
                'e' | 'E' => {
                    has_dot = true;
                    s.push(self.next_char().unwrap());
                    if let Some('+' | '-') = self.peek() {
                        s.push(self.next_char().unwrap());
                    }
                }
                _ => break,
            }
        }

        if has_dot {
            s.parse::<f64>().map(Token::Float).map_err(LexError::from)
        } else {
            s.parse::<usize>().map(Token::Int).map_err(LexError::from)
        }
    }
}
