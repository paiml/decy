//! Tests for the test generator module.
//!
//! These tests verify that the TestGenerator can:
//! - Generate unit tests from C function analysis
//! - Generate property tests for transpiled functions
//! - Generate doc tests with usage examples
//! - Generate mutation test configuration

use crate::test_generator::{TestGenConfig, TestGenerator};
use decy_hir::{HirFunction, HirParameter, HirType};

#[test]
fn test_test_generator_creation() {
    let config = TestGenConfig::default();
    let generator = TestGenerator::new(config);

    assert_eq!(generator.config().unit_tests_per_function, 5);
    assert_eq!(generator.config().property_tests_per_function, 5);
    assert_eq!(generator.config().property_test_cases, 1000);
    assert!(generator.config().generate_doc_tests);
}

#[test]
fn test_generate_unit_tests_for_simple_function() {
    let generator = TestGenerator::new(TestGenConfig::default());

    // Create a simple HIR function: fn add(a: i32, b: i32) -> i32
    let func = HirFunction::new(
        "add".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
    );

    let tests = generator.generate_tests(&func);

    // Should generate at least 5 unit tests
    assert!(
        tests.unit_tests.len() >= 5,
        "Expected at least 5 unit tests, got {}",
        tests.unit_tests.len()
    );

    // Unit tests should have proper naming
    for test in &tests.unit_tests {
        assert!(test.contains("#[test]"));
        assert!(test.contains("fn test_"));
    }
}

#[test]
fn test_generate_property_tests() {
    let generator = TestGenerator::new(TestGenConfig::default());

    let func = HirFunction::new(
        "multiply".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("x".to_string(), HirType::Int),
            HirParameter::new("y".to_string(), HirType::Int),
        ],
    );

    let tests = generator.generate_tests(&func);

    // Should generate at least 5 property tests
    assert!(
        tests.property_tests.len() >= 5,
        "Expected at least 5 property tests, got {}",
        tests.property_tests.len()
    );

    // Property tests should use proptest macro
    for test in &tests.property_tests {
        assert!(test.contains("proptest!"));
        assert!(test.contains("#[test]"));
    }
}

#[test]
fn test_generate_determinism_property() {
    let generator = TestGenerator::new(TestGenConfig::default());

    let func = HirFunction::new("process".to_string(), HirType::Void, vec![]);

    let tests = generator.generate_tests(&func);

    // Should have a determinism property test
    let has_determinism = tests
        .property_tests
        .iter()
        .any(|t| t.contains("deterministic"));

    assert!(has_determinism, "Should generate determinism property test");
}

#[test]
fn test_generate_no_panic_property() {
    let generator = TestGenerator::new(TestGenConfig::default());

    let func = HirFunction::new(
        "safe_divide".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
    );

    let tests = generator.generate_tests(&func);

    // Should have a no-panic property test
    let has_no_panic = tests
        .property_tests
        .iter()
        .any(|t| t.contains("never_panics"));

    assert!(has_no_panic, "Should generate no-panic property test");
}

#[test]
fn test_generate_doc_test() {
    let generator = TestGenerator::new(TestGenConfig::default());

    let func = HirFunction::new(
        "calculate".to_string(),
        HirType::Int,
        vec![HirParameter::new("value".to_string(), HirType::Int)],
    );

    let tests = generator.generate_tests(&func);

    // Should generate doc test
    assert!(
        !tests.doc_tests.is_empty(),
        "Should generate at least one doc test"
    );

    // Doc test should include example usage
    let doc_test = &tests.doc_tests[0];
    assert!(doc_test.contains("# Examples"));
    assert!(doc_test.contains("```"));
}

#[test]
fn test_generate_mutation_config() {
    let config = TestGenConfig {
        generate_mutation_config: true,
        ..Default::default()
    };

    let generator = TestGenerator::new(config);

    let func = HirFunction::new("increment".to_string(), HirType::Int, vec![]);

    let tests = generator.generate_tests(&func);

    // Should generate mutation test config
    assert!(
        tests.mutation_config.is_some(),
        "Should generate mutation config when enabled"
    );

    let mutation_config = tests.mutation_config.unwrap();
    assert!(mutation_config.contains("[[mutant]]"));
    assert!(mutation_config.contains("function ="));
}

