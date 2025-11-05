//! Tests for dataflow analysis module.

use crate::dataflow::*;
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType};

#[test]
fn test_build_dataflow_graph() {
    // Simple function with pointer assignment
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

    // Should have one node for ptr
    assert!(
        graph.nodes_for("ptr").is_some(),
        "Graph should track pointer variable"
    );
    let nodes = graph.nodes_for("ptr").unwrap();
    assert_eq!(nodes.len(), 1, "Should have one node for ptr");
    assert_eq!(nodes[0].kind, NodeKind::Allocation);
}

#[test]
fn test_track_pointer_assignments() {
    // Function with pointer assignment chain: ptr1 = malloc; ptr2 = ptr1
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

    // ptr2 should depend on ptr1
    assert!(
        graph.dependencies_for("ptr2").is_some(),
        "ptr2 should have dependencies"
    );
    let deps = graph.dependencies_for("ptr2").unwrap();
    assert!(deps.contains("ptr1"), "ptr2 should depend on ptr1");
}

#[test]
fn test_detect_use_after_free() {
    // Function with use-after-free: ptr = malloc; free(ptr); *ptr = 5;
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
            // Note: free() detection requires ExpressionStatement in HIR
            // For now, we'll test the detection infrastructure
            HirStatement::Assignment {
                target: "ptr".to_string(),
                value: HirExpression::IntLiteral(5),
            },
        ],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    // This is a placeholder test - actual use-after-free detection
    // will be implemented when ExpressionStatement is added to HIR
    assert!(graph.nodes_for("ptr").is_some());
}

#[test]
fn test_dependency_ordering() {
    // Test that dependencies are tracked in correct order
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "a".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(4)],
                }),
            },
            HirStatement::VariableDeclaration {
                name: "b".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Variable("a".to_string())),
            },
            HirStatement::VariableDeclaration {
                name: "c".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Variable("b".to_string())),
            },
        ],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    // Should track dependency chain: a -> b -> c
    let vars = graph.variables();
    assert_eq!(vars.len(), 3, "Should track all three variables");

    // Check that c depends on b, and b depends on a
    if let Some(c_deps) = graph.dependencies_for("c") {
        assert!(c_deps.contains("b"), "c should depend on b");
    } else {
        panic!("c should have dependencies");
    }

    if let Some(b_deps) = graph.dependencies_for("b") {
        assert!(b_deps.contains("a"), "b should depend on a");
    } else {
        panic!("b should have dependencies");
    }
}

#[test]
fn test_track_function_parameters() {
    // Function with pointer parameter
    let func = HirFunction::new_with_body(
        "process".to_string(),
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

    // Should track parameter as a node
    assert!(
        graph.nodes_for("ptr").is_some(),
        "Should track parameter pointer"
    );
    let nodes = graph.nodes_for("ptr").unwrap();
    assert_eq!(
        nodes[0].kind,
        NodeKind::Parameter,
        "Should mark as parameter"
    );
}

#[test]
fn test_track_dereference_operations() {
    // Function with dereference: int* ptr = malloc(); int x = *ptr;
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
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::Dereference(Box::new(
                    HirExpression::Variable("ptr".to_string()),
                ))),
            },
        ],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    // Should track that ptr is dereferenced
    assert!(graph.nodes_for("ptr").is_some());
}

#[test]
fn test_empty_function() {
    // Function with no pointers
    let func = HirFunction::new("empty".to_string(), HirType::Void, vec![]);

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    // Should have empty graph
    assert_eq!(
        graph.variables().len(),
        0,
        "Empty function should have no tracked variables"
    );
}

#[test]
fn test_non_pointer_variables_not_tracked() {
    // Function with integer variables (not pointers)
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

    // Should not track non-pointer variables
    assert_eq!(
        graph.variables().len(),
        0,
        "Should not track non-pointer variables"
    );
}

