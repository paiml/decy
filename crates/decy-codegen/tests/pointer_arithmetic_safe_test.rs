//! Tests for pointer arithmetic → slice indexing transformation (EXPR-ARITH-PTR validation)
//!
//! Reference: K&R §5.3, ISO C99 §6.5.6
//!
//! This module tests the transformation of C pointer arithmetic to safe Rust slice indexing.
//! Pointer arithmetic in C (ptr + offset) should be transformed to safe slice indexing (&arr[i + offset])
//! through ownership inference, eliminating the need for unsafe blocks.
//!
//! CRITICAL: This transformation is the most important for achieving <5 unsafe blocks per 1000 LOC.

use decy_codegen::CodeGenerator;
use decy_hir::{BinaryOperator, HirExpression, HirFunction, HirParameter, HirStatement, HirType};

/// Test simple pointer addition → slice indexing
///
/// C: int* p = arr;
///    int x = *(p + 1);
///
/// Rust: let p = &arr[0..];
///       let x = p[1];  // OR arr[base_index + 1]
///
/// Reference: K&R §5.3, ISO C99 §6.5.6
#[test]
#[ignore = "RED phase: Feature not yet implemented - pointer arithmetic → slice indexing"]
fn test_pointer_addition_to_slice_index() {
    let codegen = CodeGenerator::new();

    // Function that dereferences pointer + offset
    // int get_next(int* p) { return *(p + 1); }
    let func = HirFunction::new_with_body(
        "get_next".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::Dereference(
            Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("p".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            }),
        )))],
    );

    let result = codegen.generate_function(&func);

    println!("Generated code:\n{}", result);

    // Verify NO unsafe blocks
    assert!(
        !result.contains("unsafe"),
        "Pointer arithmetic should NOT generate unsafe blocks"
    );

    // Verify slice indexing pattern (either p[1] or similar safe access)
    // The exact syntax depends on how ownership inference represents this
    // but it should NOT contain wrapping_add or offset_from
    assert!(
        !result.contains("wrapping_add"),
        "Should not use wrapping_add (unsafe pointer arithmetic)"
    );
    assert!(
        !result.contains("offset_from"),
        "Should not use offset_from (unsafe pointer arithmetic)"
    );

    // Should use safe slice indexing (exact format TBD by implementation)
    // This test will FAIL initially (RED phase) until we implement the transformation
}

/// Test pointer subtraction → slice indexing
///
/// C: int* p = arr + 5;
///    int x = *(p - 2);
///
/// Rust: let p = &arr[5..];
///       let x = p[0 - 2];  // OR arr[base_index - 2]
///
/// Reference: K&R §5.3, ISO C99 §6.5.6
#[test]
#[ignore = "RED phase: Feature not yet implemented - pointer arithmetic → slice indexing"]
fn test_pointer_subtraction_to_slice_index() {
    let codegen = CodeGenerator::new();

    let func = HirFunction::new_with_body(
        "get_prev".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::Dereference(
            Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Subtract,
                left: Box::new(HirExpression::Variable("p".to_string())),
                right: Box::new(HirExpression::IntLiteral(2)),
            }),
        )))],
    );

    let result = codegen.generate_function(&func);

    println!("Generated code:\n{}", result);

    // Verify NO unsafe blocks
    assert!(
        !result.contains("unsafe"),
        "Pointer subtraction should NOT generate unsafe blocks"
    );
    assert!(
        !result.contains("wrapping_sub"),
        "Should not use wrapping_sub (unsafe pointer arithmetic)"
    );
}

/// Test pointer difference (ptr - ptr) → length calculation
///
/// C: int* start = arr;
///    int* end = arr + 10;
///    int len = end - start;
///
/// Rust: let start = &arr[0];
///       let end = &arr[10];
///       let len = 10 - 0;  // OR end_index - start_index
///
/// Reference: K&R §5.3, ISO C99 §6.5.6
#[test]
#[ignore = "RED phase: Feature not yet implemented - pointer arithmetic → slice indexing"]
fn test_pointer_difference_to_index_difference() {
    let codegen = CodeGenerator::new();

    // Function that computes pointer difference
    // int distance(int* start, int* end) { return end - start; }
    let func = HirFunction::new_with_body(
        "distance".to_string(),
        HirType::Int,
        vec![
            HirParameter::new(
                "start".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            ),
            HirParameter::new("end".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        ],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Subtract,
            left: Box::new(HirExpression::Variable("end".to_string())),
            right: Box::new(HirExpression::Variable("start".to_string())),
        }))],
    );

    let result = codegen.generate_function(&func);

    println!("Generated code:\n{}", result);

    // Verify NO unsafe blocks
    assert!(
        !result.contains("unsafe"),
        "Pointer difference should NOT generate unsafe blocks"
    );
    assert!(
        !result.contains("offset_from"),
        "Should not use offset_from (unsafe pointer arithmetic)"
    );
}

