/// Function Call Statement Tests
/// Tests for DECY-066: Parser should extract standalone function call statements
///
/// Tests that function calls used as statements (not in expressions)
/// are properly parsed as Statement::FunctionCall.
use decy_parser::parser::{CParser, Statement};

#[test]
fn test_parse_printf_statement() {
    // RED phase test for DECY-066
    let c_code = r#"
int printf(const char* format, ...);

int main() {
    printf("Hello, World!\n");
    return 0;
}
"#;

    let parser = CParser::new().expect("Failed to create parser");
    let ast = parser.parse(c_code).expect("Failed to parse");

    let main_func = ast.functions().iter().find(|f| f.name == "main");
    assert!(main_func.is_some(), "main function should be parsed");

    let main_func = main_func.unwrap();

    // Should have 2 statements: printf call + return
    assert_eq!(
        main_func.body.len(),
        2,
        "main should have 2 statements (printf + return), got {}. Statements: {:?}",
        main_func.body.len(),
        main_func.body
    );

    // First statement should be a function call
    match &main_func.body[0] {
        Statement::FunctionCall {
            function,
            arguments,
        } => {
            assert_eq!(function, "printf");
            assert_eq!(arguments.len(), 1, "printf should have 1 argument");
        }
        other => panic!(
            "Expected Statement::FunctionCall for printf, got {:?}",
            other
        ),
    }

    // Second statement should be return
    assert!(
        matches!(main_func.body[1], Statement::Return(_)),
        "Second statement should be Return"
    );
}

#[test]
fn test_parse_free_statement() {
    let c_code = r#"
void free(void* ptr);

void cleanup(void* ptr) {
    free(ptr);
}
"#;

    let parser = CParser::new().expect("Failed to create parser");
    let ast = parser.parse(c_code).expect("Failed to parse");

    let cleanup_func = ast.functions().iter().find(|f| f.name == "cleanup");
    assert!(cleanup_func.is_some(), "cleanup function should be parsed");

    let cleanup_func = cleanup_func.unwrap();

    // Should have 1 statement: free call
    assert_eq!(
        cleanup_func.body.len(),
        1,
        "cleanup should have 1 statement (free), got {}",
        cleanup_func.body.len()
    );

    // Statement should be a function call to free
    match &cleanup_func.body[0] {
        Statement::FunctionCall {
            function,
            arguments,
        } => {
            assert_eq!(function, "free");
            assert_eq!(arguments.len(), 1, "free should have 1 argument");
        }
        other => panic!("Expected Statement::FunctionCall for free, got {:?}", other),
    }
}

#[test]
fn test_parse_multiple_function_call_statements() {
    let c_code = r#"
int printf(const char* format, ...);
void free(void* ptr);

void example(void* p1, void* p2) {
    printf("Freeing memory\n");
    free(p1);
    free(p2);
    printf("Done\n");
}
"#;

    let parser = CParser::new().expect("Failed to create parser");
    let ast = parser.parse(c_code).expect("Failed to parse");

    let example_func = ast.functions().iter().find(|f| f.name == "example");
    assert!(example_func.is_some(), "example function should be parsed");

    let example_func = example_func.unwrap();

    // Should have 4 statements: printf, free, free, printf
    assert_eq!(
        example_func.body.len(),
        4,
        "example should have 4 function call statements, got {}",
        example_func.body.len()
    );

    // All should be function calls
    for (i, stmt) in example_func.body.iter().enumerate() {
        assert!(
            matches!(stmt, Statement::FunctionCall { .. }),
            "Statement {} should be FunctionCall, got {:?}",
            i,
            stmt
        );
    }
}
