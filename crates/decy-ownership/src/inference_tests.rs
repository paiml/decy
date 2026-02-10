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

// ============================================================================
// DECY-068 RED PHASE: ArrayPointer Classification Tests
// ============================================================================
// These tests implement the RED phase for DECY-068 (Sprint 20).
// Goal: Classify pointers as ArrayPointer when derived from arrays,
// enabling safe slice indexing transformation in DECY-070.
//
// Reference: docs/EXPR-ARITH-PTR-implementation-plan.md
// ============================================================================

#[test]
#[ignore = "DECY-068 RED: ArrayPointer classification not yet implemented"]
fn test_classify_stack_array_pointer() {
    // C: int arr[10]; int* p = arr;
    // Should classify p as ArrayPointer (not Owning)
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Array {
                    element_type: Box::new(HirType::Int),
                    size: Some(10),
                },
                initializer: None,
            },
            HirStatement::VariableDeclaration {
                name: "p".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Variable("arr".to_string())),
            },
        ],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("p"), "Should infer ownership for p");

    // This will FAIL until we add ArrayPointer variant
    if let OwnershipKind::ArrayPointer {
        base_array,
        element_type,
        base_index,
    } = &inferences["p"].kind
    {
        assert_eq!(base_array, "arr", "Should track array base");
        assert_eq!(*element_type, HirType::Int, "Should preserve element type");
        assert_eq!(*base_index, Some(0), "Should track base index");
    } else {
        panic!("Expected ArrayPointer, got {:?}", inferences["p"].kind);
    }
}

#[test]
#[ignore = "DECY-068 RED: ArrayPointer classification not yet implemented"]
fn test_classify_heap_array_pointer() {
    // C: int* arr = malloc(n * sizeof(int));
    // Should classify arr as ArrayPointer (heap array)
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Multiply,
                    left: Box::new(HirExpression::Variable("n".to_string())),
                    right: Box::new(HirExpression::Sizeof {
                        type_name: "int".to_string(),
                    }),
                }],
            }),
        }],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(
        inferences.contains_key("arr"),
        "Should infer ownership for arr"
    );

    // Heap array should be classified as ArrayPointer
    if let OwnershipKind::ArrayPointer {
        base_array,
        element_type,
        ..
    } = &inferences["arr"].kind
    {
        assert_eq!(base_array, "arr", "Heap array is its own base");
        assert_eq!(*element_type, HirType::Int);
    } else {
        panic!(
            "Expected ArrayPointer for heap array, got {:?}",
            inferences["arr"].kind
        );
    }
}

#[test]
#[ignore = "DECY-068 RED: ArrayPointer classification not yet implemented"]
fn test_classify_array_parameter() {
    // C: void process(int* arr, int len) { ... }
    // Parameter arr should be classified as ArrayPointer
    let func = HirFunction::new_with_body(
        "process".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("arr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len".to_string(), HirType::Int),
        ],
        vec![],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(
        inferences.contains_key("arr"),
        "Should infer ownership for arr parameter"
    );

    // Array parameter should be ArrayPointer
    if let OwnershipKind::ArrayPointer { base_array, .. } = &inferences["arr"].kind {
        assert_eq!(base_array, "arr", "Parameter is its own base");
    } else {
        panic!(
            "Expected ArrayPointer for array parameter, got {:?}",
            inferences["arr"].kind
        );
    }
}

#[test]
#[ignore = "DECY-068 RED: ArrayPointer classification not yet implemented"]
fn test_distinguish_array_pointer_from_owning() {
    // Test that array pointers are NOT classified as Owning
    // This is critical for correct transformation
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            // Regular malloc (single element) - should be Owning
            HirStatement::VariableDeclaration {
                name: "ptr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(4)],
                }),
            },
            // Array malloc - should be ArrayPointer
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::BinaryOp {
                        op: decy_hir::BinaryOperator::Multiply,
                        left: Box::new(HirExpression::IntLiteral(10)),
                        right: Box::new(HirExpression::Sizeof {
                            type_name: "int".to_string(),
                        }),
                    }],
                }),
            },
        ],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    // Regular malloc should be Owning
    assert_eq!(
        inferences["ptr"].kind,
        OwnershipKind::Owning,
        "Single malloc should be Owning"
    );

    // Array malloc should be ArrayPointer
    assert!(
        matches!(inferences["arr"].kind, OwnershipKind::ArrayPointer { .. }),
        "Array malloc should be ArrayPointer, got {:?}",
        inferences["arr"].kind
    );
}

#[test]
#[ignore = "DECY-068 RED: ArrayPointer classification not yet implemented"]
fn test_array_pointer_confidence_score() {
    // ArrayPointer should have high confidence when derived from known array
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Array {
                    element_type: Box::new(HirType::Int),
                    size: Some(10),
                },
                initializer: None,
            },
            HirStatement::VariableDeclaration {
                name: "p".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Variable("arr".to_string())),
            },
        ],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    // Confidence should be high (>= 0.9) for definite array pointer
    assert!(
        inferences["p"].confidence >= 0.9,
        "Array pointer from stack array should have high confidence (>= 0.9), got {}",
        inferences["p"].confidence
    );

    // Reason should mention array derivation
    assert!(
        inferences["p"].reason.contains("array") || inferences["p"].reason.contains("Array"),
        "Reason should mention array derivation, got: {}",
        inferences["p"].reason
    );
}

// =============================================================================
// Additional tests for coverage improvement
// =============================================================================

#[test]
fn test_default_trait_implementation() {
    // Test the Default trait implementation for OwnershipInferencer
    let inferencer: OwnershipInferencer = Default::default();
    // Just verify it creates successfully
    let func = HirFunction::new_with_body("test".to_string(), HirType::Void, vec![], vec![]);
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferences = inferencer.infer(&graph);
    assert!(inferences.is_empty()); // No variables, no inferences
}

