//! Unit tests for the C parser (DECY-001 RED phase).
//!
//! These tests are intentionally failing to follow EXTREME TDD methodology.

use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        // Test that we can create a parser
        let result = CParser::new();
        assert!(result.is_ok(), "Parser creation should succeed");
    }

    #[test]
    fn test_parse_simple_main_function() {
        // RED PHASE: This test will FAIL until we implement clang-sys parsing
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int main() { return 0; }";

        let ast = parser
            .parse(source)
            .expect("Parsing simple main function should succeed");

        // Verify we got one function
        assert_eq!(
            ast.functions().len(),
            1,
            "Should parse exactly one function"
        );

        // Verify function details
        let func = &ast.functions()[0];
        assert_eq!(func.name, "main", "Function name should be 'main'");
        assert_eq!(func.return_type, Type::Int, "Return type should be int");
        assert_eq!(func.parameters.len(), 0, "main() should have no parameters");
    }

    #[test]
    fn test_parse_function_with_parameters() {
        // RED PHASE: This test will FAIL
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int add(int a, int b) { return a + b; }";

        let ast = parser
            .parse(source)
            .expect("Parsing function with parameters should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "add");
        assert_eq!(func.return_type, Type::Int);
        assert_eq!(func.parameters.len(), 2);

        assert_eq!(func.parameters[0].name, "a");
        assert_eq!(func.parameters[0].param_type, Type::Int);

        assert_eq!(func.parameters[1].name, "b");
        assert_eq!(func.parameters[1].param_type, Type::Int);
    }

    #[test]
    fn test_parse_function_with_return_value() {
        // RED PHASE: This test will FAIL
        let parser = CParser::new().expect("Parser creation failed");
        let source = "float calculate(int x) { return x * 2.5; }";

        let ast = parser
            .parse(source)
            .expect("Parsing function with return value should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "calculate");
        assert_eq!(func.return_type, Type::Float);
        assert_eq!(func.parameters.len(), 1);
        assert_eq!(func.parameters[0].param_type, Type::Int);
    }

    #[test]
    fn test_parse_syntax_error_handling() {
        // RED PHASE: This test will FAIL
        let parser = CParser::new().expect("Parser creation failed");
        let invalid_source = "int incomplete(";

        let result = parser.parse(invalid_source);
        assert!(
            result.is_err(),
            "Parsing invalid syntax should return an error"
        );
    }

    #[test]
    fn test_parse_empty_input() {
        // RED PHASE: This test will FAIL
        let parser = CParser::new().expect("Parser creation failed");
        let empty_source = "";

        let result = parser.parse(empty_source);
        // Empty input should either return empty AST or error (design decision)
        // For now, we expect it to succeed with empty AST
        assert!(result.is_ok(), "Parsing empty input should succeed");

        if let Ok(ast) = result {
            assert_eq!(
                ast.functions().len(),
                0,
                "Empty input should have no functions"
            );
        }
    }

    #[test]
    fn test_parse_multiple_functions() {
        // RED PHASE: This test will FAIL
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            int add(int a, int b) { return a + b; }
            int subtract(int a, int b) { return a - b; }
        "#;

        let ast = parser
            .parse(source)
            .expect("Parsing multiple functions should succeed");

        assert_eq!(ast.functions().len(), 2, "Should parse two functions");
        assert_eq!(ast.functions()[0].name, "add");
        assert_eq!(ast.functions()[1].name, "subtract");
    }

    #[test]
    fn test_parse_void_function() {
        // RED PHASE: This test will FAIL
        let parser = CParser::new().expect("Parser creation failed");
        let source = "void print_hello() { }";

        let ast = parser
            .parse(source)
            .expect("Parsing void function should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "print_hello");
        assert_eq!(func.return_type, Type::Void);
        assert_eq!(func.parameters.len(), 0);
    }

    #[test]
    fn test_parse_pointer_parameter() {
        // RED PHASE: This test will FAIL
        let parser = CParser::new().expect("Parser creation failed");
        let source = "void process(int* data) { }";

        let ast = parser
            .parse(source)
            .expect("Parsing pointer parameter should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "process");
        assert_eq!(func.return_type, Type::Void);
        assert_eq!(func.parameters.len(), 1);

        // Check that parameter is a pointer
        match &func.parameters[0].param_type {
            Type::Pointer(inner) => {
                assert_eq!(**inner, Type::Int, "Should be pointer to int");
            }
            _ => panic!("Parameter should be a pointer type"),
        }
    }
}
