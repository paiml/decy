//! Tests for output parameter detection.
//!
//! These tests verify that we can detect C output parameters and transform them
//! to idiomatic Rust return values.

use decy_analyzer::output_params::{OutputParamDetector, ParameterKind};
use decy_hir::{BinaryOperator, HirExpression, HirFunction, HirParameter, HirStatement, HirType};

/// Helper: Create a simple function for testing
fn create_test_function(
    name: &str,
    params: Vec<HirParameter>,
    return_type: HirType,
    body: Vec<HirStatement>,
) -> HirFunction {
    HirFunction::new_with_body(name.to_string(), return_type, params, body)
}

/// Helper: Create a pointer parameter
fn create_pointer_param(name: &str) -> HirParameter {
    HirParameter::new(name.to_string(), HirType::Pointer(Box::new(HirType::Int)))
}

/// Helper: Create a char pointer parameter (for strings)
fn create_char_pointer_param(name: &str) -> HirParameter {
    HirParameter::new(name.to_string(), HirType::Pointer(Box::new(HirType::Char)))
}

// ============================================================================
// TEST 1: Basic output parameter detection (write-before-read)
// ============================================================================

#[test]
fn test_detect_simple_output_parameter() {
    // C code:
    // int parse(const char* input, int* result) {
    //     *result = 42;  // Write before read
    //     return 0;      // Success
    // }

    let func = create_test_function(
        "parse",
        vec![
            create_char_pointer_param("input"),
            create_pointer_param("result"),
        ],
        HirType::Int,
        vec![
            // *result = 42
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("result".to_string()),
                value: HirExpression::IntLiteral(42),
            },
            // return 0
            HirStatement::Return(Some(HirExpression::IntLiteral(0))),
        ],
    );

    let detector = OutputParamDetector::new();
    let output_params = detector.detect(&func);

    assert_eq!(output_params.len(), 1);
    assert_eq!(output_params[0].name, "result");
    assert_eq!(output_params[0].kind, ParameterKind::Output);
    assert!(output_params[0].is_fallible);
}

// ============================================================================
// TEST 2: Distinguish input parameters (read-before-write)
// ============================================================================

#[test]
fn test_input_parameter_not_detected_as_output() {
    // C code:
    // int increment(int* value) {
    //     int old = *value;  // Read before write
    //     *value = old + 1;
    //     return 0;
    // }

    let func = create_test_function(
        "increment",
        vec![create_pointer_param("value")],
        HirType::Int,
        vec![
            // int old = *value
            HirStatement::VariableDeclaration {
                name: "old".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::Dereference(Box::new(
                    HirExpression::Variable("value".to_string()),
                ))),
            },
            // *value = old + 1
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("value".to_string()),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("old".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            },
            HirStatement::Return(Some(HirExpression::IntLiteral(0))),
        ],
    );

    let detector = OutputParamDetector::new();
    let output_params = detector.detect(&func);

    // Should be classified as input-output, not pure output
    assert_eq!(output_params.len(), 0);
}

// ============================================================================
// TEST 3: Multiple output parameters
// ============================================================================

#[test]
fn test_multiple_output_parameters() {
    // C code:
    // int parse_coords(const char* input, int* x, int* y) {
    //     *x = 10;
    //     *y = 20;
    //     return 0;
    // }

    let func = create_test_function(
        "parse_coords",
        vec![
            create_char_pointer_param("input"),
            create_pointer_param("x"),
            create_pointer_param("y"),
        ],
        HirType::Int,
        vec![
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("x".to_string()),
                value: HirExpression::IntLiteral(10),
            },
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("y".to_string()),
                value: HirExpression::IntLiteral(20),
            },
            HirStatement::Return(Some(HirExpression::IntLiteral(0))),
        ],
    );

    let detector = OutputParamDetector::new();
    let output_params = detector.detect(&func);

    assert_eq!(output_params.len(), 2);

    let x_param = output_params.iter().find(|p| p.name == "x").unwrap();
    let y_param = output_params.iter().find(|p| p.name == "y").unwrap();

    assert_eq!(x_param.kind, ParameterKind::Output);
    assert_eq!(y_param.kind, ParameterKind::Output);
}

// ============================================================================
// TEST 4: Fallible operations (return value indicates success/failure)
// ============================================================================

#[test]
fn test_fallible_operation_detection() {
    // C code:
    // int try_parse(const char* input, int* result) {
    //     if (input == NULL) return -1;  // Error
    //     *result = 42;
    //     return 0;  // Success
    // }

    let func = create_test_function(
        "try_parse",
        vec![
            create_char_pointer_param("input"),
            create_pointer_param("result"),
        ],
        HirType::Int,
        vec![
            HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::Equal,
                    left: Box::new(HirExpression::Variable("input".to_string())),
                    right: Box::new(HirExpression::NullLiteral),
                },
                then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(-1)))],
                else_block: None,
            },
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("result".to_string()),
                value: HirExpression::IntLiteral(42),
            },
            HirStatement::Return(Some(HirExpression::IntLiteral(0))),
        ],
    );

    let detector = OutputParamDetector::new();
    let output_params = detector.detect(&func);

    assert_eq!(output_params.len(), 1);
    assert_eq!(output_params[0].name, "result");
    assert!(
        output_params[0].is_fallible,
        "Should detect fallible operation"
    );
}

