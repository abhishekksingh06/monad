use std::fmt::Display;

use crate::span::Spanned;
use internment::Intern;

#[derive(Debug, Clone, PartialEq)]
pub struct Ident(Intern<String>);

impl Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl AsRef<String> for Ident {
    fn as_ref(&self) -> &String {
        self.0.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Char,
    Bool,
    Real,
    Unit,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Type::Int => "int",
            Type::Char => "char",
            Type::Bool => "bool",
            Type::Real => "real",
            Type::Unit => "()",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(usize),
    Char(char),
    Bool(bool),
    Real(f64),
    Unit,
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Int(v) => write!(f, "{v}"),
            Literal::Char(c) => write!(f, "'{c}'"),
            Literal::Bool(b) => write!(f, "{b}"),
            Literal::Real(x) => write!(f, "{x}"),
            Literal::Unit => write!(f, "()"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Sub,
    Mul,
    Div,
    Rem,

    Eq,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,

    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorrowOp {
    Ref,    // &x
    RefMut, // &mut x
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "div",
            BinaryOp::Rem => "mod",

            BinaryOp::Eq => "=",
            BinaryOp::NotEq => "<>",
            BinaryOp::Less => "<",
            BinaryOp::LessEq => "<=",
            BinaryOp::Greater => ">",
            BinaryOp::GreaterEq => ">=",

            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
        };
        f.write_str(s)
    }
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            UnaryOp::Neg => "~",
            UnaryOp::Not => "not",
        };
        f.write_str(s)
    }
}

impl Display for BorrowOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BorrowOp::Ref => "&",
            BorrowOp::RefMut => "&mut",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Value(Literal),
    Local(Ident),
    Unary {
        op: Spanned<UnaryOp>,
        expr: Box<Spanned<Expr>>,
    },
    Borrow {
        op: Spanned<BorrowOp>,
        expr: Box<Spanned<Expr>>,
    },
    Apply {
        callee: Box<Spanned<Expr>>,
        arg: Box<Spanned<Expr>>,
    },
    Binary {
        left: Box<Spanned<Self>>,
        op: Spanned<BinaryOp>,
        right: Box<Spanned<Self>>,
    },
    Let {
        stmts: Vec<Spanned<Stmt>>,
        expr: Box<Spanned<Self>>,
    },
    If {
        condition: Box<Spanned<Self>>,
        then_expr: Box<Spanned<Expr>>,
        else_expr: Box<Spanned<Self>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Val {
    name: Spanned<Ident>,
    ty: Option<Type>,
    expr: Spanned<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Val(Val),
    Assign {
        target: Spanned<Ident>,
        value: Spanned<Expr>,
    },
    While {
        condition: Spanned<Expr>,
        body: Box<Spanned<Stmt>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum FuncParam {
    Ident(Ident),
    Typed { param: Box<FuncParam>, ty: Type },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Func {
    pub name: Spanned<Ident>,
    pub params: Vec<Spanned<FuncParam>>,
    pub ty: Option<Spanned<Type>>,
    pub expr: Spanned<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    Val(Val),
    Func(Func),
}