#[test]
fn test_custom_test_counts() {
    let config = TestGenConfig {
        unit_tests_per_function: 10,
        property_tests_per_function: 8,
        property_test_cases: 5000,
        generate_doc_tests: true,
        generate_mutation_config: false,
        behavior_equivalence_tests: false,
    };

    let generator = TestGenerator::new(config);

    let func = HirFunction::new("test".to_string(), HirType::Void, vec![]);

    let tests = generator.generate_tests(&func);

    assert!(tests.unit_tests.len() >= 10);
    assert!(tests.property_tests.len() >= 8);
}

#[test]
fn test_analyze_test_scenarios_for_pointer_parameter() {
    let generator = TestGenerator::new(TestGenConfig::default());

    // Function with pointer parameter: void process(int* data)
    let func = HirFunction::new(
        "process".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "data".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
    );

    let tests = generator.generate_tests(&func);

    // Should generate tests for null pointer cases
    let has_null_test = tests
        .unit_tests
        .iter()
        .any(|t| t.contains("null") || t.contains("None"));

    assert!(
        has_null_test,
        "Should generate null/None test for pointer parameter"
    );
}

#[test]
fn test_generate_tests_for_function_with_box_type() {
    let generator = TestGenerator::new(TestGenConfig::default());

    // Function returning Box<T>
    let func = HirFunction::new(
        "create_value".to_string(),
        HirType::Box(Box::new(HirType::Int)),
        vec![],
    );

    let tests = generator.generate_tests(&func);

    // Should generate tests that work with Box<T>
    assert!(!tests.unit_tests.is_empty());
    assert!(!tests.property_tests.is_empty());
}

#[test]
fn test_disable_doc_test_generation() {
    let config = TestGenConfig {
        generate_doc_tests: false,
        ..Default::default()
    };

    let generator = TestGenerator::new(config);

    let func = HirFunction::new("test".to_string(), HirType::Void, vec![]);

    let tests = generator.generate_tests(&func);

    assert!(
        tests.doc_tests.is_empty(),
        "Should not generate doc tests when disabled"
    );
}

// ============================================================================
// Additional coverage: default_test_value all type branches
// ============================================================================

#[test]
fn test_default_test_value_void() {
    assert_eq!(TestGenerator::default_test_value(&HirType::Void), "()");
}

#[test]
fn test_default_test_value_int() {
    assert_eq!(TestGenerator::default_test_value(&HirType::Int), "42");
}

#[test]
fn test_default_test_value_unsigned_int() {
    assert_eq!(
        TestGenerator::default_test_value(&HirType::UnsignedInt),
        "42u32"
    );
}

#[test]
fn test_default_test_value_float() {
    assert_eq!(
        TestGenerator::default_test_value(&HirType::Float),
        "3.14"
    );
}

#[test]
fn test_default_test_value_double() {
    assert_eq!(
        TestGenerator::default_test_value(&HirType::Double),
        "2.718"
    );
}

#[test]
fn test_default_test_value_char() {
    assert_eq!(TestGenerator::default_test_value(&HirType::Char), "b'A'");
}

#[test]
fn test_default_test_value_signed_char() {
    assert_eq!(
        TestGenerator::default_test_value(&HirType::SignedChar),
        "65i8"
    );
}

#[test]
fn test_default_test_value_pointer() {
    assert_eq!(
        TestGenerator::default_test_value(&HirType::Pointer(Box::new(HirType::Int))),
        "std::ptr::null_mut()"
    );
}

#[test]
fn test_default_test_value_box() {
    assert_eq!(
        TestGenerator::default_test_value(&HirType::Box(Box::new(HirType::Int))),
        "Box::new(42)"
    );
}

#[test]
fn test_default_test_value_vec() {
    assert_eq!(
        TestGenerator::default_test_value(&HirType::Vec(Box::new(HirType::Int))),
        "Vec::new()"
    );
}

#[test]
fn test_default_test_value_option() {
    assert_eq!(
        TestGenerator::default_test_value(&HirType::Option(Box::new(HirType::Int))),
        "Some(42)"
    );
}