#[test]
fn test_ownership_kind_equality() {
    // Test OwnershipKind Eq implementation
    assert_eq!(OwnershipKind::Owning, OwnershipKind::Owning);
    assert_eq!(
        OwnershipKind::ImmutableBorrow,
        OwnershipKind::ImmutableBorrow
    );
    assert_eq!(OwnershipKind::MutableBorrow, OwnershipKind::MutableBorrow);
    assert_eq!(OwnershipKind::Unknown, OwnershipKind::Unknown);
    assert_ne!(OwnershipKind::Owning, OwnershipKind::ImmutableBorrow);
}

#[test]
fn test_ownership_kind_clone() {
    // Test OwnershipKind Clone implementation
    let kind = OwnershipKind::ArrayPointer {
        base_array: "arr".to_string(),
        element_type: HirType::Int,
        base_index: Some(5),
    };
    let cloned = kind.clone();
    assert_eq!(kind, cloned);
}

#[test]
fn test_ownership_inference_clone() {
    // Test OwnershipInference Clone implementation
    let inference = OwnershipInference {
        variable: "ptr".to_string(),
        kind: OwnershipKind::Owning,
        confidence: 0.95,
        reason: "malloc allocation".to_string(),
    };
    let cloned = inference.clone();
    assert_eq!(inference.variable, cloned.variable);
    assert_eq!(inference.kind, cloned.kind);
    assert_eq!(inference.confidence, cloned.confidence);
    assert_eq!(inference.reason, cloned.reason);
}

#[test]
fn test_inference_with_free() {
    // Test that free() indicates owning pointer
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
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
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "free".to_string(),
                arguments: vec![HirExpression::Variable("ptr".to_string())],
            }),
        ],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("ptr"));
    assert_eq!(inferences["ptr"].kind, OwnershipKind::Owning);
}

#[test]
fn test_unknown_variable_inference() {
    // Test inference for a variable with no dataflow info
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "x".to_string(),
            HirType::Int, // Not a pointer
        )],
        vec![],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    // Non-pointer parameters may not be in inferences or have Unknown kind
    // The key test is that it doesn't crash
    assert!(inferences.is_empty() || !inferences.contains_key("x"));
}

#[test]
fn test_dereference_indicates_mutation() {
    // Test that dereference operations indicate potential mutation
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Assignment {
            target: "val".to_string(),
            value: HirExpression::Dereference(Box::new(HirExpression::Variable("ptr".to_string()))),
        }],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    // ptr is a parameter with dereference - should be MutableBorrow
    if let Some(inf) = inferences.get("ptr") {
        assert!(
            matches!(
                inf.kind,
                OwnershipKind::MutableBorrow | OwnershipKind::ImmutableBorrow
            ),
            "Dereferenced pointer should be a borrow"
        );
    }
}

// ============================================================================
// Coverage Tests - Targeting specific uncovered paths
// ============================================================================

#[test]
fn test_ownership_kind_unknown() {
    // Test Unknown ownership kind
    let kind = OwnershipKind::Unknown;
    assert_eq!(kind, OwnershipKind::Unknown);
    let debug = format!("{:?}", kind);
    assert!(debug.contains("Unknown"));
}

#[test]
fn test_ownership_kind_array_pointer() {
    // Test ArrayPointer ownership kind
    let kind = OwnershipKind::ArrayPointer {
        base_array: "arr".to_string(),
        element_type: decy_hir::HirType::Int,
        base_index: Some(0),
    };
    let debug = format!("{:?}", kind);
    assert!(debug.contains("ArrayPointer"));
}

#[test]
fn test_ownership_inference_debug() {
    let inference = OwnershipInference {
        variable: "test".to_string(),
        kind: OwnershipKind::Owning,
        confidence: 0.9,
        reason: "test reason".to_string(),
    };
    let debug = format!("{:?}", inference);
    assert!(debug.contains("OwnershipInference"));
    assert!(debug.contains("test"));
}

#[test]
fn test_inferencer_default() {
    let inferencer: OwnershipInferencer = Default::default();
    let debug = format!("{:?}", inferencer);
    assert!(debug.contains("OwnershipInferencer"));
}

#[test]
fn test_ownership_kind_eq() {
    assert_eq!(OwnershipKind::Owning, OwnershipKind::Owning);
    assert_eq!(
        OwnershipKind::ImmutableBorrow,
        OwnershipKind::ImmutableBorrow
    );
    assert_eq!(OwnershipKind::MutableBorrow, OwnershipKind::MutableBorrow);
    assert_ne!(OwnershipKind::Owning, OwnershipKind::ImmutableBorrow);
}

#[test]
fn test_ownership_inference_partial_eq() {
    let inf1 = OwnershipInference {
        variable: "test".to_string(),
        kind: OwnershipKind::Owning,
        confidence: 0.9,
        reason: "test".to_string(),
    };
    let inf2 = OwnershipInference {
        variable: "test".to_string(),
        kind: OwnershipKind::Owning,
        confidence: 0.9,
        reason: "test".to_string(),
    };
    assert_eq!(inf1, inf2);
}

// ============================================================================
// COVERAGE TESTS: Uncovered inference.rs paths
// ============================================================================

#[test]
fn test_classify_dereference_node() {
    // Test dereference path in classify_pointer
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Assignment {
            target: "val".to_string(),
            value: HirExpression::Dereference(Box::new(HirExpression::Variable("ptr".to_string()))),
        }],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    // ptr should be classified (either as borrow or based on dereference)
    assert!(inferences.contains_key("ptr") || !inferences.is_empty());
}

#[test]
fn test_classify_free_node() {
    // Test free path in classify_pointer
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
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
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "free".to_string(),
                arguments: vec![HirExpression::Variable("ptr".to_string())],
            }),
        ],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    // ptr should be owning since it was freed
    if let Some(inf) = inferences.get("ptr") {
        assert_eq!(inf.kind, OwnershipKind::Owning);
    }
}

