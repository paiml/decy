//! Tests for typedef support in HIR (DECY-023 RED phase).

use super::*;

#[test]
fn test_create_simple_typedef() {
    // typedef int Integer;
    let typedef = HirTypedef {
        name: "Integer".to_string(),
        underlying_type: HirType::Int,
    };

    assert_eq!(typedef.name(), "Integer");
    assert_eq!(typedef.underlying_type(), &HirType::Int);
}

#[test]
fn test_typedef_pointer_type() {
    // typedef int* IntPtr;
    let typedef = HirTypedef {
        name: "IntPtr".to_string(),
        underlying_type: HirType::Pointer(Box::new(HirType::Int)),
    };

    assert_eq!(typedef.name(), "IntPtr");
    match typedef.underlying_type() {
        HirType::Pointer(inner) => {
            assert_eq!(**inner, HirType::Int);
        }
        _ => panic!("Expected pointer type"),
    }
}

#[test]
fn test_typedef_struct_type() {
    // typedef struct Point Point;
    let typedef = HirTypedef {
        name: "Point".to_string(),
        underlying_type: HirType::Struct("Point".to_string()),
    };

    assert_eq!(typedef.name(), "Point");
    match typedef.underlying_type() {
        HirType::Struct(name) => {
            assert_eq!(name, "Point");
        }
        _ => panic!("Expected struct type"),
    }
}

#[test]
fn test_typedef_array_type() {
    // typedef int IntArray[10];
    let typedef = HirTypedef {
        name: "IntArray".to_string(),
        underlying_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    };

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
fn test_typedef_clone() {
    let typedef = HirTypedef {
        name: "MyInt".to_string(),
        underlying_type: HirType::Int,
    };

    let cloned = typedef.clone();
    assert_eq!(typedef, cloned);
}

#[test]
fn test_multiple_typedefs() {
    // typedef int Integer;
    // typedef Integer SignedInt;
    let typedef1 = HirTypedef {
        name: "Integer".to_string(),
        underlying_type: HirType::Int,
    };

    let typedef2 = HirTypedef {
        name: "SignedInt".to_string(),
        underlying_type: HirType::Int, // After resolution
    };

    assert_eq!(typedef1.name(), "Integer");
    assert_eq!(typedef2.name(), "SignedInt");
    assert_eq!(typedef1.underlying_type(), typedef2.underlying_type());
}
