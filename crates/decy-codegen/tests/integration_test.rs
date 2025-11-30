//! Integration tests for the complete malloc-to-Box transformation pipeline.
//!
//! These tests demonstrate the end-to-end transformation from C code patterns
//! to safe, idiomatic Rust code using Box<T>.

use decy_analyzer::patterns::PatternDetector;
use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirStatement, HirType};

#[test]
fn test_complete_malloc_to_box_pipeline() {
    // Simulates C code:
    // void process() {
    //     int* ptr = malloc(sizeof(int));
    // }

    let func = HirFunction::new_with_body(
        "process".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "ptr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::IntLiteral(4)],
            }),
        }],
    );

    // Step 1: Detect Box candidates
    let detector = PatternDetector::new();
    let candidates = detector.find_box_candidates(&func);
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].variable, "ptr");

    // Step 2: Generate code - DECY-130 now transforms malloc automatically
    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // DECY-130: malloc is now automatically transformed to Vec
    // When pointer type is used with malloc, type becomes Vec
    assert!(code.contains("Vec<i32>") || code.contains("Vec::<u8>"),
        "malloc should be transformed to Vec. Got: {}", code);
    assert!(!code.contains("malloc(4)"),
        "malloc should not appear in output. Got: {}", code);

    // Step 3: Generate code with Box transformation (explicit Box hints)
    let code_with = codegen.generate_function_with_box_transform(&func, &candidates);
    assert!(code_with.contains("Box<i32>"));
    assert!(code_with.contains("Box::new(0)"));
    assert!(!code_with.contains("malloc"));
    assert!(!code_with.contains("*mut"));

    // Verify complete transformation
    assert!(code_with.contains("fn process() {"));
    assert!(code_with.contains("let mut ptr: Box<i32> = Box::new(0);"));
}

#[test]
fn test_multiple_malloc_transformations() {
    // Simulates C code:
    // void multi_alloc() {
    //     int* x = malloc(sizeof(int));
    //     char* y = malloc(sizeof(char));
    // }

    let func = HirFunction::new_with_body(
        "multi_alloc".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(4)],
                }),
            },
            HirStatement::VariableDeclaration {
                name: "y".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Char)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(1)],
                }),
            },
        ],
    );

    let detector = PatternDetector::new();
    let candidates = detector.find_box_candidates(&func);
    assert_eq!(candidates.len(), 2);

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function_with_box_transform(&func, &candidates);

    // Verify both transformations
    assert!(code.contains("let mut x: Box<i32> = Box::new(0);"));
    assert!(code.contains("let mut y: Box<u8> = Box::new(0);"));
    assert!(!code.contains("malloc"));
    assert!(!code.contains("*mut"));
}

#[test]
fn test_mixed_malloc_and_regular_variables() {
    // Simulates C code with both malloc and regular variables:
    // void mixed() {
    //     int regular = 42;
    //     int* allocated = malloc(sizeof(int));
    // }

    let func = HirFunction::new_with_body(
        "mixed".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "regular".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(42)),
            },
            HirStatement::VariableDeclaration {
                name: "allocated".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(4)],
                }),
            },
        ],
    );

    let detector = PatternDetector::new();
    let candidates = detector.find_box_candidates(&func);
    assert_eq!(candidates.len(), 1); // Only the malloc

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function_with_box_transform(&func, &candidates);

    // Regular variable unchanged
    assert!(code.contains("let mut regular: i32 = 42;"));
    // Malloc transformed
    assert!(code.contains("let mut allocated: Box<i32> = Box::new(0);"));
    assert!(!code.contains("malloc"));
}

#[test]
fn test_different_types_transformed() {
    // Test transformation with different C types

    let test_cases = vec![
        (HirType::Int, "Box<i32>"),
        (HirType::Char, "Box<u8>"),
        (HirType::Float, "Box<f32>"),
        (HirType::Double, "Box<f64>"),
    ];

    for (hir_type, expected_rust_type) in test_cases {
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![HirStatement::VariableDeclaration {
                name: "ptr".to_string(),
                var_type: HirType::Pointer(Box::new(hir_type.clone())),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(8)],
                }),
            }],
        );

        let detector = PatternDetector::new();
        let candidates = detector.find_box_candidates(&func);
        let codegen = CodeGenerator::new();
        let code = codegen.generate_function_with_box_transform(&func, &candidates);

        assert!(
            code.contains(expected_rust_type),
            "Expected {} in generated code for type {:?}",
            expected_rust_type,
            hir_type
        );
        assert!(code.contains("Box::new"));
    }
}

#[test]
fn test_transformation_preserves_function_structure() {
    // Complex function with multiple statements
    // void complex() {
    //     int* data = malloc(sizeof(int));
    //     int value = 100;
    //     return;
    // }

    let func = HirFunction::new_with_body(
        "complex".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "data".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(4)],
                }),
            },
            HirStatement::VariableDeclaration {
                name: "value".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(100)),
            },
            HirStatement::Return(None),
        ],
    );

    let detector = PatternDetector::new();
    let candidates = detector.find_box_candidates(&func);
    let codegen = CodeGenerator::new();
    let code = codegen.generate_function_with_box_transform(&func, &candidates);

    // Verify structure preserved
    assert!(code.contains("fn complex() {"));
    assert!(code.contains("let mut data: Box<i32> = Box::new(0);"));
    assert!(code.contains("let mut value: i32 = 100;"));
    assert!(code.contains("return;"));
    assert!(code.ends_with("}"));
}

#[test]
fn test_no_transformation_without_malloc() {
    // Function without malloc should not be transformed
    let func = HirFunction::new_with_body(
        "no_malloc".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(42)),
            },
            HirStatement::Return(Some(HirExpression::Variable("x".to_string()))),
        ],
    );

    let detector = PatternDetector::new();
    let candidates = detector.find_box_candidates(&func);
    assert_eq!(candidates.len(), 0);

    let codegen = CodeGenerator::new();
    let code_without = codegen.generate_function(&func);
    let code_with = codegen.generate_function_with_box_transform(&func, &candidates);

    // Should be identical since no transformation needed
    assert_eq!(code_without, code_with);
    assert!(!code_with.contains("Box"));
}
