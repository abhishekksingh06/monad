use crate::span::{SourceId, Span, Spanned};
use internment::Intern;
use logos::Logos;
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

#[derive(Debug, Clone, PartialEq, PartialOrd, Logos)]
#[logos(skip r"[ \t\n\f]+")] // skip whitespace
#[logos(error = LexError)]
pub enum Token {
    #[token("fun")]
    KwFun,
    #[token("int")]
    KwInt,
    #[token("bool")]
    KwBool,
    #[token("float")]
    KwFloat,
    #[token("char")]
    KwChar,
    #[token("unit")]
    KwUnit,
    #[token(",")]
    Comma,
    #[token("::")]
    Cons,
    #[token("=")]
    Eq,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[regex(r"[+-]?((\d+\.\d*|\.\d+)([eE][+-]?\d+)?|\d+[eE][+-]?\d+)", |lex| lex.slice().parse())]
    Float(f64),
    #[regex(r"[0-9]+", |lex| lex.slice().parse())]
    Int(usize),
    #[token("true", |_| true)]
    #[token("false", |_| false)]
    Bool(bool),
    #[regex(r"'(?:[^'\\\n\r]|\\.)*'", parse_char)]
    Char(char),
    #[regex(r"[a-z_][a-zA-Z0-9_]*", |lex| Intern::new(lex.slice().to_string()))]
    Ident(Intern<String>),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::KwFun => f.write_str("fun"),
            Token::KwInt => f.write_str("int"),
            Token::KwBool => f.write_str("bool"),
            Token::Comma => f.write_str(","),
            Token::Cons => f.write_str("::"),
            Token::Eq => f.write_str("="),
            Token::LParen => f.write_str("("),
            Token::RParen => f.write_str(")"),
            Token::Ident(v) => f.write_str(v),
            Token::Int(v) => write!(f, "{v}"),
            Token::Float(v) => write!(f, "{v}"),
            Token::Bool(v) => write!(f, "{v}"),
            Token::Char(v) => write!(f, "'{v}'"),
            Token::KwFloat => f.write_str("float"),
            Token::KwChar => f.write_str("char"),
            Token::KwUnit => f.write_str("unit"),
        }
    }
}

fn parse_char<'src>(lex: &logos::Lexer<'src, Token>) -> Result<char, LexError> {
    let slice = lex.slice();
    let content = &slice[1..slice.len() - 1];

    if content.is_empty() {
        return Err(LexError::EmptyChar);
    }

    let mut chars = content.chars();
    let first = chars.next().unwrap();

    if first == '\\' {
        let escaped = chars.next();
        if chars.next().is_some() {
            return Err(LexError::MultiChar);
        }
        match escaped {
            Some('\'') => Ok('\''),
            Some('\\') => Ok('\\'),
            Some('n') => Ok('\n'),
            Some('r') => Ok('\r'),
            Some('t') => Ok('\t'),
            Some('0') => Ok('\0'),
            Some(_) => Err(LexError::UnknownEscape),
            None => Err(LexError::UnknownEscape),
        }
    } else if chars.next().is_none() {
        Ok(first)
    } else {
        Err(LexError::MultiChar)
    }
}

// Collect all tokens and errors; lexer continues after invalid tokens
pub fn lex(src_id: SourceId, input: &str) -> Result<Vec<Spanned<Token>>, Vec<Spanned<LexError>>> {
    let lexer = Token::lexer(input);
    let mut tokens = Vec::new();
    let mut errors = Vec::new();

    for (result, range) in lexer.spanned() {
        let span = Span::new(src_id, range);
        match result {
            Ok(token) => tokens.push((token, span)),
            Err(err) => errors.push((err, span)),
        }
    }

    if errors.is_empty() {
        Ok(tokens)
    } else {
        Err(errors)
    }
}

