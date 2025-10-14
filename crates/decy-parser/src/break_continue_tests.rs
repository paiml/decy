//! RED PHASE Tests for Break/Continue Statements (DECY-042)
//!
//! These tests are EXPECTED TO FAIL initially.
//! They define the behavior we want to implement for:
//! - Break statements in loops
//! - Continue statements in loops
//! - Break/continue in nested loops

use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // RED PHASE: Break statement in while loop
    // ============================================================================

    #[test]
    fn test_break_in_while_loop() {
        // RED: Test that break is parsed correctly in a while loop
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            int find_value(int target) {
                int i = 0;
                while (i < 100) {
                    if (i == target) {
                        break;
                    }
                    i = i + 1;
                }
                return i;
            }
        "#;

        let ast = parser.parse(source).expect("Parsing should succeed");
        let func = &ast.functions()[0];

        // Check while loop structure
        assert_eq!(
            func.body.len(),
            3,
            "Should have 3 statements: i=0, while, return"
        );

        if let Statement::While { body, .. } = &func.body[1] {
            assert_eq!(body.len(), 2, "While loop should have 2 statements");

            // First statement should be an if with break inside
            if let Statement::If { then_block, .. } = &body[0] {
                assert_eq!(then_block.len(), 1, "If block should have 1 statement");

                // RED: This will fail because we don't have Break yet
                match &then_block[0] {
                    Statement::Break => {
                        // Success - we found a break statement
                    }
                    other => panic!("Expected Break statement, got {:?}", other),
                }
            } else {
                panic!("Expected if statement as first statement in while loop");
            }
        } else {
            panic!("Expected while loop as second statement");
        }
    }

    // ============================================================================
    // RED PHASE: Continue statement in while loop
    // ============================================================================

    #[test]
    fn test_continue_in_while_loop() {
        // RED: Test that continue is parsed correctly in a while loop
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            int count_even(int n) {
                int i = 0;
                int count = 0;
                while (i < n) {
                    i = i + 1;
                    if (i % 2 == 1) {
                        continue;
                    }
                    count = count + 1;
                }
                return count;
            }
        "#;

        let ast = parser.parse(source).expect("Parsing should succeed");
        let func = &ast.functions()[0];

        // Check while loop structure
        if let Statement::While { body, .. } = &func.body[2] {
            assert_eq!(body.len(), 3, "While loop should have 3 statements");

            // Second statement should be an if with continue inside
            if let Statement::If { then_block, .. } = &body[1] {
                assert_eq!(then_block.len(), 1, "If block should have 1 statement");

                // RED: This will fail because we don't have Continue yet
                match &then_block[0] {
                    Statement::Continue => {
                        // Success - we found a continue statement
                    }
                    other => panic!("Expected Continue statement, got {:?}", other),
                }
            } else {
                panic!("Expected if statement as second statement in while loop");
            }
        } else {
            panic!("Expected while loop as third statement");
        }
    }

    // ============================================================================
    // RED PHASE: Break statement in for loop
    // ============================================================================

    #[test]
    fn test_break_in_for_loop() {
        // RED: Test that break is parsed correctly in a for loop
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            int find_divisor(int n) {
                int i;
                for (i = 2; i < n; i = i + 1) {
                    if (n % i == 0) {
                        break;
                    }
                }
                return i;
            }
        "#;

        let ast = parser.parse(source).expect("Parsing should succeed");
        let func = &ast.functions()[0];

        // Check for loop structure
        // Note: Variable declarations before for loops are parsed as separate statements
        assert!(func.body.len() >= 2, "Should have at least 2 statements");

        // Find the for loop (may not be first statement if variables declared before it)
        let for_stmt_idx = func
            .body
            .iter()
            .position(|s| matches!(s, Statement::For { .. }))
            .expect("Should have a for loop");

        if let Statement::For { body, .. } = &func.body[for_stmt_idx] {
            assert_eq!(body.len(), 1, "For loop should have 1 statement");

            // Should be an if with break inside
            if let Statement::If { then_block, .. } = &body[0] {
                assert_eq!(then_block.len(), 1, "If block should have 1 statement");

                // RED: This will fail because we don't have Break yet
                match &then_block[0] {
                    Statement::Break => {
                        // Success
                    }
                    other => panic!("Expected Break statement, got {:?}", other),
                }
            } else {
                panic!("Expected if statement in for loop");
            }
        } else {
            panic!("Expected for loop as first statement");
        }
    }

    // ============================================================================
    // RED PHASE: Continue statement in for loop
    // ============================================================================

    #[test]
    fn test_continue_in_for_loop() {
        // RED: Test that continue is parsed correctly in a for loop
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            int sum_even(int n) {
                int sum = 0;
                int i;
                for (i = 0; i < n; i = i + 1) {
                    if (i % 2 == 1) {
                        continue;
                    }
                    sum = sum + i;
                }
                return sum;
            }
        "#;

        let ast = parser.parse(source).expect("Parsing should succeed");
        let func = &ast.functions()[0];

        // Find the for loop (may not be at index 1 if variables declared)
        let for_stmt_idx = func
            .body
            .iter()
            .position(|s| matches!(s, Statement::For { .. }))
            .expect("Should have a for loop");

        if let Statement::For { body, .. } = &func.body[for_stmt_idx] {
            assert_eq!(body.len(), 2, "For loop should have 2 statements");

            // First statement should be an if with continue inside
            if let Statement::If { then_block, .. } = &body[0] {
                assert_eq!(then_block.len(), 1, "If block should have 1 statement");

                // RED: This will fail because we don't have Continue yet
                match &then_block[0] {
                    Statement::Continue => {
                        // Success
                    }
                    other => panic!("Expected Continue statement, got {:?}", other),
                }
            } else {
                panic!("Expected if statement in for loop");
            }
        } else {
            panic!("Expected for loop as second statement");
        }
    }

    // ============================================================================
    // RED PHASE: Nested loops with break
    // ============================================================================

    #[test]
    fn test_break_in_nested_loops() {
        // RED: Test that break only exits the innermost loop
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            int find_pair(int target) {
                int i;
                int j;
                for (i = 0; i < 10; i = i + 1) {
                    for (j = 0; j < 10; j = j + 1) {
                        if (i + j == target) {
                            break;
                        }
                    }
                }
                return i;
            }
        "#;

        let ast = parser.parse(source).expect("Parsing should succeed");
        let func = &ast.functions()[0];

        // Find the outer for loop
        let for_stmt_idx = func
            .body
            .iter()
            .position(|s| matches!(s, Statement::For { .. }))
            .expect("Should have a for loop");

        // Outer for loop
        if let Statement::For {
            body: outer_body, ..
        } = &func.body[for_stmt_idx]
        {
            // Inner for loop
            if let Statement::For {
                body: inner_body, ..
            } = &outer_body[0]
            {
                // If statement with break
                if let Statement::If { then_block, .. } = &inner_body[0] {
                    // RED: This will fail
                    match &then_block[0] {
                        Statement::Break => {
                            // Success - break parsed in nested loop
                        }
                        other => panic!("Expected Break statement, got {:?}", other),
                    }
                } else {
                    panic!("Expected if statement in inner loop");
                }
            } else {
                panic!("Expected inner for loop");
            }
        } else {
            panic!("Expected outer for loop");
        }
    }

    // ============================================================================
    // RED PHASE: Nested loops with continue
    // ============================================================================

    #[test]
    fn test_continue_in_nested_loops() {
        // RED: Test that continue only affects the innermost loop
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            int count_pairs(int limit) {
                int count = 0;
                int i;
                int j;
                for (i = 0; i < 10; i = i + 1) {
                    for (j = 0; j < 10; j = j + 1) {
                        if (i == j) {
                            continue;
                        }
                        if (i + j < limit) {
                            count = count + 1;
                        }
                    }
                }
                return count;
            }
        "#;

        let ast = parser.parse(source).expect("Parsing should succeed");
        let func = &ast.functions()[0];

        // Find the outer for loop
        let for_stmt_idx = func
            .body
            .iter()
            .position(|s| matches!(s, Statement::For { .. }))
            .expect("Should have a for loop");

        // Navigate to nested loop structure
        if let Statement::For {
            body: outer_body, ..
        } = &func.body[for_stmt_idx]
        {
            if let Statement::For {
                body: inner_body, ..
            } = &outer_body[0]
            {
                // First if with continue
                if let Statement::If { then_block, .. } = &inner_body[0] {
                    // RED: This will fail
                    match &then_block[0] {
                        Statement::Continue => {
                            // Success
                        }
                        other => panic!("Expected Continue statement, got {:?}", other),
                    }
                } else {
                    panic!("Expected if statement in inner loop");
                }
            } else {
                panic!("Expected inner for loop");
            }
        } else {
            panic!("Expected outer for loop");
        }
    }

    // NOTE: We don't test break/continue outside loops because clang itself
    // rejects these as syntax errors during parsing. Semantic validation
    // is handled by clang before we even see the AST.

    // ============================================================================
    // RED PHASE: Real-world pattern - early exit from search
    // ============================================================================

    #[test]
    fn test_break_in_search_pattern() {
        // RED: Test common pattern of breaking when found
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            int linear_search(int* arr, int size, int target) {
                int i;
                int found = 0;
                for (i = 0; i < size; i = i + 1) {
                    if (arr[i] == target) {
                        found = 1;
                        break;
                    }
                }
                return found;
            }
        "#;

        let ast = parser.parse(source).expect("Parsing should succeed");
        let func = &ast.functions()[0];

        // Find the for loop
        let for_stmt_idx = func
            .body
            .iter()
            .position(|s| matches!(s, Statement::For { .. }))
            .expect("Should have a for loop");

        // Navigate to for loop
        if let Statement::For { body, .. } = &func.body[for_stmt_idx] {
            // If statement
            if let Statement::If { then_block, .. } = &body[0] {
                assert_eq!(then_block.len(), 2, "If block should have 2 statements");

                // Second statement should be break
                // RED: This will fail
                match &then_block[1] {
                    Statement::Break => {
                        // Success
                    }
                    other => panic!("Expected Break statement, got {:?}", other),
                }
            } else {
                panic!("Expected if statement in for loop");
            }
        } else {
            panic!("Expected for loop");
        }
    }

    // ============================================================================
    // RED PHASE: Real-world pattern - skip processing with continue
    // ============================================================================

    #[test]
    fn test_continue_skip_pattern() {
        // RED: Test common pattern of skipping certain values
        let parser = CParser::new().expect("Parser creation failed");
        let source = r#"
            int sum_positive(int* arr, int size) {
                int sum = 0;
                int i;
                for (i = 0; i < size; i = i + 1) {
                    if (arr[i] < 0) {
                        continue;
                    }
                    sum = sum + arr[i];
                }
                return sum;
            }
        "#;

        let ast = parser.parse(source).expect("Parsing should succeed");
        let func = &ast.functions()[0];

        // Find the for loop
        let for_stmt_idx = func
            .body
            .iter()
            .position(|s| matches!(s, Statement::For { .. }))
            .expect("Should have a for loop");

        // Navigate to for loop
        if let Statement::For { body, .. } = &func.body[for_stmt_idx] {
            assert_eq!(body.len(), 2, "For loop should have 2 statements");

            // First statement is if with continue
            if let Statement::If { then_block, .. } = &body[0] {
                assert_eq!(then_block.len(), 1, "If block should have 1 statement");

                // RED: This will fail
                match &then_block[0] {
                    Statement::Continue => {
                        // Success
                    }
                    other => panic!("Expected Continue statement, got {:?}", other),
                }
            } else {
                panic!("Expected if statement in for loop");
            }
        } else {
            panic!("Expected for loop");
        }
    }
}