/// Test pointer arithmetic in array access
///
/// C: int get_element(int* arr, int base, int offset) {
///        return arr[base + offset];
///    }
///
/// Rust: fn get_element(arr: &[i32], base: i32, offset: i32) -> i32 {
///         arr[(base + offset) as usize]
///       }
///
/// Reference: K&R §5.3, ISO C99 §6.5.6
#[test]
#[ignore = "RED phase: Feature not yet implemented - pointer arithmetic → slice indexing"]
fn test_pointer_array_access_with_arithmetic() {
    let codegen = CodeGenerator::new();

    let func = HirFunction::new_with_body(
        "get_element".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("arr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("base".to_string(), HirType::Int),
            HirParameter::new("offset".to_string(), HirType::Int),
        ],
        vec![HirStatement::Return(Some(HirExpression::ArrayIndex {
            array: Box::new(HirExpression::Variable("arr".to_string())),
            index: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("base".to_string())),
                right: Box::new(HirExpression::Variable("offset".to_string())),
            }),
        }))],
    );

    let result = codegen.generate_function(&func);

    println!("Generated code:\n{}", result);

    // Verify NO unsafe blocks
    assert!(
        !result.contains("unsafe"),
        "Array access with arithmetic should NOT generate unsafe blocks"
    );

    // Verify safe array indexing
    assert!(
        result.contains("[") && result.contains("]"),
        "Should use safe slice indexing syntax"
    );
}

/// Test multiple pointer arithmetic operations in sequence
///
/// C: int sum_range(int* arr, int start, int end) {
///        int sum = 0;
///        for (int* p = arr + start; p < arr + end; p++) {
///            sum += *p;
///        }
///        return sum;
///    }
///
/// Rust: fn sum_range(arr: &[i32], start: usize, end: usize) -> i32 {
///         let mut sum = 0;
///         for i in start..end {
///             sum += arr[i];
///         }
///         sum
///       }
///
/// Reference: K&R §5.3, ISO C99 §6.5.6
#[test]
#[ignore = "RED phase: Feature not yet implemented - pointer arithmetic → slice indexing"]
fn test_pointer_arithmetic_in_loop() {
    let codegen = CodeGenerator::new();

    // Simplified version: just the pointer arithmetic part
    // int* p = arr + start;
    // This is a variable declaration with pointer arithmetic initialization
    let _init_expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::Variable("start".to_string())),
    };

    // For now, test that individual pointer arithmetic operations don't produce unsafe
    let func = HirFunction::new_with_body(
        "get_offset".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![
            HirParameter::new("arr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("offset".to_string(), HirType::Int),
        ],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("arr".to_string())),
            right: Box::new(HirExpression::Variable("offset".to_string())),
        }))],
    );

    let result = codegen.generate_function(&func);

    println!("Generated code:\n{}", result);

    // Verify NO unsafe blocks
    assert!(
        !result.contains("unsafe"),
        "Pointer arithmetic should NOT generate unsafe blocks"
    );
    assert!(
        !result.contains("wrapping_add"),
        "Should not use unsafe pointer methods"
    );
}

/// Test pointer arithmetic with negative offset
///
/// C: int get_before(int* p, int offset) {
///        return *(p - offset);
///    }
///
/// Rust: fn get_before(arr: &[i32], index: usize, offset: usize) -> i32 {
///         arr[index - offset]
///       }
///
/// Reference: K&R §5.3, ISO C99 §6.5.6
#[test]
#[ignore = "RED phase: Feature not yet implemented - pointer arithmetic → slice indexing"]
fn test_pointer_arithmetic_negative_offset() {
    let codegen = CodeGenerator::new();

    let func = HirFunction::new_with_body(
        "get_before".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("offset".to_string(), HirType::Int),
        ],
        vec![HirStatement::Return(Some(HirExpression::Dereference(
            Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Subtract,
                left: Box::new(HirExpression::Variable("p".to_string())),
                right: Box::new(HirExpression::Variable("offset".to_string())),
            }),
        )))],
    );

    let result = codegen.generate_function(&func);

    println!("Generated code:\n{}", result);

    // Verify NO unsafe blocks
    assert!(
        !result.contains("unsafe"),
        "Pointer arithmetic with negative offset should NOT generate unsafe blocks"
    );
    assert!(
        !result.contains("wrapping_sub"),
        "Should not use unsafe pointer methods"
    );
}

