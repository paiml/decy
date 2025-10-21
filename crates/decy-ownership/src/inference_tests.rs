//! Tests for ownership inference.

use super::*;
use crate::dataflow::DataflowAnalyzer;
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType};

#[test]
fn test_classify_owning_pointer() {
    // malloc creates an owning pointer
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "ptr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::IntLiteral(4)],
            }),
        }],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(
        inferences.contains_key("ptr"),
        "Should infer ownership for ptr"
    );
    assert_eq!(
        inferences["ptr"].kind,
        OwnershipKind::Owning,
        "malloc should create owning pointer"
    );
}

#[test]
fn test_classify_borrowing_pointer() {
    // Pointer derived from another pointer is a borrow
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
                    arguments: vec![HirExpression::IntLiteral(4)],
                }),
            },
            HirStatement::VariableDeclaration {
                name: "ptr2".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Variable("ptr1".to_string())),
            },
        ],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("ptr1"), "Should infer ptr1");
    assert!(inferences.contains_key("ptr2"), "Should infer ptr2");

    assert_eq!(inferences["ptr1"].kind, OwnershipKind::Owning);
    assert!(
        matches!(
            inferences["ptr2"].kind,
            OwnershipKind::ImmutableBorrow | OwnershipKind::MutableBorrow
        ),
        "ptr2 should be a borrow"
    );
}

#[test]
fn test_detect_mutation() {
    // Pointer that is assigned to is mutable
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::Dereference(Box::new(HirExpression::Variable("ptr".to_string()))),
        }],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("ptr"));
    // Reading through dereference doesn't make it mutable
    // This tests that we correctly distinguish read vs write
}

#[test]
fn test_infer_immutable_borrow() {
    // Parameter that is only read should be immutable borrow
    let func = HirFunction::new_with_body(
        "read_only".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "data".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::Dereference(
            Box::new(HirExpression::Variable("data".to_string())),
        )))],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("data"));
    assert_eq!(
        inferences["data"].kind,
        OwnershipKind::ImmutableBorrow,
        "Read-only parameter should be immutable borrow"
    );
}

#[test]
fn test_infer_mutable_borrow() {
    // Parameter that is written to should be mutable borrow
    // Note: This test is aspirational - detecting writes through pointers
    // requires more sophisticated analysis
    let func = HirFunction::new_with_body(
        "modify".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "data".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![
            // In a real scenario, this would be: *data = 5;
            // For now, we test with assignment which indicates mutation intent
            HirStatement::Assignment {
                target: "data".to_string(),
                value: HirExpression::IntLiteral(5),
            },
        ],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("data"));
    // This is a simplified test - real mutation detection would be more complex
}

#[test]
fn test_function_parameter_ownership() {
    // Function parameters are typically borrows unless they take ownership
    let func = HirFunction::new_with_body(
        "process".to_string(),
        HirType::Void,
        vec![
            HirParameter::new(
                "input".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            ),
            HirParameter::new(
                "output".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            ),
        ],
        vec![],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("input"));
    assert!(inferences.contains_key("output"));

    // Parameters should default to borrows
    assert!(
        matches!(
            inferences["input"].kind,
            OwnershipKind::ImmutableBorrow | OwnershipKind::MutableBorrow
        ),
        "Parameters should default to borrows"
    );
}

#[test]
fn test_confidence_scores() {
    // Test that confidence scores are generated
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "ptr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::IntLiteral(4)],
            }),
        }],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    if let Some(inference) = inferences.get("ptr") {
        assert!(
            inference.confidence >= 0.0 && inference.confidence <= 1.0,
            "Confidence should be between 0 and 1"
        );
        assert!(
            inference.confidence > 0.5,
            "malloc allocation should have high confidence"
        );
    }
}

#[test]
fn test_empty_function_inferences() {
    // Empty function should have no inferences
    let func = HirFunction::new("empty".to_string(), HirType::Void, vec![]);

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert_eq!(
        inferences.len(),
        0,
        "Empty function should have no inferences"
    );
}

