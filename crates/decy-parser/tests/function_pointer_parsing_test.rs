//! Parser tests for function pointer declarations (DECY-024 RED phase)
//!
//! This test suite follows EXTREME TDD methodology - all tests should FAIL initially.
//! Tests verify that the parser correctly extracts function pointer declarations from C code.
//!
//! References:
//! - K&R §5.11: Pointers to Functions
//! - ISO C99 §6.7.5.3: Function declarators

use decy_parser::CParser;

#[test]
fn test_function_pointer_simple() {
    // Test that simple function pointer variable is parsed
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int (*callback)(int);
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    // Should have one variable declaration with function pointer type
    assert_eq!(ast.variables().len(), 1, "Should parse one variable");

    let var = &ast.variables()[0];
    assert_eq!(var.name(), "callback");
    assert!(var.is_function_pointer(), "Should be recognized as function pointer");
}

#[test]
fn test_function_pointer_with_multiple_params() {
    // Test that function pointer with multiple parameters is parsed
    let parser = CParser::new().expect("Parser creation failed");
    let source = "int (*add)(int, int);";

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.variables().len(), 1);

    let var = &ast.variables()[0];
    assert_eq!(var.name(), "add");
    assert!(var.is_function_pointer());

    // Should have 2 parameters
    assert_eq!(var.function_pointer_param_count(), 2);
}

#[test]
fn test_function_pointer_void_return() {
    // Test that function pointer with void return type is parsed
    let parser = CParser::new().expect("Parser creation failed");
    let source = "void (*handler)(int);";

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.variables().len(), 1);

    let var = &ast.variables()[0];
    assert_eq!(var.name(), "handler");
    assert!(var.is_function_pointer());
    assert!(var.function_pointer_has_void_return());
}

#[test]
fn test_function_pointer_no_params() {
    // Test that function pointer with no parameters is parsed
    let parser = CParser::new().expect("Parser creation failed");
    let source = "int (*get_value)(void);";

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.variables().len(), 1);

    let var = &ast.variables()[0];
    assert_eq!(var.name(), "get_value");
    assert!(var.is_function_pointer());
    assert_eq!(var.function_pointer_param_count(), 0);
}

#[test]
fn test_function_pointer_in_struct() {
    // Test that function pointer as struct field is parsed
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        struct Handler {
            int (*callback)(int);
            void (*on_error)(char*);
        };
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.structs().len(), 1, "Should parse one struct");

    let struct_def = &ast.structs()[0];
    assert_eq!(struct_def.name(), "Handler");
    assert_eq!(struct_def.fields().len(), 2, "Should have two fields");

    // Check callback field
    let callback_field = &struct_def.fields()[0];
    assert_eq!(callback_field.name(), "callback");
    assert!(callback_field.is_function_pointer(), "First field should be function pointer");

    // Check on_error field
    let error_field = &struct_def.fields()[1];
    assert_eq!(error_field.name(), "on_error");
    assert!(error_field.is_function_pointer(), "Second field should be function pointer");
}

#[test]
fn test_function_pointer_as_parameter() {
    // Test that function pointer as function parameter is parsed
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void apply(int value, int (*operation)(int)) {
            operation(value);
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.functions().len(), 1);

    let func = &ast.functions()[0];
    assert_eq!(func.name, "apply");
    assert_eq!(func.parameters.len(), 2, "Should have two parameters");

    // Second parameter should be function pointer
    let operation_param = &func.parameters[1];
    assert_eq!(operation_param.name, "operation");
    assert!(operation_param.is_function_pointer(), "Parameter should be function pointer");
}

#[test]
fn test_callback_pattern_with_typedef() {
    // Test common callback pattern using typedef
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        typedef int (*Callback)(int, int);

        int invoke(Callback cb, int a, int b) {
            return cb(a, b);
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    // Should have typedef
    assert_eq!(ast.typedefs().len(), 1);
    let typedef = &ast.typedefs()[0];
    assert_eq!(typedef.name(), "Callback");
    assert!(typedef.is_function_pointer());

    // Should have function using the typedef
    assert_eq!(ast.functions().len(), 1);
    let func = &ast.functions()[0];
    assert_eq!(func.name, "invoke");
    assert_eq!(func.parameters.len(), 3);
}

#[test]
fn test_function_pointer_with_float_params() {
    // Test that function pointer with different parameter types is parsed
    let parser = CParser::new().expect("Parser creation failed");
    let source = "float (*compute)(float, float);";

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.variables().len(), 1);

    let var = &ast.variables()[0];
    assert_eq!(var.name(), "compute");
    assert!(var.is_function_pointer());
}

#[test]
fn test_function_pointer_with_pointer_params() {
    // Test that function pointer with pointer parameters is parsed
    let parser = CParser::new().expect("Parser creation failed");
    let source = "void (*process)(int*, char*);";

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.variables().len(), 1);

    let var = &ast.variables()[0];
    assert_eq!(var.name(), "process");
    assert!(var.is_function_pointer());
}

#[test]
fn test_multiple_function_pointers() {
    // Test that multiple function pointer declarations are parsed
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int (*add)(int, int);
        int (*subtract)(int, int);
        int (*multiply)(int, int);
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.variables().len(), 3, "Should parse three function pointers");

    assert_eq!(ast.variables()[0].name(), "add");
    assert_eq!(ast.variables()[1].name(), "subtract");
    assert_eq!(ast.variables()[2].name(), "multiply");

    for var in ast.variables() {
        assert!(var.is_function_pointer(), "{} should be function pointer", var.name());
    }
}
