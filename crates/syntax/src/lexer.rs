use logos::Logos;
use thiserror::Error;

use crate::span::{SourceId, Span, Spanned};

#[derive(Debug, Clone, Default, PartialEq, Error)]
pub enum LexError {
    #[error("invalid integer literal: {0}")]
    InvalidInt(#[from] std::num::ParseIntError),
    #[error("invalid float literal: {0}")]
    InvalidFloat(#[from] std::num::ParseFloatError),
    #[error("character literal cannot be empty")]
    EmptyChar,
    #[error("character literal must contain exactly one character")]
    MultiChar,
    #[error("unknown escape sequence in character literal")]
    UnknownEscape,
    #[error("invalid character literal")]
    InvalidChar,
    #[default]
    #[error("invalid token")]
    InvalidToken,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Logos)]
#[logos(skip r"[ \t\n\f]+")] // skip whitespace
#[logos(error = LexError)]
pub enum Token<'src> {
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
    #[regex(r"[a-z_][a-zA-Z0-9_]*", |lex| lex.slice())]
    Ident(&'src str),
}

impl<'src> std::fmt::Display for Token<'src> {
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
        }
    }
}

fn parse_char<'src>(lex: &logos::Lexer<'src, Token<'src>>) -> Result<char, LexError> {
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
pub fn lex<'src>(
    src_id: SourceId,
    input: &'src str,
) -> Result<Vec<Spanned<Token<'src>>>, Vec<Spanned<LexError>>> {
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
