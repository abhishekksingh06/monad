use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::{
    ast::{BinaryOp, BorrowOp, Expr, Ident, Literal, Type, UnaryOp},
    lexer::Token,
    span::{Span, Spanned, SpannedExt},
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

    #[error("expected type, found {found}")]
    #[diagnostic(
        code(parse::expected_type),
        help("a type name was expected here (e.g. int, bool, real, char)")
    )]
    ExpectedType {
        found: Token,
        #[label("type expected here")]
        span: SourceSpan,
    },

    #[error("expected expression")]
    #[diagnostic(
        code(parse::expected_primary),
        help("expected a literal, identifier, or parenthesized expression")
    )]
    ExpectedPrimary {
        #[label("here")]
        span: SourceSpan,
    },

    #[error("expected `{expected}`")]
    #[diagnostic(
        code(parse::expected_delimiter),
        help("add the missing `{expected}` to close this `{opened}`")
    )]
    ExpectedDelimiter {
        expected: Token,
        opened: Token,

        #[label("opened here")]
        open_span: SourceSpan,

        #[label("parser reached here")]
        end_span: SourceSpan,
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

    fn parse_type(&mut self) -> ParserResult<Spanned<Type>> {
        let (token, span) = self.advance();
        match token {
            Token::KwInt => Ok((Type::Int, span)),
            Token::KwUnit => Ok((Type::Unit, span)),
            Token::KwReal => Ok((Type::Real, span)),
            Token::KwChar => Ok((Type::Char, span)),
            _ => Err(ParseError::ExpectedType {
                found: token,
                span: span.into(),
            }),
        }
    }

    fn parse_primary(&mut self) -> ParserResult<Spanned<Expr>> {
        let (token, span) = self.advance();

        match token {
            Token::Int(v) => Ok((Expr::Literal(Literal::Int(v)), span)),
            Token::Real(x) => Ok((Expr::Literal(Literal::Real(x)), span)),
            Token::Char(c) => Ok((Expr::Literal(Literal::Char(c)), span)),
            Token::Ident(s) => Ok((Expr::Local(Ident(s)), span)),

            Token::LParen => {
                if *self.peek() == Token::RParen {
                    let (_, r_span) = self.advance();
                    let span = span.merge(r_span);
                    return Ok((Expr::Literal(Literal::Unit), span));
                }

                let (expr, expr_span) = self.parse_expr()?;

                match self.peek() {
                    Token::RParen => {
                        let (_, r_span) = self.advance();
                        let span = span.merge(expr_span).merge(r_span);
                        Ok((expr, span))
                    }
                    _ => Err(ParseError::ExpectedDelimiter {
                        opened: Token::LParen,
                        expected: Token::RParen,
                        open_span: span.into(),
                        end_span: expr_span.into(),
                    }),
                }
            }

            _ => Err(ParseError::ExpectedPrimary { span: span.into() }),
        }
    }

    fn parse_unary(&mut self) -> ParserResult<Spanned<Expr>> {
        match self.peek() {
            Token::KwNot | Token::Tilde => {
                let op = match self.peek() {
                    Token::KwNot => UnaryOp::Not,
                    Token::Tilde => UnaryOp::Neg,
                    _ => unreachable!(),
                };
                let (_, op_span) = self.advance();
                let (expr, expr_span) = self.parse_unary()?;
                let span = op_span.clone().merge(expr_span.clone());
                Ok((
                    Expr::Unary {
                        op: (op, op_span),
                        expr: Box::new((expr, expr_span)),
                    },
                    span,
                ))
            }
            Token::And => {
                let (_, op_span) = self.advance();
                let (op, op_span) = if *self.peek() == Token::KwMut {
                    let (_, mut_span) = self.advance();
                    (BorrowOp::RefMut, op_span.merge(mut_span))
                } else {
                    (BorrowOp::Ref, op_span)
                };
                let (expr, expr_span) = self.parse_unary()?;
                let span = op_span.clone().merge(expr_span.clone());
                Ok((
                    Expr::Borrow {
                        op: (op, op_span),
                        expr: Box::new((expr, expr_span)),
                    },
                    span,
                ))
            }
            _ => self.parse_primary(),
        }
    }

    #[inline]
    pub fn binary(
        left: Spanned<Expr>,
        op: BinaryOp,
        op_span: Span,
        right: Spanned<Expr>,
    ) -> Spanned<Expr> {
        let span = left.span().merge(right.span());
        (
            Expr::Binary {
                left: Box::new(left),
                right: Box::new(right),
                op: (op, op_span),
            },
            span,
        )
    }

    fn parse_multiplicative(&mut self) -> Result<Spanned<Expr>, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Token::Star => BinaryOp::Mul,
                Token::KwDiv => BinaryOp::Div,
                Token::KwMod => BinaryOp::Rem,
                _ => break,
            };
            let (_, op_span) = self.advance();
            let right = self.parse_unary()?;
            left = Self::binary(left, op, op_span, right);
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Spanned<Expr>, ParseError> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.peek() {
                Token::Plus => BinaryOp::And,
                Token::Minus => BinaryOp::Sub,
                _ => break,
            };
            let (_, op_span) = self.advance();
            let right = self.parse_multiplicative()?;
            left = Self::binary(left, op, op_span, right);
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Spanned<Expr>, ParseError> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.peek() {
                Token::Gt => BinaryOp::Greater,
                Token::GtEq => BinaryOp::GreaterEq,
                Token::Less => BinaryOp::Less,
                Token::LessEq => BinaryOp::LessEq,
                Token::NotEq => BinaryOp::NotEq,
                Token::Eq => BinaryOp::Eq,
                _ => break,
            };
            let (_, op_span) = self.advance();
            let right = self.parse_additive()?;
            left = Self::binary(left, op, op_span, right);
        }
        Ok(left)
    }

    fn parse_and_op(&mut self) -> Result<Spanned<Expr>, ParseError> {
        let mut left = self.parse_comparison()?;
        while let Token::AndAnd = self.peek() {
            let (_, op_span) = self.advance();
            let right = self.parse_comparison()?;
            left = Self::binary(left, BinaryOp::And, op_span, right);
        }
        Ok(left)
    }

    fn parse_or_op(&mut self) -> Result<Spanned<Expr>, ParseError> {
        let mut left = self.parse_and_op()?;
        while let Token::Or = self.peek() {
            let (_, op_span) = self.advance();
            let right = self.parse_and_op()?;
            left = Self::binary(left, BinaryOp::Or, op_span, right);
        }
        Ok(left)
    }

    fn parse_expr(&mut self) -> Result<Spanned<Expr>, ParseError> {
        self.parse_or_op()
    }

    pub fn parse_code(&mut self) -> Result<Spanned<Expr>, ParseError> {
        self.parse_or_op()
    }
}
