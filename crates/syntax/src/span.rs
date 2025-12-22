use std::ops::Range;

use miette::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    #[inline]
    pub fn new(start: u32, end: u32) -> Self {
        debug_assert!(start <= end);
        Self { start, end }
    }

    #[inline]
    pub fn len(self) -> u32 {
        self.end - self.start
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    #[inline]
    pub fn join(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

impl From<Range<u32>> for Span {
    fn from(range: Range<u32>) -> Self {
        Span::new(range.start, range.end)
    }
}

impl From<Span> for Range<u32> {
    fn from(span: Span) -> Self {
        span.start..span.end
    }
}

impl From<Span> for SourceSpan {
    fn from(span: Span) -> Self {
        (span.start as usize, span.len() as usize).into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Spanned<T> {
    pub inner: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    #[inline]
    pub fn new(inner: T, span: Span) -> Self {
        Self { inner, span }
    }

    #[inline]
    pub fn map<U, F>(self, f: F) -> Spanned<U>
    where
        F: FnOnce(T) -> U,
    {
        Spanned {
            inner: f(self.inner),
            span: self.span,
        }
    }

    #[inline]
    pub fn as_ref(&self) -> Spanned<&T> {
        Spanned {
            inner: &self.inner,
            span: self.span,
        }
    }

    #[inline]
    pub fn as_mut(&mut self) -> Spanned<&mut T> {
        Spanned {
            inner: &mut self.inner,
            span: self.span,
        }
    }
    #[inline]
    pub fn with_span(self, span: Span) -> Self {
        Spanned { span, ..self }
    }
}

impl<T> From<(T, Span)> for Spanned<T> {
    fn from((inner, span): (T, Span)) -> Self {
        Spanned::new(inner, span)
    }
}

impl<T> From<(T, Range<u32>)> for Spanned<T> {
    fn from((inner, range): (T, Range<u32>)) -> Self {
        Spanned::new(inner, range.into())
    }
}