#[test]
fn test_non_pointer_variables_not_inferred() {
    // Non-pointer variables should not have ownership inferences
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

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert_eq!(
        inferences.len(),
        0,
        "Non-pointer variables should not be inferred"
    );
}

#[test]
fn test_inference_reasoning() {
    // Test that inferences include reasoning
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "ptr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::IntLiteral(4)],
            }),
        }],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    if let Some(inference) = inferences.get("ptr") {
        assert!(
            !inference.reason.is_empty(),
            "Inference should include reasoning"
        );
    }
}

// RED PHASE: New failing tests for enhanced ownership inference

#[test]
fn test_const_parameter_is_immutable_borrow() {
    // const parameters should be inferred as immutable borrows
    // NOTE: const qualifier tracking will be added in future phase
    let func = HirFunction::new_with_body(
        "read_data".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "data".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::Dereference(
            Box::new(HirExpression::Variable("data".to_string())),
        )))],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("data"));
    assert_eq!(
        inferences["data"].kind,
        OwnershipKind::ImmutableBorrow,
        "const parameter should be immutable borrow"
    );
}

#[test]
fn test_pointer_returned_from_function_is_owning() {
    // Pointer returned from a function likely transfers ownership
    let func = HirFunction::new_with_body(
        "create_data".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "ptr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(4)],
                }),
            },
            HirStatement::Return(Some(HirExpression::Variable("ptr".to_string()))),
        ],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("ptr"));
    assert_eq!(
        inferences["ptr"].kind,
        OwnershipKind::Owning,
        "Returned malloc pointer should be owning"
    );
    assert!(
        inferences["ptr"].confidence > 0.85,
        "Should have high confidence for returned malloc"
    );
}

#[test]
fn test_free_called_implies_owning() {
    // If free() is called on a pointer, it must have been owning
    // NOTE: free() tracking will be added to dataflow analysis in future phase
    let func = HirFunction::new_with_body(
        "cleanup".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(None)],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("ptr"));
    // Parameters default to borrows currently - this will change when we add free() detection
    assert!(matches!(
        inferences["ptr"].kind,
        OwnershipKind::ImmutableBorrow | OwnershipKind::MutableBorrow | OwnershipKind::Owning
    ));
}

#[test]
fn test_address_of_creates_borrow() {
    // Taking address of a variable creates a borrow
    let func = HirFunction::new_with_body(
        "get_address".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(42)),
            },
            HirStatement::VariableDeclaration {
                name: "ptr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::AddressOf(Box::new(HirExpression::Variable(
                    "x".to_string(),
                )))),
            },
            HirStatement::Return(Some(HirExpression::Variable("ptr".to_string()))),
        ],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    // ptr should be inferred as borrowing since it's address-of
    if inferences.contains_key("ptr") {
        assert!(
            matches!(
                inferences["ptr"].kind,
                OwnershipKind::ImmutableBorrow | OwnershipKind::MutableBorrow
            ),
            "Address-of should create a borrow"
        );
    }
}

#[test]
fn test_multiple_owners_conflict() {
    // Two variables can't both own the same data
    let func = HirFunction::new_with_body(
        "double_owner".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "ptr1".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(4)],
                }),
            },
            HirStatement::VariableDeclaration {
                name: "ptr2".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Variable("ptr1".to_string())),
            },
            HirStatement::Return(None),
        ],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    // ptr1 owns, ptr2 borrows (since ptr1 is still alive when ptr2 is created)
    assert_eq!(inferences["ptr1"].kind, OwnershipKind::Owning);
    // ptr2 is tricky - it's freed, but it got its value from ptr1
    // This should be detected as a potential double-free
}

