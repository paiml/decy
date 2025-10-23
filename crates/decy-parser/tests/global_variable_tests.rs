/// Global variable parsing tests
/// RED phase for DECY-058: Global variable support
///
/// These tests validate parsing of global variable declarations at file scope.
/// Storage class specifiers (static, extern, const) will be added in GREEN phase.
use decy_parser::{CParser, Expression};

#[test]
fn test_parse_simple_global_variable() {
    let source = r#"
        int global_count;

        int main() {
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    // Should have 1 global variable
    assert_eq!(
        result.variables().len(),
        1,
        "Should parse 1 global variable"
    );

    let global = &result.variables()[0];
    assert_eq!(global.name(), "global_count");
}

#[test]
fn test_parse_global_with_initializer() {
    let source = r#"
        int initialized_global = 42;

        int main() {
            return initialized_global;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.variables().len(), 1);

    let global = &result.variables()[0];
    assert_eq!(global.name(), "initialized_global");
    assert!(global.initializer().is_some(), "Should have initializer");

    // Verify the initializer is the integer 42
    match global.initializer() {
        Some(Expression::IntLiteral(val)) => {
            assert_eq!(*val, 42, "Initializer should be 42");
        }
        _ => panic!("Expected IntLiteral(42) initializer"),
    }
}

#[test]
fn test_parse_multiple_globals() {
    let source = r#"
        int global1 = 1;
        int global2 = 2;
        int global3 = 3;

        int main() {
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(
        result.variables().len(),
        3,
        "Should parse 3 global variables"
    );

    assert_eq!(result.variables()[0].name(), "global1");
    assert_eq!(result.variables()[1].name(), "global2");
    assert_eq!(result.variables()[2].name(), "global3");
}

#[test]
fn test_global_vs_local_distinction() {
    let source = r#"
        int global_var = 10;

        int add(int x) {
            int local_var = 20;
            return x + local_var + global_var;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    // Should have 1 global variable (local_var should NOT be in globals)
    assert_eq!(result.variables().len(), 1, "Should only have 1 global");
    assert_eq!(result.variables()[0].name(), "global_var");

    // Should have 1 function
    assert_eq!(result.functions().len(), 1);
    assert_eq!(result.functions()[0].name, "add");
}

#[test]
fn test_global_with_pointer_type() {
    let source = r#"
        int* global_ptr;

        int main() {
            return 0;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.variables().len(), 1);

    let global = &result.variables()[0];
    assert_eq!(global.name(), "global_ptr");
    // Type checking will be verified once parsing works
}

#[test]
fn test_global_array() {
    let source = r#"
        int global_array[10];

        int main() {
            return global_array[0];
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.variables().len(), 1);

    let global = &result.variables()[0];
    assert_eq!(global.name(), "global_array");
}

#[test]
fn test_function_and_global_together() {
    let source = r#"
        int counter = 0;

        void increment() {
            counter++;
        }

        int get_counter() {
            return counter;
        }
    "#;

    let parser = CParser::new().expect("Parser creation failed");
    let result = parser.parse(source).expect("Should parse successfully");

    assert_eq!(result.variables().len(), 1, "Should have 1 global");
    assert_eq!(result.functions().len(), 2, "Should have 2 functions");

    assert_eq!(result.variables()[0].name(), "counter");
    assert_eq!(result.functions()[0].name, "increment");
    assert_eq!(result.functions()[1].name, "get_counter");
}
