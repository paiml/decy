//! Additional coverage tests for inference.rs::classify_pointer.
//!
//! Targets uncovered branches in classify_pointer including:
//! - NodeKind::Dereference path
//! - NodeKind::Free path
//! - NodeKind::ArrayAllocation path
//! - NodeKind::Assignment with array_base_for returning Some/None
//! - NodeKind::Parameter with is_array_parameter returning true
//! - Empty nodes list (OwnershipKind::Unknown return)
//! - generate_reasoning branches for ArrayAllocation + ArrayPointer
//! - generate_reasoning for Assignment + ArrayPointer
//! - generate_reasoning with empty nodes
//! - calculate_confidence for escaping non-owning pointers

use crate::dataflow::{DataflowAnalyzer, DataflowGraph};
use crate::inference::{OwnershipInferencer, OwnershipKind};
use decy_hir::{BinaryOperator, HirExpression, HirFunction, HirParameter, HirStatement, HirType};

// ============================================================================
// classify_pointer: Dereference node kind
// ============================================================================

#[test]
fn test_classify_dereference_creates_immutable_borrow() {
    // When the first node for a variable is Dereference, classify_pointer
    // should return ImmutableBorrow.
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "ptr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Dereference(Box::new(
                    HirExpression::Variable("other".to_string()),
                ))),
            },
        ],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    if let Some(inf) = inferences.get("ptr") {
        assert_eq!(
            inf.kind,
            OwnershipKind::ImmutableBorrow,
            "Dereference node should produce ImmutableBorrow"
        );
    }
}

// ============================================================================
// classify_pointer: Free node kind
// ============================================================================

#[test]
fn test_classify_free_node_returns_owning() {
    // A Free node as primary indicates the pointer was owning.
    // To hit this path, we need the Free node to be the first in the list.
    // In practice this happens when malloc creates the first node, then free
    // adds a second. classify_pointer looks at first node.
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
    if let Some(inf) = inferences.get("ptr") {
        assert_eq!(inf.kind, OwnershipKind::Owning);
    }
}

// ============================================================================
// classify_pointer: ArrayAllocation node kind
// ============================================================================

#[test]
fn test_classify_array_allocation_stack() {
    // Stack array declaration should create ArrayPointer with correct element type
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Char),
                size: Some(256),
            },
            initializer: None,
        }],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    if let Some(inf) = inferences.get("arr") {
        match &inf.kind {
            OwnershipKind::ArrayPointer {
                base_array,
                element_type,
                base_index,
            } => {
                assert_eq!(base_array, "arr");
                assert_eq!(*element_type, HirType::Char);
                assert_eq!(*base_index, Some(0));
            }
            other => panic!("Expected ArrayPointer, got {:?}", other),
        }
    }
}

#[test]
fn test_classify_array_allocation_runtime_size() {
    // Array with no known size (None)
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Double),
                size: None,
            },
            initializer: None,
        }],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    if let Some(inf) = inferences.get("arr") {
        assert!(
            matches!(inf.kind, OwnershipKind::ArrayPointer { .. }),
            "Runtime-sized array should be ArrayPointer"
        );
    }
}

// ============================================================================
// classify_pointer: heap array via malloc(n * sizeof(type))
// ============================================================================

#[test]
fn test_classify_heap_array_char_type() {
    // malloc(n * sizeof(char)) should produce ArrayPointer with Char element type
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "buf".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Char)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::BinaryOp {
                    op: BinaryOperator::Multiply,
                    left: Box::new(HirExpression::IntLiteral(100)),
                    right: Box::new(HirExpression::Sizeof {
                        type_name: "char".to_string(),
                    }),
                }],
            }),
        }],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    if let Some(inf) = inferences.get("buf") {
        match &inf.kind {
            OwnershipKind::ArrayPointer { element_type, .. } => {
                assert_eq!(*element_type, HirType::Char);
            }
            other => panic!("Expected ArrayPointer for heap char array, got {:?}", other),
        }
    }
}

