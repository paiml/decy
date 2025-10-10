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
