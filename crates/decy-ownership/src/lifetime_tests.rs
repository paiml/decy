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

// ============================================================================
// Additional Coverage Tests - Targeting 55+ uncovered lines
// ============================================================================

#[test]
fn test_expression_uses_variable_binary_op_in_return() {
    // Cover BinaryOp branch in expression_uses_variable via return
    let func = HirFunction::new_with_body(
        "binop_test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "a".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(10)),
            },
            HirStatement::Return(Some(HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::IntLiteral(5)),
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["a"].escapes,
        "Variable in binary op return should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_binary_op_right_side() {
    // Cover BinaryOp right-side branch in expression_uses_variable
    let func = HirFunction::new_with_body(
        "binop_right_test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "b".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(20)),
            },
            HirStatement::Return(Some(HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Multiply,
                left: Box::new(HirExpression::IntLiteral(3)),
                right: Box::new(HirExpression::Variable("b".to_string())),
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["b"].escapes,
        "Variable on right side of binary op should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_dereference_in_return() {
    // Cover Dereference branch in expression_uses_variable via return
    let func = HirFunction::new_with_body(
        "deref_test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "ptr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::NullLiteral),
            },
            HirStatement::Return(Some(HirExpression::Dereference(Box::new(
                HirExpression::Variable("ptr".to_string()),
            )))),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["ptr"].escapes,
        "Variable in dereference should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_address_of_in_return() {
    // Cover AddressOf branch in expression_uses_variable via return
    let func = HirFunction::new_with_body(
        "addr_test".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(42)),
            },
            HirStatement::Return(Some(HirExpression::AddressOf(Box::new(
                HirExpression::Variable("x".to_string()),
            )))),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["x"].escapes,
        "Variable in address-of should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_unary_op_in_return() {
    // Cover UnaryOp branch in expression_uses_variable via return
    let func = HirFunction::new_with_body(
        "unary_test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "n".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(5)),
            },
            HirStatement::Return(Some(HirExpression::UnaryOp {
                op: decy_hir::UnaryOperator::Minus,
                operand: Box::new(HirExpression::Variable("n".to_string())),
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["n"].escapes,
        "Variable in unary op should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_function_call_in_return() {
    // Cover FunctionCall arguments branch in expression_uses_variable via return
    let func = HirFunction::new_with_body(
        "funcall_test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "val".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(7)),
            },
            HirStatement::Return(Some(HirExpression::FunctionCall {
                function: "compute".to_string(),
                arguments: vec![HirExpression::Variable("val".to_string())],
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["val"].escapes,
        "Variable in function call args should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_field_access_in_return() {
    // Cover FieldAccess branch in expression_uses_variable via return
    let func = HirFunction::new_with_body(
        "field_test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "obj".to_string(),
                var_type: HirType::Struct("Point".to_string()),
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::Return(Some(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("obj".to_string())),
                field: "x".to_string(),
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["obj"].escapes,
        "Variable in field access should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_pointer_field_access_in_return() {
    // Cover PointerFieldAccess branch in expression_uses_variable via return
    let func = HirFunction::new_with_body(
        "ptr_field_test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "ptr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
                initializer: Some(HirExpression::NullLiteral),
            },
            HirStatement::Return(Some(HirExpression::PointerFieldAccess {
                pointer: Box::new(HirExpression::Variable("ptr".to_string())),
                field: "value".to_string(),
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["ptr"].escapes,
        "Variable in pointer field access should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_array_index_in_return() {
    // Cover ArrayIndex branch in expression_uses_variable via return
    let func = HirFunction::new_with_body(
        "arr_idx_test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::NullLiteral),
            },
            HirStatement::Return(Some(HirExpression::ArrayIndex {
                array: Box::new(HirExpression::Variable("arr".to_string())),
                index: Box::new(HirExpression::IntLiteral(0)),
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["arr"].escapes,
        "Variable in array index should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_array_index_variable_in_return() {
    // Cover ArrayIndex index branch (the index expression uses the variable)
    let func = HirFunction::new_with_body(
        "arr_idx_var_test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "idx".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(2)),
            },
            HirStatement::Return(Some(HirExpression::ArrayIndex {
                array: Box::new(HirExpression::NullLiteral),
                index: Box::new(HirExpression::Variable("idx".to_string())),
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        lifetimes["idx"].escapes,
        "Variable used as array index should be detected as escaping"
    );
}

#[test]
fn test_expression_uses_variable_literal_returns_false() {
    // Cover literal branches: IntLiteral, FloatLiteral, StringLiteral, CharLiteral, NullLiteral, Sizeof
    // A variable NOT used in any of these expressions should NOT escape
    let func = HirFunction::new_with_body(
        "literal_test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "unused".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(100)),
            },
            HirStatement::Return(Some(HirExpression::IntLiteral(42))),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        !lifetimes["unused"].escapes,
        "Variable not in return expression should not escape"
    );
}

#[test]
fn test_check_if_escapes_no_return() {
    // Cover check_if_escapes with Return(None) and non-Return statements
    let func = HirFunction::new_with_body(
        "no_return_test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(5)),
            },
            HirStatement::Return(None),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        !lifetimes["x"].escapes,
        "Variable should not escape when Return(None)"
    );
}

#[test]
fn test_is_nested_in_nonexistent_scope() {
    // Cover the None branch in is_nested_in when scope doesn't exist
    let tree = ScopeTree::new();
    // Checking a non-existent inner scope should return false
    assert!(!tree.is_nested_in(999, 0));
}

#[test]
fn test_analyze_statements_other_statement_types() {
    // Cover the _ => { index += 1; } branch in analyze_statements
    // Use statements that don't match VariableDeclaration, If, or While
    let func = HirFunction::new_with_body(
        "misc_stmts".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::Break,
            HirStatement::Continue,
            HirStatement::Expression(HirExpression::IntLiteral(42)),
            HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::IntLiteral(0),
            },
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);

    // Should still have root scope, no crashes
    assert_eq!(scope_tree.scopes().len(), 1);
}

#[test]
fn test_compound_literal_no_variable_usage() {
    // Cover CompoundLiteral branch returning false when variable not present
    let func = HirFunction::new_with_body(
        "compound_no_var".to_string(),
        HirType::Struct("Point".to_string()),
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "unused".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::Return(Some(HirExpression::CompoundLiteral {
                literal_type: HirType::Struct("Point".to_string()),
                initializers: vec![
                    HirExpression::IntLiteral(1),
                    HirExpression::IntLiteral(2),
                ],
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        !lifetimes["unused"].escapes,
        "Variable not in compound literal should not escape"
    );
}

#[test]
fn test_function_call_no_matching_variable_args() {
    // Cover FunctionCall branch when variable is not among arguments
    let func = HirFunction::new_with_body(
        "call_no_match".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "v".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::Return(Some(HirExpression::FunctionCall {
                function: "foo".to_string(),
                arguments: vec![HirExpression::IntLiteral(99)],
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        !lifetimes["v"].escapes,
        "Variable not in function call args should not escape"
    );
}

#[test]
fn test_track_lifetimes_empty_function() {
    // Cover empty function body with no variables
    let func = HirFunction::new_with_body(
        "empty".to_string(),
        HirType::Void,
        vec![],
        vec![],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(lifetimes.is_empty(), "Empty function should have no lifetimes");
}

#[test]
fn test_detect_dangling_pointers_empty_lifetimes() {
    // Cover detect_dangling_pointers with empty map
    let analyzer = LifetimeAnalyzer::new();
    let lifetimes = HashMap::new();
    let dangling = analyzer.detect_dangling_pointers(&lifetimes);
    assert!(dangling.is_empty());
}

#[test]
fn test_detect_dangling_pointers_non_escaping_nested() {
    // Variable in nested scope but does NOT escape should NOT be flagged
    let mut lifetimes = HashMap::new();
    lifetimes.insert(
        "x".to_string(),
        VariableLifetime {
            name: "x".to_string(),
            declared_in_scope: 2,
            first_use: 3,
            last_use: 5,
            escapes: false,
        },
    );

    let analyzer = LifetimeAnalyzer::new();
    let dangling = analyzer.detect_dangling_pointers(&lifetimes);
    assert!(dangling.is_empty(), "Non-escaping variable should not be flagged");
}

#[test]
fn test_infer_lifetime_relationships_empty_lifetimes() {
    // Cover empty lifetimes map in infer_lifetime_relationships
    let analyzer = LifetimeAnalyzer::new();
    let tree = ScopeTree::new();
    let lifetimes = HashMap::new();
    let relationships = analyzer.infer_lifetime_relationships(&lifetimes, &tree);
    assert!(relationships.is_empty());
}

#[test]
fn test_infer_lifetime_relationships_single_variable() {
    // Single variable should produce no pairs
    let analyzer = LifetimeAnalyzer::new();
    let tree = ScopeTree::new();
    let mut lifetimes = HashMap::new();
    lifetimes.insert(
        "only".to_string(),
        VariableLifetime {
            name: "only".to_string(),
            declared_in_scope: 0,
            first_use: 0,
            last_use: 5,
            escapes: false,
        },
    );
    let relationships = analyzer.infer_lifetime_relationships(&lifetimes, &tree);
    assert!(relationships.is_empty(), "Single variable should have no relationships");
}

#[test]
fn test_compare_lifetimes_outlived_by() {
    // Test the OutlivedBy branch: scope1 is nested in scope2
    let mut tree = ScopeTree::new();
    let inner_scope = tree.add_scope(0, (1, 5));
    tree.add_variable(0, "outer".to_string());
    tree.add_variable(inner_scope, "inner".to_string());

    let analyzer = LifetimeAnalyzer::new();
    let mut lifetimes = HashMap::new();
    lifetimes.insert(
        "inner".to_string(),
        VariableLifetime {
            name: "inner".to_string(),
            declared_in_scope: inner_scope,
            first_use: 1,
            last_use: 5,
            escapes: false,
        },
    );
    lifetimes.insert(
        "outer".to_string(),
        VariableLifetime {
            name: "outer".to_string(),
            declared_in_scope: 0,
            first_use: 0,
            last_use: 10,
            escapes: false,
        },
    );

    let relationships = analyzer.infer_lifetime_relationships(&lifetimes, &tree);

    // One of the pairs should show Outlives or OutlivedBy
    let has_outlives_or_outlived_by = relationships.values().any(|r| {
        matches!(r, LifetimeRelation::Outlives | LifetimeRelation::OutlivedBy)
    });
    assert!(has_outlives_or_outlived_by, "Should detect Outlives/OutlivedBy relationship");
}

#[test]
fn test_while_loop_with_nested_if_and_variables() {
    // Cover complex nesting: while containing if with else
    let func = HirFunction::new_with_body(
        "complex_loop".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::While {
                condition: HirExpression::IntLiteral(1),
                body: vec![
                    HirStatement::VariableDeclaration {
                        name: "loop_var".to_string(),
                        var_type: HirType::Int,
                        initializer: Some(HirExpression::IntLiteral(0)),
                    },
                    HirStatement::If {
                        condition: HirExpression::Variable("loop_var".to_string()),
                        then_block: vec![
                            HirStatement::VariableDeclaration {
                                name: "then_v".to_string(),
                                var_type: HirType::Int,
                                initializer: Some(HirExpression::IntLiteral(1)),
                            },
                        ],
                        else_block: Some(vec![
                            HirStatement::VariableDeclaration {
                                name: "else_v".to_string(),
                                var_type: HirType::Int,
                                initializer: Some(HirExpression::IntLiteral(2)),
                            },
                        ]),
                    },
                ],
            },
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);

    // Should have: root, while-body, then-block, else-block = 4 scopes
    assert!(
        scope_tree.scopes().len() >= 4,
        "Should have scopes for root, while, then, and else. Got {}",
        scope_tree.scopes().len()
    );
}

#[test]
fn test_deeply_nested_scopes_is_nested_in() {
    // Cover multi-level ancestor chain traversal in is_nested_in
    let mut tree = ScopeTree::new();
    let level1 = tree.add_scope(0, (1, 10));
    let level2 = tree.add_scope(level1, (2, 8));
    let level3 = tree.add_scope(level2, (3, 6));

    // level3 is nested in level1 (grandchild)
    assert!(tree.is_nested_in(level3, level1));
    // level3 is nested in root
    assert!(tree.is_nested_in(level3, 0));
    // root is NOT nested in level3
    assert!(!tree.is_nested_in(0, level3));
    // level1 is NOT nested in level3
    assert!(!tree.is_nested_in(level1, level3));
}

#[test]
fn test_string_method_call_no_match() {
    // Cover StringMethodCall branch when variable is NOT in receiver or args
    let func = HirFunction::new_with_body(
        "str_no_match".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "unused".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::Return(Some(HirExpression::StringMethodCall {
                receiver: Box::new(HirExpression::StringLiteral("test".to_string())),
                method: "len".to_string(),
                arguments: vec![HirExpression::IntLiteral(0)],
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        !lifetimes["unused"].escapes,
        "Variable not in string method call should not escape"
    );
}

#[test]
fn test_realloc_variable_not_present() {
    // Cover Realloc branch when variable is NOT in pointer or new_size
    let func = HirFunction::new_with_body(
        "realloc_no_var".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "unused".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::Return(Some(HirExpression::Realloc {
                pointer: Box::new(HirExpression::NullLiteral),
                new_size: Box::new(HirExpression::IntLiteral(100)),
            })),
        ],
    );

    let analyzer = LifetimeAnalyzer::new();
    let scope_tree = analyzer.build_scope_tree(&func);
    let lifetimes = analyzer.track_lifetimes(&func, &scope_tree);

    assert!(
        !lifetimes["unused"].escapes,
        "Variable not in realloc should not escape"
    );
}
