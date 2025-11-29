//! C-specific decision context

use crate::decisions::CDecisionCategory;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Context for C→Rust transpilation decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CDecisionContext {
    /// Original C construct (pointer, array, etc.)
    pub c_construct: CConstruct,

    /// Decision category
    pub category: CDecisionCategory,

    /// Source location in C
    pub c_span: Option<SourceSpan>,

    /// Target location in generated Rust
    pub rust_span: Option<SourceSpan>,

    /// Surrounding C code for pattern matching
    pub c_context: String,

    /// HIR node hash for deduplication
    pub hir_hash: u64,
}

/// Source span (file, line, column)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSpan {
    pub file: String,
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl fmt::Display for SourceSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.start_line, self.start_col)
    }
}

impl SourceSpan {
    /// Create a span for a single line
    pub fn line(file: impl Into<String>, line: usize) -> Self {
        Self {
            file: file.into(),
            start_line: line,
            start_col: 1,
            end_line: line,
            end_col: 1,
        }
    }

    /// Check if this span overlaps with another
    pub fn overlaps(&self, other: &Self) -> bool {
        self.file == other.file
            && self.start_line <= other.end_line
            && self.end_line >= other.start_line
    }
}

/// C language construct being transpiled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CConstruct {
    /// Raw pointer: *T
    RawPointer {
        is_const: bool,
        pointee: String,
    },
    /// Array: T[N] or T[]
    Array {
        element: String,
        size: Option<usize>,
    },
    /// String: char* or const char*
    String {
        is_const: bool,
    },
    /// Struct with potential pointer fields
    Struct {
        name: String,
        has_pointers: bool,
    },
    /// Union
    Union {
        name: String,
    },
    /// Function pointer
    FunctionPointer {
        signature: String,
    },
    /// void*
    VoidPointer,
}

impl fmt::Display for CConstruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RawPointer { is_const, pointee } => {
                if *is_const {
                    write!(f, "const {}*", pointee)
                } else {
                    write!(f, "{}*", pointee)
                }
            }
            Self::Array { element, size } => {
                if let Some(n) = size {
                    write!(f, "{}[{}]", element, n)
                } else {
                    write!(f, "{}[]", element)
                }
            }
            Self::String { is_const } => {
                if *is_const {
                    write!(f, "const char*")
                } else {
                    write!(f, "char*")
                }
            }
            Self::Struct { name, .. } => write!(f, "struct {}", name),
            Self::Union { name } => write!(f, "union {}", name),
            Self::FunctionPointer { signature } => write!(f, "(*)({})", signature),
            Self::VoidPointer => write!(f, "void*"),
        }
    }
}

/// Lifetime decision for generated Rust code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifetimeDecision {
    /// Lifetime can be elided
    Elided,
    /// Explicit lifetime annotation needed
    Explicit(String),
    /// Static lifetime
    Static,
    /// Bound to input lifetime
    InputBound(String),
}

impl CDecisionContext {
    /// Create a new context
    pub fn new(construct: CConstruct, category: CDecisionCategory) -> Self {
        Self {
            c_construct: construct,
            category,
            c_span: None,
            rust_span: None,
            c_context: String::new(),
            hir_hash: 0,
        }
    }

    /// Add C source span
    pub fn with_c_span(mut self, span: SourceSpan) -> Self {
        self.c_span = Some(span);
        self
    }

