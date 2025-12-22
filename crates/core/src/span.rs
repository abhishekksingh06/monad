use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        debug_assert!(start <= end, "invalid span: {start}..{end}");
        Self { start, end }
    }

    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

impl From<Range<u32>> for Span {
    fn from(val: Range<u32>) -> Self {
        Span::new(val.start, val.end)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Spanned<T> {
    pub inner: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(inner: T, span: Span) -> Self {
        Self { inner, span }
    }

    pub fn map<U, F>(self, f: F) -> Spanned<U>
    where
        F: FnOnce(T) -> U,
    {
        Spanned::new(f(self.inner), self.span)
    }
}

impl<T> From<(T, Range<u32>)> for Spanned<T> {
    fn from(value: (T, Range<u32>)) -> Self {
        Spanned::new(value.0, value.1.into())
    }
}

impl<T> From<(T, Span)> for Spanned<T> {
    fn from(value: (T, Span)) -> Self {
        Spanned::new(value.0, value.1)
    }
}
