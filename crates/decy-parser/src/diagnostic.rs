//! Structured diagnostic reporting for C parse errors.
//!
//! Provides rustc-style error messages with source locations, code snippets,
//! explanatory notes, and actionable fix suggestions extracted from clang.

use colored::Colorize;
use std::fmt;

/// Severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Informational note
    Note,
    /// Compiler warning
    Warning,
    /// Compiler error (blocks compilation)
    Error,
    /// Fatal error (stops processing)
    Fatal,
}

impl Severity {
    /// Returns the display tag for this severity level.
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Fatal => "fatal",
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tag())
    }
}

/// Category of the error for the `error[category]` display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Parse/syntax error
    Parse,
    /// Type error
    Type,
    /// Semantic error
    Semantic,
    /// I/O error
    Io,
    /// Transpilation error
    Transpile,
    /// Internal error
    Internal,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse => write!(f, "parse"),
            Self::Type => write!(f, "type"),
            Self::Semantic => write!(f, "semantic"),
            Self::Io => write!(f, "io"),
            Self::Transpile => write!(f, "transpile"),
            Self::Internal => write!(f, "internal"),
        }
    }
}

/// A structured diagnostic extracted from the C parser (clang-sys).
///
/// Contains source location, code snippet, categorized explanation, and
/// actionable fix suggestion, formatted in rustc-style output.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Severity: Note, Warning, Error, Fatal
    pub severity: Severity,
    /// Error message from clang (e.g. "expected ')'")
    pub message: String,
    /// Source file path
    pub file: Option<String>,
    /// 1-based line number
    pub line: Option<u32>,
    /// 1-based column number
    pub column: Option<u32>,
    /// Clang diagnostic category (e.g. "Parse Issue")
    pub category: Option<String>,
    /// Suggested fixes from clang fix-its
    pub fix_its: Vec<String>,
    /// 3-line code snippet with caret indicator
    pub snippet: Option<String>,
    /// Explanatory note (WHY it failed)
    pub note: Option<String>,
    /// Actionable help (HOW to fix)
    pub help: Option<String>,
}

impl Diagnostic {
    /// Create a new diagnostic with the given severity and message.
    pub fn new(severity: Severity, message: impl Into<String>) -> Self {
        Self {
            severity,
            message: message.into(),
            file: None,
            line: None,
            column: None,
            category: None,
            fix_its: Vec::new(),
            snippet: None,
            note: None,
            help: None,
        }
    }

    /// Infer the error category from the clang category or message content.
    pub fn error_category(&self) -> ErrorCategory {
        if let Some(ref cat) = self.category {
            let cat_lower = cat.to_lowercase();
            if cat_lower.contains("parse") || cat_lower.contains("syntax") {
                return ErrorCategory::Parse;
            }
            if cat_lower.contains("type") {
                return ErrorCategory::Type;
            }
            if cat_lower.contains("semantic") {
                return ErrorCategory::Semantic;
            }
        }

        let msg = self.message.to_lowercase();
        if msg.contains("expected") || msg.contains("unterminated") || msg.contains("extraneous") {
            ErrorCategory::Parse
        } else if msg.contains("incompatible") || msg.contains("implicit conversion") {
            ErrorCategory::Type
        } else if msg.contains("undeclared") || msg.contains("redefinition") {
            ErrorCategory::Semantic
        } else {
            ErrorCategory::Parse
        }
    }