#[test]
fn test_multiple_pointer_allocations() {
    // Function with multiple pointer allocations
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
                var_type: HirType::Pointer(Box::new(HirType::Char)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(8)],
                }),
            },
        ],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    // Should track both pointers
    assert_eq!(graph.variables().len(), 2, "Should track both pointers");
    assert!(graph.nodes_for("ptr1").is_some());
    assert!(graph.nodes_for("ptr2").is_some());
}

// ============================================================================
// DECY-067 RED PHASE: Array Allocation Detection Tests
// ============================================================================
// These tests implement the RED phase for DECY-067 (Sprint 20).
// Goal: Detect when pointers are derived from arrays (stack, heap, parameters)
// to enable safe slice indexing transformation.
//
// Reference: docs/EXPR-ARITH-PTR-implementation-plan.md
// ============================================================================

#[test]
#[ignore = "DECY-067 RED: Array allocation detection not yet implemented"]
fn test_detect_stack_array_allocation() {
    // C: int arr[10];
    // Should detect as NodeKind::ArrayAllocation { size: 10, element_type: Int }
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

    // Should detect array allocation
    let nodes = graph
        .nodes_for("arr")
        .expect("Array should be tracked in dataflow");
    assert_eq!(nodes.len(), 1, "Should have one node for array");

    // This will FAIL until we implement NodeKind::ArrayAllocation
    if let NodeKind::ArrayAllocation { size, element_type } = &nodes[0].kind {
        assert_eq!(*size, Some(10));
        assert_eq!(*element_type, HirType::Int);
    } else {
        panic!(
            "Expected ArrayAllocation node kind, got {:?}",
            nodes[0].kind
        );
    }
}

#[test]
#[ignore = "DECY-067 RED: Array allocation detection not yet implemented"]
fn test_detect_heap_array_allocation() {
    // C: int* arr = malloc(n * sizeof(int));
    // Should detect multiplication pattern → heap array allocation
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

    let nodes = graph
        .nodes_for("arr")
        .expect("Heap array should be tracked");
    assert_eq!(nodes.len(), 1);

    // Should detect as array allocation (heap)
    if let NodeKind::ArrayAllocation { size, element_type } = &nodes[0].kind {
        assert_eq!(*size, None); // Runtime size
        assert_eq!(*element_type, HirType::Int);
    } else {
        panic!("Expected ArrayAllocation for malloc pattern");
    }
}

#[test]
#[ignore = "DECY-067 RED: Array base tracking not yet implemented"]
fn test_track_pointer_from_array() {
    // C: int arr[10]; int* p = arr;
    // Pointer p should track that it's derived from array arr
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

    // Pointer p should have array base tracked
    let p_nodes = graph.nodes_for("p").expect("Pointer p should be tracked");

    // This will FAIL until we add array base tracking
    if let NodeKind::Assignment { source } = &p_nodes[0].kind {
        assert_eq!(source, "arr", "Should track array base");

        // Should also have array base metadata
        let array_base = graph
            .array_base_for("p")
            .expect("Pointer should have array base tracked");
        assert_eq!(array_base, "arr");
    } else {
        panic!("Expected Assignment with ArrayBase tracking");
    }
}

#[test]
#[ignore = "DECY-067 RED: Array parameter detection not yet implemented"]
fn test_track_array_parameter() {
    // C: void process(int* arr, int len) { ... }
    // Parameter arr should be classified as array pointer (if paired with length param)
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

    // arr parameter should be tracked
    let arr_nodes = graph
        .nodes_for("arr")
        .expect("Array parameter should be tracked");
    assert_eq!(arr_nodes.len(), 1);

    // Should be marked as array pointer parameter
    // This will FAIL until we add array parameter detection
    assert_eq!(
        arr_nodes[0].kind,
        NodeKind::Parameter,
        "Should be parameter node"
    );

    // Check for array metadata (pointer + length parameter pattern)
    let is_array_param = graph
        .is_array_parameter("arr")
        .expect("Should have array parameter metadata");
    assert!(
        is_array_param,
        "arr should be detected as array parameter (has length param)"
    );
}