/// Test pointer arithmetic with multiplication (array of structs)
///
/// C: struct Point { int x; int y; };
///    struct Point* get_nth(struct Point* arr, int n) {
///        return arr + n;  // Implicit sizeof(Point) multiplication
///    }
///
/// Rust: fn get_nth(arr: &[Point], n: usize) -> &Point {
///         &arr[n]
///       }
///
/// Reference: K&R §5.3, ISO C99 §6.5.6
#[test]
#[ignore = "RED phase: Feature not yet implemented - pointer arithmetic → slice indexing"]
fn test_pointer_arithmetic_struct_array() {
    let codegen = CodeGenerator::new();

    let func = HirFunction::new_with_body(
        "get_nth".to_string(),
        HirType::Pointer(Box::new(HirType::Struct("Point".to_string()))),
        vec![
            HirParameter::new(
                "arr".to_string(),
                HirType::Pointer(Box::new(HirType::Struct("Point".to_string()))),
            ),
            HirParameter::new("n".to_string(), HirType::Int),
        ],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("arr".to_string())),
            right: Box::new(HirExpression::Variable("n".to_string())),
        }))],
    );

    let result = codegen.generate_function(&func);

    println!("Generated code:\n{}", result);

    // Verify NO unsafe blocks
    assert!(
        !result.contains("unsafe"),
        "Struct array pointer arithmetic should NOT generate unsafe blocks"
    );
    assert!(
        !result.contains("wrapping_add"),
        "Should not use unsafe pointer methods"
    );
}

/// Verify unsafe block count remains 0
///
/// This is critical for the validation goal: <5 unsafe blocks per 1000 LOC
#[test]
#[ignore = "RED phase: Feature not yet implemented - pointer arithmetic → slice indexing"]
fn test_pointer_arithmetic_transformation_unsafe_count() {
    let codegen = CodeGenerator::new();

    // Test multiple pointer arithmetic scenarios
    let test_cases = [
        // ptr + offset
        HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("p".to_string())),
            right: Box::new(HirExpression::IntLiteral(5)),
        },
        // ptr - offset
        HirExpression::BinaryOp {
            op: BinaryOperator::Subtract,
            left: Box::new(HirExpression::Variable("p".to_string())),
            right: Box::new(HirExpression::IntLiteral(3)),
        },
    ];

    let mut combined_code = String::new();

    for (i, expr) in test_cases.iter().enumerate() {
        let func = HirFunction::new_with_body(
            format!("test_{}", i),
            HirType::Pointer(Box::new(HirType::Int)),
            vec![HirParameter::new(
                "p".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            )],
            vec![HirStatement::Return(Some(expr.clone()))],
        );

        let code = codegen.generate_function(&func);
        combined_code.push_str(&code);
        combined_code.push('\n');
    }

    // Count unsafe blocks (should be 0)
    let unsafe_count = combined_code.matches("unsafe").count();
    assert_eq!(
        unsafe_count, 0,
        "Pointer arithmetic → slice indexing transformation should introduce 0 unsafe blocks. Found {} unsafe blocks.",
        unsafe_count
    );
}

/// Test that pointer arithmetic now uses safe methods (wrapping_add)
///
/// When a parameter uses pointer arithmetic (e.g., p = p + n), it stays as a raw pointer
/// and pointer arithmetic should use wrapping_add (a safe method on raw pointers).
#[test]
fn test_pointer_arithmetic_expression_is_safe() {
    let codegen = CodeGenerator::new();

    // Create a function that uses pointer arithmetic (p = p + 1) to ensure p stays raw pointer
    let func = HirFunction::new_with_body(
        "ptr_advance".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![
            // p = p + 1; (this forces p to stay as raw pointer)
            HirStatement::Assignment {
                target: "p".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("p".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            },
            HirStatement::Return(Some(HirExpression::Variable("p".to_string()))),
        ],
    );

    let result = codegen.generate_function(&func);

    // Pointer arithmetic with wrapping_add is safe - no unsafe block needed for the arithmetic itself
    assert!(
        result.contains("wrapping_add"),
        "Should use wrapping_add for pointer arithmetic: {}",
        result
    );
}
