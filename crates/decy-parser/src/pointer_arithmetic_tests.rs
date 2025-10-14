//! RED PHASE Tests for Pointer Arithmetic (DECY-041)
//!
//! These tests are EXPECTED TO FAIL initially.
//! They define the behavior we want to implement for:
//! - Increment/decrement operators (++, --)
//! - Compound assignment operators (+=, -=, *=, /=, %=)
//! - Pointer arithmetic patterns

use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // RED PHASE: Post-increment operator (ptr++)
    // ============================================================================

    #[test]
    fn test_post_increment_in_for_loop() {
        // RED: Test that ptr++ is parsed correctly
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            void traverse(int* arr, int size) {
                int* ptr = arr;
                int i;
                for (i = 0; i < size; i++) {
                    ptr++;
                }
            }
        "#;

        let ast = parser.parse(source).expect("Parsing should succeed");
        let func = &ast.functions()[0];

        // Check that ptr++ appears in the for loop body
        if let Statement::For { body, .. } = &func.body[2] {
            // Should have ptr++ as a statement
            assert_eq!(body.len(), 1, "For loop should have 1 statement");

            // RED: This will fail because we don't have PostIncrement yet
            match &body[0] {
                Statement::PostIncrement { target } => {
                    assert_eq!(target, "ptr", "Should increment ptr");
                }
                other => panic!("Expected PostIncrement statement, got {:?}", other),
            }
        } else {
            panic!("Expected for loop as third statement");
        }
    }

    #[test]
    fn test_pre_increment() {
        // RED: Test that ++ptr is parsed correctly
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            void test() {
                int* ptr;
                ++ptr;
            }
        "#;

        let ast = parser.parse(source).expect("Parsing should succeed");
        let func = &ast.functions()[0];

        // RED: This will fail because we don't have PreIncrement yet
        match &func.body[1] {
            Statement::PreIncrement { target } => {
                assert_eq!(target, "ptr", "Should increment ptr");
            }
            other => panic!("Expected PreIncrement statement, got {:?}", other),
        }
    }

    #[test]
    fn test_post_decrement() {
        // RED: Test that ptr-- is parsed correctly
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            void test() {
                int* ptr;
                ptr--;
            }
        "#;

        let ast = parser.parse(source).expect("Parsing should succeed");
        let func = &ast.functions()[0];

        // RED: This will fail because we don't have PostDecrement yet
        match &func.body[1] {
            Statement::PostDecrement { target } => {
                assert_eq!(target, "ptr", "Should decrement ptr");
            }
            other => panic!("Expected PostDecrement statement, got {:?}", other),
        }
    }

    #[test]
    fn test_pre_decrement() {
        // RED: Test that --ptr is parsed correctly
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            void test() {
                int* ptr;
                --ptr;
            }
        "#;

        let ast = parser.parse(source).expect("Parsing should succeed");
        let func = &ast.functions()[0];

        // RED: This will fail because we don't have PreDecrement yet
        match &func.body[1] {
            Statement::PreDecrement { target } => {
                assert_eq!(target, "ptr", "Should decrement ptr");
            }
            other => panic!("Expected PreDecrement statement, got {:?}", other),
        }
    }

    // ============================================================================
    // RED PHASE: Compound assignment operators (+=, -=, etc.)
    // ============================================================================

    #[test]
    fn test_plus_equals_operator() {
        // RED: Test that += is parsed correctly
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            void test() {
                int* ptr;
                int offset = 5;
                ptr += offset;
            }
        "#;

        let ast = parser.parse(source).expect("Parsing should succeed");
        let func = &ast.functions()[0];

        // RED: This will fail because we don't have CompoundAssignment yet
        match &func.body[2] {
            Statement::CompoundAssignment { target, op, value } => {
                assert_eq!(target, "ptr", "Should assign to ptr");
                assert_eq!(*op, BinaryOperator::Add, "Should use + operator");
                match value {
                    Expression::Variable(name) => assert_eq!(name, "offset"),
                    _ => panic!("Expected variable 'offset' as value"),
                }
            }
            other => panic!("Expected CompoundAssignment statement, got {:?}", other),
        }
    }

    #[test]
    fn test_minus_equals_operator() {
        // RED: Test that -= is parsed correctly
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            void test() {
                int* ptr;
                ptr -= 3;
            }
        "#;

        let ast = parser.parse(source).expect("Parsing should succeed");
        let func = &ast.functions()[0];

        // RED: This will fail
        match &func.body[1] {
            Statement::CompoundAssignment { target, op, value } => {
                assert_eq!(target, "ptr", "Should assign to ptr");
                assert_eq!(*op, BinaryOperator::Subtract, "Should use - operator");
                match value {
                    Expression::IntLiteral(n) => assert_eq!(*n, 3),
                    _ => panic!("Expected int literal 3"),
                }
            }
            other => panic!("Expected CompoundAssignment statement, got {:?}", other),
        }
    }

    #[test]
    fn test_multiply_equals_operator() {
        // RED: Test that *= is parsed correctly
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            void test() {
                int x = 10;
                x *= 2;
            }
        "#;

        let ast = parser.parse(source).expect("Parsing should succeed");
        let func = &ast.functions()[0];

        // RED: This will fail
        match &func.body[1] {
            Statement::CompoundAssignment {
                target,
                op,
                value: _,
            } => {
                assert_eq!(target, "x", "Should assign to x");
                assert_eq!(*op, BinaryOperator::Multiply, "Should use * operator");
            }
            other => panic!("Expected CompoundAssignment statement, got {:?}", other),
        }
    }

    #[test]
    fn test_divide_equals_operator() {
        // RED: Test that /= is parsed correctly
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            void test() {
                int x = 10;
                x /= 2;
            }
        "#;

        let ast = parser.parse(source).expect("Parsing should succeed");
        let func = &ast.functions()[0];

        // RED: This will fail
        match &func.body[1] {
            Statement::CompoundAssignment {
                target,
                op,
                value: _,
            } => {
                assert_eq!(target, "x", "Should assign to x");
                assert_eq!(*op, BinaryOperator::Divide, "Should use / operator");
            }
            other => panic!("Expected CompoundAssignment statement, got {:?}", other),
        }
    }

    #[test]
    fn test_modulo_equals_operator() {
        // RED: Test that %= is parsed correctly
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            void test() {
                int x = 10;
                x %= 3;
            }
        "#;

        let ast = parser.parse(source).expect("Parsing should succeed");
        let func = &ast.functions()[0];

        // RED: This will fail
        match &func.body[1] {
            Statement::CompoundAssignment {
                target,
                op,
                value: _,
            } => {
                assert_eq!(target, "x", "Should assign to x");
                assert_eq!(*op, BinaryOperator::Modulo, "Should use % operator");
            }
            other => panic!("Expected CompoundAssignment statement, got {:?}", other),
        }
    }

    // ============================================================================
    // RED PHASE: Increment/Decrement in expressions (not statements)
    // ============================================================================

    #[test]
    fn test_post_increment_in_expression() {
        // RED: Test that ptr++ can be used in expressions
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            int test() {
                int* ptr;
                int x;
                x = *ptr++;
                return x;
            }
        "#;

        let ast = parser.parse(source).expect("Parsing should succeed");
        let func = &ast.functions()[0];

        // RED: This will fail because we don't support ++ in expressions yet
        match &func.body[2] {
            Statement::Assignment { target, value } => {
                assert_eq!(target, "x");
                // value should be Dereference(PostIncrement(ptr))
                match value {
                    Expression::Dereference(inner) => match &**inner {
                        Expression::PostIncrement { operand } => match &**operand {
                            Expression::Variable(name) => assert_eq!(name, "ptr"),
                            _ => panic!("Expected variable ptr"),
                        },
                        _ => panic!("Expected PostIncrement inside dereference"),
                    },
                    _ => panic!("Expected dereference expression"),
                }
            }
            other => panic!("Expected assignment, got {:?}", other),
        }
    }

    #[test]
    fn test_pre_increment_in_expression() {
        // RED: Test that ++ptr can be used in expressions
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            int test() {
                int* ptr;
                int x;
                x = *++ptr;
                return x;
            }
        "#;

        let ast = parser.parse(source).expect("Parsing should succeed");
        let func = &ast.functions()[0];

        // RED: This will fail
        match &func.body[2] {
            Statement::Assignment { target, value } => {
                assert_eq!(target, "x");
                // value should be Dereference(PreIncrement(ptr))
                match value {
                    Expression::Dereference(inner) => match &**inner {
                        Expression::PreIncrement { operand } => match &**operand {
                            Expression::Variable(name) => assert_eq!(name, "ptr"),
                            _ => panic!("Expected variable ptr"),
                        },
                        _ => panic!("Expected PreIncrement inside dereference"),
                    },
                    _ => panic!("Expected dereference expression"),
                }
            }
            other => panic!("Expected assignment, got {:?}", other),
        }
    }

    // ============================================================================
    // RED PHASE: Real-world pointer arithmetic patterns
    // ============================================================================

    #[test]
    fn test_array_traversal_pattern() {
        // RED: Test common array traversal with pointer arithmetic
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            int sum_array(int* arr, int size) {
                int sum = 0;
                int* end = arr + size;
                while (arr < end) {
                    sum += *arr;
                    arr++;
                }
                return sum;
            }
        "#;

        let ast = parser.parse(source).expect("Parsing should succeed");
        let func = &ast.functions()[0];

        // Should have: sum=0, end=arr+size, while loop with arr++ inside
        assert_eq!(func.body.len(), 4, "Should have 4 statements");

        // RED: Multiple things will fail here:
        // 1. arr + size in pointer context
        // 2. += operator
        // 3. arr++ statement
    }

    #[test]
    fn test_pointer_difference() {
        // RED: Test pointer subtraction for size calculation
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            int get_size(int* start, int* end) {
                return end - start;
            }
        "#;

        let ast = parser.parse(source).expect("Parsing should succeed");
        let func = &ast.functions()[0];

        // Should parse end - start as binary operation
        match &func.body[0] {
            Statement::Return(Some(expr)) => {
                match expr {
                    Expression::BinaryOp {
                        op,
                        left: _,
                        right: _,
                    } => {
                        assert_eq!(*op, BinaryOperator::Subtract);
                        // This should work already, but let's verify
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
            _ => panic!("Expected return statement"),
        }
    }
}
