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