    /// Add surrounding context
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.c_context = context.into();
        self
    }

    /// Convert to context strings for pattern matching
    pub fn to_context_strings(&self) -> Vec<String> {
        vec![
            self.c_construct.to_string(),
            self.category.to_string(),
            self.c_context.clone(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_construct_display() {
        let ptr = CConstruct::RawPointer {
            is_const: true,
            pointee: "int".into(),
        };
        assert_eq!(ptr.to_string(), "const int*");

        let arr = CConstruct::Array {
            element: "char".into(),
            size: Some(256),
        };
        assert_eq!(arr.to_string(), "char[256]");
    }

    #[test]
    fn test_c_construct_all_variants() {
        // RawPointer mutable
        let ptr_mut = CConstruct::RawPointer {
            is_const: false,
            pointee: "float".into(),
        };
        assert_eq!(ptr_mut.to_string(), "float*");

        // Array without size
        let arr_unsized = CConstruct::Array {
            element: "int".into(),
            size: None,
        };
        assert_eq!(arr_unsized.to_string(), "int[]");

        // String const
        let str_const = CConstruct::String { is_const: true };
        assert_eq!(str_const.to_string(), "const char*");

        // String mutable
        let str_mut = CConstruct::String { is_const: false };
        assert_eq!(str_mut.to_string(), "char*");

        // Struct
        let s = CConstruct::Struct {
            name: "Node".into(),
            has_pointers: true,
        };
        assert_eq!(s.to_string(), "struct Node");

        // Union
        let u = CConstruct::Union {
            name: "Data".into(),
        };
        assert_eq!(u.to_string(), "union Data");

        // Function pointer
        let fp = CConstruct::FunctionPointer {
            signature: "int, int".into(),
        };
        assert_eq!(fp.to_string(), "(*)(int, int)");

        // Void pointer
        let vp = CConstruct::VoidPointer;
        assert_eq!(vp.to_string(), "void*");
    }

    #[test]
    fn test_source_span_overlap() {
        let span1 = SourceSpan::line("test.c", 10);
        let span2 = SourceSpan::line("test.c", 10);
        assert!(span1.overlaps(&span2));

        let span3 = SourceSpan::line("test.c", 20);
        assert!(!span1.overlaps(&span3));

        let span4 = SourceSpan::line("other.c", 10);
        assert!(!span1.overlaps(&span4));
    }

    #[test]
    fn test_source_span_display() {
        let span = SourceSpan::line("src/main.c", 42);
        assert_eq!(span.to_string(), "src/main.c:42:1");
    }

    #[test]
    fn test_source_span_multi_line() {
        let span = SourceSpan {
            file: "test.c".into(),
            start_line: 10,
            start_col: 5,
            end_line: 15,
            end_col: 10,
        };

        let single = SourceSpan::line("test.c", 12);
        assert!(span.overlaps(&single)); // 12 is within 10-15

        let outside = SourceSpan::line("test.c", 20);
        assert!(!span.overlaps(&outside));
    }

    #[test]
    fn test_context_strings() {
        let ctx = CDecisionContext::new(
            CConstruct::RawPointer {
                is_const: false,
                pointee: "int".into(),
            },
            CDecisionCategory::PointerOwnership,
        )
        .with_context("function argument");

        let strings = ctx.to_context_strings();
        assert_eq!(strings.len(), 3);
        assert!(strings[0].contains("int*"));
        assert!(strings[1].contains("pointer_ownership"));
    }

    #[test]
    fn test_context_with_c_span() {
        let ctx = CDecisionContext::new(
            CConstruct::VoidPointer,
            CDecisionCategory::RawPointerCast,
        )
        .with_c_span(SourceSpan::line("test.c", 100));

        assert!(ctx.c_span.is_some());
        assert_eq!(ctx.c_span.as_ref().unwrap().start_line, 100);
    }

    #[test]
    fn test_lifetime_decision_variants() {
        let elided = LifetimeDecision::Elided;
        assert!(matches!(elided, LifetimeDecision::Elided));

        let explicit = LifetimeDecision::Explicit("'a".into());
        assert!(matches!(explicit, LifetimeDecision::Explicit(_)));

        let static_lt = LifetimeDecision::Static;
        assert!(matches!(static_lt, LifetimeDecision::Static));

        let bound = LifetimeDecision::InputBound("'input".into());
        assert!(matches!(bound, LifetimeDecision::InputBound(_)));
    }

    #[test]
    fn test_context_default_values() {
        let ctx = CDecisionContext::new(
            CConstruct::VoidPointer,
            CDecisionCategory::UnsafeBlock,
        );

        assert!(ctx.c_span.is_none());
        assert!(ctx.rust_span.is_none());
        assert!(ctx.c_context.is_empty());
        assert_eq!(ctx.hir_hash, 0);
    }
}
