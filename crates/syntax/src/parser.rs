use std::usize;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::{
    ast::{self, Literal},
    lexer::Token,
    span::{Span, Spanned},
};

#[derive(Debug, Error, Diagnostic)]
pub enum ParseError {
    #[error("expected {expected}, found {found}")]
    #[diagnostic(
        code(parse::unexpected_token),
        help("ensure the token order matches the grammar")
    )]
    UnexpectedToken {
        expected: Token,
        found: Token,
        #[label("here")]
        span: SourceSpan,
    },

    #[error("unexpected end of input")]
    #[diagnostic(
        code(parse::unexpected_eof),
        help("try adding a missing expression or closing delimiter")
    )]
    UnexpectedEOF,

    #[error("expected a literal, found {found}")]
    #[diagnostic(
        code(parse::expected_literal),
        help("use a valid literal such as an integer, float, boolean, or character")
    )]
    ExpectedLiteral {
        found: Token,
        #[label("not a literal")]
        span: SourceSpan,
    },
}

pub type ParserResult<T> = Result<T, ParseError>;

#[derive(Clone)]
pub struct Parser {
    tokens: Vec<Spanned<Token>>,
    pos: usize,
    len: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Spanned<Token>>) -> Self {
        Parser {
            len: tokens.len(),
            tokens,
            pos: 0,
        }
    }

    #[inline]
    fn current(&self) -> &Spanned<Token> {
        &self.tokens[self.pos]
    }

    #[inline]
    fn peek(&self) -> &Token {
        &self.current().0
    }

    #[inline]
    fn advance(&mut self) -> Spanned<Token> {
        let tok = self.current().clone();
        if self.pos < self.len - 1 {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: Token) -> Result<Spanned<Token>, ParseError> {
        let (token, span) = self.current().clone();
        if token == expected {
            Ok(self.advance())
        } else {
            Err(ParseError::UnexpectedToken {
                expected,
                found: token,
                span: span.into(),
            })
        }
    }

    fn parse_literal(&mut self) -> ParserResult<Spanned<Literal>> {
        let (token, span) = self.advance();
        match token {
            Token::Int(v) => Ok((Literal::Int(v), span)),
            Token::Real(x) => Ok((Literal::Real(x), span)),
            Token::Char(c) => Ok((Literal::Char(c), span)),
            Token::LParen if *self.peek() == Token::RParen => {
                let (_, other_span) = self.advance();
                Ok((Literal::Unit, span.merge(other_span)))
            }
            _ => Err(ParseError::ExpectedLiteral {
                found: token,
                span: span.into(),
            }),
        }
    }
}

