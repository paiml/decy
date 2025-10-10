//! Pattern detection for identifying `Box<T>` and `Vec<T>` candidates.
//!
//! Analyzes HIR to find malloc/free patterns that can be replaced with safe Rust types.

use decy_hir::{HirExpression, HirFunction, HirStatement};

/// Represents a detected `Box<T>` pattern candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxCandidate {
    /// Variable name that holds the allocated pointer
    pub variable: String,
    /// Statement index where malloc occurs
    pub malloc_index: usize,
    /// Statement index where free occurs (if found)
    pub free_index: Option<usize>,
}

/// Represents a detected `Vec<T>` pattern candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VecCandidate {
    /// Variable name that holds the allocated array pointer
    pub variable: String,
    /// Statement index where malloc occurs
    pub malloc_index: usize,
    /// Statement index where free occurs (if found)
    pub free_index: Option<usize>,
    /// Expression representing the array capacity (number of elements)
    pub capacity_expr: Option<HirExpression>,
}

/// Pattern detector for identifying `Box<T>` candidates.
#[derive(Debug, Clone)]
pub struct PatternDetector;

impl PatternDetector {
    /// Create a new pattern detector.
    pub fn new() -> Self {
        Self
    }

    /// Analyze a function to find `Box<T>` candidates.
    ///
    /// Detects patterns like:
    /// ```c
    /// T* ptr = malloc(sizeof(T));
    /// // ... use ptr ...
    /// free(ptr);
    /// ```
    pub fn find_box_candidates(&self, func: &HirFunction) -> Vec<BoxCandidate> {
        let mut candidates = Vec::new();
        let body = func.body();

        // Track malloc calls assigned to variables
        for (idx, stmt) in body.iter().enumerate() {
            if let Some(var_name) = self.is_malloc_assignment(stmt) {
                // Look for corresponding free
                let free_idx = self.find_free_call(body, idx + 1, &var_name);

                candidates.push(BoxCandidate {
                    variable: var_name,
                    malloc_index: idx,
                    free_index: free_idx,
                });
            }
        }

        candidates
    }

