#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(line: usize, start: usize, end: usize) -> Self {
        Span { line, start, end }
    }
}
