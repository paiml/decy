//! Tests for scope-based lifetime analysis.

use super::*;
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType};

#[test]
fn test_build_scope_tree() {
    // Test building a basic scope tree for a function with nested scopes
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(5)),
            },
            HirStatement::If {
                condition: HirExpression::Variable("x".to_string()),
                then_block: vec![HirStatement::VariableDeclaration {
                    name: "y".to_string(),
                    var_type: HirType::Int,
                    initializer: Some(HirExpression::IntLiteral(10)),
                }],
                else_block: None,
            },
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);

    // Should have at least 2 scopes: function scope and if-block scope
    assert!(
        scope_tree.scopes().len() >= 2,
        "Should have function scope and nested scopes"
    );

    // Root scope should contain 'x'
    let root_scope = scope_tree.get_scope(0).unwrap();
    assert!(
        root_scope.variables.contains(&"x".to_string()),
        "Root scope should contain variable x"
    );
}

#[test]
fn test_track_variable_lifetimes() {
    // Test tracking lifetimes of variables in different scopes
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(5)),
            },
            HirStatement::Return(Some(HirExpression::Variable("x".to_string()))),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes.contains_key("x"),
        "Should track lifetime of variable x"
    );

    let x_lifetime = &lifetimes["x"];
    assert_eq!(x_lifetime.name, "x");
    assert_eq!(x_lifetime.declared_in_scope, 0);
}

#[test]
fn test_detect_dangling_pointer() {
    // Test detecting potential dangling pointer issues
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![],
        vec![
            HirStatement::If {
                condition: HirExpression::IntLiteral(1),
                then_block: vec![HirStatement::VariableDeclaration {
                    name: "local_ptr".to_string(),
                    var_type: HirType::Pointer(Box::new(HirType::Int)),
                    initializer: Some(HirExpression::IntLiteral(0)),
                }],
                else_block: None,
            },
            // Returning a pointer to local variable would be dangerous
            HirStatement::Return(Some(HirExpression::Variable("local_ptr".to_string()))),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);
    let dangling = analyzer.detect_dangling_pointers(&lifetimes);

    // Should detect that local_ptr may be dangling
    assert!(
        dangling.contains(&"local_ptr".to_string()),
        "Should detect dangling pointer for local_ptr"
    );
}

#[test]
fn test_lifetime_relationships() {
    // Test inferring lifetime relationships between variables
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "outer".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(1)),
            },
            HirStatement::If {
                condition: HirExpression::IntLiteral(1),
                then_block: vec![HirStatement::VariableDeclaration {
                    name: "inner".to_string(),
                    var_type: HirType::Int,
                    initializer: Some(HirExpression::IntLiteral(2)),
                }],
                else_block: None,
            },
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);
    let relationships = analyzer.infer_lifetime_relationships(&lifetimes, &scope_tree);

    // 'outer' should outlive 'inner' (outer is in parent scope)
    if let Some(relation) = relationships.get(&("outer".to_string(), "inner".to_string())) {
        assert!(
            matches!(relation, LifetimeRelation::Outlives),
            "outer should outlive inner"
        );
    }
}

#[test]
fn test_nested_scopes() {
    // Test building scope tree with multiple levels of nesting
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "a".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(1)),
            },
            HirStatement::If {
                condition: HirExpression::IntLiteral(1),
                then_block: vec![
                    HirStatement::VariableDeclaration {
                        name: "b".to_string(),
                        var_type: HirType::Int,
                        initializer: Some(HirExpression::IntLiteral(2)),
                    },
                    HirStatement::If {
                        condition: HirExpression::IntLiteral(1),
                        then_block: vec![HirStatement::VariableDeclaration {
                            name: "c".to_string(),
                            var_type: HirType::Int,
                            initializer: Some(HirExpression::IntLiteral(3)),
                        }],
                        else_block: None,
                    },
                ],
                else_block: None,
            },
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);

    // Should have at least 3 scopes: function, outer if, inner if
    assert!(
        scope_tree.scopes().len() >= 3,
        "Should have multiple nested scopes"
    );

    // Check that scope tree correctly represents nesting
    // The root scope (0) should contain 'a'
    let root = scope_tree.get_scope(0).unwrap();
    assert!(root.variables.contains(&"a".to_string()));
}