#[test]
#[ignore = "DECY-067 RED: Multidimensional array detection not yet implemented"]
fn test_detect_multidimensional_array() {
    // C: int matrix[5][10];
    // Should detect as 2D array allocation
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "matrix".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Array {
                    element_type: Box::new(HirType::Int),
                    size: Some(10),
                }),
                size: Some(5),
            },
            initializer: None,
        }],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    let nodes = graph.nodes_for("matrix").expect("Matrix should be tracked");

    // Should detect as array allocation with nested array type
    if let NodeKind::ArrayAllocation { size, element_type } = &nodes[0].kind {
        assert_eq!(*size, Some(5));
        // Element type should be array of int[10]
        assert!(matches!(
            element_type,
            HirType::Array {
                element_type: _,
                size: Some(10)
            }
        ));
    } else {
        panic!("Expected ArrayAllocation for multidimensional array");
    }
}
// ============================================================================
// DECY-071 Tests: Array Parameter Detection Heuristics
// ============================================================================

/// Test detection of array parameter with length: fn(int* arr, int len)
#[test]
// DECY-071 GREEN: Test now active
fn test_detect_array_parameter_with_length() {
    // C: void process(int* arr, int len) { }
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

    // Should detect arr as array parameter (has length param)
    assert_eq!(
        graph.is_array_parameter("arr"),
        Some(true),
        "Should detect arr as array parameter (followed by length param)"
    );
}

/// Test detection of array parameter with size: fn(char* buf, size_t size)
#[test]
// DECY-071 GREEN: Test now active
fn test_detect_array_parameter_with_size() {
    // C: void process(char* buf, size_t size) { }
    let func = HirFunction::new_with_body(
        "process".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("buf".to_string(), HirType::Pointer(Box::new(HirType::Char))),
            HirParameter::new("size".to_string(), HirType::Int), // size_t → int
        ],
        vec![],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    // Should detect buf as array parameter (common naming: buf + size)
    assert_eq!(
        graph.is_array_parameter("buf"),
        Some(true),
        "Should detect buf as array parameter (common naming pattern)"
    );
}

