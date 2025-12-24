use std::ops::Range;

pub type SourceId = usize;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Span {
    src: SourceId,
    pub range: Range<usize>,
}

impl Span {
    pub fn new(src: SourceId, start: usize, end: usize) -> Self {
        Self {
            src,
            range: start..end,
        }
    }

    pub fn len(&self) -> usize {
        self.range.end - self.range.start
    }

    pub fn is_empty(&self) -> bool {
        self.range.start == self.range.end
    }
}

impl ariadne::Span for Span {
    type SourceId = SourceId;

    fn source(&self) -> &Self::SourceId {
        &self.src
    }

    fn start(&self) -> usize {
        self.range.start
    }

    fn end(&self) -> usize {
        self.range.end
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

pub type Spanned<T> = (T, Span);