    /// Build a 3-line code snippet with a caret pointing at the error column.
    pub fn build_snippet(source: &str, line: u32, column: Option<u32>) -> Option<String> {
        let lines: Vec<&str> = source.lines().collect();
        let line_idx = line.checked_sub(1)? as usize;
        if line_idx >= lines.len() {
            return None;
        }

        let mut out = String::new();
        let gutter_width = format!("{}", line_idx + 2).len().max(2);

        // Line before (context)
        if line_idx > 0 {
            out.push_str(&format!(
                "{:>width$}|    {}\n",
                line_idx,
                lines[line_idx - 1],
                width = gutter_width
            ));
        }

        // Error line
        out.push_str(&format!(
            "{:>width$}|    {}\n",
            line_idx + 1,
            lines[line_idx],
            width = gutter_width
        ));

        // Caret line
        if let Some(col) = column {
            let col_idx = (col as usize).saturating_sub(1);
            let padding = " ".repeat(col_idx);
            out.push_str(&format!(
                "{:>width$}|    {}^\n",
                "",
                padding,
                width = gutter_width
            ));
        }

        // Line after (context)
        if line_idx + 1 < lines.len() {
            out.push_str(&format!(
                "{:>width$}|    {}\n",
                line_idx + 2,
                lines[line_idx + 1],
                width = gutter_width
            ));
        }

        Some(out)
    }

    /// Populate `note` and `help` based on common C error patterns.
    pub fn infer_note_and_help(&mut self) {
        let msg = self.message.to_lowercase();

        if msg.contains("expected ')'") {
            self.note = Some("Unclosed parenthesis in expression or function call.".into());
            self.help = Some("Add the missing ')' to close the expression.".into());
        } else if msg.contains("expected ';'") {
            self.note = Some("Missing semicolon after statement.".into());
            self.help = Some("Add ';' at the end of the statement.".into());
        } else if msg.contains("use of undeclared identifier")
            || msg.contains("undeclared identifier")
        {
            self.note = Some("Variable or function not declared in this scope.".into());
            self.help =
                Some("Declare the variable before use, or check for typos in the name.".into());
        } else if msg.contains("implicit declaration of function") {
            self.note = Some("Function called before it is declared.".into());
            self.help = Some(
                "Add a #include for the header that declares this function, or add a forward declaration.".into(),
            );
        } else if msg.contains("incompatible") && msg.contains("type") {
            self.note = Some("Type mismatch in assignment or return value.".into());
            self.help =
                Some("Ensure the types match, or add an explicit cast if intentional.".into());
        } else if msg.contains("expected '}'") {
            self.note = Some("Unclosed brace in block or struct definition.".into());
            self.help = Some("Add the missing '}' to close the block.".into());
        } else if msg.contains("redefinition of") {
            self.note = Some("This name was already defined earlier in the same scope.".into());
            self.help = Some("Rename one of the definitions, or use a different scope.".into());
        } else if msg.contains("expected expression") {
            self.note =
                Some("The parser expected a value or expression but found something else.".into());
            self.help = Some("Check for missing operands or misplaced punctuation.".into());
        } else if !self.fix_its.is_empty() {
            self.help = Some(self.fix_its.join("; "));
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Line 1: error[category]: message
        let category = self.error_category();
        let header = format!("{}[{}]", self.severity.tag(), category);
        let colored_header = match self.severity {
            Severity::Error | Severity::Fatal => header.red().bold().to_string(),
            Severity::Warning => header.yellow().bold().to_string(),
            Severity::Note => header.cyan().bold().to_string(),
        };
        writeln!(f, "{}: {}", colored_header, self.message.bold())?;

        // Line 2: --> file:line:col
        if let Some(ref file) = self.file {
            let loc = match (self.line, self.column) {
                (Some(l), Some(c)) => format!("{}:{}:{}", file, l, c),
                (Some(l), None) => format!("{}:{}", file, l),
                _ => file.clone(),
            };
            writeln!(f, " {} {}", "-->".blue().bold(), loc)?;
        }

        // Code snippet
        if let Some(ref snippet) = self.snippet {
            // Color the gutter (pipe characters) blue
            for line in snippet.lines() {
                if let Some(pipe_pos) = line.find('|') {
                    let gutter = &line[..=pipe_pos];
                    let rest = &line[pipe_pos + 1..];
                    if rest.trim() == "^" || rest.contains('^') {
                        // Caret line: gutter blue, caret red
                        writeln!(f, " {}{}", gutter.blue(), rest.red())?;
                    } else {
                        writeln!(f, " {}{}", gutter.blue(), rest)?;
                    }
                } else {
                    writeln!(f, " {}", line)?;
                }
            }
        }

        // note: explanation
        if let Some(ref note) = self.note {
            writeln!(f, "  {}: {}", "note".cyan().bold(), note)?;
        }

        // help: suggestion
        if let Some(ref help) = self.help {
            writeln!(f, "  {}: {}", "help".green().bold(), help)?;
        }

        Ok(())
    }
}

/// Error wrapping one or more diagnostics from the C parser.
///
/// Implements `std::error::Error` so it propagates through `anyhow::Result` chains
/// and can be downcast in `main.rs` for rich formatting.
#[derive(Debug)]
pub struct DiagnosticError {
    /// All diagnostics collected from the parse attempt
    pub diagnostics: Vec<Diagnostic>,
}

impl DiagnosticError {
    /// Create a new `DiagnosticError` with the given diagnostics.
    pub fn new(diagnostics: Vec<Diagnostic>) -> Self {
        Self { diagnostics }
    }
}

impl fmt::Display for DiagnosticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for diag in &self.diagnostics {
            write!(f, "{}", diag)?;
        }
        let error_count = self
            .diagnostics
            .iter()
            .filter(|d| d.severity >= Severity::Error)
            .count();
        if error_count > 0 {
            write!(
                f,
                "{}",
                format!(
                    "aborting due to {} previous error{}",
                    error_count,
                    if error_count == 1 { "" } else { "s" }
                )
                .red()
                .bold()
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for DiagnosticError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Note < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Error < Severity::Fatal);
    }