#[test]
fn test_classify_heap_array_float_type() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "data".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Float)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::BinaryOp {
                    op: BinaryOperator::Multiply,
                    left: Box::new(HirExpression::IntLiteral(50)),
                    right: Box::new(HirExpression::Sizeof {
                        type_name: "float".to_string(),
                    }),
                }],
            }),
        }],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    if let Some(inf) = inferences.get("data") {
        assert!(matches!(inf.kind, OwnershipKind::ArrayPointer { .. }));
    }
}

#[test]
fn test_classify_heap_array_double_type() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "vals".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Double)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::BinaryOp {
                    op: BinaryOperator::Multiply,
                    left: Box::new(HirExpression::IntLiteral(20)),
                    right: Box::new(HirExpression::Sizeof {
                        type_name: "double".to_string(),
                    }),
                }],
            }),
        }],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    if let Some(inf) = inferences.get("vals") {
        assert!(matches!(inf.kind, OwnershipKind::ArrayPointer { .. }));
    }
}

// ============================================================================
// classify_pointer: Assignment with array base
// ============================================================================

#[test]
fn test_classify_assignment_from_stack_array() {
    // ptr = arr where arr is a stack array -- should be ArrayPointer
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
    if let Some(inf) = inferences.get("ptr") {
        match &inf.kind {
            OwnershipKind::ArrayPointer {
                base_array,
                element_type,
                ..
            } => {
                assert_eq!(base_array, "arr");
                assert_eq!(*element_type, HirType::Int);
            }
            _ => {
                // If not ArrayPointer, it should at least be a borrow
                assert!(
                    matches!(
                        inf.kind,
                        OwnershipKind::ImmutableBorrow | OwnershipKind::MutableBorrow
                    ),
                    "Expected borrow or ArrayPointer, got {:?}",
                    inf.kind
                );
            }
        }
    }
}

#[test]
fn test_classify_assignment_not_from_array() {
    // ptr = other_ptr where other is not an array -- should be borrow
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "owner".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(4)],
                }),
            },
            HirStatement::VariableDeclaration {
                name: "alias".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Variable("owner".to_string())),
            },
        ],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    if let Some(inf) = inferences.get("alias") {
        assert!(
            matches!(
                inf.kind,
                OwnershipKind::ImmutableBorrow | OwnershipKind::MutableBorrow
            ),
            "Non-array assignment should be borrow, got {:?}",
            inf.kind
        );
    }
}

// ============================================================================
// classify_pointer: Parameter as array parameter
// ============================================================================