/// Test detection of array parameter with count: fn(int* array, int count)
#[test]
// DECY-071 GREEN: Test now active
fn test_detect_array_parameter_with_count() {
    // C: void process(int* array, int count) { }
    let func = HirFunction::new_with_body(
        "process".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("array".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("count".to_string(), HirType::Int),
        ],
        vec![],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    // Should detect array as array parameter (name: array + count)
    assert_eq!(
        graph.is_array_parameter("array"),
        Some(true),
        "Should detect array as array parameter (array + count naming)"
    );
}

/// Test NON-detection of single pointer: fn(int* ptr)
#[test]
fn test_no_detect_single_pointer_without_length() {
    // C: void process(int* ptr) { }
    let func = HirFunction::new_with_body(
        "process".to_string(),
        HirType::Void,
        vec![HirParameter::new("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)))],
        vec![],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    // Should NOT detect ptr as array parameter (no length param)
    assert_eq!(
        graph.is_array_parameter("ptr"),
        Some(false),
        "Should NOT detect ptr as array parameter (no length param)"
    );
}

/// Test detection with array indexing in body: arr[i]
#[test]
// DECY-071 GREEN: Test now active
fn test_detect_array_parameter_with_indexing_usage() {
    // C: void process(int* arr, int len) { arr[0] = 1; }
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

    // Should detect arr as array parameter (indexing usage confirms)
    assert_eq!(
        graph.is_array_parameter("arr"),
        Some(true),
        "Should detect arr as array parameter (has indexing usage)"
    );
}

/// Test NON-detection of pointer with arithmetic (not array-like)
#[test]
fn test_no_detect_pointer_with_arithmetic() {
    // C: void process(int* ptr, int offset) { int* p = ptr + offset; }
    let func = HirFunction::new_with_body(
        "process".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("offset".to_string(), HirType::Int),
        ],
        vec![HirStatement::VariableDeclaration {
            name: "p".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("ptr".to_string())),
                right: Box::new(HirExpression::Variable("offset".to_string())),
            }),
        }],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    // Should NOT detect ptr as array (pointer arithmetic is not array-like)
    // Note: This could be debatable, but conservatively we say pointer arithmetic
    // with offset suggests non-array usage
    assert_eq!(
        graph.is_array_parameter("ptr"),
        Some(false),
        "Should NOT detect ptr as array (has pointer arithmetic)"
    );
}

/// Test detection of multiple array parameters
#[test]
// DECY-071 GREEN: Test now active
fn test_detect_multiple_array_parameters() {
    // C: void merge(int* arr1, int len1, int* arr2, int len2) { }
    let func = HirFunction::new_with_body(
        "merge".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("arr1".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len1".to_string(), HirType::Int),
            HirParameter::new("arr2".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len2".to_string(), HirType::Int),
        ],
        vec![],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    // Should detect both arr1 and arr2 as array parameters
    assert_eq!(
        graph.is_array_parameter("arr1"),
        Some(true),
        "Should detect arr1 as array parameter"
    );
    assert_eq!(
        graph.is_array_parameter("arr2"),
        Some(true),
        "Should detect arr2 as array parameter"
    );
}

/// Test detection with struct pointer (should NOT be array)
#[test]
fn test_no_detect_struct_pointer() {
    // C: void process(struct Point* ptr, int count) { }
    let func = HirFunction::new_with_body(
        "process".to_string(),
        HirType::Void,
        vec![
            HirParameter::new(
                "ptr".to_string(),
                HirType::Pointer(Box::new(HirType::Struct("Point".to_string()))),
            ),
            HirParameter::new("count".to_string(), HirType::Int),
        ],
        vec![],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    // Struct pointer with count is ambiguous - could be array of structs
    // For now, conservatively return false to avoid false positives
    assert_eq!(
        graph.is_array_parameter("ptr"),
        Some(false),
        "Should conservatively NOT detect struct pointer as array"
    );
}

/// Test detection confidence with strong signals
#[test]
// DECY-071 GREEN: Test now active
fn test_detect_array_parameter_high_confidence() {
    // C: void fill_array(int* arr, int len) { for(int i=0; i<len; i++) arr[i]=0; }
    // Strong signals: arr name, len param, array indexing
    let func = HirFunction::new_with_body(
        "fill_array".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("arr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len".to_string(), HirType::Int),
        ],
        vec![HirStatement::For {
            init: Some(Box::new(HirStatement::VariableDeclaration {
                name: "i".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            })),
            condition: HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::LessThan,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::Variable("len".to_string())),
            },
            increment: Some(Box::new(HirStatement::Assignment {
                target: "i".to_string(),
                value: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("i".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            })),
            body: vec![HirStatement::ArrayIndexAssignment {
                array: Box::new(HirExpression::Variable("arr".to_string())),
                index: Box::new(HirExpression::Variable("i".to_string())),
                value: HirExpression::IntLiteral(0),
            }],
        }],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    // Should detect arr with HIGH confidence (multiple strong signals)
    assert_eq!(
        graph.is_array_parameter("arr"),
        Some(true),
        "Should detect arr as array parameter with high confidence"
    );
}

/// Test edge case: pointer parameter without name information
#[test]
fn test_unnamed_pointer_parameter() {
    // C: void process(int*, int) { }  (unnamed parameters)
    let func = HirFunction::new_with_body(
        "process".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("_arg0".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("_arg1".to_string(), HirType::Int),
        ],
        vec![],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    // Can't use name heuristics, but pattern (ptr + int) still suggests array
    // This test documents current behavior - can be debated
    assert_eq!(
        graph.is_array_parameter("_arg0"),
        Some(false),
        "Unnamed parameter should default to false (conservative)"
    );
}
