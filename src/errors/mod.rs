pub(crate) mod runtime_errors;
use crate::well_known as wk;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
    pub offset: usize,
}

impl Span {
    pub fn new(line: usize, col: usize, offset: usize) -> Self {
        Self { line, col, offset }
    }

    pub fn unknown() -> Self {
        Self {
            line: 0,
            col: 0,
            offset: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Error {
    pub kind: ErrorKind,
    pub span: Option<Span>,
    pub file: Option<String>,
    pub backtrace: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    ParseError(String),
    TypeError(String),
    ReferenceError(String),
    SyntaxError(String),
    RuntimeError(String),
    InternalError(String),
}

// Macro for generating backward-compatible error type constructors
macro_rules! define_error_constructor {
    ($name:ident, $variant:ident) => {
        pub fn $name(msg: String) -> Self {
            Self {
                kind: ErrorKind::$variant(msg),
                span: None,
                file: None,
                backtrace: None,
            }
        }
    };
}

// Backward-compatible enum-style constructors
#[allow(non_snake_case)]
impl Error {
    define_error_constructor!(ParseError, ParseError);
    define_error_constructor!(TypeError, TypeError);
    define_error_constructor!(ReferenceError, ReferenceError);
    define_error_constructor!(SyntaxError, SyntaxError);
    define_error_constructor!(RuntimeError, RuntimeError);
    define_error_constructor!(InternalError, InternalError);

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        if self.file.is_none() {
            self.file = Some(file.into());
        }
        self
    }

    pub fn with_backtrace(mut self, backtrace: String) -> Self {
        self.backtrace = Some(backtrace);
        self
    }

    pub fn kind_name(&self) -> &str {
        self.as_str()
    }

    pub fn as_str(&self) -> &'static str {
        match &self.kind {
            ErrorKind::ParseError(_) => wk::PARSE_ERROR,
            ErrorKind::TypeError(_) => wk::TYPE_ERROR,
            ErrorKind::ReferenceError(_) => wk::REFERENCE_ERROR,
            ErrorKind::SyntaxError(_) => wk::SYNTAX_ERROR,
            ErrorKind::RuntimeError(_) => wk::RUNTIME_ERROR,
            ErrorKind::InternalError(_) => wk::INTERNAL_ERROR,
        }
    }

    pub fn message(&self) -> &str {
        match &self.kind {
            ErrorKind::ParseError(m) => m,
            ErrorKind::TypeError(m) => m,
            ErrorKind::ReferenceError(m) => m,
            ErrorKind::SyntaxError(m) => m,
            ErrorKind::RuntimeError(m) => m,
            ErrorKind::InternalError(m) => m,
        }
    }

    pub fn display(&self, source: Option<&str>) -> String {
        let mut out = String::new();
        let kind_name = self.kind_name();
        let msg = self.message();

        out.push_str(&format!("\x1B[31m{}: {}\x1B[0m\n", kind_name, msg));

        if let Some(span) = &self.span {
            if span.line > 0 {
                let source_code = self
                    .file
                    .as_ref()
                    .and_then(|f| std::fs::read_to_string(f).ok())
                    .or_else(|| source.map(|s| s.to_string()));
                if let Some(source_code) = source_code {
                    let lines: Vec<&str> = source_code.lines().collect();
                    if span.line > 0 && span.line <= lines.len() {
                        let file = self.file.as_deref().unwrap_or("<script>");
                        let context_start = span.line.saturating_sub(2).max(1);
                        let context_end = (span.line + 1).min(lines.len());
                        let gutter_width = format!("{}", context_end).len();

                        for i in context_start..=context_end {
                            let line_num = i;
                            let line_content = lines[line_num - 1];
                            out.push_str(&format!(
                                "\x1B[90m{:>width$} | {}\x1B[0m\n",
                                line_num,
                                line_content,
                                width = gutter_width
                            ));
                            if line_num == span.line {
                                let padding: String =
                                    " ".repeat(gutter_width + 3 + span.col.saturating_sub(1));
                                let line_content = lines[line_num - 1];
                                let caret_len = if line_content.len() > span.col.saturating_sub(1) {
                                    let remaining = &line_content[span.col.saturating_sub(1)..];
                                    let len = remaining
                                        .find(char::is_whitespace)
                                        .unwrap_or(remaining.len());
                                    if len == 0 {
                                        1
                                    } else {
                                        len
                                    }
                                } else {
                                    1
                                };
                                out.push_str(&format!(
                                    "{}\x1B[31m{}\x1B[0m\n",
                                    padding,
                                    "^".repeat(caret_len)
                                ));
                            }
                        }
                        out.push_str(&format!("\x1B[90m{}\x1B[0m\n", file));
                    }
                }
            }
        }

        if let Some(backtrace) = &self.backtrace {
            out.push_str(backtrace);
            out.push('\n');
        }

        out
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind_name(), self.message())?;
        if let Some(bt) = &self.backtrace {
            write!(f, "{}", bt)?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub fn parse_error(msg: impl Into<String>) -> Error {
    Error::ParseError(msg.into())
}

pub fn type_error(msg: impl Into<String>) -> Error {
    Error::TypeError(msg.into())
}

pub fn reference_error(msg: impl Into<String>) -> Error {
    Error::ReferenceError(msg.into())
}

pub fn runtime_error(msg: impl Into<String>) -> Error {
    Error::RuntimeError(msg.into())
}
