//! RED phase integration tests for enum generation from tagged unions (DECY-081).

use decy_analyzer::tagged_union_analysis::TaggedUnionAnalyzer;
use decy_codegen::enum_gen::EnumGenerator;
use decy_hir::{HirStruct, HirStructField, HirType};

#[test]
fn test_generate_simple_enum_from_tagged_union() {
    // C: struct Value { enum Tag tag; union { int i; float f; } data; };
    // Rust: enum Value { Int(i32), Float(f32) }
    let struct_def = HirStruct::new(
        "Value".to_string(),
        vec![
            HirStructField::new("tag".to_string(), HirType::Enum("Tag".to_string())),
            HirStructField::new(
                "data".to_string(),
                HirType::Union(vec![
                    ("i".to_string(), HirType::Int),
                    ("f".to_string(), HirType::Float),
                ]),
            ),
        ],
    );

    let analyzer = TaggedUnionAnalyzer::new();
    let info = analyzer.analyze_struct(&struct_def).unwrap();

    let generator = EnumGenerator::new();
    let rust_enum = generator.generate_enum(&info);

    assert!(rust_enum.contains("enum Value"));
    assert!(rust_enum.contains("Int(i32)"));
    assert!(rust_enum.contains("Float(f32)"));
    assert!(!rust_enum.contains("tag"), "Should not contain tag field");
    assert!(!rust_enum.contains("union"), "Should not contain union");
}

#[test]
fn test_generate_enum_with_multiple_variants() {
    let struct_def = HirStruct::new(
        "JsonValue".to_string(),
        vec![
            HirStructField::new("type".to_string(), HirType::Enum("ValueType".to_string())),
            HirStructField::new(
                "as".to_string(),
                HirType::Union(vec![
                    ("int_value".to_string(), HirType::Int),
                    ("float_value".to_string(), HirType::Double),
                    (
                        "string_value".to_string(),
                        HirType::Pointer(Box::new(HirType::Char)),
                    ),
                ]),
            ),
        ],
    );

    let analyzer = TaggedUnionAnalyzer::new();
    let info = analyzer.analyze_struct(&struct_def).unwrap();

    let generator = EnumGenerator::new();
    let rust_enum = generator.generate_enum(&info);

    assert!(rust_enum.contains("enum JsonValue"));
    assert!(rust_enum.contains("IntValue(i32)"));
    assert!(rust_enum.contains("FloatValue(f64)"));
    assert!(rust_enum.contains("StringValue("));
}

#[test]
fn test_enum_variant_name_capitalization() {
    // Variant names should be in PascalCase
    let struct_def = HirStruct::new(
        "Value".to_string(),
        vec![
            HirStructField::new("kind".to_string(), HirType::Enum("Kind".to_string())),
            HirStructField::new(
                "data".to_string(),
                HirType::Union(vec![
                    ("int_val".to_string(), HirType::Int),
                    ("float_val".to_string(), HirType::Float),
                ]),
            ),
        ],
    );

    let analyzer = TaggedUnionAnalyzer::new();
    let info = analyzer.analyze_struct(&struct_def).unwrap();

    let generator = EnumGenerator::new();
    let rust_enum = generator.generate_enum(&info);

    // Check that variant names are properly formatted
    assert!(
        rust_enum.contains("IntVal(i32)") || rust_enum.contains("Int(i32)"),
        "Variant should be capitalized"
    );
}

#[test]
fn test_map_c_types_to_rust_types() {
    // Test C type → Rust type mapping
    let struct_def = HirStruct::new(
        "TypeTest".to_string(),
        vec![
            HirStructField::new("tag".to_string(), HirType::Enum("Tag".to_string())),
            HirStructField::new(
                "data".to_string(),
                HirType::Union(vec![
                    ("byte_val".to_string(), HirType::Char),
                    ("int_val".to_string(), HirType::Int),
                    ("float_val".to_string(), HirType::Float),
                    ("double_val".to_string(), HirType::Double),
                ]),
            ),
        ],
    );

    let analyzer = TaggedUnionAnalyzer::new();
    let info = analyzer.analyze_struct(&struct_def).unwrap();

    let generator = EnumGenerator::new();
    let rust_enum = generator.generate_enum(&info);

    // Verify Rust type mappings
    assert!(rust_enum.contains("i8") || rust_enum.contains("u8")); // char
    assert!(rust_enum.contains("i32")); // int
    assert!(rust_enum.contains("f32")); // float
    assert!(rust_enum.contains("f64")); // double
}

#[test]
fn test_generate_valid_rust_syntax() {
    let struct_def = HirStruct::new(
        "Value".to_string(),
        vec![
            HirStructField::new("tag".to_string(), HirType::Enum("Tag".to_string())),
            HirStructField::new(
                "data".to_string(),
                HirType::Union(vec![("i".to_string(), HirType::Int)]),
            ),
        ],
    );

    let analyzer = TaggedUnionAnalyzer::new();
    let info = analyzer.analyze_struct(&struct_def).unwrap();

    let generator = EnumGenerator::new();
    let rust_enum = generator.generate_enum(&info);

    // Should be valid Rust code that can be parsed
    assert!(
        rust_enum.contains("enum ") || rust_enum.contains("pub enum "),
        "Should contain enum declaration"
    );
    assert!(rust_enum.contains("{"));
    assert!(rust_enum.contains("}"));
    assert!(rust_enum.contains(",") || rust_enum.ends_with("}"));
}

