//! End-to-end integration tests for lifetime annotation generation (DECY-076).
//!
//! Tests that lifetime analysis results are correctly integrated into code generation,
//! producing compilable Rust code with proper lifetime annotations.
//!
//! Coverage:
//! - Function signatures with lifetime annotations
//! - Reference parameters and return types
//! - Multiple lifetimes
//! - Lifetime elision cases
//! - Struct definitions with lifetime parameters

use decy_codegen::CodeGenerator;
use decy_hir::{HirFunction, HirParameter, HirType};

// ============================================================================
// FUNCTION LIFETIME GENERATION: Single reference parameter
// ============================================================================

#[test]
fn test_function_with_single_reference_parameter() {
    // C: int* identity(int* ptr)
    // Expected Rust: fn identity<'a>(mut ptr: &'a i32) -> &'a i32

    let func = HirFunction::new(
        "identity".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Reference {
                inner: Box::new(HirType::Int),
                mutable: false,
            },
        )],
    );

    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_function(&func);

    // Should have lifetime parameter
    assert!(
        rust_code.contains("<'a>") || rust_code.contains("'a"),
        "Expected lifetime parameter, got: {}",
        rust_code
    );

    // Should have lifetime on parameter
    assert!(
        rust_code.contains("&'a i32") || rust_code.contains("& 'a i32"),
        "Expected lifetime on parameter, got: {}",
        rust_code
    );

    // Should have lifetime on return type
    assert!(
        rust_code.contains("-> &'a i32") || rust_code.contains("-> & 'a i32"),
        "Expected lifetime on return type, got: {}",
        rust_code
    );
}

#[test]
fn test_function_with_mutable_reference_lifetime() {
    // C: int* modify(int* ptr)
    // Expected Rust: fn modify<'a>(mut ptr: &'a mut i32) -> &'a mut i32

    let func = HirFunction::new(
        "modify".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: true,
        },
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Reference {
                inner: Box::new(HirType::Int),
                mutable: true,
            },
        )],
    );

    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_function(&func);

    // Should have lifetime parameter
    assert!(
        rust_code.contains("<'a>") || rust_code.contains("'a"),
        "Expected lifetime parameter, got: {}",
        rust_code
    );

    // Should have lifetime on mutable parameter
    assert!(
        rust_code.contains("&'a mut") || rust_code.contains("& 'a mut"),
        "Expected lifetime on mutable parameter, got: {}",
        rust_code
    );

    // Should have lifetime on mutable return type
    assert!(
        rust_code.contains("-> &'a mut i32") || rust_code.contains("-> & 'a mut i32"),
        "Expected lifetime on mutable return type, got: {}",
        rust_code
    );
}

// ============================================================================
// MULTIPLE LIFETIMES: Distinct lifetime parameters
// ============================================================================

#[test]
fn test_function_with_multiple_reference_parameters() {
    // C: int* choose(int* a, int* b, int flag)
    // Expected Rust: fn choose<'a, 'b>(mut a: &'a i32, mut b: &'b i32, mut flag: i32) -> &'a i32

    let func = HirFunction::new(
        "choose".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
        vec![
            HirParameter::new(
                "a".to_string(),
                HirType::Reference {
                    inner: Box::new(HirType::Int),
                    mutable: false,
                },
            ),
            HirParameter::new(
                "b".to_string(),
                HirType::Reference {
                    inner: Box::new(HirType::Int),
                    mutable: false,
                },
            ),
            HirParameter::new("flag".to_string(), HirType::Int),
        ],
    );

    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_function(&func);

    // Should have at least one lifetime parameter
    // (May have 'a and 'b, or may unify them - implementation choice)
    assert!(
        rust_code.contains("'a"),
        "Expected lifetime parameters, got: {}",
        rust_code
    );

    // Should have lifetimes on parameters
    assert!(
        rust_code.contains("&'") || rust_code.contains("& '"),
        "Expected lifetimes on reference parameters, got: {}",
        rust_code
    );

    // Non-reference parameter should not have lifetime
    assert!(
        rust_code.contains("flag: i32") || rust_code.contains("mut flag: i32"),
        "Expected non-reference parameter without lifetime, got: {}",
        rust_code
    );
}

// ============================================================================
// LIFETIME ELISION: No explicit annotations needed
// ============================================================================

#[test]
fn test_function_with_no_references_no_lifetimes() {
    // C: int add(int a, int b)
    // Expected Rust: fn add(mut a: i32, mut b: i32) -> i32

    let func = HirFunction::new(
        "add".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
    );

    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_function(&func);

    // Should NOT have lifetime parameters
    assert!(
        !rust_code.contains("<'") && !rust_code.contains("< '"),
        "Should not have lifetime parameters for non-reference function, got: {}",
        rust_code
    );

    // Should NOT have lifetime annotations
    assert!(
        !rust_code.contains("&'"),
        "Should not have lifetime annotations, got: {}",
        rust_code
    );
}

