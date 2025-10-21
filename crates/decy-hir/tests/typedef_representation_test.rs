//! HIR tests for typedef representation (DECY-023 RED phase)
//!
//! This test suite verifies that HirTypedef correctly represents C typedef declarations.
//!
//! References:
//! - K&R §6.7: Type Names
//! - ISO C99 §6.7.7: Type definitions

use decy_hir::{HirType, HirTypedef};

#[test]
fn test_typedef_simple_int() {
    // Test creating a simple typedef
    let typedef = HirTypedef::new("MyInt".to_string(), HirType::Int);

    assert_eq!(typedef.name(), "MyInt");
    assert_eq!(typedef.underlying_type(), &HirType::Int);
}

#[test]
fn test_typedef_float() {
    let typedef = HirTypedef::new("MyFloat".to_string(), HirType::Float);

    assert_eq!(typedef.name(), "MyFloat");
    assert_eq!(typedef.underlying_type(), &HirType::Float);
}

#[test]
fn test_typedef_pointer() {
    let typedef = HirTypedef::new(
        "IntPtr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );

    assert_eq!(typedef.name(), "IntPtr");

    match typedef.underlying_type() {
        HirType::Pointer(inner) => {
            assert_eq!(**inner, HirType::Int);
        }
        _ => panic!("Expected pointer type"),
    }
}

#[test]
#[ignore]
fn test_typedef_struct() {
    // RED: Test typedef for struct type
    // This will fail until we add struct typedef support
    let typedef = HirTypedef::new(
        "Point".to_string(),
        HirType::Struct("Point".to_string()),
    );

    assert_eq!(typedef.name(), "Point");

    match typedef.underlying_type() {
        HirType::Struct(name) => {
            assert_eq!(name, "Point");
        }
        _ => panic!("Expected struct type"),
    }
}

#[test]
#[ignore]
fn test_typedef_function_pointer() {
    // RED: Test typedef for function pointer
    // This will fail until we add function pointer type support
    let typedef = HirTypedef::new(
        "Callback".to_string(),
        HirType::FunctionPointer {
            param_types: vec![HirType::Int, HirType::Int],
            return_type: Box::new(HirType::Int),
        },
    );

    assert_eq!(typedef.name(), "Callback");

    match typedef.underlying_type() {
        HirType::FunctionPointer { param_types, return_type } => {
            assert_eq!(param_types.len(), 2);
            assert_eq!(param_types[0], HirType::Int);
            assert_eq!(param_types[1], HirType::Int);
            assert_eq!(**return_type, HirType::Int);
        }
        _ => panic!("Expected function pointer type"),
    }
}

#[test]
fn test_typedef_char_pointer() {
    let typedef = HirTypedef::new(
        "String".to_string(),
        HirType::Pointer(Box::new(HirType::Char)),
    );

    assert_eq!(typedef.name(), "String");

    match typedef.underlying_type() {
        HirType::Pointer(inner) => {
            assert_eq!(**inner, HirType::Char);
        }
        _ => panic!("Expected pointer type"),
    }
}

#[test]
#[ignore]
fn test_typedef_array() {
    // RED: Test typedef for array type
    // This will fail until we add array typedef support
    let typedef = HirTypedef::new(
        "IntArray".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );

    assert_eq!(typedef.name(), "IntArray");

    match typedef.underlying_type() {
        HirType::Array { element_type, size } => {
            assert_eq!(**element_type, HirType::Int);
            assert_eq!(*size, Some(10));
        }
        _ => panic!("Expected array type"),
    }
}

#[test]
fn test_typedef_multiple() {
    // Test creating multiple typedefs
    let typedef1 = HirTypedef::new("Int32".to_string(), HirType::Int);
    let typedef2 = HirTypedef::new("Float32".to_string(), HirType::Float);
    let typedef3 = HirTypedef::new("Int64".to_string(), HirType::Int);

    assert_eq!(typedef1.name(), "Int32");
    assert_eq!(typedef2.name(), "Float32");
    assert_eq!(typedef3.name(), "Int64");
}

#[test]
fn test_typedef_name_preservation() {
    // Test that typedef names are preserved exactly
    let names = vec!["MyType", "my_type", "MY_TYPE", "Type123", "T"];

    for name in names {
        let typedef = HirTypedef::new(name.to_string(), HirType::Int);
        assert_eq!(typedef.name(), name, "Name should be preserved exactly");
    }
}

#[test]
#[ignore]
fn test_typedef_unsigned() {
    // RED: Test typedef for unsigned type
    // This will fail until we add unsigned type support
    // For now, use Int as a placeholder
    let typedef = HirTypedef::new("uint".to_string(), HirType::Int);

    assert_eq!(typedef.name(), "uint");
    assert_eq!(typedef.underlying_type(), &HirType::Int);
}
