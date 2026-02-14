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
            HirType::Bool => "Bool".to_string(),
            HirType::Int => "Int".to_string(),
            HirType::UnsignedInt => "UInt".to_string(), // DECY-158
            HirType::Float => "Float".to_string(),
            HirType::Double => "Double".to_string(),
            HirType::Char => "Char".to_string(),
            HirType::SignedChar => "SignedChar".to_string(), // DECY-250
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
            // DECY-172: Type aliases use the alias name as variant name
            HirType::TypeAlias(name) => name.clone(),
        }
    }

    /// Map HIR type to Rust type string.
    fn map_hir_type_to_rust(hir_type: &HirType) -> String {
        match hir_type {
            HirType::Void => "()".to_string(),
            HirType::Bool => "bool".to_string(),
            HirType::Int => "i32".to_string(),
            HirType::UnsignedInt => "u32".to_string(), // DECY-158
            HirType::Float => "f32".to_string(),
            HirType::Double => "f64".to_string(),
            HirType::Char => "u8".to_string(),
            HirType::SignedChar => "i8".to_string(), // DECY-250
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
            // DECY-172: Preserve typedef names
            HirType::TypeAlias(name) => name.clone(),
        }
    }
}

impl Default for EnumGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use decy_analyzer::tagged_union_analysis::{TaggedUnionInfo, VariantInfo};

    // ============================================================================
    // ENUM GENERATOR CONSTRUCTION TESTS
    // ============================================================================

    #[test]
    fn test_enum_generator_new() {
        let gen = EnumGenerator::new();
        // Generator is a unit struct, just verify construction
        assert!(std::mem::size_of_val(&gen) == 0);
    }

    #[test]
    fn test_enum_generator_default() {
        let gen: EnumGenerator = Default::default();
        assert!(std::mem::size_of_val(&gen) == 0);
    }

    // ============================================================================
    // ENUM GENERATION TESTS
    // ============================================================================

    #[test]
    fn test_generate_enum_single_variant() {
        let gen = EnumGenerator::new();
        let info = TaggedUnionInfo {
            struct_name: "Value".to_string(),
            tag_field_name: "tag".to_string(),
            union_field_name: "data".to_string(),
            variants: vec![VariantInfo {
                name: "int_value".to_string(),
                payload_type: HirType::Int,
            }],
        };

        let result = gen.generate_enum(&info);
        assert!(result.contains("#[derive(Debug, Clone, PartialEq)]"));
        assert!(result.contains("pub enum Value"));
        assert!(result.contains("IntValue(i32)"));
    }

    #[test]
    fn test_generate_enum_multiple_variants() {
        let gen = EnumGenerator::new();
        let info = TaggedUnionInfo {
            struct_name: "Value".to_string(),
            tag_field_name: "tag".to_string(),
            union_field_name: "data".to_string(),
            variants: vec![
                VariantInfo {
                    name: "int_value".to_string(),
                    payload_type: HirType::Int,
                },
                VariantInfo {
                    name: "float_value".to_string(),
                    payload_type: HirType::Float,
                },
                VariantInfo {
                    name: "double_value".to_string(),
                    payload_type: HirType::Double,
                },
            ],
        };

        let result = gen.generate_enum(&info);
        assert!(result.contains("IntValue(i32),"));
        assert!(result.contains("FloatValue(f32),"));
        assert!(result.contains("DoubleValue(f64)"));
    }

    #[test]
    fn test_generate_enum_void_variant() {
        let gen = EnumGenerator::new();
        let info = TaggedUnionInfo {
            struct_name: "Option".to_string(),
            tag_field_name: "tag".to_string(),
            union_field_name: "data".to_string(),
            variants: vec![
                VariantInfo {
                    name: "none".to_string(),
                    payload_type: HirType::Void,
                },
                VariantInfo {
                    name: "some_int".to_string(),
                    payload_type: HirType::Int,
                },
            ],
        };

        let result = gen.generate_enum(&info);
        // Void variants should be unit variants (no payload)
        assert!(result.contains("None,"));
        assert!(result.contains("SomeInt(i32)"));
    }

    #[test]
    fn test_generate_enum_last_variant_no_trailing_comma() {
        let gen = EnumGenerator::new();
        let info = TaggedUnionInfo {
            struct_name: "Test".to_string(),
            tag_field_name: "tag".to_string(),
            union_field_name: "data".to_string(),
            variants: vec![VariantInfo {
                name: "value".to_string(),
                payload_type: HirType::Int,
            }],
        };

        let result = gen.generate_enum(&info);
        // Last non-void variant should not have trailing comma
        assert!(result.contains("Value(i32)\n}"));
    }

    // ============================================================================
    // VARIANT NAME CAPITALIZATION TESTS
    // ============================================================================

    #[test]
    fn test_capitalize_variant_name_snake_case() {
        let result = EnumGenerator::capitalize_variant_name("int_value", &HirType::Int);
        assert_eq!(result, "IntValue");
    }

    #[test]
    fn test_capitalize_variant_name_short_name_uses_type() {
        // Short names (<=2 chars) should derive from type
        let result = EnumGenerator::capitalize_variant_name("i", &HirType::Int);
        assert_eq!(result, "Int");

        let result = EnumGenerator::capitalize_variant_name("f", &HirType::Float);
        assert_eq!(result, "Float");

        let result = EnumGenerator::capitalize_variant_name("d", &HirType::Double);
        assert_eq!(result, "Double");
    }

    #[test]
    fn test_capitalize_variant_name_two_char_name() {
        let result = EnumGenerator::capitalize_variant_name("id", &HirType::Int);
        assert_eq!(result, "Int"); // 2 chars is still "short"
    }

    #[test]
    fn test_capitalize_variant_name_three_char_name() {
        let result = EnumGenerator::capitalize_variant_name("val", &HirType::Int);
        assert_eq!(result, "Val"); // 3 chars is not short
    }

    #[test]
    fn test_capitalize_variant_name_empty_parts_fallback() {
        // Name with only underscores should fall back to type-based name
        let result = EnumGenerator::capitalize_variant_name("___", &HirType::Int);
        assert_eq!(result, "Int");
    }

    #[test]
    fn test_capitalize_variant_name_single_word() {
        let result = EnumGenerator::capitalize_variant_name("value", &HirType::Int);
        assert_eq!(result, "Value");
    }

    #[test]
    fn test_capitalize_variant_name_multiple_underscores() {
        let result = EnumGenerator::capitalize_variant_name("my__long__name", &HirType::Int);
        assert_eq!(result, "MyLongName");
    }

    // ============================================================================
    // TYPE-BASED VARIANT NAME TESTS
    // ============================================================================

    #[test]
    fn test_type_based_variant_name_primitives() {
        assert_eq!(
            EnumGenerator::type_based_variant_name(&HirType::Void),
            "None"
        );
        assert_eq!(EnumGenerator::type_based_variant_name(&HirType::Int), "Int");
        assert_eq!(
            EnumGenerator::type_based_variant_name(&HirType::UnsignedInt),
            "UInt"
        );
        assert_eq!(
            EnumGenerator::type_based_variant_name(&HirType::Float),
            "Float"
        );
        assert_eq!(
            EnumGenerator::type_based_variant_name(&HirType::Double),
            "Double"
        );
        assert_eq!(
            EnumGenerator::type_based_variant_name(&HirType::Char),
            "Char"
        );
    }

    #[test]
    fn test_type_based_variant_name_char_pointer_is_string() {
        let char_ptr = HirType::Pointer(Box::new(HirType::Char));
        assert_eq!(EnumGenerator::type_based_variant_name(&char_ptr), "String");
    }

    #[test]
    fn test_type_based_variant_name_void_pointer() {
        let void_ptr = HirType::Pointer(Box::new(HirType::Void));
        assert_eq!(EnumGenerator::type_based_variant_name(&void_ptr), "Pointer");
    }

    #[test]
    fn test_type_based_variant_name_other_pointer() {
        let int_ptr = HirType::Pointer(Box::new(HirType::Int));
        assert_eq!(EnumGenerator::type_based_variant_name(&int_ptr), "Pointer");
    }

    #[test]
    fn test_type_based_variant_name_box() {
        let boxed = HirType::Box(Box::new(HirType::Int));
        assert_eq!(EnumGenerator::type_based_variant_name(&boxed), "Boxed");
    }

    #[test]
    fn test_type_based_variant_name_vec() {
        let vec = HirType::Vec(Box::new(HirType::Int));
        assert_eq!(EnumGenerator::type_based_variant_name(&vec), "Vec");
    }

    #[test]
    fn test_type_based_variant_name_option() {
        let opt = HirType::Option(Box::new(HirType::Int));
        assert_eq!(EnumGenerator::type_based_variant_name(&opt), "Option");
    }

    #[test]
    fn test_type_based_variant_name_reference() {
        let ref_type = HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        };
        assert_eq!(EnumGenerator::type_based_variant_name(&ref_type), "Ref");
    }

    #[test]
    fn test_type_based_variant_name_struct() {
        let struct_type = HirType::Struct("MyStruct".to_string());
        assert_eq!(
            EnumGenerator::type_based_variant_name(&struct_type),
            "MyStruct"
        );
    }

    #[test]
    fn test_type_based_variant_name_enum() {
        let enum_type = HirType::Enum("MyEnum".to_string());
        assert_eq!(EnumGenerator::type_based_variant_name(&enum_type), "MyEnum");
    }

    #[test]
    fn test_type_based_variant_name_union() {
        let union_type = HirType::Union(vec![]);
        assert_eq!(EnumGenerator::type_based_variant_name(&union_type), "Union");
    }

    #[test]
    fn test_type_based_variant_name_array() {
        let array = HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        };
        assert_eq!(EnumGenerator::type_based_variant_name(&array), "Array");
    }

    #[test]
    fn test_type_based_variant_name_function_pointer() {
        let fn_ptr = HirType::FunctionPointer {
            param_types: vec![HirType::Int],
            return_type: Box::new(HirType::Void),
        };
        assert_eq!(EnumGenerator::type_based_variant_name(&fn_ptr), "Function");
    }

    #[test]
    fn test_type_based_variant_name_string_types() {
        assert_eq!(
            EnumGenerator::type_based_variant_name(&HirType::StringLiteral),
            "String"
        );
        assert_eq!(
            EnumGenerator::type_based_variant_name(&HirType::OwnedString),
            "String"
        );
        assert_eq!(
            EnumGenerator::type_based_variant_name(&HirType::StringReference),
            "String"
        );
    }

    #[test]
    fn test_type_based_variant_name_type_alias() {
        let alias = HirType::TypeAlias("size_t".to_string());
        assert_eq!(EnumGenerator::type_based_variant_name(&alias), "size_t");
    }

    // ============================================================================
    // TYPE MAPPING TESTS
    // ============================================================================

    #[test]
    fn test_map_hir_type_primitives() {
        assert_eq!(EnumGenerator::map_hir_type_to_rust(&HirType::Void), "()");
        assert_eq!(EnumGenerator::map_hir_type_to_rust(&HirType::Int), "i32");
        assert_eq!(
            EnumGenerator::map_hir_type_to_rust(&HirType::UnsignedInt),
            "u32"
        );
        assert_eq!(EnumGenerator::map_hir_type_to_rust(&HirType::Float), "f32");
        assert_eq!(EnumGenerator::map_hir_type_to_rust(&HirType::Double), "f64");
        assert_eq!(EnumGenerator::map_hir_type_to_rust(&HirType::Char), "u8");
    }

    #[test]
    fn test_map_hir_type_char_pointer_to_string() {
        let char_ptr = HirType::Pointer(Box::new(HirType::Char));
        assert_eq!(EnumGenerator::map_hir_type_to_rust(&char_ptr), "String");
    }

    #[test]
    fn test_map_hir_type_void_pointer() {
        let void_ptr = HirType::Pointer(Box::new(HirType::Void));
        assert_eq!(EnumGenerator::map_hir_type_to_rust(&void_ptr), "*mut ()");
    }

    #[test]
    fn test_map_hir_type_other_pointer() {
        let int_ptr = HirType::Pointer(Box::new(HirType::Int));
        assert_eq!(EnumGenerator::map_hir_type_to_rust(&int_ptr), "*mut i32");
    }

    #[test]
    fn test_map_hir_type_box() {
        let boxed = HirType::Box(Box::new(HirType::Int));
        assert_eq!(EnumGenerator::map_hir_type_to_rust(&boxed), "Box<i32>");
    }

    #[test]
    fn test_map_hir_type_vec() {
        let vec = HirType::Vec(Box::new(HirType::Float));
        assert_eq!(EnumGenerator::map_hir_type_to_rust(&vec), "Vec<f32>");
    }

    #[test]
    fn test_map_hir_type_option() {
        let opt = HirType::Option(Box::new(HirType::Double));
        assert_eq!(EnumGenerator::map_hir_type_to_rust(&opt), "Option<f64>");
    }

    #[test]
    fn test_map_hir_type_immutable_reference() {
        let ref_type = HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        };
        assert_eq!(EnumGenerator::map_hir_type_to_rust(&ref_type), "&i32");
    }

    #[test]
    fn test_map_hir_type_mutable_reference() {
        let ref_type = HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: true,
        };
        assert_eq!(EnumGenerator::map_hir_type_to_rust(&ref_type), "&mut i32");
    }

    #[test]
    fn test_map_hir_type_struct() {
        let struct_type = HirType::Struct("Point".to_string());
        assert_eq!(EnumGenerator::map_hir_type_to_rust(&struct_type), "Point");
    }

    #[test]
    fn test_map_hir_type_enum() {
        let enum_type = HirType::Enum("Color".to_string());
        assert_eq!(EnumGenerator::map_hir_type_to_rust(&enum_type), "Color");
    }

    #[test]
    fn test_map_hir_type_union() {
        let union_type = HirType::Union(vec![]);
        assert_eq!(
            EnumGenerator::map_hir_type_to_rust(&union_type),
            "/* Union */"
        );
    }

    #[test]
    fn test_map_hir_type_array_with_size() {
        let array = HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        };
        assert_eq!(EnumGenerator::map_hir_type_to_rust(&array), "[i32; 10]");
    }

    #[test]
    fn test_map_hir_type_array_without_size() {
        let array = HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None,
        };
        assert_eq!(EnumGenerator::map_hir_type_to_rust(&array), "[i32]");
    }

    #[test]
    fn test_map_hir_type_function_pointer_void_return() {
        let fn_ptr = HirType::FunctionPointer {
            param_types: vec![HirType::Int, HirType::Float],
            return_type: Box::new(HirType::Void),
        };
        assert_eq!(EnumGenerator::map_hir_type_to_rust(&fn_ptr), "fn(i32, f32)");
    }

    #[test]
    fn test_map_hir_type_function_pointer_with_return() {
        let fn_ptr = HirType::FunctionPointer {
            param_types: vec![HirType::Int],
            return_type: Box::new(HirType::Int),
        };
        assert_eq!(
            EnumGenerator::map_hir_type_to_rust(&fn_ptr),
            "fn(i32) -> i32"
        );
    }

    #[test]
    fn test_map_hir_type_function_pointer_no_params() {
        let fn_ptr = HirType::FunctionPointer {
            param_types: vec![],
            return_type: Box::new(HirType::Int),
        };
        assert_eq!(EnumGenerator::map_hir_type_to_rust(&fn_ptr), "fn() -> i32");
    }

    #[test]
    fn test_map_hir_type_string_types() {
        assert_eq!(
            EnumGenerator::map_hir_type_to_rust(&HirType::StringLiteral),
            "&str"
        );
        assert_eq!(
            EnumGenerator::map_hir_type_to_rust(&HirType::OwnedString),
            "String"
        );
        assert_eq!(
            EnumGenerator::map_hir_type_to_rust(&HirType::StringReference),
            "&str"
        );
    }

    #[test]
    fn test_map_hir_type_type_alias() {
        let alias = HirType::TypeAlias("size_t".to_string());
        assert_eq!(EnumGenerator::map_hir_type_to_rust(&alias), "size_t");
    }

    // ============================================================================
    // INTEGRATION TESTS - COMPLETE ENUM GENERATION
    // ============================================================================

    #[test]
    fn test_generate_enum_complete_value_type() {
        let gen = EnumGenerator::new();
        let info = TaggedUnionInfo {
            struct_name: "Value".to_string(),
            tag_field_name: "type".to_string(),
            union_field_name: "data".to_string(),
            variants: vec![
                VariantInfo {
                    name: "i".to_string(), // Short name (1 char) → uses type
                    payload_type: HirType::Int,
                },
                VariantInfo {
                    name: "f".to_string(), // Short name (1 char) → uses type
                    payload_type: HirType::Float,
                },
                VariantInfo {
                    name: "s".to_string(), // Short name (1 char) → uses type
                    payload_type: HirType::Pointer(Box::new(HirType::Char)),
                },
            ],
        };

        let result = gen.generate_enum(&info);

        // Check derives
        assert!(result.contains("#[derive(Debug, Clone, PartialEq)]"));

        // Check enum declaration
        assert!(result.contains("pub enum Value {"));

        // Short names (<=2 chars) should use type-based variants
        assert!(result.contains("Int(i32)"));
        assert!(result.contains("Float(f32)"));
        assert!(result.contains("String(String)")); // char* → String (type-based)
    }

    #[test]
    fn test_generate_enum_with_nested_types() {
        let gen = EnumGenerator::new();
        let info = TaggedUnionInfo {
            struct_name: "Container".to_string(),
            tag_field_name: "kind".to_string(),
            union_field_name: "data".to_string(),
            variants: vec![
                VariantInfo {
                    name: "boxed_int".to_string(),
                    payload_type: HirType::Box(Box::new(HirType::Int)),
                },
                VariantInfo {
                    name: "vec_float".to_string(),
                    payload_type: HirType::Vec(Box::new(HirType::Float)),
                },
            ],
        };

        let result = gen.generate_enum(&info);
        assert!(result.contains("BoxedInt(Box<i32>)"));
        assert!(result.contains("VecFloat(Vec<f32>)"));
    }
}
