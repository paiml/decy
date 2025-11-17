//! Tagged union pattern detection (DECY-080).
//!
//! Detects C tagged union patterns and extracts variant information.
//!
//! # Overview
//!
//! This module analyzes C structs to detect the "tagged union" pattern, where a struct
//! combines an enum discriminant (tag) with a union containing variant data. This is a
//! common C idiom for creating variant types, but it's unsafe because the compiler doesn't
//! verify that the tag matches the union field being accessed.
//!
//! # Tagged Union Pattern
//!
//! A typical C tagged union looks like:
//!
//! ```c
//! enum ValueType { TYPE_INT, TYPE_FLOAT, TYPE_STRING };
//!
//! struct Value {
//!     enum ValueType tag;  // Discriminant
//!     union {              // Payload
//!         int int_val;
//!         float float_val;
//!         char* string_val;
//!     } data;
//! };
//! ```
//!
//! This module detects such patterns and extracts the tag field, union field, and variant
//! information, enabling transformation to Rust's type-safe `enum` with pattern matching.
//!
//! # Example
//!
//! ```no_run
//! use decy_analyzer::tagged_union_analysis::TaggedUnionAnalyzer;
//! use decy_hir::{HirStruct, HirStructField, HirType};
//!
//! // C struct: struct Value { enum Tag tag; union { int i; float f; } data; };
//! let struct_def = HirStruct::new(
//!     "Value".to_string(),
//!     vec![
//!         HirStructField::new("tag".to_string(), HirType::Enum("Tag".to_string())),
//!         HirStructField::new("data".to_string(), HirType::Union(vec![
//!             ("i".to_string(), HirType::Int),
//!             ("f".to_string(), HirType::Float),
//!         ])),
//!     ],
//! );
//!
//! let analyzer = TaggedUnionAnalyzer::new();
//! let info = analyzer.analyze_struct(&struct_def);
//!
//! assert!(info.is_some());
//! let info = info.unwrap();
//! assert_eq!(info.struct_name, "Value");
//! assert_eq!(info.tag_field_name, "tag");
//! assert_eq!(info.union_field_name, "data");
//! assert_eq!(info.variants.len(), 2);
//! ```
//!
//! # Algorithm
//!
//! The detection algorithm:
//!
//! 1. Scan struct fields for the first enum type (tag discriminant)
//! 2. Scan struct fields for the first union type (variant payload)
//! 3. If both exist and the union is non-empty, extract variant metadata
//! 4. Return `TaggedUnionInfo` with complete information for code generation
//!
//! Empty unions are rejected because they represent invalid tagged unions.

use decy_hir::{HirStruct, HirType};

/// Information about a variant in a tagged union.
///
/// Each variant corresponds to a field in the C union, representing one possible
/// type that the tagged union can hold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantInfo {
    /// Name of the variant (union field name).
    ///
    /// Example: `"int_val"` for `union { int int_val; float float_val; }`
    pub name: String,

    /// Type of the variant payload.
    ///
    /// Example: `HirType::Int` for the `int int_val` field
    pub payload_type: HirType,
}

/// Information about a detected tagged union.
///
/// Contains all metadata needed to transform a C tagged union into a Rust enum.
#[derive(Debug, Clone, PartialEq)]
pub struct TaggedUnionInfo {
    /// Name of the struct.
    ///
    /// Example: `"Value"` for `struct Value { ... }`
    pub struct_name: String,

    /// Name of the tag field (enum discriminant).
    ///
    /// Example: `"tag"` for `enum Tag tag;`
    pub tag_field_name: String,

    /// Name of the union field (variant payload).
    ///
    /// Example: `"data"` for `union { ... } data;`
    pub union_field_name: String,

    /// List of variants extracted from the union.
    ///
    /// Each variant represents one possible type/value for the tagged union.
    pub variants: Vec<VariantInfo>,
}

/// Analyzes structs to detect tagged union patterns.
///
/// This analyzer identifies C structs that follow the tagged union idiom and extracts
/// the necessary metadata for safe transformation to Rust enums.
pub struct TaggedUnionAnalyzer;

impl TaggedUnionAnalyzer {
    /// Create a new tagged union analyzer.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_analyzer::tagged_union_analysis::TaggedUnionAnalyzer;
    ///
    /// let analyzer = TaggedUnionAnalyzer::new();
    /// ```
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
    pub fn analyze_struct(&self, struct_def: &HirStruct) -> Option<TaggedUnionInfo> {
        let fields = struct_def.fields();

        // Find the first enum field (tag)
        let tag_field = fields
            .iter()
            .find(|f| matches!(f.field_type(), HirType::Enum(_)))?;

        // Find the first union field (data)
        let union_field = fields
            .iter()
            .find(|f| matches!(f.field_type(), HirType::Union(_)))?;

        // Extract union variants
        if let HirType::Union(variants) = union_field.field_type() {
            // Empty union is not a valid tagged union
            if variants.is_empty() {
                return None;
            }

            // Build variant info
            let variant_infos: Vec<VariantInfo> = variants
                .iter()
                .map(|(name, payload_type)| VariantInfo {
                    name: name.clone(),
                    payload_type: payload_type.clone(),
                })
                .collect();

            Some(TaggedUnionInfo {
                struct_name: struct_def.name().to_string(),
                tag_field_name: tag_field.name().to_string(),
                union_field_name: union_field.name().to_string(),
                variants: variant_infos,
            })
        } else {
            None
        }
    }
}

impl Default for TaggedUnionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
