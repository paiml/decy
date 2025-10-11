//! Tests for function pointer support in HIR (DECY-024 RED phase).

use super::*;

#[test]
fn test_create_function_pointer_type() {
    // C: int (*func_ptr)(int, int);
    // Rust: fn(i32, i32) -> i32
    let func_ptr_type = HirType::FunctionPointer {
        param_types: vec![HirType::Int, HirType::Int],
        return_type: Box::new(HirType::Int),
    };

    match func_ptr_type {
        HirType::FunctionPointer {
            param_types,
            return_type,
        } => {
            assert_eq!(param_types.len(), 2);
            assert_eq!(param_types[0], HirType::Int);
            assert_eq!(param_types[1], HirType::Int);
            assert_eq!(*return_type, HirType::Int);
        }
        _ => panic!("Expected FunctionPointer type"),
    }
}

#[test]
fn test_function_pointer_void_return() {
    // C: void (*callback)(void);
    // Rust: fn()
    let func_ptr_type = HirType::FunctionPointer {
        param_types: vec![],
        return_type: Box::new(HirType::Void),
    };

    match func_ptr_type {
        HirType::FunctionPointer {
            param_types,
            return_type,
        } => {
            assert_eq!(param_types.len(), 0);
            assert_eq!(*return_type, HirType::Void);
        }
        _ => panic!("Expected FunctionPointer type"),
    }
}

#[test]
fn test_function_pointer_single_param() {
    // C: void (*handler)(int);
    // Rust: fn(i32)
    let func_ptr_type = HirType::FunctionPointer {
        param_types: vec![HirType::Int],
        return_type: Box::new(HirType::Void),
    };

    match func_ptr_type {
        HirType::FunctionPointer {
            param_types,
            return_type,
        } => {
            assert_eq!(param_types.len(), 1);
            assert_eq!(param_types[0], HirType::Int);
            assert_eq!(*return_type, HirType::Void);
        }
        _ => panic!("Expected FunctionPointer type"),
    }
}

#[test]
fn test_function_pointer_with_pointer_param() {
    // C: void (*callback)(int*);
    // Rust: fn(*mut i32)
    let func_ptr_type = HirType::FunctionPointer {
        param_types: vec![HirType::Pointer(Box::new(HirType::Int))],
        return_type: Box::new(HirType::Void),
    };

    match func_ptr_type {
        HirType::FunctionPointer {
            param_types,
            return_type,
        } => {
            assert_eq!(param_types.len(), 1);
            match &param_types[0] {
                HirType::Pointer(inner) => {
                    assert_eq!(**inner, HirType::Int);
                }
                _ => panic!("Expected pointer parameter"),
            }
            assert_eq!(*return_type, HirType::Void);
        }
        _ => panic!("Expected FunctionPointer type"),
    }
}

#[test]
fn test_function_pointer_typedef() {
    // C: typedef int (*BinaryOp)(int, int);
    // Rust: type BinaryOp = fn(i32, i32) -> i32;
    let typedef = HirTypedef::new(
        "BinaryOp".to_string(),
        HirType::FunctionPointer {
            param_types: vec![HirType::Int, HirType::Int],
            return_type: Box::new(HirType::Int),
        },
    );

    assert_eq!(typedef.name(), "BinaryOp");
    match typedef.underlying_type() {
        HirType::FunctionPointer {
            param_types,
            return_type,
        } => {
            assert_eq!(param_types.len(), 2);
            assert_eq!(**return_type, HirType::Int);
        }
        _ => panic!("Expected FunctionPointer type"),
    }
}

#[test]
fn test_function_pointer_variable_declaration() {
    // C: int (*add_func)(int, int) = &add;
    // Rust: let add_func: fn(i32, i32) -> i32 = add;
    let var_decl = HirStatement::VariableDeclaration {
        name: "add_func".to_string(),
        var_type: HirType::FunctionPointer {
            param_types: vec![HirType::Int, HirType::Int],
            return_type: Box::new(HirType::Int),
        },
        initializer: Some(HirExpression::Variable("add".to_string())),
    };

    match var_decl {
        HirStatement::VariableDeclaration {
            name,
            var_type,
            initializer,
        } => {
            assert_eq!(name, "add_func");
            match var_type {
                HirType::FunctionPointer { .. } => {
                    // Type is correct
                }
                _ => panic!("Expected FunctionPointer type"),
            }
            assert!(initializer.is_some());
        }
        _ => panic!("Expected VariableDeclaration"),
    }
}

#[test]
fn test_function_pointer_clone() {
    let func_ptr = HirType::FunctionPointer {
        param_types: vec![HirType::Int],
        return_type: Box::new(HirType::Void),
    };

    let cloned = func_ptr.clone();
    assert_eq!(func_ptr, cloned);
}

#[test]
fn test_function_pointer_equality() {
    let func_ptr1 = HirType::FunctionPointer {
        param_types: vec![HirType::Int, HirType::Int],
        return_type: Box::new(HirType::Int),
    };

    let func_ptr2 = HirType::FunctionPointer {
        param_types: vec![HirType::Int, HirType::Int],
        return_type: Box::new(HirType::Int),
    };

    assert_eq!(func_ptr1, func_ptr2);
}