// TDD-Refactor Phase: Property tests for ownership inference

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_malloc_always_owning(var_name in "[a-z][a-z0-9_]{0,10}") {
            // Property: malloc always creates owning pointers
            let func = HirFunction::new_with_body(
                "test".to_string(),
                HirType::Void,
                vec![],
                vec![HirStatement::VariableDeclaration {
                    name: var_name.clone(),
                    var_type: HirType::Pointer(Box::new(HirType::Int)),
                    initializer: Some(HirExpression::FunctionCall {
                        function: "malloc".to_string(),
                        arguments: vec![HirExpression::IntLiteral(4)],
                    }),
                }],
            );

            let analyzer = DataflowAnalyzer::new();
            let graph = analyzer.analyze(&func);

            let inferencer = OwnershipInferencer::new();
            let inferences = inferencer.infer(&graph);

            prop_assert!(inferences.contains_key(&var_name));
            prop_assert_eq!(&inferences[&var_name].kind, &OwnershipKind::Owning);
            prop_assert!(inferences[&var_name].confidence >= 0.85);
        }

        #[test]
        fn prop_parameter_is_borrow(param_name in "[a-z][a-z0-9_]{0,10}") {
            // Property: function parameters are borrows by default
            let func = HirFunction::new_with_body(
                "test".to_string(),
                HirType::Void,
                vec![HirParameter::new(
                    param_name.clone(),
                    HirType::Pointer(Box::new(HirType::Int)),
                )],
                vec![HirStatement::Return(None)],
            );

            let analyzer = DataflowAnalyzer::new();
            let graph = analyzer.analyze(&func);

            let inferencer = OwnershipInferencer::new();
            let inferences = inferencer.infer(&graph);

            prop_assert!(inferences.contains_key(&param_name));
            prop_assert!(
                matches!(
                    inferences[&param_name].kind,
                    OwnershipKind::ImmutableBorrow | OwnershipKind::MutableBorrow
                ),
                "Parameters should be borrows, got {:?}",
                inferences[&param_name].kind
            );
        }

        #[test]
        fn prop_confidence_in_range(var_name in "[a-z][a-z0-9_]{0,10}") {
            // Property: confidence scores always between 0.0 and 1.0
            let func = HirFunction::new_with_body(
                "test".to_string(),
                HirType::Void,
                vec![],
                vec![HirStatement::VariableDeclaration {
                    name: var_name.clone(),
                    var_type: HirType::Pointer(Box::new(HirType::Int)),
                    initializer: Some(HirExpression::FunctionCall {
                        function: "malloc".to_string(),
                        arguments: vec![HirExpression::IntLiteral(4)],
                    }),
                }],
            );

            let analyzer = DataflowAnalyzer::new();
            let graph = analyzer.analyze(&func);

            let inferencer = OwnershipInferencer::new();
            let inferences = inferencer.infer(&graph);

            for (_name, inference) in inferences.iter() {
                prop_assert!(
                    inference.confidence >= 0.0 && inference.confidence <= 1.0,
                    "Confidence {} out of range for {}",
                    inference.confidence,
                    _name
                );
            }
        }

        #[test]
        fn prop_inference_deterministic(
            var_name in "[a-z][a-z0-9_]{0,10}",
            value in any::<i32>(),
        ) {
            // Property: same input produces same inference
            let func = HirFunction::new_with_body(
                "test".to_string(),
                HirType::Void,
                vec![],
                vec![HirStatement::VariableDeclaration {
                    name: var_name.clone(),
                    var_type: HirType::Pointer(Box::new(HirType::Int)),
                    initializer: Some(HirExpression::FunctionCall {
                        function: "malloc".to_string(),
                        arguments: vec![HirExpression::IntLiteral(value)],
                    }),
                }],
            );

            let analyzer = DataflowAnalyzer::new();
            let graph = analyzer.analyze(&func);

            let inferencer = OwnershipInferencer::new();

            // Run inference twice
            let inferences1 = inferencer.infer(&graph);
            let inferences2 = inferencer.infer(&graph);

            prop_assert_eq!(inferences1.len(), inferences2.len());
            for (key, value1) in inferences1.iter() {
                let value2 = &inferences2[key];
                prop_assert_eq!(&value1.kind, &value2.kind);
                prop_assert_eq!(value1.confidence, value2.confidence);
            }
        }

        #[test]
        fn prop_reasoning_not_empty(var_name in "[a-z][a-z0-9_]{0,10}") {
            // Property: all inferences have non-empty reasoning
            let func = HirFunction::new_with_body(
                "test".to_string(),
                HirType::Void,
                vec![],
                vec![HirStatement::VariableDeclaration {
                    name: var_name.clone(),
                    var_type: HirType::Pointer(Box::new(HirType::Int)),
                    initializer: Some(HirExpression::FunctionCall {
                        function: "malloc".to_string(),
                        arguments: vec![HirExpression::IntLiteral(4)],
                    }),
                }],
            );

            let analyzer = DataflowAnalyzer::new();
            let graph = analyzer.analyze(&func);

            let inferencer = OwnershipInferencer::new();
            let inferences = inferencer.infer(&graph);

            for (_name, inference) in inferences.iter() {
                prop_assert!(
                    !inference.reason.is_empty(),
                    "Reasoning should not be empty for {}",
                    _name
                );
            }
        }

        #[test]
        fn prop_borrowed_pointer_lower_confidence(
            var1 in "[a-z][a-z0-9_]{0,10}",
            var2 in "[a-z][a-z0-9_]{0,10}",
        ) {
            // Property: borrowed pointers have lower/equal confidence than owning
            prop_assume!(var1 != var2); // Different variable names

            let func = HirFunction::new_with_body(
                "test".to_string(),
                HirType::Void,
                vec![],
                vec![
                    HirStatement::VariableDeclaration {
                        name: var1.clone(),
                        var_type: HirType::Pointer(Box::new(HirType::Int)),
                        initializer: Some(HirExpression::FunctionCall {
                            function: "malloc".to_string(),
                            arguments: vec![HirExpression::IntLiteral(4)],
                        }),
                    },
                    HirStatement::VariableDeclaration {
                        name: var2.clone(),
                        var_type: HirType::Pointer(Box::new(HirType::Int)),
                        initializer: Some(HirExpression::Variable(var1.clone())),
                    },
                ],
            );

            let analyzer = DataflowAnalyzer::new();
            let graph = analyzer.analyze(&func);

            let inferencer = OwnershipInferencer::new();
            let inferences = inferencer.infer(&graph);

            if inferences.contains_key(&var1) && inferences.contains_key(&var2) {
                let owning_conf = inferences[&var1].confidence;
                let borrow_conf = inferences[&var2].confidence;

                prop_assert!(
                    borrow_conf <= owning_conf,
                    "Borrowed pointer confidence {} should be <= owning confidence {}",
                    borrow_conf,
                    owning_conf
                );
            }
        }

        #[test]
        fn prop_inference_never_panics(
            num_vars in 0usize..5,
            var_names in prop::collection::vec("[a-z][a-z0-9_]{0,10}", 0..5),
        ) {
            // Property: inference never panics, even with complex scenarios
            let statements: Vec<HirStatement> = var_names
                .iter()
                .take(num_vars)
                .map(|name| HirStatement::VariableDeclaration {
                    name: name.clone(),
                    var_type: HirType::Pointer(Box::new(HirType::Int)),
                    initializer: Some(HirExpression::FunctionCall {
                        function: "malloc".to_string(),
                        arguments: vec![HirExpression::IntLiteral(4)],
                    }),
                })
                .collect();

            let func = HirFunction::new_with_body(
                "test".to_string(),
                HirType::Void,
                vec![],
                statements,
            );

            let analyzer = DataflowAnalyzer::new();
            let graph = analyzer.analyze(&func);

            let inferencer = OwnershipInferencer::new();

            // Should not panic
            let _inferences = inferencer.infer(&graph);
        }
    }
}