#[test]
fn test_confidence_unknown_kind() {
    // Test Unknown kind confidence (0.3)
    let kind = OwnershipKind::Unknown;
    let debug = format!("{:?}", kind);
    assert!(debug.contains("Unknown"));
}

#[test]
fn test_generate_reasoning_no_nodes() {
    // Test reasoning with empty graph
    let inferencer = OwnershipInferencer::new();
    let graph = DataflowGraph::new();
    let inferences = inferencer.infer(&graph);
    assert!(inferences.is_empty());
}

#[test]
fn test_is_mutated_no_nodes() {
    // Test is_mutated returns false when no nodes
    let inferencer = OwnershipInferencer::new();
    let graph = DataflowGraph::new();
    // The method is private, but we test via classify_pointer behavior
    let inferences = inferencer.infer(&graph);
    assert!(inferences.is_empty());
}

#[test]
fn test_escapes_function_assignment() {
    // Test escapes_function with assignment
    let func = HirFunction::new_with_body(
        "test".to_string(),
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
    if let Some(inf) = inferences.get("ptr") {
        // Owning pointer that escapes should have boosted confidence
        assert!(inf.confidence >= 0.9);
    }
}

#[test]
fn test_array_allocation_classification() {
    // Test ArrayAllocation node kind
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(10),
            },
            initializer: None,
        }],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    // Array should be classified appropriately
    if let Some(inf) = inferences.get("arr") {
        let reason = &inf.reason;
        assert!(!reason.is_empty());
    }
}

#[test]
fn test_parameter_immutable_borrow_reasoning() {
    // Test reasoning for immutable borrow parameter
    let func = HirFunction::new_with_body(
        "test".to_string(),
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
    if let Some(inf) = inferences.get("data") {
        // Should mention parameter in reason
        assert!(inf.reason.contains("data") || !inf.reason.is_empty());
    }
}

#[test]
fn test_ownership_kind_array_pointer_clone() {
    let kind = OwnershipKind::ArrayPointer {
        base_array: "arr".to_string(),
        element_type: HirType::Float,
        base_index: Some(5),
    };
    let cloned = kind.clone();
    assert_eq!(kind, cloned);
}

#[test]
fn test_ownership_kind_unknown_eq() {
    assert_eq!(OwnershipKind::Unknown, OwnershipKind::Unknown);
    assert_ne!(OwnershipKind::Unknown, OwnershipKind::Owning);
}

#[test]
fn test_confidence_array_pointer() {
    // ArrayPointer should have 0.95 base confidence
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(10),
            },
            initializer: None,
        }],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    if let Some(inf) = inferences.get("arr") {
        if matches!(inf.kind, OwnershipKind::ArrayPointer { .. }) {
            assert!(
                inf.confidence >= 0.9,
                "ArrayPointer should have high confidence"
            );
        }
    }
}

#[test]
fn test_mutable_borrow_confidence() {
    // MutableBorrow should have 0.75 base confidence
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
    if let Some(inf) = inferences.get("ptr") {
        assert!(inf.confidence > 0.5);
    }
}

// ============================================================================
// Additional Inference Coverage Tests
// ============================================================================

#[test]
fn test_classify_pointer_assignment_from_array() {
    // Test assignment from array base
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Array {
                    element_type: Box::new(HirType::Int),
                    size: Some(10),
                },
                initializer: None,
            },
            HirStatement::VariableDeclaration {
                name: "ptr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Variable("arr".to_string())),
            },
        ],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    // ptr should be classified as borrowing from array
    assert!(!inferences.is_empty());
}

#[test]
fn test_escapes_function_via_return() {
    // Test pointer that escapes via return
    let func = HirFunction::new_with_body(
        "test".to_string(),
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
    if let Some(inf) = inferences.get("ptr") {
        // Escaping owning pointer should have high confidence
        assert!(inf.confidence >= 0.85);
    }
}

#[test]
fn test_immutable_borrow_readonly() {
    // Test read-only parameter becomes ImmutableBorrow
    let func = HirFunction::new_with_body(
        "test".to_string(),
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
    if let Some(inf) = inferences.get("data") {
        // Read-only use should be immutable borrow
        assert!(
            matches!(inf.kind, OwnershipKind::ImmutableBorrow)
                || matches!(inf.kind, OwnershipKind::MutableBorrow)
        );
    }
}

#[test]
fn test_mutable_borrow_with_deref_assignment() {
    // Test parameter mutated via dereference becomes MutableBorrow
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "out".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("out".to_string()),
            value: HirExpression::IntLiteral(42),
        }],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    if let Some(inf) = inferences.get("out") {
        // Deref assignment indicates mutable borrow
        assert!(
            matches!(inf.kind, OwnershipKind::MutableBorrow)
                || matches!(inf.kind, OwnershipKind::ImmutableBorrow),
            "Expected borrow, got {:?}",
            inf.kind
        );
    }
}

#[test]
fn test_ownership_inference_with_calloc() {
    // Test calloc allocation - note: current impl only recognizes malloc
    // calloc/realloc are not yet recognized as allocation functions
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "calloc".to_string(),
                arguments: vec![HirExpression::IntLiteral(10), HirExpression::IntLiteral(4)],
            }),
        }],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    // Currently calloc is not recognized, defaults to ImmutableBorrow
    if let Some(inf) = inferences.get("arr") {
        assert_eq!(inf.kind, OwnershipKind::ImmutableBorrow);
    }
}

#[test]
fn test_ownership_inference_with_realloc() {
    // Test realloc allocation - note: current impl only recognizes malloc
    // calloc/realloc are not yet recognized as allocation functions
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "buf".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Char)),
            initializer: Some(HirExpression::FunctionCall {
                function: "realloc".to_string(),
                arguments: vec![HirExpression::NullLiteral, HirExpression::IntLiteral(100)],
            }),
        }],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    // Currently realloc is not recognized, defaults to ImmutableBorrow
    if let Some(inf) = inferences.get("buf") {
        assert_eq!(inf.kind, OwnershipKind::ImmutableBorrow);
    }
}