    #[test]
    fn test_severity_tags() {
        assert_eq!(Severity::Note.tag(), "note");
        assert_eq!(Severity::Warning.tag(), "warning");
        assert_eq!(Severity::Error.tag(), "error");
        assert_eq!(Severity::Fatal.tag(), "fatal");
    }

    #[test]
    fn test_error_category_from_clang_category() {
        let mut d = Diagnostic::new(Severity::Error, "expected ')'");
        d.category = Some("Parse Issue".into());
        assert_eq!(d.error_category(), ErrorCategory::Parse);

        d.category = Some("Semantic Issue".into());
        assert_eq!(d.error_category(), ErrorCategory::Semantic);
    }

    #[test]
    fn test_error_category_from_message() {
        let d = Diagnostic::new(Severity::Error, "expected ';' after expression");
        assert_eq!(d.error_category(), ErrorCategory::Parse);

        let d = Diagnostic::new(Severity::Error, "use of undeclared identifier 'x'");
        assert_eq!(d.error_category(), ErrorCategory::Semantic);

        let d = Diagnostic::new(Severity::Error, "incompatible pointer types");
        assert_eq!(d.error_category(), ErrorCategory::Type);
    }

    #[test]
    fn test_display_contains_header() {
        let d = Diagnostic::new(Severity::Error, "expected ')'");
        let output = format!("{}", d);
        assert!(output.contains("error[parse]"));
        assert!(output.contains("expected ')'"));
    }

    #[test]
    fn test_display_contains_location() {
        let mut d = Diagnostic::new(Severity::Error, "expected ')'");
        d.file = Some("test.c".into());
        d.line = Some(15);
        d.column = Some(22);
        let output = format!("{}", d);
        assert!(output.contains("-->"));
        assert!(output.contains("test.c:15:22"));
    }

    #[test]
    fn test_display_contains_snippet() {
        let mut d = Diagnostic::new(Severity::Error, "expected ')'");
        d.snippet = Some("14|    int y = 10;\n15|    int x = foo(bar;\n  |                    ^\n16|    return 0;\n".into());
        let output = format!("{}", d);
        assert!(output.contains("|"));
        assert!(output.contains("^"));
    }

    #[test]
    fn test_display_contains_note_and_help() {
        let mut d = Diagnostic::new(Severity::Error, "expected ')'");
        d.note = Some("Unclosed parenthesis.".into());
        d.help = Some("Add ')' to close.".into());
        let output = format!("{}", d);
        assert!(output.contains("note:"));
        assert!(output.contains("help:"));
        assert!(output.contains("Unclosed parenthesis."));
        assert!(output.contains("Add ')' to close."));
    }

