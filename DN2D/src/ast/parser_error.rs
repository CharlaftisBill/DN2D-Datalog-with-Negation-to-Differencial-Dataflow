use crate::lexer::Span;


#[derive(Debug)]
pub struct ParserError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Parser Error at Line {}, Col {}-{}: {}",
            self.span.line, self.span.start, self.span.end, self.message
        )
    }
}
impl std::error::Error for ParserError {}