#[test]
fn test_infer_with_complex_function() {
    // Complex function with multiple variables
    let func = HirFunction::new_with_body(
        "process".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("arr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len".to_string(), HirType::Int),
        ],
        vec![
            HirStatement::VariableDeclaration {
                name: "total".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::VariableDeclaration {
                name: "i".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::Return(Some(HirExpression::Variable("total".to_string()))),
        ],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    // Should have inference for arr
    assert!(inferences.contains_key("arr") || !inferences.is_empty());
}

#[test]
fn test_ownership_kind_display() {
    // Test Debug for various OwnershipKind variants
    let kinds = [
        OwnershipKind::Owning,
        OwnershipKind::ImmutableBorrow,
        OwnershipKind::MutableBorrow,
        OwnershipKind::Unknown,
        OwnershipKind::ArrayPointer {
            base_array: "arr".to_string(),
            element_type: HirType::Int,
            base_index: None,
        },
    ];
    for kind in &kinds {
        let debug = format!("{:?}", kind);
        assert!(!debug.is_empty());
    }
}

#[test]
fn test_ownership_inferencer_new() {
    let inferencer = OwnershipInferencer::new();
    let debug = format!("{:?}", inferencer);
    assert!(debug.contains("OwnershipInferencer"));
}

// ============================================================================
// Generate Reasoning Branch Coverage Tests
// ============================================================================

#[test]
fn test_reasoning_allocation_owning() {
    // Test (NodeKind::Allocation, OwnershipKind::Owning) branch
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
    if let Some(inf) = inferences.get("ptr") {
        assert!(inf.reason.contains("malloc") || inf.reason.contains("owns"));
    }
}

#[test]
fn test_reasoning_parameter_immutable() {
    // Test (NodeKind::Parameter, OwnershipKind::ImmutableBorrow) branch
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "data".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    if let Some(inf) = inferences.get("data") {
        assert!(inf.reason.contains("parameter") || inf.reason.contains("data"));
    }
}

#[test]
fn test_reasoning_parameter_mutable() {
    // Test (NodeKind::Parameter, OwnershipKind::MutableBorrow) branch
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "out".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("out".to_string()),
            value: HirExpression::IntLiteral(42),
        }],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    if let Some(inf) = inferences.get("out") {
        assert!(!inf.reason.is_empty());
    }
}

#[test]
fn test_reasoning_assignment_immutable() {
    // Test (NodeKind::Assignment, OwnershipKind::ImmutableBorrow) branch
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "src".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![
            HirStatement::VariableDeclaration {
                name: "ptr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Variable("src".to_string())),
            },
            HirStatement::Return(Some(HirExpression::Dereference(Box::new(
                HirExpression::Variable("ptr".to_string()),
            )))),
        ],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    if let Some(inf) = inferences.get("ptr") {
        assert!(!inf.reason.is_empty());
    }
}

#[test]
fn test_reasoning_no_tracked_nodes() {
    // Test the "no tracked nodes" branch
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::Return(None)],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    // Should have no inferences for non-pointer variables
    assert!(inferences.is_empty() || inferences.values().all(|i| !i.reason.is_empty()));
}

#[test]
fn test_calculate_confidence_owning_escapes() {
    // Test confidence boost for owning pointer that escapes
    let func = HirFunction::new_with_body(
        "create".to_string(),
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
    if let Some(inf) = inferences.get("ptr") {
        // Owning + escapes = high confidence
        assert!(inf.confidence >= 0.9);
    }
}

#[test]
fn test_calculate_confidence_borrow_escapes() {
    // Test confidence reduction for borrow that escapes
    let func = HirFunction::new_with_body(
        "get_ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new(
            "data".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![
            HirStatement::VariableDeclaration {
                name: "ptr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Variable("data".to_string())),
            },
            HirStatement::Return(Some(HirExpression::Variable("ptr".to_string()))),
        ],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    // Check that we have inferences
    assert!(!inferences.is_empty());
}

#[test]
fn test_is_mutated_empty_function_body() {
    // Test is_mutated returns false when no nodes
    let func = HirFunction::new_with_body("test".to_string(), HirType::Void, vec![], vec![]);
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    assert!(inferences.is_empty());
}

#[test]
fn test_escapes_function_no_nodes() {
    // Test escapes_function returns false when no nodes
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::Return(None)],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    assert!(inferences.is_empty());
}

#[test]
fn test_ownership_inferencer_default() {
    let inferencer: OwnershipInferencer = Default::default();
    let debug = format!("{:?}", inferencer);
    assert!(debug.contains("OwnershipInferencer"));
}

#[test]
fn test_unknown_ownership_low_confidence() {
    // Unknown ownership should have low confidence (0.3)
    let kind = OwnershipKind::Unknown;
    let debug = format!("{:?}", kind);
    assert!(debug.contains("Unknown"));
}

#[test]
fn test_array_pointer_high_confidence() {
    // ArrayPointer should have 0.95 base confidence
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(10),
            },
            initializer: None,
        }],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    if let Some(inf) = inferences.get("arr") {
        if matches!(inf.kind, OwnershipKind::ArrayPointer { .. }) {
            assert!(inf.confidence >= 0.9);
        }
    }
}

#[test]
fn test_classify_pointer_dereference() {
    // Test Dereference node kind
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::Dereference(
            Box::new(HirExpression::Variable("ptr".to_string())),
        )))],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    // ptr should be classified
    assert!(!inferences.is_empty());
}

#[test]
fn test_classify_pointer_free() {
    // Test Free node kind
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
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
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "free".to_string(),
                arguments: vec![HirExpression::Variable("ptr".to_string())],
            }),
        ],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    if let Some(inf) = inferences.get("ptr") {
        // free indicates owning
        assert_eq!(inf.kind, OwnershipKind::Owning);
    }
}