#[test]
fn test_while_loop_scopes() {
    // Test scope analysis for while loops
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "i".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::While {
                condition: HirExpression::IntLiteral(1),
                body: vec![HirStatement::VariableDeclaration {
                    name: "temp".to_string(),
                    var_type: HirType::Int,
                    initializer: Some(HirExpression::IntLiteral(5)),
                }],
            },
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);

    // Should have at least 2 scopes: function and while loop
    assert!(scope_tree.scopes().len() >= 2);

    // Root scope should have 'i'
    let root = scope_tree.get_scope(0).unwrap();
    assert!(root.variables.contains(&"i".to_string()));
}

#[test]
fn test_else_block_scopes() {
    // Test that else blocks get their own scopes
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::If {
            condition: HirExpression::IntLiteral(1),
            then_block: vec![HirStatement::VariableDeclaration {
                name: "then_var".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(1)),
            }],
            else_block: Some(vec![HirStatement::VariableDeclaration {
                name: "else_var".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(2)),
            }]),
        }],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);

    // Should have at least 3 scopes: function, then, else
    assert!(
        scope_tree.scopes().len() >= 3,
        "Should have separate scopes for then and else"
    );
}

#[test]
fn test_function_parameters_not_tracked_in_tree() {
    // Test that function parameters are handled separately
    // (they're not part of statement-based scope analysis)
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "param".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);

    // Parameters are part of function scope but not explicitly tracked as variables
    // in the scope tree (they have function-level lifetime by default)
    assert!(!scope_tree.scopes().is_empty());
}

#[test]
fn test_escaping_variables() {
    // Test detection of variables that escape the function
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "result".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(42)),
            },
            HirStatement::Return(Some(HirExpression::Variable("result".to_string()))),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    let result_lifetime = &lifetimes["result"];
    assert!(
        result_lifetime.escapes,
        "Variable returned from function should be marked as escaping"
    );
}

#[test]
fn test_independent_branches() {
    // Test that variables in different if branches have independent lifetimes
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::If {
                condition: HirExpression::IntLiteral(1),
                then_block: vec![HirStatement::VariableDeclaration {
                    name: "then_var".to_string(),
                    var_type: HirType::Int,
                    initializer: Some(HirExpression::IntLiteral(1)),
                }],
                else_block: None,
            },
            HirStatement::If {
                condition: HirExpression::IntLiteral(0),
                then_block: vec![HirStatement::VariableDeclaration {
                    name: "other_var".to_string(),
                    var_type: HirType::Int,
                    initializer: Some(HirExpression::IntLiteral(2)),
                }],
                else_block: None,
            },
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);
    let relationships = analyzer.infer_lifetime_relationships(&lifetimes, &scope_tree);

    // Variables in separate if blocks should have independent lifetimes
    if let Some(relation) = relationships.get(&("then_var".to_string(), "other_var".to_string())) {
        assert!(
            matches!(relation, LifetimeRelation::Independent),
            "Variables in separate branches should be independent"
        );
    }
}

#[test]
fn test_complex_scope_analysis() {
    // Integration test with complex nested structure
    let func = HirFunction::new_with_body(
        "complex".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "input".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![
            HirStatement::VariableDeclaration {
                name: "outer".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(1)),
            },
            HirStatement::While {
                condition: HirExpression::IntLiteral(1),
                body: vec![
                    HirStatement::VariableDeclaration {
                        name: "loop_var".to_string(),
                        var_type: HirType::Int,
                        initializer: Some(HirExpression::IntLiteral(2)),
                    },
                    HirStatement::If {
                        condition: HirExpression::Variable("loop_var".to_string()),
                        then_block: vec![HirStatement::VariableDeclaration {
                            name: "inner".to_string(),
                            var_type: HirType::Int,
                            initializer: Some(HirExpression::IntLiteral(3)),
                        }],
                        else_block: Some(vec![HirStatement::Break]),
                    },
                ],
            },
            HirStatement::Return(Some(HirExpression::Variable("outer".to_string()))),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);
    let relationships = analyzer.infer_lifetime_relationships(&lifetimes, &scope_tree);

    // Should successfully analyze complex nested structure
    assert!(
        scope_tree.scopes().len() >= 4,
        "Should have multiple scopes"
    );
    assert!(lifetimes.len() >= 3, "Should track multiple variables");
    assert!(!relationships.is_empty(), "Should infer relationships");

    // 'outer' should escape since it's returned
    assert!(lifetimes["outer"].escapes);
}

