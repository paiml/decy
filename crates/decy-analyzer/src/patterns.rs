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
    }
}