#[test]
fn test_default_test_value_reference() {
    assert_eq!(
        TestGenerator::default_test_value(&HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        }),
        "&42"
    );
}

#[test]
fn test_default_test_value_struct() {
    assert_eq!(
        TestGenerator::default_test_value(&HirType::Struct("Point".to_string())),
        "Point::default()"
    );
}

#[test]
fn test_default_test_value_enum() {
    assert_eq!(
        TestGenerator::default_test_value(&HirType::Enum("Color".to_string())),
        "Color::default()"
    );
}

#[test]
fn test_default_test_value_array_with_size() {
    assert_eq!(
        TestGenerator::default_test_value(&HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(5),
        }),
        "[42; 5]"
    );
}

#[test]
fn test_default_test_value_array_unsized() {
    assert_eq!(
        TestGenerator::default_test_value(&HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None,
        }),
        "&[]"
    );
}

#[test]
fn test_default_test_value_function_pointer() {
    let result = TestGenerator::default_test_value(&HirType::FunctionPointer {
        return_type: Box::new(HirType::Void),
        param_types: vec![],
    });
    assert!(result.contains("todo!"));
}

#[test]
fn test_default_test_value_string_literal() {
    assert_eq!(
        TestGenerator::default_test_value(&HirType::StringLiteral),
        r#""test string""#
    );
}

#[test]
fn test_default_test_value_owned_string() {
    assert_eq!(
        TestGenerator::default_test_value(&HirType::OwnedString),
        r#"String::from("test string")"#
    );
}

#[test]
fn test_default_test_value_string_reference() {
    assert_eq!(
        TestGenerator::default_test_value(&HirType::StringReference),
        r#""test string""#
    );
}

#[test]
fn test_default_test_value_union() {
    let result = TestGenerator::default_test_value(&HirType::Union(vec![
        ("field1".to_string(), HirType::Int),
    ]));
    assert!(result.contains("todo!"));
}

#[test]
fn test_default_test_value_type_alias() {
    assert_eq!(
        TestGenerator::default_test_value(&HirType::TypeAlias("size_t".to_string())),
        "0"
    );
}

// ============================================================================
// Additional coverage: generate_tests with mutation disabled
// ============================================================================

#[test]
fn test_mutation_config_disabled() {
    let config = TestGenConfig {
        generate_mutation_config: false,
        ..Default::default()
    };

    let generator = TestGenerator::new(config);
    let func = HirFunction::new("test".to_string(), HirType::Void, vec![]);
    let tests = generator.generate_tests(&func);

    assert!(tests.mutation_config.is_none());
}

// ============================================================================
// Additional coverage: function with Box parameter (generates null test)
// ============================================================================

#[test]
fn test_null_test_for_box_parameter() {
    let generator = TestGenerator::new(TestGenConfig::default());

    let func = HirFunction::new(
        "process_box".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "data".to_string(),
            HirType::Box(Box::new(HirType::Int)),
        )],
    );

    let tests = generator.generate_tests(&func);

    let has_null_test = tests
        .unit_tests
        .iter()
        .any(|t| t.contains("null") || t.contains("None"));

    assert!(has_null_test, "Should generate null test for Box parameter");
}

// ============================================================================
// Additional coverage: doc tests with no parameters
// ============================================================================

#[test]
fn test_doc_tests_no_params() {
    let generator = TestGenerator::new(TestGenConfig::default());

    let func = HirFunction::new("get_value".to_string(), HirType::Int, vec![]);

    let tests = generator.generate_tests(&func);
    assert!(!tests.doc_tests.is_empty());
    let doc_test = &tests.doc_tests[0];
    assert!(doc_test.contains("get_value()"));
}

// ============================================================================
// Additional coverage: config equality
// ============================================================================

#[test]
fn test_config_clone_and_eq() {
    let config = TestGenConfig::default();
    let clone = config.clone();
    assert_eq!(config, clone);
}

#[test]
fn test_config_debug_format() {
    let config = TestGenConfig::default();
    let debug = format!("{:?}", config);
    assert!(debug.contains("TestGenConfig"));
}