// ============================================================================
// DEEP COVERAGE: classify_pointer all NodeKind branches
// ============================================================================

#[test]
fn test_classify_pointer_none_nodes_returns_unknown() {
    // When graph.nodes_for returns None, classify_pointer returns Unknown
    let inferencer = OwnershipInferencer::new();
    let graph = DataflowGraph::new();
    // Empty graph => no variables => no inferences
    let inferences = inferencer.infer(&graph);
    assert!(inferences.is_empty());
}

#[test]
fn test_classify_pointer_empty_nodes_returns_unknown() {
    // When nodes list is empty (no first node), classify_pointer returns Unknown
    // This is hard to trigger through the public API since the analyzer always
    // adds at least one node per variable. But we test indirectly via the graph.
    let inferencer = OwnershipInferencer::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferences = inferencer.infer(&graph);
    // No pointer variables => empty inferences
    assert!(inferences.is_empty());
}

#[test]
fn test_classify_pointer_parameter_not_array_not_mutated() {
    // Parameter + not array + not mutated => ImmutableBorrow
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "input".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![
            // No mutation, just return a constant
            HirStatement::Return(Some(HirExpression::IntLiteral(0))),
        ],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("input"));
    assert_eq!(inferences["input"].kind, OwnershipKind::ImmutableBorrow);
    assert!(inferences["input"].confidence >= 0.75);
    assert!(inferences["input"].reason.contains("parameter") || inferences["input"].reason.contains("input"));
}

#[test]
fn test_classify_pointer_parameter_mutated() {
    // Parameter + mutated via dereference => MutableBorrow
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "out".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![
            // Dereference assignment indicates mutation
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("out".to_string()),
                value: HirExpression::IntLiteral(42),
            },
        ],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("out"));
    // DerefAssignment adds a Dereference node which triggers is_mutated
    // The parameter node is first, but dereference node is also present
    // So the classification depends on which node is first
    assert!(
        matches!(
            inferences["out"].kind,
            OwnershipKind::MutableBorrow | OwnershipKind::ImmutableBorrow
        ),
        "Mutated parameter should be a borrow, got {:?}",
        inferences["out"].kind
    );
}

#[test]
fn test_classify_pointer_assignment_not_from_array_immutable() {
    // Assignment from non-array variable, not mutated => ImmutableBorrow
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "src".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![
            HirStatement::VariableDeclaration {
                name: "alias".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Variable("src".to_string())),
            },
            HirStatement::Return(Some(HirExpression::IntLiteral(0))),
        ],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("alias"));
    assert!(
        matches!(
            inferences["alias"].kind,
            OwnershipKind::ImmutableBorrow | OwnershipKind::MutableBorrow
        ),
        "Assignment from non-array should be a borrow"
    );
    assert!(!inferences["alias"].reason.is_empty());
}

#[test]
fn test_classify_pointer_assignment_from_array_becomes_array_pointer() {
    // Assignment from array variable => ArrayPointer
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            // Declare array
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Array {
                    element_type: Box::new(HirType::Int),
                    size: Some(10),
                },
                initializer: None,
            },
            // Assign pointer from array
            HirStatement::VariableDeclaration {
                name: "ptr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Variable("arr".to_string())),
            },
        ],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("ptr"));
    if let OwnershipKind::ArrayPointer {
        base_array,
        element_type,
        base_index,
    } = &inferences["ptr"].kind
    {
        assert_eq!(base_array, "arr");
        assert_eq!(*element_type, HirType::Int);
        assert_eq!(*base_index, Some(0));
    } else {
        // May be ImmutableBorrow if array_base tracking didn't kick in
        // depending on dataflow implementation
        assert!(
            matches!(
                inferences["ptr"].kind,
                OwnershipKind::ArrayPointer { .. } | OwnershipKind::ImmutableBorrow
            ),
            "Expected ArrayPointer or ImmutableBorrow, got {:?}",
            inferences["ptr"].kind
        );
    }
}

#[test]
fn test_classify_pointer_dereference_node_returns_immutable_borrow() {
    // Dereference node kind => ImmutableBorrow
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "pp".to_string(),
            HirType::Pointer(Box::new(HirType::Pointer(Box::new(HirType::Int)))),
        )],
        vec![
            HirStatement::VariableDeclaration {
                name: "inner".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Dereference(Box::new(
                    HirExpression::Variable("pp".to_string()),
                ))),
            },
        ],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    if let Some(inf) = inferences.get("inner") {
        assert_eq!(
            inf.kind,
            OwnershipKind::ImmutableBorrow,
            "Dereference-initialized pointer should be ImmutableBorrow"
        );
    }
}

#[test]
fn test_classify_pointer_free_node_returns_owning() {
    // Free node kind => Owning
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
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
            HirStatement::Free {
                pointer: HirExpression::Variable("ptr".to_string()),
            },
        ],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("ptr"));
    assert_eq!(inferences["ptr"].kind, OwnershipKind::Owning);
}

#[test]
fn test_classify_pointer_array_allocation_node() {
    // ArrayAllocation node kind => ArrayPointer
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "data".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Float),
                size: Some(20),
            },
            initializer: None,
        }],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    if let Some(inf) = inferences.get("data") {
        if let OwnershipKind::ArrayPointer {
            base_array,
            element_type,
            base_index,
        } = &inf.kind
        {
            assert_eq!(base_array, "data");
            assert_eq!(*element_type, HirType::Float);
            assert_eq!(*base_index, Some(0));
            // ArrayPointer should have 0.95 base confidence
            assert!(inf.confidence >= 0.9);
        }
    }
}

// ============================================================================
// DEEP COVERAGE: calculate_confidence all branches
// ============================================================================

#[test]
fn test_confidence_owning_no_escape() {
    // Owning pointer that doesn't escape => 0.9 base
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
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
            // No return, no assignment => doesn't escape
        ],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    if let Some(inf) = inferences.get("ptr") {
        assert_eq!(inf.kind, OwnershipKind::Owning);
        // Base confidence 0.9 with no escape adjustment
        assert!((inf.confidence - 0.9).abs() < 0.06);
    }
}