#[test]
fn test_function_with_reference_input_only_no_explicit_lifetime() {
    // C: int dereference(int* ptr)
    // Expected Rust: fn dereference(mut ptr: &i32) -> i32
    // Lifetime elision applies - no explicit lifetime needed

    let func = HirFunction::new(
        "dereference".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Reference {
                inner: Box::new(HirType::Int),
                mutable: false,
            },
        )],
    );

    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_function(&func);

    // May or may not have explicit lifetime (elision rule applies)
    // But should have reference
    assert!(
        rust_code.contains("&i32") || rust_code.contains("&'a i32"),
        "Expected reference parameter, got: {}",
        rust_code
    );

    // Return type should be i32 (not a reference)
    assert!(
        rust_code.contains("-> i32"),
        "Expected i32 return type, got: {}",
        rust_code
    );
}

// ============================================================================
// NESTED REFERENCES: References to references
// ============================================================================

#[test]
fn test_function_with_nested_reference_types() {
    // C: int** get_ptr_to_ptr(int** ptr)
    // Expected Rust: fn get_ptr_to_ptr<'a>(mut ptr: &'a &'a i32) -> &'a &'a i32

    let func = HirFunction::new(
        "get_ptr_to_ptr".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Reference {
                inner: Box::new(HirType::Int),
                mutable: false,
            }),
            mutable: false,
        },
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Reference {
                inner: Box::new(HirType::Reference {
                    inner: Box::new(HirType::Int),
                    mutable: false,
                }),
                mutable: false,
            },
        )],
    );

    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_function(&func);

    // Should have lifetime parameter
    assert!(
        rust_code.contains("'a") || rust_code.contains("'b"),
        "Expected lifetime parameters for nested references, got: {}",
        rust_code
    );

    // Should have nested reference in parameter
    assert!(
        rust_code.contains("&&") || rust_code.contains("& &"),
        "Expected nested reference in parameter, got: {}",
        rust_code
    );

    // Should have nested reference in return type
    assert!(
        rust_code.contains("-> &&") || rust_code.contains("-> & &"),
        "Expected nested reference in return type, got: {}",
        rust_code
    );
}

// ============================================================================
// FLOAT REFERENCES: Non-integer reference types
// ============================================================================

#[test]
fn test_function_with_float_reference() {
    // C: float* process_float(float* value)
    // Expected Rust: fn process_float<'a>(mut value: &'a f32) -> &'a f32

    let func = HirFunction::new(
        "process_float".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Float),
            mutable: false,
        },
        vec![HirParameter::new(
            "value".to_string(),
            HirType::Reference {
                inner: Box::new(HirType::Float),
                mutable: false,
            },
        )],
    );

    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_function(&func);

    // Should have lifetime parameter
    assert!(
        rust_code.contains("'a"),
        "Expected lifetime parameter, got: {}",
        rust_code
    );

    // Should have lifetime on float reference
    assert!(
        rust_code.contains("&'a f32") || rust_code.contains("& 'a f32"),
        "Expected lifetime on float reference, got: {}",
        rust_code
    );
}

// ============================================================================
// COMPILATION VALIDATION: Generated code must compile
// ============================================================================

#[test]
fn test_generated_code_with_lifetimes_is_valid_rust() {
    // This test verifies that generated lifetime annotations
    // produce syntactically valid Rust code

    let func = HirFunction::new(
        "test_func".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
        vec![HirParameter::new(
            "x".to_string(),
            HirType::Reference {
                inner: Box::new(HirType::Int),
                mutable: false,
            },
        )],
    );

    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_function(&func);

    // Basic syntax checks
    assert!(rust_code.contains("fn test_func"), "Should have function name");
    assert!(rust_code.contains("("), "Should have parameter list open");
    assert!(rust_code.contains(")"), "Should have parameter list close");
    assert!(rust_code.contains("->"), "Should have return type arrow");

    // Should have balanced braces
    let open_braces = rust_code.matches('{').count();
    let close_braces = rust_code.matches('}').count();
    assert_eq!(
        open_braces, close_braces,
        "Should have balanced braces, got: {}",
        rust_code
    );

    // Lifetime syntax should be correct if present
    if rust_code.contains("'a") {
        // If 'a exists, it should be in angle brackets for function signature
        assert!(
            rust_code.contains("<'") || rust_code.contains("< '"),
            "Lifetime parameters should be in angle brackets, got: {}",
            rust_code
        );
    }
}
