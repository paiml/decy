//! Enum generation from tagged unions (DECY-081).
//!
//! Transforms C tagged unions into type-safe Rust enums with pattern matching.
//!
//! # Overview
//!
//! This module generates idiomatic Rust `enum` definitions from C tagged union patterns.
//! It takes the tagged union metadata extracted by the analyzer and produces clean,
//! type-safe Rust code that eliminates the unsafe union access pattern.
//!
//! # C Tagged Union Pattern
//!
//! C code often uses the "tagged union" pattern for variant types:
//!
//! ```c
//! enum ValueType { TYPE_INT, TYPE_FLOAT, TYPE_STRING };
//!
//! struct Value {
//!     enum ValueType tag;  // Discriminant
//!     union {              // Payload
//!         int int_value;
//!         float float_value;
//!         char* string_value;
//!     } data;
//! };
//! ```
//!
//! This is unsafe because:
//! - The compiler doesn't verify tag matches union field access
//! - Reading wrong union field causes undefined behavior
//! - No exhaustiveness checking for tag values
//!
//! # Rust Enum Transformation
//!
//! This module transforms the unsafe C pattern into safe Rust:
//!
//! ```rust
//! #[derive(Debug, Clone, PartialEq)]
//! pub enum Value {
//!     Int(i32),
//!     Float(f32),
//!     String(String),
//! }
//! ```
//!
//! Benefits:
//! - Type-safe: Compiler ensures tag matches payload
//! - Exhaustive: Pattern matching requires all variants
//! - Zero unsafe code in generated output
//!
//! # Example
//!
//! ```no_run
//! use decy_analyzer::tagged_union_analysis::TaggedUnionAnalyzer;
//! use decy_codegen::enum_gen::EnumGenerator;
//! use decy_hir::{HirStruct, HirStructField, HirType};
//!
//! // C: struct Value { enum Tag tag; union { int i; float f; } data; };
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
//! // Analyze tagged union
//! let analyzer = TaggedUnionAnalyzer::new();
//! let info = analyzer.analyze_struct(&struct_def).unwrap();
//!
//! // Generate Rust enum
//! let generator = EnumGenerator::new();
//! let rust_enum = generator.generate_enum(&info);
//!
//! // Result:
//! // #[derive(Debug, Clone, PartialEq)]
//! // pub enum Value {
//! //     Int(i32),
//! //     Float(f32),
//! // }
//! ```
//!
//! # Variant Naming
//!
//! The generator produces PascalCase variant names from C union field names:
//!
//! - Short names (≤2 chars): Derived from type (e.g., `i` with `int` → `Int`)
//! - Long names: Converted to PascalCase (e.g., `int_value` → `IntValue`)
//! - Type-based fallback: When field name is non-descriptive
//!
//! # Type Mapping
//!
//! C types are mapped to safe Rust equivalents:
//!
//! | C Type       | Rust Type  |
//! |-------------|-----------|
//! | `int`       | `i32`     |
//! | `float`     | `f32`     |
//! | `double`    | `f64`     |
//! | `char`      | `u8`      |
//! | `char*`     | `String`  |
//! | `void`      | `()`      |
//!
//! # Quality Guarantees
//!
//! - ✅ Zero unsafe code in generated output
//! - ✅ All variants derive Debug, Clone, PartialEq
//! - ✅ Public visibility for API usage
//! - ✅ Valid Rust syntax (parseable by rustc)
//! - ✅ Exhaustive pattern matching enforced

use decy_analyzer::tagged_union_analysis::TaggedUnionInfo;
use decy_hir::HirType;

/// Generator for Rust enums from C tagged unions.
pub struct EnumGenerator;

impl EnumGenerator {
    /// Create a new enum generator.
    pub fn new() -> Self {
        Self
    }

    /// Generate a Rust enum from tagged union info.
    ///
    /// # Arguments
    ///
    /// * `info` - Tagged union information from analysis
    ///
    /// # Returns
    ///
    /// Rust enum definition as a string
    pub fn generate_enum(&self, info: &TaggedUnionInfo) -> String {
        let mut result = String::new();

        // Add derive macros
        result.push_str("#[derive(Debug, Clone, PartialEq)]\n");

        // Add enum declaration
        result.push_str(&format!("pub enum {} {{\n", info.struct_name));

        // Generate variants
        for (idx, variant) in info.variants.iter().enumerate() {
            let variant_name = Self::capitalize_variant_name(&variant.name, &variant.payload_type);
            let variant_type = Self::map_hir_type_to_rust(&variant.payload_type);

            // Handle void types as unit variants
            if matches!(variant.payload_type, HirType::Void) {
                result.push_str(&format!("    {},\n", variant_name));
            } else {
                result.push_str(&format!("    {}({})", variant_name, variant_type));
                if idx < info.variants.len() - 1 {
                    result.push_str(",\n");
                } else {
                    result.push('\n');
                }
            }
        }

        result.push('}');
        result
    }