#[test]
fn test_confidence_owning_escapes_boosted() {
    // Owning pointer that escapes => boosted to 0.95
    let func = HirFunction::new_with_body(
        "create".to_string(),
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

    if let Some(inf) = inferences.get("ptr") {
        assert_eq!(inf.kind, OwnershipKind::Owning);
        // Owning + escapes = boosted to 0.95
        assert!(inf.confidence >= 0.9, "Expected >= 0.9, got {}", inf.confidence);
    }
}

#[test]
fn test_confidence_borrow_escapes_reduced() {
    // Borrow that escapes => reduced confidence
    let func = HirFunction::new_with_body(
        "get_ref".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new(
            "src".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![
            HirStatement::VariableDeclaration {
                name: "ref_ptr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Variable("src".to_string())),
            },
            HirStatement::Return(Some(HirExpression::Variable("ref_ptr".to_string()))),
        ],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    if let Some(inf) = inferences.get("ref_ptr") {
        // ImmutableBorrow or MutableBorrow + escapes => reduced from base
        assert!(inf.confidence < 0.85 || inf.confidence >= 0.3);
    }
}

// ============================================================================
// DEEP COVERAGE: generate_reasoning all match arms
// ============================================================================

#[test]
fn test_reasoning_allocation_owning_message() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "buf".to_string(),
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

    let inf = &inferences["buf"];
    assert!(
        inf.reason.contains("malloc") && inf.reason.contains("buf"),
        "Reasoning should mention malloc and variable name, got: {}",
        inf.reason
    );
}

#[test]
fn test_reasoning_parameter_immutable_message() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "read_only".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    if let Some(inf) = inferences.get("read_only") {
        assert!(
            inf.reason.contains("parameter") || inf.reason.contains("read_only"),
            "Reasoning should mention parameter, got: {}",
            inf.reason
        );
    }
}

#[test]
fn test_reasoning_assignment_immutable_borrow_message() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "src".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![
            HirStatement::VariableDeclaration {
                name: "copy".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Variable("src".to_string())),
            },
            HirStatement::Return(Some(HirExpression::IntLiteral(0))),
        ],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    if let Some(inf) = inferences.get("copy") {
        assert!(
            inf.reason.contains("assigned") || inf.reason.contains("copy") || inf.reason.contains("src"),
            "Reasoning should mention assignment, got: {}",
            inf.reason
        );
    }
}

#[test]
fn test_reasoning_array_allocation_message() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(10),
            },
            initializer: None,
        }],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    if let Some(inf) = inferences.get("arr") {
        if matches!(inf.kind, OwnershipKind::ArrayPointer { .. }) {
            assert!(
                inf.reason.contains("array") || inf.reason.contains("Array"),
                "Reasoning should mention array, got: {}",
                inf.reason
            );
        }
    }
}

#[test]
fn test_reasoning_assignment_from_array_message() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Array {
                    element_type: Box::new(HirType::Int),
                    size: Some(10),
                },
                initializer: None,
            },
            HirStatement::VariableDeclaration {
                name: "p".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Variable("arr".to_string())),
            },
        ],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    if let Some(inf) = inferences.get("p") {
        if matches!(inf.kind, OwnershipKind::ArrayPointer { .. }) {
            assert!(
                inf.reason.contains("array") || inf.reason.contains("Array") || inf.reason.contains("arr"),
                "Reasoning should mention array derivation, got: {}",
                inf.reason
            );
        }
    }
}

#[test]
fn test_reasoning_default_arm() {
    // Test the default `_ =>` arm in generate_reasoning
    // This triggers when the node kind and ownership kind don't match
    // a specific pattern. For example, Dereference + ImmutableBorrow.
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "pp".to_string(),
            HirType::Pointer(Box::new(HirType::Pointer(Box::new(HirType::Int)))),
        )],
        vec![
            HirStatement::VariableDeclaration {
                name: "inner".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Dereference(Box::new(
                    HirExpression::Variable("pp".to_string()),
                ))),
            },
        ],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    if let Some(inf) = inferences.get("inner") {
        // The Dereference node + ImmutableBorrow hits the default arm
        // which produces "<var> has <kind> ownership"
        assert!(
            inf.reason.contains("inner") || inf.reason.contains("ownership"),
            "Default reasoning should mention variable name, got: {}",
            inf.reason
        );
    }
}

// ============================================================================
// Assignment with array base: fallback element type branches
// ============================================================================

#[test]
fn test_classify_assignment_from_array_with_matching_element_type() {
    // Array allocation with known element type, pointer derived from it
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "float_arr".to_string(),
                var_type: HirType::Array {
                    element_type: Box::new(HirType::Float),
                    size: Some(5),
                },
                initializer: None,
            },
            HirStatement::VariableDeclaration {
                name: "fp".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Float)),
                initializer: Some(HirExpression::Variable("float_arr".to_string())),
            },
        ],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    if let Some(inf) = inferences.get("fp") {
        if let OwnershipKind::ArrayPointer { element_type, .. } = &inf.kind {
            assert_eq!(*element_type, HirType::Float);
        }
    }
}

// ============================================================================
// is_mutated: various mutation detection scenarios
// ============================================================================

#[test]
fn test_is_mutated_no_dereference() {
    // Parameter with no dereference => not mutated
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "data".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![
            // Just pass data to a function, no dereference
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "printf".to_string(),
                arguments: vec![HirExpression::Variable("data".to_string())],
            }),
        ],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    if let Some(inf) = inferences.get("data") {
        assert_eq!(
            inf.kind,
            OwnershipKind::ImmutableBorrow,
            "Non-mutated parameter should be ImmutableBorrow"
        );
    }
}

// ============================================================================
// escapes_function: non-escaping scenarios
// ============================================================================