    #[test]
    fn test_build_snippet_middle_of_file() {
        let source = "line 1\nline 2\nline 3\nline 4\nline 5";
        let snippet = Diagnostic::build_snippet(source, 3, Some(4)).unwrap();
        assert!(snippet.contains("line 2")); // context before
        assert!(snippet.contains("line 3")); // error line
        assert!(snippet.contains("line 4")); // context after
        assert!(snippet.contains("^")); // caret
    }

    #[test]
    fn test_build_snippet_first_line() {
        let source = "int x = foo(bar;\nint y = 10;";
        let snippet = Diagnostic::build_snippet(source, 1, Some(16)).unwrap();
        assert!(snippet.contains("int x = foo(bar;"));
        assert!(snippet.contains("^"));
        // No line before
    }

    #[test]
    fn test_build_snippet_last_line() {
        let source = "int y = 10;\nint x = foo(bar;";
        let snippet = Diagnostic::build_snippet(source, 2, Some(16)).unwrap();
        assert!(snippet.contains("int x = foo(bar;"));
        assert!(snippet.contains("^"));
        // No line after
    }

    #[test]
    fn test_build_snippet_no_column() {
        let source = "line 1\nline 2\nline 3";
        let snippet = Diagnostic::build_snippet(source, 2, None).unwrap();
        assert!(snippet.contains("line 2"));
        assert!(!snippet.contains("^")); // No caret without column
    }

    #[test]
    fn test_build_snippet_out_of_bounds() {
        let source = "line 1\nline 2";
        assert!(Diagnostic::build_snippet(source, 99, Some(1)).is_none());
        assert!(Diagnostic::build_snippet(source, 0, Some(1)).is_none());
    }

    #[test]
    fn test_infer_note_expected_paren() {
        let mut d = Diagnostic::new(Severity::Error, "expected ')'");
        d.infer_note_and_help();
        assert!(d.note.is_some());
        assert!(d.help.is_some());
        assert!(d.note.unwrap().contains("parenthesis"));
    }

    #[test]
    fn test_infer_note_expected_semicolon() {
        let mut d = Diagnostic::new(Severity::Error, "expected ';' after expression");
        d.infer_note_and_help();
        assert!(d.note.unwrap().contains("semicolon"));
    }

    #[test]
    fn test_infer_note_undeclared() {
        let mut d = Diagnostic::new(Severity::Error, "use of undeclared identifier 'foo'");
        d.infer_note_and_help();
        assert!(d.note.unwrap().contains("not declared"));
    }

    #[test]
    fn test_infer_note_falls_back_to_fix_its() {
        let mut d = Diagnostic::new(Severity::Error, "some unusual error");
        d.fix_its = vec!["insert ';'".into()];
        d.infer_note_and_help();
        assert!(d.help.unwrap().contains("insert ';'"));
    }

    #[test]
    fn test_diagnostic_error_display() {
        let d1 = Diagnostic::new(Severity::Error, "first error");
        let d2 = Diagnostic::new(Severity::Error, "second error");
        let err = DiagnosticError::new(vec![d1, d2]);
        let output = format!("{}", err);
        assert!(output.contains("first error"));
        assert!(output.contains("second error"));
        assert!(output.contains("2 previous errors"));
    }

    #[test]
    fn test_diagnostic_error_single() {
        let d = Diagnostic::new(Severity::Error, "only error");
        let err = DiagnosticError::new(vec![d]);
        let output = format!("{}", err);
        assert!(output.contains("1 previous error"));
        // No plural "s"
        assert!(!output.contains("1 previous errors"));
    }

    #[test]
    fn test_diagnostic_error_is_std_error() {
        let err: Box<dyn std::error::Error> =
            Box::new(DiagnosticError::new(vec![Diagnostic::new(
                Severity::Error,
                "test",
            )]));
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_warning_severity_display() {
        let d = Diagnostic::new(Severity::Warning, "implicit conversion loses precision");
        let output = format!("{}", d);
        assert!(output.contains("warning"));
    }
}
