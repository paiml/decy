//! HIR tests for function pointer representation (DECY-024 RED phase)
//!
//! This test suite verifies that function pointer types are correctly represented in HIR.
//!
//! References:
//! - K&R §5.11: Pointers to Functions
//! - ISO C99 §6.7.5.3: Function declarators

use decy_hir::{HirType, HirParameter};

#[test]
fn test_function_pointer_simple() {
    // Test creating a simple function pointer type
    let fn_ptr = HirType::FunctionPointer {
        param_types: vec![HirType::Int],
        return_type: Box::new(HirType::Int),
    };

    match fn_ptr {
        HirType::FunctionPointer { param_types, return_type } => {
            assert_eq!(param_types.len(), 1);
            assert_eq!(param_types[0], HirType::Int);
            assert_eq!(*return_type, HirType::Int);
        }
        _ => panic!("Expected FunctionPointer type"),
    }
}

#[test]
fn test_function_pointer_multiple_params() {
    // Test function pointer with multiple parameters
    let fn_ptr = HirType::FunctionPointer {
        param_types: vec![HirType::Int, HirType::Int],
        return_type: Box::new(HirType::Int),
    };

    match fn_ptr {
        HirType::FunctionPointer { param_types, .. } => {
            assert_eq!(param_types.len(), 2);
        }
        _ => panic!("Expected FunctionPointer type"),
    }
}

#[test]
fn test_function_pointer_void_return() {
    // Test function pointer with void return type
    let fn_ptr = HirType::FunctionPointer {
        param_types: vec![HirType::Int],
        return_type: Box::new(HirType::Void),
    };

    match fn_ptr {
        HirType::FunctionPointer { return_type, .. } => {
            assert_eq!(*return_type, HirType::Void);
        }
        _ => panic!("Expected FunctionPointer type"),
    }
}

#[test]
fn test_function_pointer_no_params() {
    // Test function pointer with no parameters
    let fn_ptr = HirType::FunctionPointer {
        param_types: vec![],
        return_type: Box::new(HirType::Int),
    };

    match fn_ptr {
        HirType::FunctionPointer { param_types, .. } => {
            assert_eq!(param_types.len(), 0);
        }
        _ => panic!("Expected FunctionPointer type"),
    }
}

#[test]
fn test_function_pointer_as_parameter() {
    // Test function with function pointer parameter
    let callback_type = HirType::FunctionPointer {
        param_types: vec![HirType::Int],
        return_type: Box::new(HirType::Int),
    };

    let callback_param = HirParameter::new("callback".to_string(), callback_type.clone());

    assert_eq!(callback_param.name(), "callback");

    match callback_param.param_type() {
        HirType::FunctionPointer { param_types, return_type } => {
            assert_eq!(param_types.len(), 1);
            assert_eq!(return_type.as_ref(), &HirType::Int);
        }
        _ => panic!("Expected FunctionPointer type"),
    }
}

#[test]
fn test_function_pointer_with_float_types() {
    // Test function pointer with float types
    let fn_ptr = HirType::FunctionPointer {
        param_types: vec![HirType::Float, HirType::Float],
        return_type: Box::new(HirType::Float),
    };

    match fn_ptr {
        HirType::FunctionPointer { param_types, return_type } => {
            assert_eq!(param_types[0], HirType::Float);
            assert_eq!(param_types[1], HirType::Float);
            assert_eq!(*return_type, HirType::Float);
        }
        _ => panic!("Expected FunctionPointer type"),
    }
}

#[test]
fn test_function_pointer_with_pointer_params() {
    // Test function pointer with pointer parameters
    let fn_ptr = HirType::FunctionPointer {
        param_types: vec![
            HirType::Pointer(Box::new(HirType::Int)),
            HirType::Pointer(Box::new(HirType::Char)),
        ],
        return_type: Box::new(HirType::Void),
    };

    match fn_ptr {
        HirType::FunctionPointer { param_types, .. } => {
            assert_eq!(param_types.len(), 2);

            match &param_types[0] {
                HirType::Pointer(inner) => assert_eq!(**inner, HirType::Int),
                _ => panic!("Expected pointer type"),
            }

            match &param_types[1] {
                HirType::Pointer(inner) => assert_eq!(**inner, HirType::Char),
                _ => panic!("Expected pointer type"),
            }
        }
        _ => panic!("Expected FunctionPointer type"),
    }
}

#[test]
fn test_function_pointer_equality() {
    // Test that identical function pointer types are equal
    let fn_ptr1 = HirType::FunctionPointer {
        param_types: vec![HirType::Int],
        return_type: Box::new(HirType::Int),
    };

    let fn_ptr2 = HirType::FunctionPointer {
        param_types: vec![HirType::Int],
        return_type: Box::new(HirType::Int),
    };

    assert_eq!(fn_ptr1, fn_ptr2);
}

#[test]
fn test_function_pointer_inequality_params() {
    // Test that function pointers with different params are not equal
    let fn_ptr1 = HirType::FunctionPointer {
        param_types: vec![HirType::Int],
        return_type: Box::new(HirType::Int),
    };

    let fn_ptr2 = HirType::FunctionPointer {
        param_types: vec![HirType::Int, HirType::Int],
        return_type: Box::new(HirType::Int),
    };

    assert_ne!(fn_ptr1, fn_ptr2);
}

#[test]
fn test_function_pointer_inequality_return() {
    // Test that function pointers with different return types are not equal
    let fn_ptr1 = HirType::FunctionPointer {
        param_types: vec![HirType::Int],
        return_type: Box::new(HirType::Int),
    };

    let fn_ptr2 = HirType::FunctionPointer {
        param_types: vec![HirType::Int],
        return_type: Box::new(HirType::Void),
    };

    assert_ne!(fn_ptr1, fn_ptr2);
}
