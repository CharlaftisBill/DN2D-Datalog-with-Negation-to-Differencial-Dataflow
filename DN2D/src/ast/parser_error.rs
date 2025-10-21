use crate::lexer::Span;


#[derive(Debug)]
pub struct ParserError {
    pub message: String,
    pub line_ref: String,
    pub span: Span,
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {

        let mut txt_pointer = String::new();
        if self.line_ref != "" { 
            txt_pointer.push_str("\n\n");
            txt_pointer.push_str(&self.line_ref);
            txt_pointer.push_str("\n");
            txt_pointer.push_str(&" ".repeat(self.span.start));
            txt_pointer.push_str(&"┗");
            txt_pointer.push_str(&"━".repeat(self.span.start - 2));
            txt_pointer.push_str(&"┛");
        }

        write!(
            f,
            "Parser Error at Line {}, Col {}-{}: {}.{}",
            self.span.line,
            self.span.start,
            self.span.end,
            self.message,
            txt_pointer
        )
    }
}
impl std::error::Error for ParserError {}