    /// Capitalize and clean up variant name to PascalCase.
    ///
    /// For short names (<=2 chars), derives a better name from the type.
    /// For longer names, converts to PascalCase.
    fn capitalize_variant_name(name: &str, payload_type: &HirType) -> String {
        // For very short names, derive from type
        if name.len() <= 2 {
            return Self::type_based_variant_name(payload_type);
        }

        // Split by underscore and capitalize each part
        let parts: Vec<String> = name
            .split('_')
            .filter(|s| !s.is_empty())
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect();

        if parts.is_empty() {
            // Fallback: derive from type
            Self::type_based_variant_name(payload_type)
        } else {
            parts.join("")
        }
    }

    /// Generate a variant name based on the payload type.
    fn type_based_variant_name(payload_type: &HirType) -> String {
        match payload_type {
            HirType::Void => "None".to_string(),
            HirType::Int => "Int".to_string(),
            HirType::UnsignedInt => "UInt".to_string(), // DECY-158
            HirType::Float => "Float".to_string(),
            HirType::Double => "Double".to_string(),
            HirType::Char => "Char".to_string(),
            HirType::Pointer(inner) if matches!(**inner, HirType::Char) => "String".to_string(),
            HirType::Pointer(inner) if matches!(**inner, HirType::Void) => "Pointer".to_string(),
            HirType::Pointer(_) => "Pointer".to_string(),
            HirType::Box(_) => "Boxed".to_string(),
            HirType::Vec(_) => "Vec".to_string(),
            HirType::Option(_) => "Option".to_string(),
            HirType::Reference { .. } => "Ref".to_string(),
            HirType::Struct(name) => name.clone(),
            HirType::Enum(name) => name.clone(),
            HirType::Union(_) => "Union".to_string(),
            HirType::Array { .. } => "Array".to_string(),
            HirType::FunctionPointer { .. } => "Function".to_string(),
            HirType::StringLiteral | HirType::OwnedString | HirType::StringReference => {
                "String".to_string()
            }
        }
    }

    /// Map HIR type to Rust type string.
    fn map_hir_type_to_rust(hir_type: &HirType) -> String {
        match hir_type {
            HirType::Void => "()".to_string(),
            HirType::Int => "i32".to_string(),
            HirType::UnsignedInt => "u32".to_string(), // DECY-158
            HirType::Float => "f32".to_string(),
            HirType::Double => "f64".to_string(),
            HirType::Char => "u8".to_string(),
            HirType::Pointer(inner) => {
                // Check if it's char* which should be String
                if matches!(**inner, HirType::Char) {
                    "String".to_string()
                } else if matches!(**inner, HirType::Void) {
                    "*mut ()".to_string()
                } else {
                    format!("*mut {}", Self::map_hir_type_to_rust(inner))
                }
            }
            HirType::Box(inner) => format!("Box<{}>", Self::map_hir_type_to_rust(inner)),
            HirType::Vec(inner) => format!("Vec<{}>", Self::map_hir_type_to_rust(inner)),
            HirType::Option(inner) => format!("Option<{}>", Self::map_hir_type_to_rust(inner)),
            HirType::Reference { inner, mutable } => {
                if *mutable {
                    format!("&mut {}", Self::map_hir_type_to_rust(inner))
                } else {
                    format!("&{}", Self::map_hir_type_to_rust(inner))
                }
            }
            HirType::Struct(name) => name.clone(),
            HirType::Enum(name) => name.clone(),
            HirType::Union(_) => "/* Union */".to_string(),
            HirType::Array { element_type, size } => {
                if let Some(n) = size {
                    format!("[{}; {}]", Self::map_hir_type_to_rust(element_type), n)
                } else {
                    format!("[{}]", Self::map_hir_type_to_rust(element_type))
                }
            }
            HirType::FunctionPointer {
                param_types,
                return_type,
            } => {
                let params: Vec<String> =
                    param_types.iter().map(Self::map_hir_type_to_rust).collect();
                let params_str = params.join(", ");
                if matches!(**return_type, HirType::Void) {
                    format!("fn({})", params_str)
                } else {
                    format!(
                        "fn({}) -> {}",
                        params_str,
                        Self::map_hir_type_to_rust(return_type)
                    )
                }
            }
            HirType::StringLiteral => "&str".to_string(),
            HirType::OwnedString => "String".to_string(),
            HirType::StringReference => "&str".to_string(),
        }
    }
}

impl Default for EnumGenerator {
    fn default() -> Self {
        Self::new()
    }
}