    /// Check if a statement is an assignment from malloc.
    ///
    /// Patterns matched:
    /// - `T* ptr = malloc(...)`  (VariableDeclaration)
    /// - `ptr = malloc(...)`     (Assignment)
    fn is_malloc_assignment(&self, stmt: &HirStatement) -> Option<String> {
        match stmt {
            HirStatement::VariableDeclaration {
                name,
                initializer: Some(expr),
                ..
            } => {
                if self.is_malloc_call(expr) {
                    Some(name.clone())
                } else {
                    None
                }
            }
            HirStatement::Assignment { target, value } => {
                if self.is_malloc_call(value) {
                    Some(target.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Check if an expression is a malloc call.
    fn is_malloc_call(&self, expr: &HirExpression) -> bool {
        matches!(
            expr,
            HirExpression::FunctionCall { function, .. } if function == "malloc"
        )
    }

    /// Find a free call for a specific variable after a given statement index.
    fn find_free_call(
        &self,
        body: &[HirStatement],
        start_idx: usize,
        var_name: &str,
    ) -> Option<usize> {
        for (offset, stmt) in body[start_idx..].iter().enumerate() {
            if self.is_free_call(stmt, var_name) {
                return Some(start_idx + offset);
            }
        }
        None
    }

    /// Check if a statement is a free call for a specific variable.
    ///
    /// Free call detection requires ExpressionStatement support in HIR.
    /// This will be implemented in a future phase when ExpressionStatement is added.
    /// For now, free_index in BoxCandidate will always be None.
    fn is_free_call(&self, _stmt: &HirStatement, _var_name: &str) -> bool {
        // Free call detection requires ExpressionStatement support in HIR.
        // This will be implemented in a future phase when ExpressionStatement is added.
        // For now, free_index in BoxCandidate will always be None.
        false
    }

    /// Analyze a function to find `Vec<T>` candidates.
    ///
    /// Detects patterns like:
    /// ```c
    /// T* arr = malloc(n * sizeof(T));
    /// // ... use arr as array ...
    /// free(arr);
    /// ```
    pub fn find_vec_candidates(&self, func: &HirFunction) -> Vec<VecCandidate> {
        let mut candidates = Vec::new();
        let body = func.body();

        // Track malloc calls assigned to variables that use array allocation pattern
        for (idx, stmt) in body.iter().enumerate() {
            if let Some((var_name, malloc_expr)) = self.is_malloc_assignment_expr(stmt) {
                // Check if this is an array allocation pattern (n * sizeof(T))
                if self.is_array_size_expr(malloc_expr) {
                    let capacity = self.extract_capacity(malloc_expr);

                    // Look for corresponding free (same logic as Box)
                    let free_idx = self.find_free_call(body, idx + 1, &var_name);

                    candidates.push(VecCandidate {
                        variable: var_name,
                        malloc_index: idx,
                        free_index: free_idx,
                        capacity_expr: capacity,
                    });
                }
            }
        }

        candidates
    }

    /// Check if a statement is an assignment from malloc, returning var name and malloc expr.
    ///
    /// Similar to is_malloc_assignment but returns the malloc call expression for analysis.
    fn is_malloc_assignment_expr<'a>(
        &self,
        stmt: &'a HirStatement,
    ) -> Option<(String, &'a HirExpression)> {
        match stmt {
            HirStatement::VariableDeclaration {
                name,
                initializer: Some(expr),
                ..
            } => {
                if let HirExpression::FunctionCall {
                    function,
                    arguments,
                } = expr
                {
                    if function == "malloc" && !arguments.is_empty() {
                        return Some((name.clone(), &arguments[0]));
                    }
                }
                None
            }
            HirStatement::Assignment { target, value } => {
                if let HirExpression::FunctionCall {
                    function,
                    arguments,
                } = value
                {
                    if function == "malloc" && !arguments.is_empty() {
                        return Some((target.clone(), &arguments[0]));
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Check if an expression represents array allocation: n * sizeof(T) pattern
    ///
    /// Looks for multiplication expressions that indicate array sizing.
    fn is_array_size_expr(&self, expr: &HirExpression) -> bool {
        matches!(
            expr,
            HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Multiply,
                ..
            }
        )
    }

    /// Extract capacity from array size expression (n * sizeof(T))
    ///
    /// Returns the left operand of the multiplication, which typically
    /// represents the number of elements (capacity).
    fn extract_capacity(&self, expr: &HirExpression) -> Option<HirExpression> {
        if let HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::Multiply,
            left,
            ..
        } = expr
        {
            Some((**left).clone())
        } else {
            None
        }
    }
}

impl Default for PatternDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use decy_hir::{HirParameter, HirType};

    #[test]
    fn test_detect_malloc_in_variable_declaration() {
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![HirStatement::VariableDeclaration {
                name: "ptr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(100)],
                }),
            }],
        );

        let detector = PatternDetector::new();
        let candidates = detector.find_box_candidates(&func);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].variable, "ptr");
        assert_eq!(candidates[0].malloc_index, 0);
    }

    #[test]
    fn test_detect_malloc_in_assignment() {
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![HirParameter::new(
                "ptr".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            )],
            vec![HirStatement::Assignment {
                target: "ptr".to_string(),
                value: HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(50)],
                },
            }],
        );

        let detector = PatternDetector::new();
        let candidates = detector.find_box_candidates(&func);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].variable, "ptr");
        assert_eq!(candidates[0].malloc_index, 0);
    }

    #[test]
    fn test_no_malloc_detected() {
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Int,
            vec![],
            vec![
                HirStatement::VariableDeclaration {
                    name: "x".to_string(),
                    var_type: HirType::Int,
                    initializer: Some(HirExpression::IntLiteral(42)),
                },
                HirStatement::Return(Some(HirExpression::Variable("x".to_string()))),
            ],
        );

        let detector = PatternDetector::new();
        let candidates = detector.find_box_candidates(&func);

        assert_eq!(candidates.len(), 0);
    }

    #[test]
    fn test_multiple_malloc_calls() {
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                HirStatement::VariableDeclaration {
                    name: "ptr1".to_string(),
                    var_type: HirType::Pointer(Box::new(HirType::Int)),
                    initializer: Some(HirExpression::FunctionCall {
                        function: "malloc".to_string(),
                        arguments: vec![HirExpression::IntLiteral(100)],
                    }),
                },
                HirStatement::VariableDeclaration {
                    name: "ptr2".to_string(),
                    var_type: HirType::Pointer(Box::new(HirType::Char)),
                    initializer: Some(HirExpression::FunctionCall {
                        function: "malloc".to_string(),
                        arguments: vec![HirExpression::IntLiteral(200)],
                    }),
                },
            ],
        );

        let detector = PatternDetector::new();
        let candidates = detector.find_box_candidates(&func);

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].variable, "ptr1");
        assert_eq!(candidates[1].variable, "ptr2");
    }

    #[test]
    fn test_malloc_from_other_function() {
        // Should NOT detect allocate() as malloc
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![HirStatement::VariableDeclaration {
                name: "ptr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "allocate".to_string(),
                    arguments: vec![HirExpression::IntLiteral(100)],
                }),
            }],
        );

        let detector = PatternDetector::new();
        let candidates = detector.find_box_candidates(&func);

        assert_eq!(candidates.len(), 0);
    }

    // Vec candidate detection tests
    #[test]
    fn test_detect_vec_array_allocation_in_variable_declaration() {
        // Pattern: int* arr = malloc(n * sizeof(int));
        // Should be detected as Vec<i32> candidate
        let n_expr = HirExpression::Variable("n".to_string());
        let sizeof_expr = HirExpression::IntLiteral(4); // sizeof(int) = 4
        let size_expr = HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::Multiply,
            left: Box::new(n_expr.clone()),
            right: Box::new(sizeof_expr),
        };

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![size_expr],
                }),
            }],
        );

        let detector = PatternDetector::new();
        let candidates = detector.find_vec_candidates(&func);

        assert_eq!(candidates.len(), 1, "Should detect one Vec candidate");
        assert_eq!(candidates[0].variable, "arr");
        assert_eq!(candidates[0].malloc_index, 0);
    }

    #[test]
    fn test_detect_vec_with_literal_capacity() {
        // Pattern: int* arr = malloc(10 * sizeof(int));
        let capacity = HirExpression::IntLiteral(10);
        let sizeof_expr = HirExpression::IntLiteral(4);
        let size_expr = HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::Multiply,
            left: Box::new(capacity.clone()),
            right: Box::new(sizeof_expr),
        };

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![size_expr],
                }),
            }],
        );

        let detector = PatternDetector::new();
        let candidates = detector.find_vec_candidates(&func);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].variable, "arr");
        assert!(
            candidates[0].capacity_expr.is_some(),
            "Should extract capacity expression"
        );
    }

    #[test]
    fn test_vec_vs_box_distinction() {
        // Box pattern: malloc(sizeof(T)) - single element
        // Vec pattern: malloc(n * sizeof(T)) - array
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                // Box candidate: single element
                HirStatement::VariableDeclaration {
                    name: "single".to_string(),
                    var_type: HirType::Pointer(Box::new(HirType::Int)),
                    initializer: Some(HirExpression::FunctionCall {
                        function: "malloc".to_string(),
                        arguments: vec![HirExpression::IntLiteral(4)], // just sizeof(int)
                    }),
                },
                // Vec candidate: array
                HirStatement::VariableDeclaration {
                    name: "array".to_string(),
                    var_type: HirType::Pointer(Box::new(HirType::Int)),
                    initializer: Some(HirExpression::FunctionCall {
                        function: "malloc".to_string(),
                        arguments: vec![HirExpression::BinaryOp {
                            op: decy_hir::BinaryOperator::Multiply,
                            left: Box::new(HirExpression::IntLiteral(10)),
                            right: Box::new(HirExpression::IntLiteral(4)),
                        }],
                    }),
                },
            ],
        );

        let detector = PatternDetector::new();
        let box_candidates = detector.find_box_candidates(&func);
        let vec_candidates = detector.find_vec_candidates(&func);

        // Box detector should find both (it's less specific)
        // Vec detector should only find the array pattern
        assert_eq!(vec_candidates.len(), 1, "Should find only array pattern");
        assert_eq!(vec_candidates[0].variable, "array");

        // The "single" allocation should be detected as Box only
        // (Box detector will find it, Vec detector won't)
        assert!(box_candidates.iter().any(|c| c.variable == "single"));
    }

    #[test]
    fn test_no_vec_detected_for_non_array_malloc() {
        // malloc without multiplication pattern should not be Vec
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![HirStatement::VariableDeclaration {
                name: "ptr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(100)],
                }),
            }],
        );

        let detector = PatternDetector::new();
        let candidates = detector.find_vec_candidates(&func);

        assert_eq!(candidates.len(), 0, "Should not detect non-array as Vec");
    }

    #[test]
    fn test_multiple_vec_allocations() {
        let size1 = HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::Multiply,
            left: Box::new(HirExpression::IntLiteral(10)),
            right: Box::new(HirExpression::IntLiteral(4)),
        };

        let size2 = HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("count".to_string())),
            right: Box::new(HirExpression::IntLiteral(8)),
        };

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                HirStatement::VariableDeclaration {
                    name: "arr1".to_string(),
                    var_type: HirType::Pointer(Box::new(HirType::Int)),
                    initializer: Some(HirExpression::FunctionCall {
                        function: "malloc".to_string(),
                        arguments: vec![size1],
                    }),
                },
                HirStatement::VariableDeclaration {
                    name: "arr2".to_string(),
                    var_type: HirType::Pointer(Box::new(HirType::Double)),
                    initializer: Some(HirExpression::FunctionCall {
                        function: "malloc".to_string(),
                        arguments: vec![size2],
                    }),
                },
            ],
        );

        let detector = PatternDetector::new();
        let candidates = detector.find_vec_candidates(&func);

        assert_eq!(candidates.len(), 2, "Should detect both Vec candidates");
        assert_eq!(candidates[0].variable, "arr1");
        assert_eq!(candidates[1].variable, "arr2");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use decy_hir::{HirExpression, HirFunction, HirStatement, HirType};
    use proptest::prelude::*;

    proptest! {
        /// Property: Detector never panics on any function
        #[test]
        fn property_detector_never_panics(
            func_name in "[a-z_][a-z0-9_]{0,10}",
            var_name in "[a-z_][a-z0-9_]{0,10}",
            size in 1i32..1000
        ) {
            let func = HirFunction::new_with_body(
                func_name,
                HirType::Void,
                vec![],
                vec![HirStatement::VariableDeclaration {
                    name: var_name,
                    var_type: HirType::Pointer(Box::new(HirType::Int)),
                    initializer: Some(HirExpression::FunctionCall {
                        function: "malloc".to_string(),
                        arguments: vec![HirExpression::IntLiteral(size)],
                    }),
                }],
            );

            let detector = PatternDetector::new();
            let _candidates = detector.find_box_candidates(&func);
            // If we get here without panic, test passes
        }

        /// Property: Every malloc detection has a valid malloc_index
        #[test]
        fn property_malloc_index_valid(
            var_name in "[a-z_][a-z0-9_]{0,10}",
            size in 1i32..1000
        ) {
            let body = vec![
                HirStatement::VariableDeclaration {
                    name: "x".to_string(),
                    var_type: HirType::Int,
                    initializer: Some(HirExpression::IntLiteral(0)),
                },
                HirStatement::VariableDeclaration {
                    name: var_name.clone(),
                    var_type: HirType::Pointer(Box::new(HirType::Int)),
                    initializer: Some(HirExpression::FunctionCall {
                        function: "malloc".to_string(),
                        arguments: vec![HirExpression::IntLiteral(size)],
                    }),
                },
            ];

            let func = HirFunction::new_with_body(
                "test".to_string(),
                HirType::Void,
                vec![],
                body.clone(),
            );

            let detector = PatternDetector::new();
            let candidates = detector.find_box_candidates(&func);

            // Should find exactly one candidate
            prop_assert_eq!(candidates.len(), 1);
            // Index should be valid (within body length)
            prop_assert!(candidates[0].malloc_index < body.len());
            // Index should point to the malloc statement
            prop_assert_eq!(candidates[0].malloc_index, 1);
        }

        /// Property: Detected variable names match actual variable names
        #[test]
        fn property_variable_name_preserved(
            var_name in "[a-z_][a-z0-9_]{0,10}",
            size in 1i32..1000
        ) {
            let func = HirFunction::new_with_body(
                "test".to_string(),
                HirType::Void,
                vec![],
                vec![HirStatement::VariableDeclaration {
                    name: var_name.clone(),
                    var_type: HirType::Pointer(Box::new(HirType::Int)),
                    initializer: Some(HirExpression::FunctionCall {
                        function: "malloc".to_string(),
                        arguments: vec![HirExpression::IntLiteral(size)],
                    }),
                }],
            );

            let detector = PatternDetector::new();
            let candidates = detector.find_box_candidates(&func);

            prop_assert_eq!(candidates.len(), 1);
            prop_assert_eq!(&candidates[0].variable, &var_name);
        }

        /// Property: Detection is deterministic
        #[test]
        fn property_detection_deterministic(
            var_name in "[a-z_][a-z0-9_]{0,10}",
            size in 1i32..1000
        ) {
            let func = HirFunction::new_with_body(
                "test".to_string(),
                HirType::Void,
                vec![],
                vec![HirStatement::VariableDeclaration {
                    name: var_name,
                    var_type: HirType::Pointer(Box::new(HirType::Int)),
                    initializer: Some(HirExpression::FunctionCall {
                        function: "malloc".to_string(),
                        arguments: vec![HirExpression::IntLiteral(size)],
                    }),
                }],
            );

            let detector = PatternDetector::new();
            let candidates1 = detector.find_box_candidates(&func);
            let candidates2 = detector.find_box_candidates(&func);

            prop_assert_eq!(candidates1, candidates2);
        }

        // Vec candidate property tests
        /// Property: Vec detector never panics
        #[test]
        fn property_vec_detector_never_panics(
            func_name in "[a-z_][a-z0-9_]{0,10}",
            var_name in "[a-z_][a-z0-9_]{0,10}",
            capacity in 1i32..1000,
            elem_size in 1i32..16
        ) {
            let size_expr = HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Multiply,
                left: Box::new(HirExpression::IntLiteral(capacity)),
                right: Box::new(HirExpression::IntLiteral(elem_size)),
            };

            let func = HirFunction::new_with_body(
                func_name,
                HirType::Void,
                vec![],
                vec![HirStatement::VariableDeclaration {
                    name: var_name,
                    var_type: HirType::Pointer(Box::new(HirType::Int)),
                    initializer: Some(HirExpression::FunctionCall {
                        function: "malloc".to_string(),
                        arguments: vec![size_expr],
                    }),
                }],
            );

            let detector = PatternDetector::new();
            let _candidates = detector.find_vec_candidates(&func);
            // If we get here without panic, test passes
        }

        /// Property: Vec detection is deterministic
        #[test]
        fn property_vec_detection_deterministic(
            var_name in "[a-z_][a-z0-9_]{0,10}",
            capacity in 1i32..100,
            elem_size in 1i32..16
        ) {
            let size_expr = HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Multiply,
                left: Box::new(HirExpression::IntLiteral(capacity)),
                right: Box::new(HirExpression::IntLiteral(elem_size)),
            };

            let func = HirFunction::new_with_body(
                "test".to_string(),
                HirType::Void,
                vec![],
                vec![HirStatement::VariableDeclaration {
                    name: var_name,
                    var_type: HirType::Pointer(Box::new(HirType::Int)),
                    initializer: Some(HirExpression::FunctionCall {
                        function: "malloc".to_string(),
                        arguments: vec![size_expr],
                    }),
                }],
            );

            let detector = PatternDetector::new();
            let candidates1 = detector.find_vec_candidates(&func);
            let candidates2 = detector.find_vec_candidates(&func);

            prop_assert_eq!(candidates1, candidates2);
        }

        /// Property: Detected variable names match actual variable names
        #[test]
        fn property_vec_variable_name_preserved(
            var_name in "[a-z_][a-z0-9_]{0,10}",
            capacity in 1i32..100,
            elem_size in 1i32..16
        ) {
            let size_expr = HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Multiply,
                left: Box::new(HirExpression::IntLiteral(capacity)),
                right: Box::new(HirExpression::IntLiteral(elem_size)),
            };

            let func = HirFunction::new_with_body(
                "test".to_string(),
                HirType::Void,
                vec![],
                vec![HirStatement::VariableDeclaration {
                    name: var_name.clone(),
                    var_type: HirType::Pointer(Box::new(HirType::Int)),
                    initializer: Some(HirExpression::FunctionCall {
                        function: "malloc".to_string(),
                        arguments: vec![size_expr],
                    }),
                }],
            );

            let detector = PatternDetector::new();
            let candidates = detector.find_vec_candidates(&func);

            if !candidates.is_empty() {
                prop_assert_eq!(&candidates[0].variable, &var_name);
            }
        }

        /// Property: Vec malloc_index is always valid
        #[test]
        fn property_vec_malloc_index_valid(
            var_name in "[a-z_][a-z0-9_]{0,10}",
            capacity in 1i32..100,
            elem_size in 1i32..16
        ) {
            let size_expr = HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Multiply,
                left: Box::new(HirExpression::IntLiteral(capacity)),
                right: Box::new(HirExpression::IntLiteral(elem_size)),
            };

            let body = vec![
                HirStatement::VariableDeclaration {
                    name: "x".to_string(),
                    var_type: HirType::Int,
                    initializer: Some(HirExpression::IntLiteral(0)),
                },
                HirStatement::VariableDeclaration {
                    name: var_name,
                    var_type: HirType::Pointer(Box::new(HirType::Int)),
                    initializer: Some(HirExpression::FunctionCall {
                        function: "malloc".to_string(),
                        arguments: vec![size_expr],
                    }),
                },
            ];

            let func = HirFunction::new_with_body(
                "test".to_string(),
                HirType::Void,
                vec![],
                body.clone(),
            );

            let detector = PatternDetector::new();
            let candidates = detector.find_vec_candidates(&func);

            for candidate in candidates {
                prop_assert!(candidate.malloc_index < body.len());
            }
        }
    }
}
