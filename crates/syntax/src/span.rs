use miette::SourceSpan;
use std::ops::Range;

pub type SourceId = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub src: SourceId,
    pub range: Range<usize>,
}

impl Default for Span {
    fn default() -> Self {
        Self {
            src: 0,
            range: 0..0,
        }
    }
}

impl Span {
    #[inline]
    pub fn new(src: SourceId, range: Range<usize>) -> Self {
        Self { src, range }
    }

    #[inline]
    pub fn start(&self) -> usize {
        self.range.start
    }

    #[inline]
    pub fn end(&self) -> usize {
        self.range.end
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.range.end - self.range.start
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.range.is_empty()
    }

    /// Merge two spans that originate from the same source.
    #[inline]
    pub fn merge(self, other: Self) -> Self {
        debug_assert_eq!(
            self.src, other.src,
            "cannot merge spans from different sources"
        );

        Self {
            src: self.src,
            range: self.range.start.min(other.range.start)..self.range.end.max(other.range.end),
        }
    }
}

impl From<Span> for SourceSpan {
    #[inline]
    fn from(span: Span) -> Self {
        SourceSpan::new(span.range.start.into(), span.range.end - span.range.start)
    }
}

pub type Spanned<T> = (T, Span);

pub trait SpannedExt {
    fn span(&self) -> Span;
}

impl<T> SpannedExt for Spanned<T> {
    #[inline]
    fn span(&self) -> Span {
        self.1.clone()
    }
}
