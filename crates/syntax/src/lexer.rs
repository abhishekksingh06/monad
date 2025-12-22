use std::ops::Range;

use crate::span::Span;

/// A value annotated with a source span.
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

    /// Map the inner value while preserving the span.
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

    /// Borrow the inner value.
    #[inline]
    pub fn as_ref(&self) -> Spanned<&T> {
        Spanned {
            inner: &self.inner,
            span: self.span,
        }
    }

    /// Mutably borrow the inner value.
    #[inline]
    pub fn as_mut(&mut self) -> Spanned<&mut T> {
        Spanned {
            inner: &mut self.inner,
            span: self.span,
        }
    }

    /// Replace the span.
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
