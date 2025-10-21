//! Parser tests for typedef declarations (DECY-023 RED phase)
//!
//! This test suite follows EXTREME TDD methodology - all tests should FAIL initially.
//! Tests verify that the parser correctly extracts typedef declarations from C code.
//!
//! References:
//! - K&R §6.7: Type Names
//! - ISO C99 §6.7.7: Type definitions

use decy_parser::CParser;

#[test]
fn test_typedef_simple_int() {
    // Test that simple typedef is parsed
    let parser = CParser::new().expect("Parser creation failed");
    let source = "typedef int MyInt;";

    let ast = parser.parse(source).expect("Parsing should succeed");

    // Should have one typedef
    assert_eq!(ast.typedefs().len(), 1, "Should parse one typedef");

    let typedef = &ast.typedefs()[0];
    assert_eq!(typedef.name(), "MyInt");
    assert_eq!(typedef.underlying_type(), "int");
}

#[test]
fn test_typedef_pointer() {
    // Test that pointer typedef is parsed
    let parser = CParser::new().expect("Parser creation failed");
    let source = "typedef int* IntPtr;";

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.typedefs().len(), 1);

    let typedef = &ast.typedefs()[0];
    assert_eq!(typedef.name(), "IntPtr");
    assert!(typedef.is_pointer(), "Should be recognized as pointer type");
}

#[test]
fn test_typedef_struct() {
    // Test that struct typedef is parsed
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        typedef struct {
            int x;
            int y;
        } Point;
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.typedefs().len(), 1);

    let typedef = &ast.typedefs()[0];
    assert_eq!(typedef.name(), "Point");
    assert!(typedef.is_struct(), "Should be recognized as struct type");
}

#[test]
fn test_typedef_named_struct() {
    // Test that named struct typedef is parsed
    let parser = CParser::new().expect("Parser creation failed");
    let source = "typedef struct Point { int x; int y; } Point;";

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.typedefs().len(), 1);

    let typedef = &ast.typedefs()[0];
    assert_eq!(typedef.name(), "Point");
}

#[test]
fn test_typedef_function_pointer() {
    // Test that function pointer typedef is parsed
    let parser = CParser::new().expect("Parser creation failed");
    let source = "typedef int (*Callback)(int, int);";

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.typedefs().len(), 1);

    let typedef = &ast.typedefs()[0];
    assert_eq!(typedef.name(), "Callback");
    assert!(typedef.is_function_pointer(), "Should be recognized as function pointer");
}

#[test]
fn test_typedef_multiple_declarations() {
    // Test that multiple typedefs are parsed
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        typedef int MyInt;
        typedef float MyFloat;
        typedef char* String;
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.typedefs().len(), 3, "Should parse three typedefs");

    assert_eq!(ast.typedefs()[0].name(), "MyInt");
    assert_eq!(ast.typedefs()[1].name(), "MyFloat");
    assert_eq!(ast.typedefs()[2].name(), "String");
}

#[test]
fn test_typedef_with_function() {
    // Test that typedefs work alongside functions
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        typedef int MyInt;

        MyInt add(MyInt a, MyInt b) {
            return a + b;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.typedefs().len(), 1);
    assert_eq!(ast.functions().len(), 1);

    // Function should use the typedef
    let func = &ast.functions()[0];
    assert_eq!(func.name, "add");
}

#[test]
fn test_typedef_unsigned() {
    // Test that unsigned typedef is parsed
    let parser = CParser::new().expect("Parser creation failed");
    let source = "typedef unsigned int uint;";

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.typedefs().len(), 1);

    let typedef = &ast.typedefs()[0];
    assert_eq!(typedef.name(), "uint");
}

#[test]
fn test_typedef_const() {
    // Test that const typedef is parsed
    let parser = CParser::new().expect("Parser creation failed");
    let source = "typedef const char* ConstString;";

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.typedefs().len(), 1);

    let typedef = &ast.typedefs()[0];
    assert_eq!(typedef.name(), "ConstString");
}

#[test]
fn test_typedef_array() {
    // Test that array typedef is parsed
    let parser = CParser::new().expect("Parser creation failed");
    let source = "typedef int IntArray[10];";

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.typedefs().len(), 1);

    let typedef = &ast.typedefs()[0];
    assert_eq!(typedef.name(), "IntArray");
    assert!(typedef.is_array(), "Should be recognized as array type");
}