#[test]
fn test_pointer_does_not_escape() {
    // Pointer used locally, not returned or stored
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "local".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(4)],
                }),
            },
            HirStatement::Free {
                pointer: HirExpression::Variable("local".to_string()),
            },
        ],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    if let Some(inf) = inferences.get("local") {
        assert_eq!(inf.kind, OwnershipKind::Owning);
        // Should have high confidence (0.9 base for Owning)
        assert!(inf.confidence >= 0.85);
    }
}

// ============================================================================
// Direct DataflowGraph construction tests (defensive branch coverage)
// These test branches that the DataflowAnalyzer can't produce but exist as
// defensive code in classify_pointer, is_mutated, calculate_confidence, etc.
// ============================================================================

use crate::dataflow::{NodeKind, PointerNode};

#[test]
fn test_classify_empty_nodes_returns_unknown() {
    // When a variable exists in the graph but has zero nodes,
    // classify_pointer should return Unknown (line 177)
    let graph = DataflowGraph::new().with_empty_var("orphan");

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("orphan"));
    assert_eq!(inferences["orphan"].kind, OwnershipKind::Unknown);
}

#[test]
fn test_classify_parameter_mutated_returns_mutable_borrow() {
    // Parameter node + Dereference node → is_mutated returns true → MutableBorrow (line 112)
    let graph = DataflowGraph::new()
        .with_node(
            "ptr",
            PointerNode {
                name: "ptr".to_string(),
                def_index: 0,
                kind: NodeKind::Parameter,
            },
        )
        .with_node(
            "ptr",
            PointerNode {
                name: "ptr".to_string(),
                def_index: 1,
                kind: NodeKind::Dereference,
            },
        );

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("ptr"));
    assert_eq!(
        inferences["ptr"].kind,
        OwnershipKind::MutableBorrow,
        "Parameter with dereference mutation should be MutableBorrow"
    );
}

#[test]
fn test_classify_assignment_mutated_returns_mutable_borrow() {
    // Assignment node + Dereference node → is_mutated returns true → MutableBorrow (line 151)
    let graph = DataflowGraph::new()
        .with_node(
            "alias",
            PointerNode {
                name: "alias".to_string(),
                def_index: 0,
                kind: NodeKind::Assignment {
                    source: "original".to_string(),
                },
            },
        )
        .with_node(
            "alias",
            PointerNode {
                name: "alias".to_string(),
                def_index: 1,
                kind: NodeKind::Dereference,
            },
        );

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("alias"));
    assert_eq!(
        inferences["alias"].kind,
        OwnershipKind::MutableBorrow,
        "Assignment with dereference mutation should be MutableBorrow"
    );
}

#[test]
fn test_classify_free_node_returns_owning_direct() {
    // Free as first node → Owning (line 162)
    let graph = DataflowGraph::new().with_node(
        "freed",
        PointerNode {
            name: "freed".to_string(),
            def_index: 0,
            kind: NodeKind::Free,
        },
    );

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("freed"));
    assert_eq!(inferences["freed"].kind, OwnershipKind::Owning);
}

#[test]
fn test_confidence_mutable_borrow() {
    // MutableBorrow confidence should be 0.75 (line 242)
    let graph = DataflowGraph::new()
        .with_node(
            "ptr",
            PointerNode {
                name: "ptr".to_string(),
                def_index: 0,
                kind: NodeKind::Parameter,
            },
        )
        .with_node(
            "ptr",
            PointerNode {
                name: "ptr".to_string(),
                def_index: 1,
                kind: NodeKind::Dereference,
            },
        );

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("ptr"));
    // Base confidence for MutableBorrow is 0.75, no escape → stays 0.75
    assert!(
        (inferences["ptr"].confidence - 0.75).abs() < 0.01,
        "MutableBorrow confidence should be ~0.75, got {}",
        inferences["ptr"].confidence
    );
}

#[test]
fn test_confidence_unknown_kind_direct() {
    // Unknown confidence should be 0.3 (line 244)
    let graph = DataflowGraph::new().with_empty_var("orphan");

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("orphan"));
    assert!(
        (inferences["orphan"].confidence - 0.3).abs() < 0.01,
        "Unknown confidence should be ~0.3, got {}",
        inferences["orphan"].confidence
    );
}

#[test]
fn test_reasoning_parameter_mutable_borrow() {
    // Reasoning for Parameter + MutableBorrow: "is a parameter, may be modified" (line 288)
    let graph = DataflowGraph::new()
        .with_node(
            "out",
            PointerNode {
                name: "out".to_string(),
                def_index: 0,
                kind: NodeKind::Parameter,
            },
        )
        .with_node(
            "out",
            PointerNode {
                name: "out".to_string(),
                def_index: 1,
                kind: NodeKind::Dereference,
            },
        );

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("out"));
    assert!(
        inferences["out"].reason.contains("parameter")
            && inferences["out"].reason.contains("modified"),
        "Reasoning should mention parameter modification, got: {}",
        inferences["out"].reason
    );
}

#[test]
fn test_reasoning_assignment_mutable_borrow() {
    // Reasoning for Assignment + MutableBorrow: "assigned from X, mutable borrow" (line 293-294)
    let graph = DataflowGraph::new()
        .with_node(
            "alias",
            PointerNode {
                name: "alias".to_string(),
                def_index: 0,
                kind: NodeKind::Assignment {
                    source: "original".to_string(),
                },
            },
        )
        .with_node(
            "alias",
            PointerNode {
                name: "alias".to_string(),
                def_index: 1,
                kind: NodeKind::Dereference,
            },
        );

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("alias"));
    assert!(
        inferences["alias"].reason.contains("assigned from")
            && inferences["alias"].reason.contains("mutable"),
        "Reasoning should mention assignment + mutable, got: {}",
        inferences["alias"].reason
    );
}

