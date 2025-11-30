//! DECY-161: Tests for array parameter with pointer arithmetic.
//!
//! When an array parameter uses pointer arithmetic (arr++, arr = arr + n),
//! it must stay as a raw pointer, NOT be transformed to a slice.
//! Slices don't support pointer arithmetic.

use decy_hir::{BinaryOperator, HirExpression, HirFunction, HirParameter, HirStatement, HirType};

/// Create a function that uses pointer arithmetic on array parameter.
/// C equivalent:
/// ```c
/// int sum_array(int* arr, int size) {
///     int sum = 0;
///     int* end = arr + size;
///     while (arr < end) {
///         sum += *arr;
///         arr++;  // Pointer arithmetic - arr must be raw pointer!
///     }
///     return sum;
/// }
/// ```
fn create_array_ptr_arithmetic_function() -> HirFunction {
    HirFunction::new_with_body(
        "sum_array".to_string(),
        HirType::Int,
        vec![
            HirParameter::new(
                "arr".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            ),
            HirParameter::new("size".to_string(), HirType::Int),
        ],
        vec![
            // int sum = 0;
            HirStatement::VariableDeclaration {
                name: "sum".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            // int* end = arr + size;
            HirStatement::VariableDeclaration {
                name: "end".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("arr".to_string())),
                    right: Box::new(HirExpression::Variable("size".to_string())),
                }),
            },
            // while (arr < end) { ... arr++; }
            HirStatement::While {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::LessThan,
                    left: Box::new(HirExpression::Variable("arr".to_string())),
                    right: Box::new(HirExpression::Variable("end".to_string())),
                },
                body: vec![
                    // sum += *arr;
                    HirStatement::Assignment {
                        target: "sum".to_string(),
                        value: HirExpression::BinaryOp {
                            op: BinaryOperator::Add,
                            left: Box::new(HirExpression::Variable("sum".to_string())),
                            right: Box::new(HirExpression::Dereference(Box::new(
                                HirExpression::Variable("arr".to_string()),
                            ))),
                        },
                    },
                    // arr++; (becomes arr = arr + 1)
                    HirStatement::Assignment {
                        target: "arr".to_string(),
                        value: HirExpression::BinaryOp {
                            op: BinaryOperator::Add,
                            left: Box::new(HirExpression::Variable("arr".to_string())),
                            right: Box::new(HirExpression::IntLiteral(1)),
                        },
                    },
                ],
            },
            // return sum;
            HirStatement::Return(Some(HirExpression::Variable("sum".to_string()))),
        ],
    )
}

#[test]
fn test_array_param_with_ptr_arithmetic_stays_raw_pointer() {
    // DECY-161: When array param uses pointer arithmetic, it must stay as *mut T
    // NOT be transformed to &[T] slice (slices don't support arr++ or arr + n)
    let func = create_array_ptr_arithmetic_function();

    let generator = decy_codegen::CodeGenerator::new();
    let rust_code = generator.generate_function(&func);

    println!("Generated code:\n{}", rust_code);

    // Should have raw pointer in signature, NOT slice
    assert!(
        rust_code.contains("*mut i32") || rust_code.contains("*const i32"),
        "Array param with pointer arithmetic should stay as raw pointer. Got:\n{}",
        rust_code
    );

    // Should NOT contain slice syntax for arr parameter
    assert!(
        !rust_code.contains("arr: &[i32]") && !rust_code.contains("arr: &mut [i32]"),
        "Array param with pointer arithmetic should NOT be slice. Got:\n{}",
        rust_code
    );
}

#[test]
fn test_array_param_with_ptr_arithmetic_uses_mut() {
    // When pointer arithmetic is used on arr, the signature should have mut
    // because arr is reassigned (arr = arr + 1)
    let func = create_array_ptr_arithmetic_function();

    let generator = decy_codegen::CodeGenerator::new();
    let rust_code = generator.generate_function(&func);

    println!("Generated code:\n{}", rust_code);

    // Should have 'mut arr' since arr is reassigned
    assert!(
        rust_code.contains("mut arr: *mut"),
        "Array param with pointer arithmetic should be 'mut arr: *mut'. Got:\n{}",
        rust_code
    );
}
