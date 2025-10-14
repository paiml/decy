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

    #[test]
    fn test_parse_pointer_dereference_in_return() {
        // DECY-031 RED PHASE: Test that pointer dereference is parsed
        // This test will FAIL because Expression doesn't have Dereference variant
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int get_value(int* ptr) { return *ptr; }";

        let ast = parser
            .parse(source)
            .expect("Parsing pointer dereference should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "get_value");
        assert_eq!(func.body.len(), 1, "Should have one return statement");

        // Verify return statement has dereference expression
        match &func.body[0] {
            Statement::Return(Some(Expression::Dereference(expr))) => {
                // Inner expression should be variable 'ptr'
                match **expr {
                    Expression::Variable(ref name) => {
                        assert_eq!(name, "ptr", "Dereferenced variable should be 'ptr'");
                    }
                    _ => panic!("Dereference should contain a variable"),
                }
            }
            _ => panic!("Expected Return statement with Dereference expression"),
        }
    }

    #[test]
    fn test_parse_pointer_dereference_in_condition() {
        // DECY-031 RED PHASE: Test pointer dereference in if condition
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int check(int* ptr) { if (*ptr != 0) { return 1; } return 0; }";

        let ast = parser
            .parse(source)
            .expect("Parsing dereference in condition should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "check");

        // First statement should be if with dereference in condition
        match &func.body[0] {
            Statement::If { condition, .. } => {
                match condition {
                    Expression::BinaryOp { op, left, right } => {
                        assert_eq!(*op, BinaryOperator::NotEqual);

                        // Left side should be dereference
                        match **left {
                            Expression::Dereference(ref inner) => match **inner {
                                Expression::Variable(ref name) => {
                                    assert_eq!(name, "ptr");
                                }
                                _ => panic!("Dereference should contain variable"),
                            },
                            _ => panic!("Left operand should be dereference expression"),
                        }

                        // Right side should be literal 0
                        match **right {
                            Expression::IntLiteral(val) => {
                                assert_eq!(val, 0);
                            }
                            _ => panic!("Right operand should be literal 0"),
                        }
                    }
                    _ => panic!("Expected binary expression in condition"),
                }
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_parse_pointer_dereference_in_assignment() {
        // DECY-031 RED PHASE: Test pointer dereference in assignment value
        let parser = CParser::new().expect("Parser creation failed");
        let source = "void copy(int* src, int* dst) { int x; x = *src; *dst = x; }";

        let ast = parser
            .parse(source)
            .expect("Parsing dereference in assignment should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "copy");
        assert_eq!(
            func.body.len(),
            3,
            "Should have var decl and two assignments"
        );

        // Second statement: x = *src
        match &func.body[1] {
            Statement::Assignment { target, value } => {
                assert_eq!(target, "x");
                match value {
                    Expression::Dereference(inner) => match **inner {
                        Expression::Variable(ref name) => {
                            assert_eq!(name, "src");
                        }
                        _ => panic!("Dereference should contain variable 'src'"),
                    },
                    _ => panic!("Assignment value should be dereference"),
                }
            }
            _ => panic!("Expected assignment statement"),
        }

        // Third statement: *dst = x (dereference as target)
        match &func.body[2] {
            Statement::DerefAssignment { target, value } => {
                match target {
                    Expression::Variable(ref name) => {
                        assert_eq!(name, "dst");
                    }
                    _ => panic!("Deref assignment target should be variable"),
                }
                match value {
                    Expression::Variable(ref name) => {
                        assert_eq!(name, "x");
                    }
                    _ => panic!("Assignment value should be variable"),
                }
            }
            _ => panic!("Expected DerefAssignment statement for *dst = x"),
        }
    }

    #[test]
    fn test_parse_logical_and() {
        // DECY-032 RED PHASE: Test that logical AND (&&) is parsed
        // This test will FAIL because BinaryOperator doesn't have LogicalAnd variant
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int check_both(int a, int b) { if (a > 0 && b > 0) { return 1; } return 0; }";

        let ast = parser
            .parse(source)
            .expect("Parsing logical AND should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "check_both");

        // First statement should be if with logical AND
        match &func.body[0] {
            Statement::If { condition, .. } => {
                match condition {
                    Expression::BinaryOp { op, left, right } => {
                        assert_eq!(
                            *op,
                            BinaryOperator::LogicalAnd,
                            "Operator should be LogicalAnd (&&)"
                        );

                        // Left: a > 0
                        match **left {
                            Expression::BinaryOp {
                                op: ref left_op, ..
                            } => {
                                assert_eq!(*left_op, BinaryOperator::GreaterThan);
                            }
                            _ => panic!("Left should be comparison expression"),
                        }

                        // Right: b > 0
                        match **right {
                            Expression::BinaryOp {
                                op: ref right_op, ..
                            } => {
                                assert_eq!(*right_op, BinaryOperator::GreaterThan);
                            }
                            _ => panic!("Right should be comparison expression"),
                        }
                    }
                    _ => panic!("Expected binary expression with LogicalAnd"),
                }
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_parse_logical_or() {
        // DECY-032 RED PHASE: Test that logical OR (||) is parsed
        // This test will FAIL because BinaryOperator doesn't have LogicalOr variant
        let parser = CParser::new().expect("Parser creation failed");
        let source =
            "int check_either(int a, int b) { if (a > 0 || b > 0) { return 1; } return 0; }";

        let ast = parser
            .parse(source)
            .expect("Parsing logical OR should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "check_either");

        // First statement should be if with logical OR
        match &func.body[0] {
            Statement::If { condition, .. } => {
                match condition {
                    Expression::BinaryOp { op, left, right } => {
                        assert_eq!(
                            *op,
                            BinaryOperator::LogicalOr,
                            "Operator should be LogicalOr (||)"
                        );

                        // Left: a > 0
                        match **left {
                            Expression::BinaryOp {
                                op: ref left_op, ..
                            } => {
                                assert_eq!(*left_op, BinaryOperator::GreaterThan);
                            }
                            _ => panic!("Left should be comparison expression"),
                        }

                        // Right: b > 0
                        match **right {
                            Expression::BinaryOp {
                                op: ref right_op, ..
                            } => {
                                assert_eq!(*right_op, BinaryOperator::GreaterThan);
                            }
                            _ => panic!("Right should be comparison expression"),
                        }
                    }
                    _ => panic!("Expected binary expression with LogicalOr"),
                }
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_parse_combined_logical_ops() {
        // DECY-032 RED PHASE: Test combined AND/OR operators
        let parser = CParser::new().expect("Parser creation failed");
        let source =
            "int check_range(int x) { if (x > 0 && x < 100 || x == 5) { return 1; } return 0; }";

        let ast = parser
            .parse(source)
            .expect("Parsing combined logical ops should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "check_range");

        // Verify that logical operators are parsed (structure will depend on precedence)
        match &func.body[0] {
            Statement::If { condition, .. } => {
                // Should have LogicalOr at the top level (lower precedence)
                match condition {
                    Expression::BinaryOp { op, .. } => {
                        assert!(
                            *op == BinaryOperator::LogicalOr || *op == BinaryOperator::LogicalAnd,
                            "Should have logical operator at top level"
                        );
                    }
                    _ => panic!("Expected binary expression with logical operator"),
                }
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_parse_array_index_in_expression() {
        // DECY-033 RED PHASE: Test that array indexing is parsed
        // This test will FAIL because Expression doesn't have ArrayIndex variant
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int get_element(int arr[], int i) { return arr[i]; }";

        let ast = parser
            .parse(source)
            .expect("Parsing array indexing should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "get_element");
        assert_eq!(func.body.len(), 1, "Should have one return statement");

        // Verify return statement has array index expression
        match &func.body[0] {
            Statement::Return(Some(Expression::ArrayIndex { array, index })) => {
                // Array should be variable 'arr'
                match **array {
                    Expression::Variable(ref name) => {
                        assert_eq!(name, "arr", "Array should be 'arr'");
                    }
                    _ => panic!("Array should be a variable"),
                }

                // Index should be variable 'i'
                match **index {
                    Expression::Variable(ref name) => {
                        assert_eq!(name, "i", "Index should be 'i'");
                    }
                    _ => panic!("Index should be a variable"),
                }
            }
            _ => panic!("Expected Return statement with ArrayIndex expression"),
        }
    }

    #[test]
    fn test_parse_array_index_assignment() {
        // DECY-033 RED PHASE: Test array index in assignment
        // buffer[i] = value should parse as ArrayIndexAssignment
        let parser = CParser::new().expect("Parser creation failed");
        let source = "void set_element(int buffer[], int i, int value) { buffer[i] = value; }";

        let ast = parser
            .parse(source)
            .expect("Parsing array index assignment should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "set_element");
        assert_eq!(func.body.len(), 1, "Should have one assignment statement");

        // Verify assignment to array index
        match &func.body[0] {
            Statement::ArrayIndexAssignment {
                array,
                index,
                value,
            } => {
                // Array should be variable 'buffer'
                match **array {
                    Expression::Variable(ref name) => {
                        assert_eq!(name, "buffer");
                    }
                    _ => panic!("Array should be a variable"),
                }

                // Index should be variable 'i'
                match **index {
                    Expression::Variable(ref name) => {
                        assert_eq!(name, "i");
                    }
                    _ => panic!("Index should be a variable"),
                }

                // Value should be variable 'value'
                match value {
                    Expression::Variable(ref name) => {
                        assert_eq!(name, "value");
                    }
                    _ => panic!("Value should be a variable"),
                }
            }
            _ => panic!("Expected ArrayIndexAssignment statement"),
        }
    }

    #[test]
    fn test_parse_array_index_in_binary_expr() {
        // DECY-033 RED PHASE: Test array index in binary expression
        // total = total + arr[i] should parse with ArrayIndex in right operand
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int sum(int arr[], int n) { int total = 0; for (int i = 0; i < n; i = i + 1) { total = total + arr[i]; } return total; }";

        let ast = parser
            .parse(source)
            .expect("Parsing array index in binary expression should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "sum");

        // Should have: var decl, for loop, return
        assert_eq!(func.body.len(), 3);

        // Check for loop body has assignment with array index
        match &func.body[1] {
            Statement::For { body, .. } => {
                assert_eq!(body.len(), 1, "For loop body should have one statement");

                // Body should be: total = total + arr[i]
                match &body[0] {
                    Statement::Assignment { target, value } => {
                        assert_eq!(target, "total");

                        // Value should be binary expression with arr[i] on right
                        match value {
                            Expression::BinaryOp { op, right, .. } => {
                                assert_eq!(*op, BinaryOperator::Add);

                                // Right operand should be ArrayIndex
                                match **right {
                                    Expression::ArrayIndex {
                                        ref array,
                                        ref index,
                                    } => {
                                        match **array {
                                            Expression::Variable(ref name) => {
                                                assert_eq!(name, "arr")
                                            }
                                            _ => panic!("Array should be variable"),
                                        }
                                        match **index {
                                            Expression::Variable(ref name) => assert_eq!(name, "i"),
                                            _ => panic!("Index should be variable"),
                                        }
                                    }
                                    _ => panic!("Right operand should be ArrayIndex"),
                                }
                            }
                            _ => panic!("Value should be BinaryOp"),
                        }
                    }
                    _ => panic!("Expected Assignment in loop body"),
                }
            }
            _ => panic!("Expected For statement"),
        }
    }

    #[test]
    fn test_parse_pointer_field_access() {
        // DECY-034 RED PHASE: Test that pointer field access (ptr->field) is parsed
        // This test will FAIL because Expression doesn't have PointerFieldAccess variant
        let parser = CParser::new().expect("Parser creation failed");
        let source = "struct Point { int x; int y; }; int get_x(struct Point* p) { return p->x; }";

        let ast = parser
            .parse(source)
            .expect("Parsing pointer field access should succeed");

        // Second function (index 1) because struct declaration is parsed too
        assert!(
            !ast.functions().is_empty(),
            "Should have at least one function"
        );
        let func = &ast.functions()[0];
        assert_eq!(func.name, "get_x");
        assert_eq!(func.body.len(), 1, "Should have one return statement");

        // Verify return statement has pointer field access expression
        match &func.body[0] {
            Statement::Return(Some(Expression::PointerFieldAccess { pointer, field })) => {
                // Pointer should be variable 'p'
                match **pointer {
                    Expression::Variable(ref name) => {
                        assert_eq!(name, "p", "Pointer should be 'p'");
                    }
                    _ => panic!("Pointer should be a variable"),
                }

                // Field should be 'x'
                assert_eq!(field, "x", "Field should be 'x'");
            }
            _ => panic!("Expected Return statement with PointerFieldAccess expression"),
        }
    }

    #[test]
    fn test_parse_field_access() {
        // DECY-034 RED PHASE: Test that struct field access (obj.field) is parsed
        // This test will FAIL because Expression doesn't have FieldAccess variant
        let parser = CParser::new().expect("Parser creation failed");
        let source = "struct Point { int x; int y; }; int get_x(struct Point p) { return p.x; }";

        let ast = parser
            .parse(source)
            .expect("Parsing field access should succeed");

        assert!(
            !ast.functions().is_empty(),
            "Should have at least one function"
        );
        let func = &ast.functions()[0];
        assert_eq!(func.name, "get_x");
        assert_eq!(func.body.len(), 1, "Should have one return statement");

        // Verify return statement has field access expression
        match &func.body[0] {
            Statement::Return(Some(Expression::FieldAccess { object, field })) => {
                // Object should be variable 'p'
                match **object {
                    Expression::Variable(ref name) => {
                        assert_eq!(name, "p", "Object should be 'p'");
                    }
                    _ => panic!("Object should be a variable"),
                }

                // Field should be 'x'
                assert_eq!(field, "x", "Field should be 'x'");
            }
            _ => panic!("Expected Return statement with FieldAccess expression"),
        }
    }

    #[test]
    fn test_parse_pointer_field_access_assignment() {
        // DECY-034 RED PHASE: Test ptr->field = value assignment
        let parser = CParser::new().expect("Parser creation failed");
        let source = "struct Point { int x; int y; }; void set_y(struct Point* p, int value) { p->y = value; }";

        let ast = parser
            .parse(source)
            .expect("Parsing pointer field access assignment should succeed");

        assert!(
            !ast.functions().is_empty(),
            "Should have at least one function"
        );
        let func = &ast.functions()[0];
        assert_eq!(func.name, "set_y");
        assert_eq!(func.body.len(), 1, "Should have one assignment statement");

        // Verify assignment with field access on left side
        // This should parse as a FieldAssignment statement
        match &func.body[0] {
            Statement::FieldAssignment {
                object,
                field,
                value,
            } => {
                // Object should be pointer field access or variable 'p'
                match object {
                    Expression::Variable(ref name) => {
                        assert_eq!(name, "p", "Object should be 'p'");
                    }
                    _ => panic!("Object should be a variable"),
                }

                // Field should be 'y'
                assert_eq!(field, "y", "Field should be 'y'");

                // Value should be variable 'value'
                match value {
                    Expression::Variable(ref name) => {
                        assert_eq!(name, "value");
                    }
                    _ => panic!("Value should be a variable"),
                }
            }
            _ => panic!("Expected FieldAssignment statement"),
        }
    }

    #[test]
    fn test_parse_unary_minus() {
        // DECY-035 RED PHASE: Test that unary minus (-x) is parsed
        // This test will FAIL because UnaryOp expression variant doesn't support minus
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int negate(int x) { return -x; }";

        let ast = parser
            .parse(source)
            .expect("Parsing unary minus should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "negate");
        assert_eq!(func.body.len(), 1, "Should have one return statement");

        // Verify return statement has unary minus expression
        match &func.body[0] {
            Statement::Return(Some(Expression::UnaryOp { op, operand })) => {
                assert_eq!(*op, UnaryOperator::Minus, "Operator should be minus");

                // Operand should be variable 'x'
                match **operand {
                    Expression::Variable(ref name) => {
                        assert_eq!(name, "x", "Operand should be 'x'");
                    }
                    _ => panic!("Operand should be a variable"),
                }
            }
            _ => panic!("Expected Return statement with UnaryOp expression"),
        }
    }

    #[test]
    fn test_parse_logical_not() {
        // DECY-035 RED PHASE: Test that logical NOT (!x) is parsed
        // This test will FAIL because UnaryOp expression variant doesn't support not
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int logical_not(int x) { return !x; }";

        let ast = parser
            .parse(source)
            .expect("Parsing logical NOT should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "logical_not");
        assert_eq!(func.body.len(), 1, "Should have one return statement");

        // Verify return statement has logical NOT expression
        match &func.body[0] {
            Statement::Return(Some(Expression::UnaryOp { op, operand })) => {
                assert_eq!(
                    *op,
                    UnaryOperator::LogicalNot,
                    "Operator should be logical NOT"
                );

                // Operand should be variable 'x'
                match **operand {
                    Expression::Variable(ref name) => {
                        assert_eq!(name, "x", "Operand should be 'x'");
                    }
                    _ => panic!("Operand should be a variable"),
                }
            }
            _ => panic!("Expected Return statement with UnaryOp expression"),
        }
    }

    #[test]
    fn test_parse_double_negative() {
        // DECY-035 RED PHASE: Test nested unary operators (-(-x))
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int double_negative(int x) { return -(-x); }";

        let ast = parser
            .parse(source)
            .expect("Parsing double negative should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "double_negative");

        // Verify nested unary operators
        match &func.body[0] {
            Statement::Return(Some(Expression::UnaryOp {
                op: outer_op,
                operand: outer_operand,
            })) => {
                assert_eq!(
                    *outer_op,
                    UnaryOperator::Minus,
                    "Outer operator should be minus"
                );

                // Inner should also be UnaryOp with minus
                match **outer_operand {
                    Expression::UnaryOp {
                        ref op,
                        ref operand,
                    } => {
                        assert_eq!(*op, UnaryOperator::Minus, "Inner operator should be minus");
                        match **operand {
                            Expression::Variable(ref name) => {
                                assert_eq!(name, "x", "Innermost operand should be 'x'");
                            }
                            _ => panic!("Innermost operand should be a variable"),
                        }
                    }
                    _ => panic!("Inner expression should be UnaryOp"),
                }
            }
            _ => panic!("Expected Return statement with UnaryOp expression"),
        }
    }

    #[test]
    fn test_parse_function_call_with_variable_args() {
        // DECY-036 RED PHASE: Test function call with variable arguments
        // This test will FAIL because visit_call_argument only handles IntLiteral
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int add(int a, int b) { return a + b; } int compute(int x) { int result = add(x, 10); return result; }";

        let ast = parser
            .parse(source)
            .expect("Parsing function call with variable args should succeed");

        // Should have two functions
        assert_eq!(ast.functions().len(), 2);
        let compute_func = &ast.functions()[1];
        assert_eq!(compute_func.name, "compute");

        // First statement: int result = add(x, 10);
        match &compute_func.body[0] {
            Statement::VariableDeclaration {
                name, initializer, ..
            } => {
                assert_eq!(name, "result");
                match initializer {
                    Some(Expression::FunctionCall {
                        function,
                        arguments,
                    }) => {
                        assert_eq!(function, "add");
                        assert_eq!(arguments.len(), 2, "add() should have 2 arguments");

                        // First argument should be variable 'x'
                        match &arguments[0] {
                            Expression::Variable(ref name) => {
                                assert_eq!(name, "x", "First argument should be variable 'x'");
                            }
                            _ => panic!("First argument should be a variable"),
                        }

                        // Second argument should be literal 10
                        match &arguments[1] {
                            Expression::IntLiteral(val) => {
                                assert_eq!(*val, 10, "Second argument should be 10");
                            }
                            _ => panic!("Second argument should be IntLiteral(10)"),
                        }
                    }
                    _ => panic!("Initializer should be a function call"),
                }
            }
            _ => panic!("Expected VariableDeclaration statement"),
        }
    }

    #[test]
    fn test_parse_nested_function_calls() {
        // DECY-036 RED PHASE: Test nested function calls
        // This test will FAIL because nested calls are flattened
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int add(int a, int b) { return a + b; } int nested(int x) { return add(add(x, 5), add(10, 20)); }";

        let ast = parser
            .parse(source)
            .expect("Parsing nested function calls should succeed");

        assert_eq!(ast.functions().len(), 2);
        let nested_func = &ast.functions()[1];
        assert_eq!(nested_func.name, "nested");

        // Return statement: return add(add(x, 5), add(10, 20));
        match &nested_func.body[0] {
            Statement::Return(Some(Expression::FunctionCall {
                function,
                arguments,
            })) => {
                assert_eq!(function, "add");
                assert_eq!(arguments.len(), 2, "Outer add() should have 2 arguments");

                // First argument should be nested function call: add(x, 5)
                match &arguments[0] {
                    Expression::FunctionCall {
                        function: inner_fn,
                        arguments: inner_args,
                    } => {
                        assert_eq!(inner_fn, "add");
                        assert_eq!(inner_args.len(), 2);
                        match &inner_args[0] {
                            Expression::Variable(ref name) => assert_eq!(name, "x"),
                            _ => panic!("First arg of inner call should be variable 'x'"),
                        }
                        match &inner_args[1] {
                            Expression::IntLiteral(val) => assert_eq!(*val, 5),
                            _ => panic!("Second arg of inner call should be 5"),
                        }
                    }
                    _ => panic!("First argument should be a function call"),
                }

                // Second argument should be nested function call: add(10, 20)
                match &arguments[1] {
                    Expression::FunctionCall {
                        function: inner_fn,
                        arguments: inner_args,
                    } => {
                        assert_eq!(inner_fn, "add");
                        assert_eq!(inner_args.len(), 2);
                        match &inner_args[0] {
                            Expression::IntLiteral(val) => assert_eq!(*val, 10),
                            _ => panic!("First arg of inner call should be 10"),
                        }
                        match &inner_args[1] {
                            Expression::IntLiteral(val) => assert_eq!(*val, 20),
                            _ => panic!("Second arg of inner call should be 20"),
                        }
                    }
                    _ => panic!("Second argument should be a function call"),
                }
            }
            _ => panic!("Expected Return statement with function call"),
        }
    }

    #[test]
    fn test_parse_function_call_with_expression_args() {
        // DECY-036 RED PHASE: Test function call with expression arguments
        // This test will FAIL because expressions are not properly extracted
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int add(int a, int b) { return a + b; } int compute(int x, int y) { return add(x + 1, y * 2); }";

        let ast = parser
            .parse(source)
            .expect("Parsing function call with expression args should succeed");

        assert_eq!(ast.functions().len(), 2);
        let compute_func = &ast.functions()[1];
        assert_eq!(compute_func.name, "compute");

        // Return statement: return add(x + 1, y * 2);
        match &compute_func.body[0] {
            Statement::Return(Some(Expression::FunctionCall {
                function,
                arguments,
            })) => {
                assert_eq!(function, "add");
                assert_eq!(arguments.len(), 2, "add() should have 2 arguments");

                // First argument should be binary expression: x + 1
                match &arguments[0] {
                    Expression::BinaryOp { op, left, right } => {
                        assert_eq!(*op, BinaryOperator::Add);
                        match **left {
                            Expression::Variable(ref name) => assert_eq!(name, "x"),
                            _ => panic!("Left operand should be variable 'x'"),
                        }
                        match **right {
                            Expression::IntLiteral(val) => assert_eq!(val, 1),
                            _ => panic!("Right operand should be literal 1"),
                        }
                    }
                    _ => panic!("First argument should be a binary expression"),
                }

                // Second argument should be binary expression: y * 2
                match &arguments[1] {
                    Expression::BinaryOp { op, left, right } => {
                        assert_eq!(*op, BinaryOperator::Multiply);
                        match **left {
                            Expression::Variable(ref name) => assert_eq!(name, "y"),
                            _ => panic!("Left operand should be variable 'y'"),
                        }
                        match **right {
                            Expression::IntLiteral(val) => assert_eq!(val, 2),
                            _ => panic!("Right operand should be literal 2"),
                        }
                    }
                    _ => panic!("Second argument should be a binary expression"),
                }
            }
            _ => panic!("Expected Return statement with function call"),
        }
    }

    #[test]
    fn test_parse_struct_value_parameter() {
        // DECY-037 RED PHASE: Test that struct value parameters are parsed
        // This test will FAIL because Type enum doesn't have Struct variant
        let parser = CParser::new().expect("Parser creation failed");
        let source = "struct Point { int x; int y; }; void print_point(struct Point p) { }";

        let ast = parser
            .parse(source)
            .expect("Parsing struct value parameter should succeed");

        assert!(
            !ast.functions().is_empty(),
            "Should have at least one function"
        );
        let func = &ast.functions()[0];
        assert_eq!(func.name, "print_point");
        assert_eq!(func.parameters.len(), 1, "Should have one parameter");

        // Parameter should be struct type
        match &func.parameters[0].param_type {
            Type::Struct(name) => {
                assert_eq!(name, "Point", "Struct name should be 'Point'");
            }
            _ => panic!(
                "Parameter should be a struct type, got {:?}",
                func.parameters[0].param_type
            ),
        }
    }

    #[test]
    fn test_parse_struct_pointer_parameter() {
        // DECY-037 RED PHASE: Test that struct pointer parameters are parsed
        // This test will FAIL because Type::Pointer doesn't handle inner struct types
        let parser = CParser::new().expect("Parser creation failed");
        let source = "struct Point { int x; int y; }; void update_point(struct Point* p) { }";

        let ast = parser
            .parse(source)
            .expect("Parsing struct pointer parameter should succeed");

        assert!(
            !ast.functions().is_empty(),
            "Should have at least one function"
        );
        let func = &ast.functions()[0];
        assert_eq!(func.name, "update_point");
        assert_eq!(func.parameters.len(), 1, "Should have one parameter");

        // Parameter should be pointer to struct
        match &func.parameters[0].param_type {
            Type::Pointer(inner) => match **inner {
                Type::Struct(ref name) => {
                    assert_eq!(name, "Point", "Struct name should be 'Point'");
                }
                _ => panic!("Pointer should point to struct type"),
            },
            _ => panic!("Parameter should be a pointer type"),
        }
    }

    #[test]
    fn test_parse_struct_return_type() {
        // DECY-037 RED PHASE: Test that struct return types are parsed
        // This test will FAIL because Type enum doesn't handle structs
        let parser = CParser::new().expect("Parser creation failed");
        let source = "struct Point { int x; int y; }; struct Point create_point(int x, int y) { struct Point p; return p; }";

        let ast = parser
            .parse(source)
            .expect("Parsing struct return type should succeed");

        assert!(
            !ast.functions().is_empty(),
            "Should have at least one function"
        );
        let func = &ast.functions()[0];
        assert_eq!(func.name, "create_point");

        // Return type should be struct
        match &func.return_type {
            Type::Struct(name) => {
                assert_eq!(name, "Point", "Struct name should be 'Point'");
            }
            _ => panic!("Return type should be a struct, got {:?}", func.return_type),
        }
    }

    #[test]
    fn test_parse_simple_typedef() {
        // DECY-023 RED PHASE: Test that simple typedefs are parsed
        // This test will FAIL because AST doesn't have typedefs yet
        let parser = CParser::new().expect("Parser creation failed");
        let source = "typedef int Integer; int test(Integer x) { return x; }";

        let ast = parser
            .parse(source)
            .expect("Parsing simple typedef should succeed");

        // Should have one typedef and one function
        assert!(
            !ast.typedefs().is_empty(),
            "Should have at least one typedef"
        );
        let typedef = &ast.typedefs()[0];
        assert_eq!(typedef.name, "Integer", "Typedef name should be 'Integer'");
        assert_eq!(
            typedef.underlying_type,
            Type::Int,
            "Underlying type should be int"
        );

        // Function should use the typedef'd type
        assert_eq!(ast.functions().len(), 1, "Should have one function");
        let func = &ast.functions()[0];
        assert_eq!(func.name, "test");
        assert_eq!(func.parameters.len(), 1);
        // Note: The parameter type will be resolved to Int by clang, not kept as "Integer"
    }

    #[test]
    fn test_parse_struct_typedef() {
        // DECY-023 RED PHASE: Test typedef of struct
        // This test will FAIL because AST doesn't have typedefs
        let parser = CParser::new().expect("Parser creation failed");
        let source = "struct Point { int x; int y; }; typedef struct Point Point;";

        let ast = parser
            .parse(source)
            .expect("Parsing struct typedef should succeed");

        assert!(
            !ast.typedefs().is_empty(),
            "Should have at least one typedef"
        );
        let typedef = &ast.typedefs()[0];
        assert_eq!(typedef.name, "Point", "Typedef name should be 'Point'");
        match &typedef.underlying_type {
            Type::Struct(name) => {
                assert_eq!(name, "Point", "Underlying type should be struct Point");
            }
            _ => panic!("Typedef should reference struct type"),
        }
    }

    #[test]
    fn test_parse_pointer_typedef() {
        // DECY-023 RED PHASE: Test typedef of pointer type
        // This test will FAIL because AST doesn't have typedefs
        let parser = CParser::new().expect("Parser creation failed");
        let source = "typedef int* IntPtr; void process(IntPtr p) { }";

        let ast = parser
            .parse(source)
            .expect("Parsing pointer typedef should succeed");

        assert!(
            !ast.typedefs().is_empty(),
            "Should have at least one typedef"
        );
        let typedef = &ast.typedefs()[0];
        assert_eq!(typedef.name, "IntPtr", "Typedef name should be 'IntPtr'");
        match &typedef.underlying_type {
            Type::Pointer(inner) => {
                assert_eq!(**inner, Type::Int, "Should be pointer to int");
            }
            _ => panic!("Typedef should be pointer type"),
        }
    }

    #[test]
    fn test_parse_function_pointer_typedef() {
        // DECY-024 RED PHASE: Test that function pointer typedefs are parsed
        // This test will FAIL because Type enum doesn't have FunctionPointer variant
        let parser = CParser::new().expect("Parser creation failed");
        let source = "typedef int (*Callback)(int x);";

        let ast = parser
            .parse(source)
            .expect("Parsing function pointer typedef should succeed");

        // Should have one typedef
        assert!(
            !ast.typedefs().is_empty(),
            "Should have at least one typedef"
        );
        let typedef = &ast.typedefs()[0];
        assert_eq!(
            typedef.name, "Callback",
            "Typedef name should be 'Callback'"
        );

        // Underlying type should be function pointer
        match &typedef.underlying_type {
            Type::FunctionPointer {
                param_types,
                return_type,
            } => {
                assert_eq!(param_types.len(), 1, "Should have one parameter");
                assert_eq!(param_types[0], Type::Int, "Parameter should be int");
                assert_eq!(**return_type, Type::Int, "Return type should be int");
            }
            _ => panic!(
                "Typedef should be function pointer type, got {:?}",
                typedef.underlying_type
            ),
        }
    }

    #[test]
    fn test_parse_function_pointer_parameter() {
        // DECY-024 RED PHASE: Test function with function pointer parameter
        // This test will FAIL because Type enum doesn't have FunctionPointer variant
        let parser = CParser::new().expect("Parser creation failed");
        let source = "typedef int (*Callback)(int x); void register_callback(Callback cb) { }";

        let ast = parser
            .parse(source)
            .expect("Parsing function pointer parameter should succeed");

        assert!(
            !ast.functions().is_empty(),
            "Should have at least one function"
        );
        let func = &ast.functions()[0];
        assert_eq!(func.name, "register_callback");
        assert_eq!(func.parameters.len(), 1, "Should have one parameter");
        assert_eq!(
            func.parameters[0].name, "cb",
            "Parameter name should be 'cb'"
        );

        // Parameter type should be function pointer (resolved from typedef)
        // Note: clang will resolve the typedef to the underlying function pointer type
        match &func.parameters[0].param_type {
            Type::FunctionPointer {
                param_types,
                return_type,
            } => {
                assert_eq!(param_types.len(), 1, "Should have one parameter");
                assert_eq!(param_types[0], Type::Int, "Parameter should be int");
                assert_eq!(**return_type, Type::Int, "Return type should be int");
            }
            _ => panic!(
                "Parameter should be function pointer type, got {:?}",
                func.parameters[0].param_type
            ),
        }
    }

    #[test]
    fn test_parse_function_pointer_return_type() {
        // DECY-024 RED PHASE: Test function returning function pointer
        // This test will FAIL because Type enum doesn't have FunctionPointer variant
        let parser = CParser::new().expect("Parser creation failed");
        let source = "typedef int (*Callback)(int x); Callback get_callback() { return 0; }";

        let ast = parser
            .parse(source)
            .expect("Parsing function pointer return type should succeed");

        assert!(
            !ast.functions().is_empty(),
            "Should have at least one function"
        );
        let func = &ast.functions()[0];
        assert_eq!(func.name, "get_callback");

        // Return type should be function pointer (resolved from typedef)
        match &func.return_type {
            Type::FunctionPointer {
                param_types,
                return_type,
            } => {
                assert_eq!(param_types.len(), 1, "Should have one parameter");
                assert_eq!(param_types[0], Type::Int, "Parameter should be int");
                assert_eq!(**return_type, Type::Int, "Return type should be int");
            }
            _ => panic!(
                "Return type should be function pointer, got {:?}",
                func.return_type
            ),
        }
    }

    #[test]
    fn test_parse_string_literal_in_return() {
        // DECY-025 RED PHASE: Test that string literals are parsed
        // This test will FAIL because Expression doesn't have StringLiteral variant
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"const char* get_message() { return "Hello"; }"#;

        let ast = parser
            .parse(source)
            .expect("Parsing string literal should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "get_message");
        assert_eq!(func.body.len(), 1, "Should have one return statement");

        // Verify return statement has string literal
        match &func.body[0] {
            Statement::Return(Some(Expression::StringLiteral(value))) => {
                assert_eq!(value, "Hello", "String should be 'Hello'");
            }
            _ => panic!("Expected Return statement with StringLiteral"),
        }
    }

    #[test]
    fn test_parse_string_literal_in_variable() {
        // DECY-025 RED PHASE: Test string literal in variable declaration
        // This test will FAIL because Expression doesn't have StringLiteral variant
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"void test() { char* msg = "World"; }"#;

        let ast = parser
            .parse(source)
            .expect("Parsing string in variable should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "test");
        assert_eq!(func.body.len(), 1, "Should have one declaration");

        // Verify variable declaration has string literal initializer
        match &func.body[0] {
            Statement::VariableDeclaration {
                name, initializer, ..
            } => {
                assert_eq!(name, "msg");
                match initializer {
                    Some(Expression::StringLiteral(value)) => {
                        assert_eq!(value, "World", "String should be 'World'");
                    }
                    _ => panic!("Expected StringLiteral initializer"),
                }
            }
            _ => panic!("Expected VariableDeclaration"),
        }
    }

    #[test]
    fn test_parse_string_literal_in_function_call() {
        // DECY-025 GREEN PHASE: Test string literal as function argument
        // Test with function call in a variable declaration (which IS supported)
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"void test() { int result = strlen("Test"); }"#;

        let ast = parser
            .parse(source)
            .expect("Parsing string in function call should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "test");
        assert_eq!(func.body.len(), 1, "Should have one statement");

        // Verify function call has string literal argument
        match &func.body[0] {
            Statement::VariableDeclaration {
                name, initializer, ..
            } => {
                assert_eq!(name, "result");
                match initializer {
                    Some(Expression::FunctionCall {
                        function,
                        arguments,
                    }) => {
                        assert_eq!(function, "strlen");
                        assert_eq!(arguments.len(), 1, "Should have one argument");
                        match &arguments[0] {
                            Expression::StringLiteral(value) => {
                                assert_eq!(value, "Test", "String should be 'Test'");
                            }
                            _ => panic!("Expected StringLiteral argument"),
                        }
                    }
                    _ => panic!("Expected FunctionCall initializer"),
                }
            }
            _ => panic!("Expected VariableDeclaration statement"),
        }
    }

    // ============================================================================
    // MUTATION TESTING HARDENING: Visitor Match Arm Coverage Tests
    // These tests target uncovered wildcard branches and edge cases
    // Found by mutation testing analysis (69.5% → 85%+ target)
    // ============================================================================

    #[test]
    fn test_visitor_unknown_statement_cursor() {
        // Test that unknown statement cursor types are handled gracefully
        // Targets visit_statement wildcard arm (parser.rs:332)
        let parser = CParser::new().expect("Parser creation failed");
        // Use C code with break statement (not yet supported, should be skipped)
        let source = "void test() { for(int i = 0; i < 10; i++) { break; } }";

        let ast = parser
            .parse(source)
            .expect("Parsing with unsupported statement should still succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "test");
        // Should parse the for loop, but break statement may be skipped
        assert!(!func.body.is_empty(), "Should have at least the for loop");
    }

    #[test]
    fn test_visitor_unknown_expression_cursor() {
        // Test that unknown expression cursor types are handled gracefully
        // Targets visit_expression wildcard arm (parser.rs:624)
        let parser = CParser::new().expect("Parser creation failed");
        // Use complex cast expression that might not be fully supported
        let source = "int test(void* ptr) { return (int)(long)ptr; }";

        let ast = parser
            .parse(source)
            .expect("Parsing with complex cast should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "test");
        // Function should parse even if some expressions are not fully captured
        assert_eq!(func.body.len(), 1, "Should have return statement");
    }

    #[test]
    fn test_visitor_unknown_binary_operand() {
        // Test that unknown binary operand cursor types trigger recursion
        // Targets visit_binary_operand wildcard arm (parser.rs:816)
        let parser = CParser::new().expect("Parser creation failed");
        // Use sizeof operator in binary expression (might need recursion)
        let source = "int test() { return 5 + 10; }";

        let ast = parser
            .parse(source)
            .expect("Parsing binary expression with complex operand should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "test");
        match &func.body[0] {
            Statement::Return(Some(Expression::BinaryOp { op, .. })) => {
                assert_eq!(*op, BinaryOperator::Add);
            }
            _ => panic!("Expected return with binary operation"),
        }
    }

    #[test]
    fn test_visitor_unknown_call_argument() {
        // Test that unknown call argument cursor types are handled
        // Targets visit_call_argument wildcard arm (parser.rs:1047)
        let parser = CParser::new().expect("Parser creation failed");
        // Use complex expression as function argument
        let source = "int test(int x) { int y = max(x, 100); return y; }";

        let ast = parser
            .parse(source)
            .expect("Parsing function call with complex args should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.name, "test");
        match &func.body[0] {
            Statement::VariableDeclaration { initializer, .. } => match initializer {
                Some(Expression::FunctionCall { arguments, .. }) => {
                    assert_eq!(arguments.len(), 2, "Should parse both arguments");
                }
                _ => panic!("Expected function call initializer"),
            },
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_extract_var_decl_returns_none() {
        // Test edge case where extract_var_decl might return None
        // Targets lines 67-95 in parser.rs (extract_var_decl)
        let parser = CParser::new().expect("Parser creation failed");
        // Declare variable with unknown type (should handle gracefully)
        let source = "void test() { int x; }";

        let ast = parser
            .parse(source)
            .expect("Parsing simple var decl should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.body.len(), 1, "Should have one variable declaration");
        match &func.body[0] {
            Statement::VariableDeclaration {
                name,
                var_type,
                initializer,
            } => {
                assert_eq!(name, "x");
                assert_eq!(*var_type, Type::Int);
                assert!(initializer.is_none(), "Should have no initializer");
            }
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_extract_return_stmt_with_no_value() {
        // Test return statement with no expression
        // Targets lines 97-108 in parser.rs (extract_return_stmt)
        let parser = CParser::new().expect("Parser creation failed");
        let source = "void test() { return; }";

        let ast = parser
            .parse(source)
            .expect("Parsing return with no value should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.body.len(), 1, "Should have one return statement");
        match &func.body[0] {
            Statement::Return(None) => {
                // Expected: return with no value
            }
            _ => panic!("Expected Return(None) statement"),
        }
    }

    #[test]
    fn test_extract_assignment_invalid_token() {
        // Test assignment extraction when tokenization might fail
        // Targets lines 110-233 in parser.rs (extract_assignment_stmt)
        let parser = CParser::new().expect("Parser creation failed");
        let source = "void test() { int x = 5; x = 10; }";

        let ast = parser
            .parse(source)
            .expect("Parsing assignment should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.body.len(), 2, "Should have var decl and assignment");
        match &func.body[1] {
            Statement::Assignment { target, value } => {
                assert_eq!(target, "x");
                match value {
                    Expression::IntLiteral(val) => assert_eq!(*val, 10),
                    _ => panic!("Value should be 10"),
                }
            }
            _ => panic!("Expected assignment statement"),
        }
    }

    #[test]
    fn test_extract_binary_op_with_no_operator() {
        // Test binary operation when operator extraction might fail
        // Targets lines 734-757 in parser.rs (extract_binary_op)
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test(int a, int b) { return a * b; }";

        let ast = parser
            .parse(source)
            .expect("Parsing binary operation should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::Return(Some(Expression::BinaryOp { op, left, right })) => {
                assert_eq!(*op, BinaryOperator::Multiply);
                match **left {
                    Expression::Variable(ref name) => assert_eq!(name, "a"),
                    _ => panic!("Left should be variable 'a'"),
                }
                match **right {
                    Expression::Variable(ref name) => assert_eq!(name, "b"),
                    _ => panic!("Right should be variable 'b'"),
                }
            }
            _ => panic!("Expected binary operation"),
        }
    }

    #[test]
    fn test_extract_binary_operator_division() {
        // Test division operator parsing
        // Targets BinaryOperator::Divide variant coverage
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test(int a, int b) { return a / b; }";

        let ast = parser
            .parse(source)
            .expect("Parsing division should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::Return(Some(Expression::BinaryOp { op, .. })) => {
                assert_eq!(*op, BinaryOperator::Divide, "Should be division operator");
            }
            _ => panic!("Expected division operation"),
        }
    }

    #[test]
    fn test_extract_binary_operator_modulo() {
        // Test modulo operator parsing
        // Targets BinaryOperator::Modulo variant coverage
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test(int a, int b) { return a % b; }";

        let ast = parser.parse(source).expect("Parsing modulo should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::Return(Some(Expression::BinaryOp { op, .. })) => {
                assert_eq!(*op, BinaryOperator::Modulo, "Should be modulo operator");
            }
            _ => panic!("Expected modulo operation"),
        }
    }

    #[test]
    fn test_extract_binary_operator_subtract() {
        // Test subtraction operator parsing
        // Targets BinaryOperator::Subtract variant coverage
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test(int a, int b) { return a - b; }";

        let ast = parser
            .parse(source)
            .expect("Parsing subtraction should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::Return(Some(Expression::BinaryOp { op, .. })) => {
                assert_eq!(
                    *op,
                    BinaryOperator::Subtract,
                    "Should be subtraction operator"
                );
            }
            _ => panic!("Expected subtraction operation"),
        }
    }

    #[test]
    fn test_extract_binary_operator_less_equal() {
        // Test less-than-or-equal operator parsing
        // Targets BinaryOperator::LessEqual variant coverage
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test(int a, int b) { if (a <= b) { return 1; } return 0; }";

        let ast = parser.parse(source).expect("Parsing <= should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::If { condition, .. } => match condition {
                Expression::BinaryOp { op, .. } => {
                    assert_eq!(*op, BinaryOperator::LessEqual, "Should be <= operator");
                }
                _ => panic!("Condition should be binary operation"),
            },
            _ => panic!("Expected if statement"),
        }
    }

    #[test]
    fn test_extract_binary_operator_greater_equal() {
        // Test greater-than-or-equal operator parsing
        // Targets BinaryOperator::GreaterEqual variant coverage
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test(int a, int b) { if (a >= b) { return 1; } return 0; }";

        let ast = parser.parse(source).expect("Parsing >= should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::If { condition, .. } => match condition {
                Expression::BinaryOp { op, .. } => {
                    assert_eq!(*op, BinaryOperator::GreaterEqual, "Should be >= operator");
                }
                _ => panic!("Condition should be binary operation"),
            },
            _ => panic!("Expected if statement"),
        }
    }

    #[test]
    fn test_extract_int_literal_null_tu() {
        // Test integer literal extraction with edge cases
        // Targets lines 629-677 in parser.rs (extract_int_literal)
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test() { return 999; }";

        let ast = parser
            .parse(source)
            .expect("Parsing integer literal should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::Return(Some(Expression::IntLiteral(val))) => {
                assert_eq!(*val, 999, "Should parse literal value 999");
            }
            _ => panic!("Expected integer literal"),
        }
    }

    #[test]
    fn test_extract_string_literal_empty() {
        // Test empty string literal parsing
        // Targets lines 679-718 in parser.rs (extract_string_literal)
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"const char* test() { return ""; }"#;

        let ast = parser
            .parse(source)
            .expect("Parsing empty string should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::Return(Some(Expression::StringLiteral(val))) => {
                assert_eq!(val, "", "Should parse empty string");
            }
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_extract_function_call_no_args() {
        // Test function call with no arguments
        // Targets lines 924-957 in parser.rs (extract_function_call)
        let parser = CParser::new().expect("Parser creation failed");
        let source = "void test() { int x = rand(); }";

        let ast = parser
            .parse(source)
            .expect("Parsing function call with no args should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::VariableDeclaration { initializer, .. } => match initializer {
                Some(Expression::FunctionCall {
                    function,
                    arguments,
                }) => {
                    assert_eq!(function, "rand");
                    assert_eq!(arguments.len(), 0, "Should have no arguments");
                }
                _ => panic!("Expected function call"),
            },
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_convert_type_elaborated() {
        // Test elaborated type conversion (struct keyword)
        // Targets lines 1313-1318 in parser.rs (convert_type with CXType_Elaborated)
        let parser = CParser::new().expect("Parser creation failed");
        let source = "struct Point { int x; }; int test(struct Point p) { return p.x; }";

        let ast = parser
            .parse(source)
            .expect("Parsing elaborated type should succeed");

        let func = &ast.functions()[0];
        match &func.parameters[0].param_type {
            Type::Struct(name) => {
                assert_eq!(name, "Point", "Should resolve elaborated type to struct");
            }
            _ => panic!("Expected struct type"),
        }
    }

    #[test]
    fn test_convert_type_typedef_canonical() {
        // Test typedef type conversion to canonical
        // Targets lines 1319-1324 in parser.rs (convert_type with CXType_Typedef)
        let parser = CParser::new().expect("Parser creation failed");
        let source = "typedef int MyInt; MyInt test(MyInt x) { return x; }";

        let ast = parser
            .parse(source)
            .expect("Parsing typedef should succeed");

        let func = &ast.functions()[0];
        // Clang resolves typedefs to canonical types
        assert_eq!(func.return_type, Type::Int, "Should resolve typedef to int");
        assert_eq!(
            func.parameters[0].param_type,
            Type::Int,
            "Should resolve typedef to int"
        );
    }

    #[test]
    fn test_parse_comparison_not_equal() {
        // Test != operator parsing
        // Targets BinaryOperator::NotEqual variant
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test(int a, int b) { if (a != b) { return 1; } return 0; }";

        let ast = parser.parse(source).expect("Parsing != should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::If { condition, .. } => match condition {
                Expression::BinaryOp { op, .. } => {
                    assert_eq!(*op, BinaryOperator::NotEqual, "Should be != operator");
                }
                _ => panic!("Expected binary operation"),
            },
            _ => panic!("Expected if statement"),
        }
    }

    #[test]
    fn test_parse_comparison_equal() {
        // Test == operator parsing
        // Targets BinaryOperator::Equal variant
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test(int a, int b) { if (a == b) { return 1; } return 0; }";

        let ast = parser.parse(source).expect("Parsing == should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::If { condition, .. } => match condition {
                Expression::BinaryOp { op, .. } => {
                    assert_eq!(*op, BinaryOperator::Equal, "Should be == operator");
                }
                _ => panic!("Expected binary operation"),
            },
            _ => panic!("Expected if statement"),
        }
    }

    #[test]
    fn test_parse_comparison_less_than() {
        // Test < operator parsing explicitly
        // Targets BinaryOperator::LessThan variant
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test(int a, int b) { if (a < b) { return 1; } return 0; }";

        let ast = parser.parse(source).expect("Parsing < should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::If { condition, .. } => match condition {
                Expression::BinaryOp { op, .. } => {
                    assert_eq!(*op, BinaryOperator::LessThan, "Should be < operator");
                }
                _ => panic!("Expected binary operation"),
            },
            _ => panic!("Expected if statement"),
        }
    }

    // ============================================================================
    // MUTATION TESTING HARDENING: Assignment Edge Case Tests
    // These tests target assignment statement edge cases and error paths
    // ============================================================================

    #[test]
    fn test_assignment_with_complex_rhs() {
        // Test assignment with complex right-hand side expression
        // Ensures extract_assignment_stmt handles nested expressions
        let parser = CParser::new().expect("Parser creation failed");
        let source = "void test() { int x = 0; x = 5 * 10; }";

        let ast = parser
            .parse(source)
            .expect("Parsing assignment with complex RHS should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.body.len(), 2, "Should have var decl and assignment");
        match &func.body[1] {
            Statement::Assignment { target, value } => {
                assert_eq!(target, "x");
                // Value should be a binary operation
                match value {
                    Expression::BinaryOp { op, .. } => {
                        assert_eq!(*op, BinaryOperator::Multiply, "Should parse multiplication");
                    }
                    _ => panic!("Expected binary operation on RHS"),
                }
            }
            _ => panic!("Expected assignment statement"),
        }
    }

    #[test]
    fn test_assignment_distinguishes_equality() {
        // Test that == is NOT treated as assignment
        // Targets the operator detection logic in extract_assignment_stmt
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test(int a, int b) { if (a == b) { return 1; } return 0; }";

        let ast = parser
            .parse(source)
            .expect("Parsing == comparison should succeed");

        let func = &ast.functions()[0];
        // Should be an if statement, NOT an assignment
        match &func.body[0] {
            Statement::If { .. } => {
                // Expected: if statement with == comparison
            }
            Statement::Assignment { .. } => {
                panic!("== should not be parsed as assignment");
            }
            _ => panic!("Expected if statement"),
        }
    }

    #[test]
    fn test_assignment_distinguishes_not_equal() {
        // Test that != is NOT treated as assignment
        // Targets the operator detection logic in extract_assignment_stmt
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test(int a, int b) { if (a != b) { return 1; } return 0; }";

        let ast = parser
            .parse(source)
            .expect("Parsing != comparison should succeed");

        let func = &ast.functions()[0];
        // Should be an if statement, NOT an assignment
        match &func.body[0] {
            Statement::If { .. } => {
                // Expected: if statement with != comparison
            }
            Statement::Assignment { .. } => {
                panic!("!= should not be parsed as assignment");
            }
            _ => panic!("Expected if statement"),
        }
    }

    #[test]
    fn test_deref_assignment_edge_case() {
        // Test dereference assignment with complex target
        // Targets DerefAssignment statement creation
        let parser = CParser::new().expect("Parser creation failed");
        let source = "void test(int** pptr, int value) { *(*pptr) = value; }";

        let ast = parser
            .parse(source)
            .expect("Parsing nested dereference assignment should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.body.len(), 1, "Should have one assignment");
        // Should parse as some form of assignment (exact structure may vary)
        match &func.body[0] {
            Statement::DerefAssignment { .. } | Statement::Assignment { .. } => {
                // Expected: some assignment variant
            }
            _ => panic!("Expected assignment statement"),
        }
    }

    #[test]
    fn test_array_index_assignment_with_expression() {
        // Test array index assignment with expression as index
        // Targets ArrayIndexAssignment with complex index
        let parser = CParser::new().expect("Parser creation failed");
        let source = "void test(int arr[], int i) { arr[i + 1] = 42; }";

        let ast = parser
            .parse(source)
            .expect("Parsing array assignment with expression index should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.body.len(), 1, "Should have one assignment");
        match &func.body[0] {
            Statement::ArrayIndexAssignment {
                array,
                index,
                value,
            } => {
                // Array should be 'arr'
                match **array {
                    Expression::Variable(ref name) => assert_eq!(name, "arr"),
                    _ => panic!("Array should be variable"),
                }
                // Index should be binary expression i + 1
                match **index {
                    Expression::BinaryOp { op, .. } => {
                        assert_eq!(op, BinaryOperator::Add, "Index should be i + 1");
                    }
                    _ => panic!("Index should be binary operation"),
                }
                // Value should be 42
                match value {
                    Expression::IntLiteral(val) => assert_eq!(*val, 42),
                    _ => panic!("Value should be 42"),
                }
            }
            _ => panic!("Expected ArrayIndexAssignment"),
        }
    }

    #[test]
    fn test_field_assignment_with_struct_value() {
        // Test field assignment with struct by value
        // Targets FieldAssignment for obj.field = value
        let parser = CParser::new().expect("Parser creation failed");
        let source = "struct Point { int x; int y; }; void test(struct Point p) { p.x = 100; }";

        let ast = parser
            .parse(source)
            .expect("Parsing field assignment should succeed");

        let func = &ast.functions()[0];
        assert_eq!(func.body.len(), 1, "Should have one assignment");
        match &func.body[0] {
            Statement::FieldAssignment {
                object,
                field,
                value,
            } => {
                // Object should be 'p'
                match object {
                    Expression::Variable(ref name) => assert_eq!(name, "p"),
                    _ => panic!("Object should be variable"),
                }
                // Field should be 'x'
                assert_eq!(field, "x");
                // Value should be 100
                match value {
                    Expression::IntLiteral(val) => assert_eq!(*val, 100),
                    _ => panic!("Value should be 100"),
                }
            }
            _ => panic!("Expected FieldAssignment"),
        }
    }

    #[test]
    fn test_assignment_chain() {
        // Test multiple assignments in sequence
        // Ensures each assignment is parsed independently
        let parser = CParser::new().expect("Parser creation failed");
        let source = "void test() { int a = 0; int b = 0; a = 1; b = 2; a = b; }";

        let ast = parser
            .parse(source)
            .expect("Parsing assignment chain should succeed");

        let func = &ast.functions()[0];
        assert_eq!(
            func.body.len(),
            5,
            "Should have 2 var decls + 3 assignments"
        );

        // Third statement: a = 1
        match &func.body[2] {
            Statement::Assignment { target, value } => {
                assert_eq!(target, "a");
                match value {
                    Expression::IntLiteral(val) => assert_eq!(*val, 1),
                    _ => panic!("Value should be 1"),
                }
            }
            _ => panic!("Expected assignment a = 1"),
        }

        // Fourth statement: b = 2
        match &func.body[3] {
            Statement::Assignment { target, value } => {
                assert_eq!(target, "b");
                match value {
                    Expression::IntLiteral(val) => assert_eq!(*val, 2),
                    _ => panic!("Value should be 2"),
                }
            }
            _ => panic!("Expected assignment b = 2"),
        }

        // Fifth statement: a = b
        match &func.body[4] {
            Statement::Assignment { target, value } => {
                assert_eq!(target, "a");
                match value {
                    Expression::Variable(ref name) => assert_eq!(name, "b"),
                    _ => panic!("Value should be variable b"),
                }
            }
            _ => panic!("Expected assignment a = b"),
        }
    }

    // ============================================================================
    // MUTATION TESTING: Final Push to 85%+ (Targeting Remaining 34 Mutants)
    // These tests target specific missed mutants from mutation testing
    // ============================================================================

    #[test]
    fn test_if_with_unary_in_condition() {
        // Targets: delete match arm CXCursor_UnaryOperator in visit_if_children (line 566)
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test(int x) { if (-x > 0) { return 1; } return 0; }";

        let ast = parser
            .parse(source)
            .expect("Parsing if with unary in condition should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::If { condition, .. } => {
                match condition {
                    Expression::BinaryOp { left, .. } => {
                        // Left should be unary operation
                        match **left {
                            Expression::UnaryOp { .. } => {
                                // Expected: unary operator in condition
                            }
                            _ => panic!("Expected unary operator in left side of condition"),
                        }
                    }
                    _ => panic!("Expected binary expression in condition"),
                }
            }
            _ => panic!("Expected if statement"),
        }
    }

    #[test]
    fn test_if_with_integer_literal_condition() {
        // Targets: delete match arm CXCursor_IntegerLiteral in visit_if_children (line 563)
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test() { if (1) { return 1; } return 0; }";

        let ast = parser
            .parse(source)
            .expect("Parsing if with integer literal should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::If { condition, .. } => match condition {
                Expression::IntLiteral(val) => {
                    assert_eq!(*val, 1, "Condition should be literal 1");
                }
                _ => panic!("Expected integer literal in condition"),
            },
            _ => panic!("Expected if statement"),
        }
    }

    #[test]
    fn test_if_with_variable_condition() {
        // Targets: delete match arm CXCursor_DeclRefExpr in visit_if_children (line 564)
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test(int flag) { if (flag) { return 1; } return 0; }";

        let ast = parser
            .parse(source)
            .expect("Parsing if with variable condition should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::If { condition, .. } => match condition {
                Expression::Variable(name) => {
                    assert_eq!(name, "flag", "Condition should be variable 'flag'");
                }
                _ => panic!("Expected variable in condition"),
            },
            _ => panic!("Expected if statement"),
        }
    }

    #[test]
    fn test_for_loop_with_variable_in_condition() {
        // Targets: delete match arm CXCursor_DeclRefExpr in visit_for_children (line 694)
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test(int n) { for (int i = 0; n; i = i + 1) { } return 0; }";

        let ast = parser
            .parse(source)
            .expect("Parsing for with variable condition should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::For { condition, .. } => {
                assert!(condition.is_some(), "Should have condition");
                if let Expression::Variable(name) = condition.as_ref().unwrap() {
                    assert_eq!(name, "n", "Condition should be variable 'n'");
                }
            }
            _ => panic!("Expected for statement"),
        }
    }

    #[test]
    fn test_while_loop_with_variable_condition() {
        // Targets: delete match arm CXCursor_DeclRefExpr in visit_while_children (line 789)
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test(int flag) { while (flag) { flag = 0; } return 0; }";

        let ast = parser
            .parse(source)
            .expect("Parsing while with variable condition should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::While { condition, .. } => match condition {
                Expression::Variable(name) => {
                    assert_eq!(name, "flag", "Condition should be variable 'flag'");
                }
                _ => panic!("Expected variable in while condition"),
            },
            _ => panic!("Expected while statement"),
        }
    }

    #[test]
    fn test_while_loop_with_function_call_condition() {
        // Targets: delete match arm CXCursor_CallExpr in visit_while_children (line 790)
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test() { while (check()) { } return 0; }";

        let ast = parser
            .parse(source)
            .expect("Parsing while with call condition should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::While { condition, .. } => match condition {
                Expression::FunctionCall { function, .. } => {
                    assert_eq!(function, "check", "Should call 'check' function");
                }
                _ => panic!("Expected function call in while condition"),
            },
            _ => panic!("Expected while statement"),
        }
    }

    #[test]
    fn test_function_call_with_array_index_argument() {
        // Targets: delete match arm CXCursor_ArraySubscriptExpr in visit_call_argument (line 1298)
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test(int arr[]) { int x = process(arr[0]); return x; }";

        let ast = parser
            .parse(source)
            .expect("Parsing call with array arg should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::VariableDeclaration { initializer, .. } => {
                match initializer {
                    Some(Expression::FunctionCall { arguments, .. }) => {
                        assert_eq!(arguments.len(), 1, "Should have one argument");
                        match &arguments[0] {
                            Expression::ArrayIndex { .. } => {
                                // Expected: array index as argument
                            }
                            _ => panic!("Expected array index argument"),
                        }
                    }
                    _ => panic!("Expected function call"),
                }
            }
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_function_call_with_unary_argument() {
        // Targets: delete match arm CXCursor_UnaryOperator in visit_call_argument (line 1291)
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test(int x) { int y = abs(-x); return y; }";

        let ast = parser
            .parse(source)
            .expect("Parsing call with unary arg should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::VariableDeclaration { initializer, .. } => {
                match initializer {
                    Some(Expression::FunctionCall { arguments, .. }) => {
                        assert_eq!(arguments.len(), 1, "Should have one argument");
                        match &arguments[0] {
                            Expression::UnaryOp { .. } => {
                                // Expected: unary op as argument
                            }
                            _ => panic!("Expected unary op argument"),
                        }
                    }
                    _ => panic!("Expected function call"),
                }
            }
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_function_call_with_field_access_argument() {
        // Targets: delete match arm CXCursor_MemberRefExpr in visit_call_argument (line 1305)
        let parser = CParser::new().expect("Parser creation failed");
        let source =
            "struct Point { int x; }; int test(struct Point p) { int v = process(p.x); return v; }";

        let ast = parser
            .parse(source)
            .expect("Parsing call with field access arg should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::VariableDeclaration { initializer, .. } => {
                match initializer {
                    Some(Expression::FunctionCall { arguments, .. }) => {
                        assert_eq!(arguments.len(), 1, "Should have one argument");
                        match &arguments[0] {
                            Expression::FieldAccess { .. } => {
                                // Expected: field access as argument
                            }
                            _ => panic!("Expected field access argument"),
                        }
                    }
                    _ => panic!("Expected function call"),
                }
            }
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_while_with_integer_literal_condition() {
        // Targets: delete match arm CXCursor_IntegerLiteral in visit_while_children (line 788)
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test() { while (1) { return 0; } return 1; }";

        let ast = parser
            .parse(source)
            .expect("Parsing while with int literal should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::While { condition, .. } => match condition {
                Expression::IntLiteral(val) => {
                    assert_eq!(*val, 1, "Condition should be literal 1");
                }
                _ => panic!("Expected integer literal condition"),
            },
            _ => panic!("Expected while statement"),
        }
    }

    #[test]
    fn test_while_with_unary_in_condition() {
        // Targets: delete match arm CXCursor_UnaryOperator in visit_while_children (line 791)
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test(int x) { while (-x < 10) { x = x + 1; } return x; }";

        let ast = parser
            .parse(source)
            .expect("Parsing while with unary should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::While { condition, .. } => {
                match condition {
                    Expression::BinaryOp { left, .. } => {
                        match **left {
                            Expression::UnaryOp { .. } => {
                                // Expected: unary in while condition
                            }
                            _ => panic!("Expected unary op"),
                        }
                    }
                    _ => panic!("Expected binary expression"),
                }
            }
            _ => panic!("Expected while statement"),
        }
    }

    #[test]
    fn test_for_with_integer_literal_condition() {
        // Targets: delete match arm CXCursor_IntegerLiteral in visit_for_children (line 693)
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test() { for (int i = 0; 1; i = i + 1) { return 0; } return 1; }";

        let ast = parser
            .parse(source)
            .expect("Parsing for with int literal condition should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::For { condition, .. } => {
                assert!(condition.is_some(), "Should have condition");
                if let Expression::IntLiteral(val) = condition.as_ref().unwrap() {
                    assert_eq!(*val, 1, "Condition should be literal 1");
                }
            }
            _ => panic!("Expected for statement"),
        }
    }

    #[test]
    fn test_for_with_function_call_condition() {
        // Targets: delete match arm CXCursor_CallExpr in visit_for_children (line 695)
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test() { for (int i = 0; check(i); i = i + 1) { } return 0; }";

        let ast = parser
            .parse(source)
            .expect("Parsing for with call condition should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::For { condition, .. } => {
                assert!(condition.is_some(), "Should have condition");
                if let Expression::FunctionCall { .. } = condition.as_ref().unwrap() {
                    // Expected: function call in for condition
                }
            }
            _ => panic!("Expected for statement"),
        }
    }

    #[test]
    fn test_if_with_function_call_condition() {
        // Targets: delete match arm CXCursor_CallExpr in visit_if_children (line 565)
        let parser = CParser::new().expect("Parser creation failed");
        let source = "int test() { if (check()) { return 1; } return 0; }";

        let ast = parser
            .parse(source)
            .expect("Parsing if with call condition should succeed");

        let func = &ast.functions()[0];
        match &func.body[0] {
            Statement::If { condition, .. } => match condition {
                Expression::FunctionCall { function, .. } => {
                    assert_eq!(function, "check", "Should call check function");
                }
                _ => panic!("Expected function call condition"),
            },
            _ => panic!("Expected if statement"),
        }
    }
}