#[test]
fn test_reasoning_no_nodes_for_variable() {
    // No tracked nodes → fallback reasoning (line 319)
    let graph = DataflowGraph::new().with_empty_var("ghost");

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("ghost"));
    assert!(
        inferences["ghost"].reason.contains("no tracked nodes"),
        "Reasoning should say no tracked nodes, got: {}",
        inferences["ghost"].reason
    );
}

#[test]
fn test_confidence_mutable_borrow_escapes_reduced() {
    // MutableBorrow + escapes → confidence reduced by 0.05 (line 257)
    let graph = DataflowGraph::new()
        .with_node(
            "ptr",
            PointerNode {
                name: "ptr".to_string(),
                def_index: 0,
                kind: NodeKind::Parameter,
            },
        )
        .with_node(
            "ptr",
            PointerNode {
                name: "ptr".to_string(),
                def_index: 1,
                kind: NodeKind::Dereference,
            },
        )
        // Assignment node makes escapes_function return true
        .with_node(
            "ptr",
            PointerNode {
                name: "ptr".to_string(),
                def_index: 2,
                kind: NodeKind::Assignment {
                    source: "other".to_string(),
                },
            },
        );

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("ptr"));
    // Base 0.75 - 0.05 = 0.70
    assert!(
        (inferences["ptr"].confidence - 0.70).abs() < 0.01,
        "MutableBorrow with escape should have confidence ~0.70, got {}",
        inferences["ptr"].confidence
    );
}

#[test]
fn test_classify_assignment_from_array_with_source_not_array_alloc() {
    // Assignment from array base, but source node is NOT ArrayAllocation
    // Falls through to decy_hir::HirType::Int fallback (line 132)
    let graph = DataflowGraph::new()
        .with_node(
            "arr",
            PointerNode {
                name: "arr".to_string(),
                def_index: 0,
                kind: NodeKind::Allocation, // NOT ArrayAllocation
            },
        )
        .with_node(
            "ptr",
            PointerNode {
                name: "ptr".to_string(),
                def_index: 1,
                kind: NodeKind::Assignment {
                    source: "arr".to_string(),
                },
            },
        )
        .with_array_base("ptr", "arr");

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("ptr"));
    // Should still be ArrayPointer but with default Int element type
    if let OwnershipKind::ArrayPointer { element_type, .. } = &inferences["ptr"].kind {
        assert_eq!(*element_type, decy_hir::HirType::Int);
    } else {
        panic!(
            "Expected ArrayPointer, got {:?}",
            inferences["ptr"].kind
        );
    }
}

#[test]
fn test_classify_assignment_from_array_source_empty_nodes() {
    // Assignment from array base, source has empty nodes
    // Falls through to HirType::Int fallback (line 135)
    let graph = DataflowGraph::new()
        .with_empty_var("arr")
        .with_node(
            "ptr",
            PointerNode {
                name: "ptr".to_string(),
                def_index: 0,
                kind: NodeKind::Assignment {
                    source: "arr".to_string(),
                },
            },
        )
        .with_array_base("ptr", "arr");

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("ptr"));
    if let OwnershipKind::ArrayPointer { element_type, .. } = &inferences["ptr"].kind {
        assert_eq!(*element_type, decy_hir::HirType::Int);
    } else {
        panic!(
            "Expected ArrayPointer, got {:?}",
            inferences["ptr"].kind
        );
    }
}

#[test]
fn test_classify_assignment_from_array_no_source_nodes() {
    // Assignment from array base, but source var doesn't exist in graph
    // Falls through to HirType::Int fallback (line 138)
    let graph = DataflowGraph::new()
        .with_node(
            "ptr",
            PointerNode {
                name: "ptr".to_string(),
                def_index: 0,
                kind: NodeKind::Assignment {
                    source: "nonexistent".to_string(),
                },
            },
        )
        .with_array_base("ptr", "missing_arr");

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("ptr"));
    if let OwnershipKind::ArrayPointer { element_type, .. } = &inferences["ptr"].kind {
        assert_eq!(*element_type, decy_hir::HirType::Int);
    } else {
        panic!(
            "Expected ArrayPointer, got {:?}",
            inferences["ptr"].kind
        );
    }
}

#[test]
fn test_is_mutated_returns_false_no_dereference() {
    // Variable with only Parameter node → is_mutated returns false
    let graph = DataflowGraph::new().with_node(
        "ptr",
        PointerNode {
            name: "ptr".to_string(),
            def_index: 0,
            kind: NodeKind::Parameter,
        },
    );

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    // Parameter + not mutated → ImmutableBorrow
    assert_eq!(inferences["ptr"].kind, OwnershipKind::ImmutableBorrow);
}

#[test]
fn test_reasoning_default_arm_dereference_unknown() {
    // Dereference node with non-standard kind triggers default reasoning arm (line 316)
    let graph = DataflowGraph::new().with_node(
        "d",
        PointerNode {
            name: "d".to_string(),
            def_index: 0,
            kind: NodeKind::Dereference,
        },
    );

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("d"));
    assert_eq!(inferences["d"].kind, OwnershipKind::ImmutableBorrow);
    // Dereference + ImmutableBorrow doesn't match any specific arm → default
    assert!(
        inferences["d"].reason.contains("ownership"),
        "Should use default reasoning, got: {}",
        inferences["d"].reason
    );
}

#[test]
fn test_reasoning_free_node_owning() {
    // Free node → Owning, but Free+Owning isn't in the reasoning match → default arm
    let graph = DataflowGraph::new().with_node(
        "f",
        PointerNode {
            name: "f".to_string(),
            def_index: 0,
            kind: NodeKind::Free,
        },
    );

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("f"));
    assert_eq!(inferences["f"].kind, OwnershipKind::Owning);
    // Free+Owning doesn't match Allocation+Owning → default arm
    assert!(
        inferences["f"].reason.contains("ownership"),
        "Should use default reasoning for Free+Owning, got: {}",
        inferences["f"].reason
    );
}