// ============================================================================
// TEST 5: Non-fallible operation (void return type)
// ============================================================================

#[test]
fn test_non_fallible_operation() {
    // C code:
    // void get_default(int* result) {
    //     *result = 42;
    // }

    let func = create_test_function(
        "get_default",
        vec![create_pointer_param("result")],
        HirType::Void,
        vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("result".to_string()),
            value: HirExpression::IntLiteral(42),
        }],
    );

    let detector = OutputParamDetector::new();
    let output_params = detector.detect(&func);

    assert_eq!(output_params.len(), 1);
    assert_eq!(output_params[0].name, "result");
    assert!(!output_params[0].is_fallible, "Void return is not fallible");
}

// ============================================================================
// TEST 6: Output parameter not written
// ============================================================================

#[test]
fn test_parameter_not_written_not_detected() {
    // C code:
    // int no_op(int* result) {
    //     return 0;  // Parameter never written
    // }

    let func = create_test_function(
        "no_op",
        vec![create_pointer_param("result")],
        HirType::Int,
        vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
    );

    let detector = OutputParamDetector::new();
    let output_params = detector.detect(&func);

    assert_eq!(
        output_params.len(),
        0,
        "Unwritten parameter should not be detected"
    );
}

// ============================================================================
// TEST 7: Conditional write (still an output parameter)
// ============================================================================

#[test]
fn test_conditional_write_detected_as_output() {
    // C code:
    // int maybe_write(int flag, int* result) {
    //     if (flag) {
    //         *result = 42;
    //     }
    //     return 0;
    // }

    let func = create_test_function(
        "maybe_write",
        vec![
            HirParameter::new("flag".to_string(), HirType::Int),
            create_pointer_param("result"),
        ],
        HirType::Int,
        vec![
            HirStatement::If {
                condition: HirExpression::Variable("flag".to_string()),
                then_block: vec![HirStatement::DerefAssignment {
                    target: HirExpression::Variable("result".to_string()),
                    value: HirExpression::IntLiteral(42),
                }],
                else_block: None,
            },
            HirStatement::Return(Some(HirExpression::IntLiteral(0))),
        ],
    );

    let detector = OutputParamDetector::new();
    let output_params = detector.detect(&func);

    assert_eq!(output_params.len(), 1);
    assert_eq!(output_params[0].name, "result");
}

// ============================================================================
// TEST 8: Non-pointer parameter (should not be detected)
// ============================================================================

#[test]
fn test_non_pointer_not_detected() {
    // C code:
    // int add(int a, int b) {
    //     return a + b;
    // }

    let func = create_test_function(
        "add",
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
        HirType::Int,
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }))],
    );

    let detector = OutputParamDetector::new();
    let output_params = detector.detect(&func);

    assert_eq!(
        output_params.len(),
        0,
        "Non-pointer parameters should not be detected"
    );
}

// ============================================================================
// TEST 9: Pointer parameter read but never written
// ============================================================================

#[test]
fn test_pointer_read_only_not_output() {
    // C code:
    // int read_value(int* ptr) {
    //     return *ptr;
    // }

    let func = create_test_function(
        "read_value",
        vec![create_pointer_param("ptr")],
        HirType::Int,
        vec![HirStatement::Return(Some(HirExpression::Dereference(
            Box::new(HirExpression::Variable("ptr".to_string())),
        )))],
    );

    let detector = OutputParamDetector::new();
    let output_params = detector.detect(&func);

    assert_eq!(
        output_params.len(),
        0,
        "Read-only pointer should not be output"
    );
}

// ============================================================================
// TEST 10: Double pointer (common for handles/resources)
// ============================================================================

#[test]
fn test_double_pointer_output() {
    // C code:
    // int create_object(Object** obj) {
    //     *obj = malloc(sizeof(Object));
    //     return 0;
    // }

    let func = create_test_function(
        "create_object",
        vec![HirParameter::new(
            "obj".to_string(),
            HirType::Pointer(Box::new(HirType::Pointer(Box::new(HirType::Struct(
                "Object".to_string(),
            ))))),
        )],
        HirType::Int,
        vec![
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("obj".to_string()),
                value: HirExpression::Malloc {
                    size: Box::new(HirExpression::Sizeof {
                        type_name: "Object".to_string(),
                    }),
                },
            },
            HirStatement::Return(Some(HirExpression::IntLiteral(0))),
        ],
    );

    let detector = OutputParamDetector::new();
    let output_params = detector.detect(&func);

    assert_eq!(output_params.len(), 1);
    assert_eq!(output_params[0].name, "obj");
    assert_eq!(output_params[0].kind, ParameterKind::Output);
}

#[test]
fn test_output_param_detector_default() {
    let detector = OutputParamDetector::default();
    let func = create_test_function("empty", vec![], HirType::Void, vec![]);
    let params = detector.detect(&func);
    assert!(params.is_empty());
}

#[test]
fn test_detect_no_parameters() {
    let func = create_test_function(
        "noop",
        vec![],
        HirType::Void,
        vec![HirStatement::Return(None)],
    );
    let detector = OutputParamDetector::new();
    assert!(detector.detect(&func).is_empty());
}

#[test]
fn test_detect_non_pointer_only_params() {
    let func = create_test_function(
        "add",
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
        HirType::Int,
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }))],
    );
    let detector = OutputParamDetector::new();
    assert!(detector.detect(&func).is_empty());
}