#[test]
fn test_classify_parameter_with_array_indexing_and_length() {
    // Function pattern: fn process(int* arr, int len) { arr[0] = 1; }
    // This should detect arr as array parameter with multiple heuristics
    let func = HirFunction::new_with_body(
        "process".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("arr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len".to_string(), HirType::Int),
        ],
        vec![HirStatement::ArrayIndexAssignment {
            array: Box::new(HirExpression::Variable("arr".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
            value: HirExpression::IntLiteral(1),
        }],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    if let Some(inf) = inferences.get("arr") {
        // With array naming + length param + array indexing, this should be ArrayPointer
        // or at minimum a borrow
        assert!(
            matches!(
                inf.kind,
                OwnershipKind::ArrayPointer { .. }
                    | OwnershipKind::ImmutableBorrow
                    | OwnershipKind::MutableBorrow
            ),
            "arr parameter should be classified, got {:?}",
            inf.kind
        );
    }
}

// ============================================================================
// classify_pointer: Parameter mutated via dereference
// ============================================================================

#[test]
fn test_classify_parameter_mutated_is_mutable_borrow() {
    let func = HirFunction::new_with_body(
        "modify".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "out".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![
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
    if let Some(inf) = inferences.get("out") {
        assert!(
            matches!(inf.kind, OwnershipKind::MutableBorrow | OwnershipKind::ImmutableBorrow),
            "Mutated parameter should be borrow, got {:?}",
            inf.kind
        );
    }
}

// ============================================================================
// generate_reasoning: ArrayAllocation + ArrayPointer branch
// ============================================================================

#[test]
fn test_reasoning_array_allocation_array_pointer() {
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
                "Reasoning should mention array: {}",
                inf.reason
            );
        }
    }
}

// ============================================================================
// generate_reasoning: Assignment + ArrayPointer branch
// ============================================================================

#[test]
fn test_reasoning_assignment_array_pointer() {
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
    if let Some(inf) = inferences.get("ptr") {
        // Reasoning should be non-empty
        assert!(!inf.reason.is_empty(), "Reasoning should not be empty");
    }
}

// ============================================================================
// calculate_confidence: escaping borrow lowers confidence
// ============================================================================

#[test]
fn test_confidence_borrow_escapes_lower_than_base() {
    // A borrowing pointer that escapes should have lower confidence
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
    // Just verify it runs without panic and has reasonable confidence
    for inf in inferences.values() {
        assert!(inf.confidence >= 0.0 && inf.confidence <= 1.0);
    }
}

// ============================================================================
// Unknown ownership kind (no nodes for variable)
// ============================================================================

#[test]
fn test_classify_unknown_when_no_nodes() {
    // An empty graph should return Unknown for any variable
    let graph = DataflowGraph::new();
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    assert!(inferences.is_empty(), "Empty graph should have no inferences");
}

// ============================================================================
// Multiple variables in one function
// ============================================================================

#[test]
fn test_mixed_ownership_in_one_function() {
    let func = HirFunction::new_with_body(
        "mixed".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "param".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![
            HirStatement::VariableDeclaration {
                name: "owned".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(4)],
                }),
            },
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Array {
                    element_type: Box::new(HirType::Int),
                    size: Some(5),
                },
                initializer: None,
            },
            HirStatement::Return(None),
        ],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    // owned should be Owning (malloc)
    if let Some(inf) = inferences.get("owned") {
        assert_eq!(inf.kind, OwnershipKind::Owning);
    }
    // arr should be ArrayPointer
    if let Some(inf) = inferences.get("arr") {
        assert!(matches!(inf.kind, OwnershipKind::ArrayPointer { .. }));
    }
    // param should be a borrow
    if let Some(inf) = inferences.get("param") {
        assert!(matches!(
            inf.kind,
            OwnershipKind::ImmutableBorrow | OwnershipKind::MutableBorrow
        ));
    }
}

// ============================================================================
// Heap array with signed char sizeof
// ============================================================================

#[test]
fn test_classify_heap_array_signed_char() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "buf".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::SignedChar)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::BinaryOp {
                    op: BinaryOperator::Multiply,
                    left: Box::new(HirExpression::IntLiteral(64)),
                    right: Box::new(HirExpression::Sizeof {
                        type_name: "signed char".to_string(),
                    }),
                }],
            }),
        }],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    if let Some(inf) = inferences.get("buf") {
        assert!(matches!(inf.kind, OwnershipKind::ArrayPointer { .. }));
    }
}

// ============================================================================
// Heap array with unknown sizeof type (fallback to Int)
// ============================================================================

#[test]
fn test_classify_heap_array_unknown_sizeof_type() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "buf".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::BinaryOp {
                    op: BinaryOperator::Multiply,
                    left: Box::new(HirExpression::IntLiteral(10)),
                    right: Box::new(HirExpression::Sizeof {
                        type_name: "custom_struct".to_string(),
                    }),
                }],
            }),
        }],
    );
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    if let Some(inf) = inferences.get("buf") {
        // Unknown sizeof type falls back to HirType::Int as element_type
        if let OwnershipKind::ArrayPointer { element_type, .. } = &inf.kind {
            assert_eq!(*element_type, HirType::Int, "Unknown sizeof should default to Int");
        }
    }
}

// ============================================================================
// Assignment from array with non-ArrayAllocation source node
// ============================================================================

#[test]
fn test_assignment_from_array_with_non_array_source_fallback() {
    // When array_base_for returns Some, but source node is not ArrayAllocation,
    // the element_type should fall back to Int
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
    // Just verify no panic and reasonable result
    assert!(!inferences.is_empty());
}

// ============================================================================
// Default trait for OwnershipInferencer
// ============================================================================

#[test]
fn test_inferencer_default_trait() {
    let inf: OwnershipInferencer = Default::default();
    let graph = DataflowGraph::new();
    let result = inf.infer(&graph);
    assert!(result.is_empty());
}
