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
    Float,
    Unit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(usize),
    Char(char),
    Bool(bool),
    Float(f64),
    Unit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Value(Value),
    Local(Ident),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Ident(Ident),
    Wildcard,
    Typed { pattern: Box<Self>, ty: Type },
    // Future pattern forms (not yet supported):
    // - Tuple patterns for destructuring multiple values
    // - Constructor patterns for algebraic data types
    // - Reference / borrow patterns for ownership-aware matching
    //
    // These are intentionally deferred until composite types,
    // ADTs, and ownership semantics are introduced.
    //
    // Tuple(Vec<Self>),
    // Constructor { name: Ident, args: Vec<Self> },
    // Ref(Box<Self>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Func {
    pub name: Spanned<Ident>,
    pub params: Vec<Spanned<Pattern>>,
    pub ty: Option<Spanned<Type>>,
    pub expr: Expr,
}