// ============================================================================
// Coverage Improvement Tests - Expression Uses Variable
// ============================================================================

#[test]
fn test_expression_uses_variable_in_cast() {
    // Test that Cast expression properly detects variable usage
    let func = HirFunction::new_with_body(
        "cast_test".to_string(),
        HirType::Float,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(42)),
            },
            HirStatement::Return(Some(HirExpression::Cast {
                expr: Box::new(HirExpression::Variable("x".to_string())),
                target_type: HirType::Float,
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["x"].escapes,
        "Variable used in cast should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_in_compound_literal() {
    // Test that CompoundLiteral expression properly detects variable usage
    let func = HirFunction::new_with_body(
        "compound_test".to_string(),
        HirType::Struct("Point".to_string()),
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(10)),
            },
            HirStatement::Return(Some(HirExpression::CompoundLiteral {
                literal_type: HirType::Struct("Point".to_string()),
                initializers: vec![
                    HirExpression::Variable("x".to_string()),
                    HirExpression::IntLiteral(20),
                ],
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["x"].escapes,
        "Variable in compound literal should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_in_is_not_null() {
    // Test that IsNotNull expression properly detects variable usage
    let func = HirFunction::new_with_body(
        "is_not_null_test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "ptr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::NullLiteral),
            },
            HirStatement::Return(Some(HirExpression::IsNotNull(Box::new(
                HirExpression::Variable("ptr".to_string()),
            )))),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["ptr"].escapes,
        "Variable in IsNotNull should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_in_calloc() {
    // Test that Calloc count expression properly detects variable usage
    let func = HirFunction::new_with_body(
        "calloc_test".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "count".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(10)),
            },
            HirStatement::Return(Some(HirExpression::Calloc {
                count: Box::new(HirExpression::Variable("count".to_string())),
                element_type: Box::new(HirType::Int),
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["count"].escapes,
        "Variable in calloc should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_in_malloc() {
    // Test that Malloc size expression properly detects variable usage
    let func = HirFunction::new_with_body(
        "malloc_test".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "size".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(100)),
            },
            HirStatement::Return(Some(HirExpression::Malloc {
                size: Box::new(HirExpression::Variable("size".to_string())),
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["size"].escapes,
        "Variable in malloc should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_in_realloc() {
    // Test that Realloc expression properly detects variable usage in pointer and new_size
    let func = HirFunction::new_with_body(
        "realloc_test".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "ptr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::NullLiteral),
            },
            HirStatement::VariableDeclaration {
                name: "new_size".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(200)),
            },
            HirStatement::Return(Some(HirExpression::Realloc {
                pointer: Box::new(HirExpression::Variable("ptr".to_string())),
                new_size: Box::new(HirExpression::Variable("new_size".to_string())),
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["ptr"].escapes,
        "Pointer in realloc should be detected as escaping"
    );
    assert!(
        lifetimes["new_size"].escapes,
        "new_size in realloc should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_in_string_method_call() {
    // Test that StringMethodCall properly detects variable usage
    let func = HirFunction::new_with_body(
        "string_method_test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "s".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Char)),
                initializer: Some(HirExpression::StringLiteral("hello".to_string())),
            },
            HirStatement::Return(Some(HirExpression::StringMethodCall {
                receiver: Box::new(HirExpression::Variable("s".to_string())),
                method: "len".to_string(),
                arguments: vec![],
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["s"].escapes,
        "Variable in string method call should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_in_string_method_call_args() {
    // Test that StringMethodCall detects variable usage in arguments
    let func = HirFunction::new_with_body(
        "string_method_arg_test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "arg".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(5)),
            },
            HirStatement::Return(Some(HirExpression::StringMethodCall {
                receiver: Box::new(HirExpression::StringLiteral("test".to_string())),
                method: "substr".to_string(),
                arguments: vec![HirExpression::Variable("arg".to_string())],
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["arg"].escapes,
        "Variable in string method args should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_in_slice_index() {
    // Test that SliceIndex properly detects variable usage
    let func = HirFunction::new_with_body(
        "slice_index_test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::NullLiteral),
            },
            HirStatement::VariableDeclaration {
                name: "idx".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::Return(Some(HirExpression::SliceIndex {
                slice: Box::new(HirExpression::Variable("arr".to_string())),
                index: Box::new(HirExpression::Variable("idx".to_string())),
                element_type: HirType::Int,
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["arr"].escapes,
        "Slice variable should be detected as escaping"
    );
    assert!(
        lifetimes["idx"].escapes,
        "Index variable should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_in_post_increment() {
    // Test that PostIncrement properly detects variable usage
    let func = HirFunction::new_with_body(
        "post_inc_test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(5)),
            },
            HirStatement::Return(Some(HirExpression::PostIncrement {
                operand: Box::new(HirExpression::Variable("x".to_string())),
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["x"].escapes,
        "Variable in post increment should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_in_pre_increment() {
    // Test that PreIncrement properly detects variable usage
    let func = HirFunction::new_with_body(
        "pre_inc_test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(5)),
            },
            HirStatement::Return(Some(HirExpression::PreIncrement {
                operand: Box::new(HirExpression::Variable("x".to_string())),
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["x"].escapes,
        "Variable in pre increment should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_in_post_decrement() {
    // Test that PostDecrement properly detects variable usage
    let func = HirFunction::new_with_body(
        "post_dec_test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(5)),
            },
            HirStatement::Return(Some(HirExpression::PostDecrement {
                operand: Box::new(HirExpression::Variable("x".to_string())),
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["x"].escapes,
        "Variable in post decrement should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_in_pre_decrement() {
    // Test that PreDecrement properly detects variable usage
    let func = HirFunction::new_with_body(
        "pre_dec_test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(5)),
            },
            HirStatement::Return(Some(HirExpression::PreDecrement {
                operand: Box::new(HirExpression::Variable("x".to_string())),
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["x"].escapes,
        "Variable in pre decrement should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_in_ternary_condition() {
    // Test that Ternary properly detects variable usage in condition
    let func = HirFunction::new_with_body(
        "ternary_cond_test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "cond".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(1)),
            },
            HirStatement::Return(Some(HirExpression::Ternary {
                condition: Box::new(HirExpression::Variable("cond".to_string())),
                then_expr: Box::new(HirExpression::IntLiteral(10)),
                else_expr: Box::new(HirExpression::IntLiteral(20)),
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["cond"].escapes,
        "Condition variable in ternary should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_in_ternary_then() {
    // Test that Ternary properly detects variable usage in then_expr
    let func = HirFunction::new_with_body(
        "ternary_then_test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "result".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(42)),
            },
            HirStatement::Return(Some(HirExpression::Ternary {
                condition: Box::new(HirExpression::IntLiteral(1)),
                then_expr: Box::new(HirExpression::Variable("result".to_string())),
                else_expr: Box::new(HirExpression::IntLiteral(0)),
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["result"].escapes,
        "Then variable in ternary should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_in_ternary_else() {
    // Test that Ternary properly detects variable usage in else_expr
    let func = HirFunction::new_with_body(
        "ternary_else_test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "fallback".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(99)),
            },
            HirStatement::Return(Some(HirExpression::Ternary {
                condition: Box::new(HirExpression::IntLiteral(0)),
                then_expr: Box::new(HirExpression::IntLiteral(0)),
                else_expr: Box::new(HirExpression::Variable("fallback".to_string())),
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["fallback"].escapes,
        "Else variable in ternary should be detected as escaping"
    );
}

// ============================================================================
// Coverage Improvement Tests - ScopeTree Methods
// ============================================================================

#[test]
fn test_scope_tree_default() {
    let tree: ScopeTree = Default::default();
    assert_eq!(
        tree.scopes().len(),
        1,
        "Default tree should have root scope"
    );
}

#[test]
fn test_lifetime_analyzer_default() {
    let analyzer: LifetimeAnalyzer = Default::default();
    let func = HirFunction::new_with_body("test".to_string(), HirType::Void, vec![], vec![]);
    let tree = analyzer.build_scope_tree(&func);
    assert_eq!(tree.scopes().len(), 1);
}

#[test]
fn test_scope_tree_add_variable_to_nonexistent_scope() {
    let mut tree = ScopeTree::new();
    // Try to add variable to scope that doesn't exist
    tree.add_variable(999, "ghost".to_string());
    // Should not panic, just no-op
    let root = tree.get_scope(0).unwrap();
    assert!(!root.variables.contains(&"ghost".to_string()));
}

#[test]
fn test_scope_tree_get_nonexistent_scope() {
    let tree = ScopeTree::new();
    assert!(tree.get_scope(999).is_none());
}

#[test]
fn test_is_nested_in_same_scope() {
    let tree = ScopeTree::new();
    // A scope is nested in itself
    assert!(tree.is_nested_in(0, 0));
}

#[test]
fn test_is_nested_in_with_child() {
    let mut tree = ScopeTree::new();
    let child_id = tree.add_scope(0, (1, 5));
    // Child is nested in root
    assert!(tree.is_nested_in(child_id, 0));
    // Root is not nested in child
    assert!(!tree.is_nested_in(0, child_id));
}

#[test]
fn test_is_nested_in_independent_branches() {
    let mut tree = ScopeTree::new();
    let child1 = tree.add_scope(0, (1, 5));
    let child2 = tree.add_scope(0, (6, 10));
    // Sibling scopes are not nested in each other
    assert!(!tree.is_nested_in(child1, child2));
    assert!(!tree.is_nested_in(child2, child1));
}

#[test]
fn test_no_dangling_for_function_scope_variable() {
    // Variable declared at function scope that escapes should NOT be flagged as dangling
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "result".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(42)),
            },
            HirStatement::Return(Some(HirExpression::Variable("result".to_string()))),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);
    let dangling = analyzer.detect_dangling_pointers(&lifetimes);

    // result is in scope 0 (function scope) so should not be dangling
    assert!(!dangling.contains(&"result".to_string()));
}

#[test]
fn test_variable_lifetime_fields() {
    let lifetime = VariableLifetime {
        name: "test".to_string(),
        declared_in_scope: 1,
        first_use: 5,
        last_use: 10,
        escapes: true,
    };

    assert_eq!(lifetime.name, "test");
    assert_eq!(lifetime.declared_in_scope, 1);
    assert_eq!(lifetime.first_use, 5);
    assert_eq!(lifetime.last_use, 10);
    assert!(lifetime.escapes);
}

#[test]
fn test_scope_fields() {
    let scope = Scope {
        id: 42,
        parent: Some(0),
        variables: vec!["x".to_string(), "y".to_string()],
        statement_range: (10, 20),
    };

    assert_eq!(scope.id, 42);
    assert_eq!(scope.parent, Some(0));
    assert_eq!(scope.variables.len(), 2);
    assert_eq!(scope.statement_range, (10, 20));
}

#[test]
fn test_lifetime_relation_variants() {
    // Test all variants can be compared
    assert_eq!(LifetimeRelation::Equal, LifetimeRelation::Equal);
    assert_eq!(LifetimeRelation::Outlives, LifetimeRelation::Outlives);
    assert_eq!(LifetimeRelation::OutlivedBy, LifetimeRelation::OutlivedBy);
    assert_eq!(LifetimeRelation::Independent, LifetimeRelation::Independent);

    // Test that different variants are not equal
    assert_ne!(LifetimeRelation::Equal, LifetimeRelation::Outlives);
}

#[test]
fn test_same_scope_variables_relationship() {
    // Variables declared in same scope should have some relationship inferred
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "a".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(1)),
            },
            HirStatement::VariableDeclaration {
                name: "b".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(2)),
            },
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);
    let relationships = analyzer.infer_lifetime_relationships(&lifetimes, &scope_tree);

    // Both a and b should be tracked
    assert!(lifetimes.contains_key("a"));
    assert!(lifetimes.contains_key("b"));

    // Relationship should exist (check either direction)
    let has_rel = relationships.contains_key(&("a".to_string(), "b".to_string()))
        || relationships.contains_key(&("b".to_string(), "a".to_string()));
    assert!(has_rel, "Should have relationship between a and b");
}
