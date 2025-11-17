//! Tagged union pattern detection (DECY-080).
//!
//! Detects C tagged union patterns and extracts variant information.

use decy_hir::{HirStruct, HirType};

/// Information about a variant in a tagged union.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantInfo {
    /// Name of the variant (union field name)
    pub name: String,
    /// Type of the variant payload
    pub payload_type: HirType,
}

/// Information about a detected tagged union.
#[derive(Debug, Clone, PartialEq)]
pub struct TaggedUnionInfo {
    /// Name of the struct
    pub struct_name: String,
    /// Name of the tag field (enum)
    pub tag_field_name: String,
    /// Name of the union field
    pub union_field_name: String,
    /// List of variants
    pub variants: Vec<VariantInfo>,
}

/// Analyzes structs to detect tagged union patterns.
pub struct TaggedUnionAnalyzer;

impl TaggedUnionAnalyzer {
    /// Create a new tagged union analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Analyze a struct to detect if it's a tagged union pattern.
    ///
    /// Returns `Some(TaggedUnionInfo)` if the struct matches the pattern:
    /// - Has at least one enum field (tag)
    /// - Has at least one union field (data)
    /// - Union is non-empty
    ///
    /// Returns `None` if the struct doesn't match the pattern.
    pub fn analyze_struct(&self, _struct_def: &HirStruct) -> Option<TaggedUnionInfo> {
        // RED phase: Stub implementation
        todo!("DECY-080: Implement tagged union detection")
    }
}

impl Default for TaggedUnionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
