//! Tests for function pointer code generation (DECY-024 RED phase).

use super::*;
use decy_hir::{HirType, HirTypedef};

#[test]
fn test_generate_simple_function_pointer_type() {
    // C: int (*func_ptr)(int, int);
    // Rust: fn(i32, i32) -> i32
    let func_ptr_type = HirType::FunctionPointer {
        param_types: vec![HirType::Int, HirType::Int],
        return_type: Box::new(HirType::Int),
    };

    let code = CodeGenerator::map_type(&func_ptr_type);

    assert!(code.contains("fn("));
    assert!(code.contains("i32"));
    assert!(code.contains("->"));
}

#[test]
fn test_generate_void_callback_type() {
    // C: void (*callback)(void);
    // Rust: fn()
    let func_ptr_type = HirType::FunctionPointer {
        param_types: vec![],
        return_type: Box::new(HirType::Void),
    };

    let code = CodeGenerator::map_type(&func_ptr_type);

    assert_eq!(code, "fn()");
}

#[test]
fn test_generate_single_param_handler() {
    // C: void (*handler)(int);
    // Rust: fn(i32)
    let func_ptr_type = HirType::FunctionPointer {
        param_types: vec![HirType::Int],
        return_type: Box::new(HirType::Void),
    };

    let code = CodeGenerator::map_type(&func_ptr_type);

    assert!(code.contains("fn(i32)"));
}

#[test]
fn test_generate_function_pointer_with_pointer_param() {
    // C: void (*callback)(int*);
    // Rust: fn(*mut i32)
    let func_ptr_type = HirType::FunctionPointer {
        param_types: vec![HirType::Pointer(Box::new(HirType::Int))],
        return_type: Box::new(HirType::Void),
    };

    let code = CodeGenerator::map_type(&func_ptr_type);

    assert!(code.contains("fn(*mut i32)"));
}

#[test]
fn test_generate_function_pointer_typedef() {
    // C: typedef int (*BinaryOp)(int, int);
    // Rust: type BinaryOp = fn(i32, i32) -> i32;
    let codegen = CodeGenerator::new();

    let typedef = HirTypedef::new(
        "BinaryOp".to_string(),
        HirType::FunctionPointer {
            param_types: vec![HirType::Int, HirType::Int],
            return_type: Box::new(HirType::Int),
        },
    );

    let code = codegen.generate_typedef(&typedef);

    assert!(code.contains("type BinaryOp = fn(i32, i32) -> i32"));
    assert!(code.ends_with(';'));
}

#[test]
fn test_generate_function_pointer_with_multiple_params() {
    // C: float (*compute)(int, float, double);
    // Rust: fn(i32, f32, f64) -> f32
    let func_ptr_type = HirType::FunctionPointer {
        param_types: vec![HirType::Int, HirType::Float, HirType::Double],
        return_type: Box::new(HirType::Float),
    };

    let code = CodeGenerator::map_type(&func_ptr_type);

    assert!(code.contains("fn(i32, f32, f64) -> f32"));
}

#[test]
fn test_generate_function_pointer_returning_pointer() {
    // C: int* (*factory)(void);
    // Rust: fn() -> *mut i32
    let func_ptr_type = HirType::FunctionPointer {
        param_types: vec![],
        return_type: Box::new(HirType::Pointer(Box::new(HirType::Int))),
    };

    let code = CodeGenerator::map_type(&func_ptr_type);

    assert!(code.contains("fn() -> *mut i32"));
}

#[test]
fn test_generate_nested_function_pointer() {
    // C: int (*(*higher_order)(int))(float);
    // Rust: fn(i32) -> fn(f32) -> i32
    let inner_func = HirType::FunctionPointer {
        param_types: vec![HirType::Float],
        return_type: Box::new(HirType::Int),
    };

    let outer_func = HirType::FunctionPointer {
        param_types: vec![HirType::Int],
        return_type: Box::new(inner_func),
    };

    let code = CodeGenerator::map_type(&outer_func);

    assert!(code.contains("fn(i32) -> fn(f32) -> i32"));
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Function pointer code generation never panics
        #[test]
        fn property_function_pointer_never_panics(
            param_count in 0usize..5,
        ) {
            let param_types: Vec<HirType> = (0..param_count)
                .map(|_| HirType::Int)
                .collect();

            let func_ptr = HirType::FunctionPointer {
                param_types,
                return_type: Box::new(HirType::Int),
            };

            let _code = CodeGenerator::map_type(&func_ptr);
            // If we get here without panic, test passes
        }

        /// Property: Generated code always starts with "fn("
        #[test]
        fn property_always_starts_with_fn(
            param_count in 0usize..5,
        ) {
            let param_types: Vec<HirType> = (0..param_count)
                .map(|_| HirType::Int)
                .collect();

            let func_ptr = HirType::FunctionPointer {
                param_types,
                return_type: Box::new(HirType::Int),
            };

            let code = CodeGenerator::map_type(&func_ptr);
            prop_assert!(code.starts_with("fn("));
        }

        /// Property: Non-void return types always contain "->"
        #[test]
        fn property_non_void_has_arrow(
            param_count in 0usize..5,
        ) {
            let param_types: Vec<HirType> = (0..param_count)
                .map(|_| HirType::Int)
                .collect();

            let func_ptr = HirType::FunctionPointer {
                param_types,
                return_type: Box::new(HirType::Int),
            };

            let code = CodeGenerator::map_type(&func_ptr);
            prop_assert!(code.contains("->"));
        }

        /// Property: Void return types never contain "->"
        #[test]
        fn property_void_no_arrow(
            param_count in 0usize..5,
        ) {
            let param_types: Vec<HirType> = (0..param_count)
                .map(|_| HirType::Int)
                .collect();

            let func_ptr = HirType::FunctionPointer {
                param_types,
                return_type: Box::new(HirType::Void),
            };

            let code = CodeGenerator::map_type(&func_ptr);
            prop_assert!(!code.contains("->"));
        }

        /// Property: Parameter count matches comma count + 1 (when params > 0)
        #[test]
        fn property_param_count_matches_commas(
            param_count in 1usize..5,
        ) {
            let param_types: Vec<HirType> = (0..param_count)
                .map(|_| HirType::Int)
                .collect();

            let func_ptr = HirType::FunctionPointer {
                param_types,
                return_type: Box::new(HirType::Void),
            };

            let code = CodeGenerator::map_type(&func_ptr);
            let comma_count = code.matches(',').count();

            // For n parameters, we expect n-1 commas
            prop_assert_eq!(comma_count, param_count - 1);
        }

        /// Property: Generated code is always valid Rust syntax (contains balanced parens)
        #[test]
        fn property_balanced_parens(
            param_count in 0usize..5,
        ) {
            let param_types: Vec<HirType> = (0..param_count)
                .map(|_| HirType::Int)
                .collect();

            let func_ptr = HirType::FunctionPointer {
                param_types,
                return_type: Box::new(HirType::Int),
            };

            let code = CodeGenerator::map_type(&func_ptr);
            let open_count = code.matches('(').count();
            let close_count = code.matches(')').count();

            prop_assert_eq!(open_count, close_count);
        }
    }
}
