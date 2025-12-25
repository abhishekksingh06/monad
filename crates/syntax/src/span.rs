use std::ops::Range;

use miette::SourceSpan;

pub type SourceId = usize;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Span {
    pub src: SourceId,
    pub range: Range<usize>,
}

impl Span {
    pub fn new(src: SourceId, range: Range<usize>) -> Self {
        Self { src, range }
    }

    pub fn len(&self) -> usize {
        self.range.end - self.range.start
    }

    pub fn is_empty(&self) -> bool {
        self.range.start == self.range.end
    }
}

impl From<&Span> for SourceSpan {
    fn from(span: &Span) -> Self {
        SourceSpan::new(span.range.start.into(), span.range.end - span.range.start)
    }
}

impl chumsky::span::Span for Span {
    type Context = SourceId;

    type Offset = usize;

    fn new(context: Self::Context, range: Range<Self::Offset>) -> Self {
        Self {
            src: context,
            range,
        }
    }

    fn context(&self) -> Self::Context {
        self.src
    }

    fn start(&self) -> Self::Offset {
        self.range.start
    }

    fn end(&self) -> Self::Offset {
        self.range.end
    }
}

pub type Spanned<T: PartialOrd> = (T, Span);
