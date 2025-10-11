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

    #[test]
    fn test_parse_double_type() {
        // Test for double type conversion (mutation testing found this gap)
        let parser = CParser::new().expect("Parser creation failed");
        let source = "double calculate_pi() { return 3.14159; }";

        let ast = parser
            .parse(source)
            .expect("Parsing double function should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "calculate_pi");
        assert_eq!(
            func.return_type,
            Type::Double,
            "Return type should be double"
        );
    }

    #[test]
    fn test_parse_double_parameter() {
        // Test for double parameter type
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int round_value(double value) { return 0; }";

        let ast = parser
            .parse(source)
            .expect("Parsing function with double parameter should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.parameters.len(), 1);
        assert_eq!(
            func.parameters[0].param_type,
            Type::Double,
            "Parameter should be double"
        );
    }

    #[test]
    fn test_parse_char_type() {
        // Test for char type conversion (mutation testing found this gap)
        let parser = CParser::new().expect("Parser creation failed");
        let source = "char get_initial() { return 'A'; }";

        let ast = parser
            .parse(source)
            .expect("Parsing char function should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "get_initial");
        assert_eq!(func.return_type, Type::Char, "Return type should be char");
    }

    #[test]
    fn test_parse_char_parameter() {
        // Test for char parameter type
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int to_uppercase(char c) { return 0; }";

        let ast = parser
            .parse(source)
            .expect("Parsing function with char parameter should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.parameters.len(), 1);
        assert_eq!(
            func.parameters[0].param_type,
            Type::Char,
            "Parameter should be char"
        );
    }

    #[test]
    fn test_parse_mixed_types() {
        // Test function with multiple different types
        let parser = CParser::new().expect("Parser creation failed");
        let source = "double complex_calc(int a, float b, double c, char d) { return 0.0; }";

        let ast = parser
            .parse(source)
            .expect("Parsing function with mixed types should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.return_type, Type::Double);
        assert_eq!(func.parameters.len(), 4);
        assert_eq!(func.parameters[0].param_type, Type::Int);
        assert_eq!(func.parameters[1].param_type, Type::Float);
        assert_eq!(func.parameters[2].param_type, Type::Double);
        assert_eq!(func.parameters[3].param_type, Type::Char);
    }

    #[test]
    fn test_parse_return_literal_value() {
        // DECY-028: Test that return statements preserve actual integer values
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int get_value() { return 42; }";

        let ast = parser
            .parse(source)
            .expect("Parsing return with literal should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "get_value");
        assert_eq!(func.body.len(), 1, "Should have one statement");

        // Verify return statement has correct value
        match &func.body[0] {
            Statement::Return(Some(Expression::IntLiteral(value))) => {
                assert_eq!(*value, 42, "Return value should be 42, not 0");
            }
            _ => panic!("Expected Return statement with IntLiteral(42)"),
        }
    }

    #[test]
    fn test_parse_binary_expression() {
        // DECY-028: Test that binary expressions are parsed
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int add(int a, int b) { return a + b; }";

        let ast = parser
            .parse(source)
            .expect("Parsing binary expression should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "add");
        assert_eq!(func.body.len(), 1, "Should have one statement");

        // Verify return statement has binary expression
        match &func.body[0] {
            Statement::Return(Some(Expression::BinaryOp { op, left, right })) => {
                assert_eq!(*op, BinaryOperator::Add, "Operator should be Add");

                // Left side should be variable 'a'
                match **left {
                    Expression::Variable(ref name) => {
                        assert_eq!(name, "a", "Left operand should be variable 'a'");
                    }
                    _ => panic!("Left operand should be a variable"),
                }

                // Right side should be variable 'b'
                match **right {
                    Expression::Variable(ref name) => {
                        assert_eq!(name, "b", "Right operand should be variable 'b'");
                    }
                    _ => panic!("Right operand should be a variable"),
                }
            }
            _ => panic!("Expected Return statement with BinaryOp expression"),
        }
    }

    #[test]
    fn test_parse_assignment_statement() {
        // DECY-028 Phase 3: Test that assignment statements are parsed
        // RED PHASE: This test will FAIL because Statement doesn't have Assignment variant
        let parser = CParser::new().expect("Parser creation failed");
        let source = "void set_value() { int x; x = 42; }";

        let ast = parser
            .parse(source)
            .expect("Parsing assignment should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "set_value");
        assert_eq!(
            func.body.len(),
            2,
            "Should have two statements: declaration and assignment"
        );

        // First statement: variable declaration
        match &func.body[0] {
            Statement::VariableDeclaration {
                name,
                var_type,
                initializer,
            } => {
                assert_eq!(name, "x");
                assert_eq!(*var_type, Type::Int);
                assert!(initializer.is_none());
            }
            _ => panic!("Expected VariableDeclaration statement"),
        }

        // Second statement: assignment
        match &func.body[1] {
            Statement::Assignment { target, value } => {
                assert_eq!(target, "x", "Assignment target should be 'x'");
                match value {
                    Expression::IntLiteral(val) => {
                        assert_eq!(*val, 42, "Assignment value should be 42");
                    }
                    _ => panic!("Assignment value should be IntLiteral(42)"),
                }
            }
            _ => panic!("Expected Assignment statement"),
        }
    }

    #[test]
    fn test_parse_if_statement() {
        // DECY-029 RED PHASE: Test that if statements are parsed
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int max(int a, int b) { if (a > b) { return a; } return b; }";

        let ast = parser
            .parse(source)
            .expect("Parsing if statement should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "max");
        assert_eq!(func.body.len(), 2, "Should have if statement and return");

        // First statement: if statement
        match &func.body[0] {
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                // Check condition: a > b
                match condition {
                    Expression::BinaryOp { op, left, right } => {
                        assert_eq!(*op, BinaryOperator::GreaterThan);
                        match **left {
                            Expression::Variable(ref name) => assert_eq!(name, "a"),
                            _ => panic!("Expected variable 'a'"),
                        }
                        match **right {
                            Expression::Variable(ref name) => assert_eq!(name, "b"),
                            _ => panic!("Expected variable 'b'"),
                        }
                    }
                    _ => panic!("Expected binary expression for condition"),
                }

                // Check then block
                assert_eq!(then_block.len(), 1, "Then block should have one statement");
                match &then_block[0] {
                    Statement::Return(Some(Expression::Variable(ref name))) => {
                        assert_eq!(name, "a");
                    }
                    _ => panic!("Expected return statement in then block"),
                }

                // Check else block
                assert!(else_block.is_none(), "Should not have else block");
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_parse_if_else_statement() {
        // DECY-029 RED PHASE: Test that if-else statements are parsed
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int max(int a, int b) { if (a > b) { return a; } else { return b; } }";

        let ast = parser
            .parse(source)
            .expect("Parsing if-else statement should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.body.len(), 1, "Should have one if-else statement");

        match &func.body[0] {
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                // Condition already tested above
                assert!(
                    matches!(condition, Expression::BinaryOp { .. }),
                    "Should have comparison"
                );

                // Then block
                assert_eq!(then_block.len(), 1);

                // Else block
                assert!(else_block.is_some(), "Should have else block");
                let else_stmts = else_block.as_ref().unwrap();
                assert_eq!(else_stmts.len(), 1, "Else block should have one statement");
                match &else_stmts[0] {
                    Statement::Return(Some(Expression::Variable(ref name))) => {
                        assert_eq!(name, "b");
                    }
                    _ => panic!("Expected return statement in else block"),
                }
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_parse_for_loop() {
        // DECY-029 RED PHASE: Test that for loops are parsed
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int sum(int n) { int s = 0; for (int i = 0; i < n; i = i + 1) { s = s + i; } return s; }";

        let ast = parser
            .parse(source)
            .expect("Parsing for loop should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "sum");
        assert_eq!(func.body.len(), 3, "Should have var decl, for loop, return");

        // Second statement: for loop
        match &func.body[1] {
            Statement::For {
                init,
                condition,
                increment,
                body,
            } => {
                // Check init: int i = 0
                assert!(init.is_some(), "Should have init statement");
                match init.as_ref().unwrap().as_ref() {
                    Statement::VariableDeclaration {
                        name,
                        var_type,
                        initializer,
                    } => {
                        assert_eq!(name, "i");
                        assert_eq!(*var_type, Type::Int);
                        assert!(initializer.is_some());
                    }
                    _ => panic!("Expected variable declaration in init"),
                }

                // Check condition: i < n
                assert!(condition.is_some(), "Should have condition");
                match condition.as_ref().unwrap() {
                    Expression::BinaryOp { op, .. } => {
                        assert_eq!(*op, BinaryOperator::LessThan);
                    }
                    _ => panic!("Expected binary expression for condition"),
                }

                // Check increment: i = i + 1
                assert!(increment.is_some(), "Should have increment");

                // Check body: s = s + i
                assert_eq!(body.len(), 1, "Loop body should have one statement");
                match &body[0] {
                    Statement::Assignment { target, .. } => {
                        assert_eq!(target, "s");
                    }
                    _ => panic!("Expected assignment in loop body"),
                }
            }
            _ => panic!("Expected For statement"),
        }
    }

    #[test]
    fn test_parse_nested_if() {
        // DECY-029 RED PHASE: Test nested if statements
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            int classify(int x) {
                if (x > 0) {
                    if (x > 10) {
                        return 2;
                    }
                    return 1;
                }
                return 0;
            }
        "#;

        let ast = parser
            .parse(source)
            .expect("Parsing nested if should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "classify");
        // Should have: if (outer), return
        assert!(
            func.body.len() >= 2,
            "Should have at least outer if and final return"
        );

        // Outer if statement
        match &func.body[0] {
            Statement::If {
                then_block,
                else_block,
                ..
            } => {
                // Then block should have nested if
                assert!(
                    then_block.len() >= 2,
                    "Should have nested if and return in then block"
                );
                match &then_block[0] {
                    Statement::If { .. } => {
                        // Nested if found
                    }
                    _ => panic!("Expected nested If statement"),
                }
                assert!(else_block.is_none(), "Outer if should not have else");
            }
            _ => panic!("Expected outer If statement"),
        }
    }

    #[test]
    fn test_parse_while_loop_with_body() {
        // DECY-029: Verify while loops parse correctly with actual body
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int countdown(int n) { while (n > 0) { n = n - 1; } return n; }";

        let ast = parser
            .parse(source)
            .expect("Parsing while loop should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "countdown");
        assert_eq!(func.body.len(), 2, "Should have while loop and return");

        // While statement
        match &func.body[0] {
            Statement::While { condition, body } => {
                // Check condition: n > 0
                match condition {
                    Expression::BinaryOp { op, .. } => {
                        assert_eq!(*op, BinaryOperator::GreaterThan);
                    }
                    _ => panic!("Expected comparison in while condition"),
                }

                // Check body
                assert_eq!(body.len(), 1, "While body should have one statement");
                match &body[0] {
                    Statement::Assignment { target, .. } => {
                        assert_eq!(target, "n");
                    }
                    _ => panic!("Expected assignment in while body"),
                }
            }
            _ => panic!("Expected While statement"),
        }
    }
}
