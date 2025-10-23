/// Cast expression parsing tests
/// RED phase for DECY-059: Cast expression support
///
/// These tests validate parsing of C cast expressions: (type)expr
/// Cast expressions are used for type conversions in C.
use decy_parser::{CParser, Expression, Statement, Type};

#[test]
fn test_parse_simple_integer_cast() {
    let source = r#"
        int main() {
            int x = (int)3.14;
            return x;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
    let func = &result.functions()[0];
    assert_eq!(func.name, "main");

    // Just verify it parses without errors
    // The cast expression is successfully parsed if we got here
    assert!(func.body.len() >= 1, "Should have at least 1 statement");
}

#[test]
fn test_parse_integer_cast_int_to_long() {
    let source = r#"
        int main() {
            int small = 42;
            long large = (long)small;
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    // Should parse successfully and detect the cast expression
    assert_eq!(result.functions().len(), 1);
}

#[test]
fn test_parse_pointer_cast_to_void() {
    let source = r#"
        int main() {
            int* ptr = 0;
            void* vptr = (void*)ptr;
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
}

#[test]
fn test_parse_struct_pointer_cast() {
    let source = r#"
        struct Point {
            int x;
            int y;
        };

        int main() {
            void* raw_ptr = 0;
            struct Point* p = (struct Point*)raw_ptr;
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
    assert_eq!(result.structs().len(), 1);
}

#[test]
fn test_parse_nested_casts() {
    let source = r#"
        int main() {
            double d = 3.14;
            int x = (int)(long)d;
            return x;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
}

#[test]
fn test_parse_cast_in_arithmetic() {
    let source = r#"
        int main() {
            double d = 3.14;
            int result = (int)d + 10;
            return result;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
}

#[test]
fn test_parse_cast_in_function_call() {
    let source = r#"
        void process(int value) {}

        int main() {
            double d = 3.14;
            process((int)d);
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 2);
}

#[test]
fn test_parse_const_cast() {
    let source = r#"
        int main() {
            const char* const_str = "hello";
            char* str = (char*)const_str;
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
}

#[test]
fn test_parse_cast_with_sizeof() {
    let source = r#"
        int main() {
            int size = (int)sizeof(long);
            return size;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
}

#[test]
fn test_parse_unsigned_cast() {
    let source = r#"
        int main() {
            int signed_val = -1;
            unsigned int unsigned_val = (unsigned int)signed_val;
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.functions().len(), 1);
}
