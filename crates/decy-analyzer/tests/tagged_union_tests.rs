//! RED phase tests for tagged union pattern detection (DECY-080).

use decy_analyzer::tagged_union_analysis::{TaggedUnionAnalyzer, TaggedUnionInfo};
use decy_hir::{HirStruct, HirStructField, HirType};

#[test]
fn test_detect_simple_tagged_union() {
    // struct Value { enum Tag tag; union { int i; float f; } data; };
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
    let result = analyzer.analyze_struct(&struct_def);

    assert!(result.is_some(), "Should detect tagged union");
    let info = result.unwrap();
    assert_eq!(info.struct_name, "Value");
    assert_eq!(info.tag_field_name, "tag");
    assert_eq!(info.union_field_name, "data");
    assert_eq!(info.variants.len(), 2);
}

#[test]
fn test_not_tagged_union_without_enum() {
    let struct_def = HirStruct::new(
        "NotTagged".to_string(),
        vec![
            HirStructField::new("x".to_string(), HirType::Int),
            HirStructField::new(
                "data".to_string(),
                HirType::Union(vec![("i".to_string(), HirType::Int)]),
            ),
        ],
    );

    let analyzer = TaggedUnionAnalyzer::new();
    assert!(analyzer.analyze_struct(&struct_def).is_none());
}

#[test]
fn test_not_tagged_union_without_union() {
    let struct_def = HirStruct::new(
        "NotTagged".to_string(),
        vec![
            HirStructField::new("tag".to_string(), HirType::Enum("Tag".to_string())),
            HirStructField::new("value".to_string(), HirType::Int),
        ],
    );

    let analyzer = TaggedUnionAnalyzer::new();
    assert!(analyzer.analyze_struct(&struct_def).is_none());
}

#[test]
fn test_extract_variant_types() {
    let struct_def = HirStruct::new(
        "Value".to_string(),
        vec![
            HirStructField::new("tag".to_string(), HirType::Enum("Tag".to_string())),
            HirStructField::new(
                "data".to_string(),
                HirType::Union(vec![
                    ("int_val".to_string(), HirType::Int),
                    ("float_val".to_string(), HirType::Float),
                    ("string_val".to_string(), HirType::Pointer(Box::new(HirType::Char))),
                ]),
            ),
        ],
    );

    let analyzer = TaggedUnionAnalyzer::new();
    let result = analyzer.analyze_struct(&struct_def).unwrap();

    assert_eq!(result.variants.len(), 3);
    assert!(result.variants.iter().any(|v| v.name == "int_val"));
    assert!(result.variants.iter().any(|v| v.name == "float_val"));
    assert!(result.variants.iter().any(|v| v.name == "string_val"));
}

#[test]
fn test_map_variant_types() {
    let struct_def = HirStruct::new(
        "Value".to_string(),
        vec![
            HirStructField::new("kind".to_string(), HirType::Enum("Kind".to_string())),
            HirStructField::new(
                "payload".to_string(),
                HirType::Union(vec![
                    ("i".to_string(), HirType::Int),
                    ("f".to_string(), HirType::Float),
                ]),
            ),
        ],
    );

    let analyzer = TaggedUnionAnalyzer::new();
    let result = analyzer.analyze_struct(&struct_def).unwrap();

    let int_variant = result.variants.iter().find(|v| v.name == "i").unwrap();
    assert_eq!(int_variant.payload_type, HirType::Int);

    let float_variant = result.variants.iter().find(|v| v.name == "f").unwrap();
    assert_eq!(float_variant.payload_type, HirType::Float);
}

#[test]
fn test_tagged_union_with_additional_fields() {
    let struct_def = HirStruct::new(
        "Value".to_string(),
        vec![
            HirStructField::new("id".to_string(), HirType::Int),
            HirStructField::new("tag".to_string(), HirType::Enum("Tag".to_string())),
            HirStructField::new(
                "data".to_string(),
                HirType::Union(vec![("i".to_string(), HirType::Int)]),
            ),
        ],
    );

    let analyzer = TaggedUnionAnalyzer::new();
    let result = analyzer.analyze_struct(&struct_def);

    assert!(result.is_some());
    let info = result.unwrap();
    assert_eq!(info.tag_field_name, "tag");
    assert_eq!(info.union_field_name, "data");
}

#[test]
fn test_empty_union() {
    let struct_def = HirStruct::new(
        "Value".to_string(),
        vec![
            HirStructField::new("tag".to_string(), HirType::Enum("Tag".to_string())),
            HirStructField::new("data".to_string(), HirType::Union(vec![])),
        ],
    );

    let analyzer = TaggedUnionAnalyzer::new();
    assert!(analyzer.analyze_struct(&struct_def).is_none());
}

#[test]
fn test_realistic_json_value() {
    // enum ValueType { TYPE_NULL, TYPE_INT, TYPE_FLOAT, TYPE_STRING };
    // struct JsonValue {
    //   enum ValueType type;
    //   union { int int_value; double float_value; char* string_value; } as;
    // };
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
    let result = analyzer.analyze_struct(&struct_def).unwrap();

    assert_eq!(result.struct_name, "JsonValue");
    assert_eq!(result.tag_field_name, "type");
    assert_eq!(result.union_field_name, "as");
    assert_eq!(result.variants.len(), 3);
}