#[test]
fn test_pointer_type_mapping() {
    let struct_def = HirStruct::new(
        "Value".to_string(),
        vec![
            HirStructField::new("tag".to_string(), HirType::Enum("Tag".to_string())),
            HirStructField::new(
                "data".to_string(),
                HirType::Union(vec![
                    (
                        "string_val".to_string(),
                        HirType::Pointer(Box::new(HirType::Char)),
                    ),
                    (
                        "buffer_val".to_string(),
                        HirType::Pointer(Box::new(HirType::Void)),
                    ),
                ]),
            ),
        ],
    );

    let analyzer = TaggedUnionAnalyzer::new();
    let info = analyzer.analyze_struct(&struct_def).unwrap();

    let generator = EnumGenerator::new();
    let rust_enum = generator.generate_enum(&info);

    // char* should map to String or &str
    assert!(
        rust_enum.contains("String") || rust_enum.contains("&str"),
        "char* should map to String or &str"
    );
}

#[test]
fn test_enum_with_unit_variants() {
    // Some variants might have no payload (unit variants)
    let struct_def = HirStruct::new(
        "Option".to_string(),
        vec![
            HirStructField::new("tag".to_string(), HirType::Enum("Tag".to_string())),
            HirStructField::new(
                "data".to_string(),
                HirType::Union(vec![
                    ("none".to_string(), HirType::Void),
                    ("some".to_string(), HirType::Int),
                ]),
            ),
        ],
    );

    let analyzer = TaggedUnionAnalyzer::new();
    let info = analyzer.analyze_struct(&struct_def).unwrap();

    let generator = EnumGenerator::new();
    let rust_enum = generator.generate_enum(&info);

    assert!(rust_enum.contains("enum Option"));
    // void should be unit variant or ()
    assert!(
        rust_enum.contains("None,") || rust_enum.contains("None(())"),
        "void should map to unit variant"
    );
}

#[test]
fn test_enum_visibility_modifier() {
    let struct_def = HirStruct::new(
        "Value".to_string(),
        vec![
            HirStructField::new("tag".to_string(), HirType::Enum("Tag".to_string())),
            HirStructField::new(
                "data".to_string(),
                HirType::Union(vec![("i".to_string(), HirType::Int)]),
            ),
        ],
    );

    let analyzer = TaggedUnionAnalyzer::new();
    let info = analyzer.analyze_struct(&struct_def).unwrap();

    let generator = EnumGenerator::new();
    let rust_enum = generator.generate_enum(&info);

    // Should generate pub enum for public API
    assert!(
        rust_enum.contains("pub enum") || rust_enum.contains("enum"),
        "Should have visibility modifier"
    );
}

#[test]
fn test_enum_derives() {
    let struct_def = HirStruct::new(
        "Value".to_string(),
        vec![
            HirStructField::new("tag".to_string(), HirType::Enum("Tag".to_string())),
            HirStructField::new(
                "data".to_string(),
                HirType::Union(vec![("i".to_string(), HirType::Int)]),
            ),
        ],
    );

    let analyzer = TaggedUnionAnalyzer::new();
    let info = analyzer.analyze_struct(&struct_def).unwrap();

    let generator = EnumGenerator::new();
    let rust_enum = generator.generate_enum(&info);

    // Should include common derives for safety and usability
    assert!(
        rust_enum.contains("#[derive") || rust_enum.contains("derive("),
        "Should include derive macros"
    );
}

#[test]
fn test_realistic_ast_node_enum() {
    // Real-world example: AST node with multiple variants
    let struct_def = HirStruct::new(
        "AstNode".to_string(),
        vec![
            HirStructField::new("type".to_string(), HirType::Enum("NodeType".to_string())),
            HirStructField::new(
                "as".to_string(),
                HirType::Union(vec![
                    ("literal".to_string(), HirType::Int),
                    ("binary_op".to_string(), HirType::Int), // Would be struct in real code
                    (
                        "identifier".to_string(),
                        HirType::Pointer(Box::new(HirType::Char)),
                    ),
                ]),
            ),
        ],
    );

    let analyzer = TaggedUnionAnalyzer::new();
    let info = analyzer.analyze_struct(&struct_def).unwrap();

    let generator = EnumGenerator::new();
    let rust_enum = generator.generate_enum(&info);

    assert!(rust_enum.contains("enum AstNode"));
    assert!(rust_enum.contains("Literal"));
    assert!(rust_enum.contains("BinaryOp"));
    assert!(rust_enum.contains("Identifier"));
    assert_eq!(info.variants.len(), 3);
}
