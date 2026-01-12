//! Tests for void* usage pattern analysis (DECY-095).
//!
//! Analyzes void* usage to infer generic type parameters for
//! transformation to Rust generics.

use decy_analyzer::void_ptr_analysis::{TypeConstraint, VoidPtrAnalyzer, VoidPtrPattern};
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType};

/// Helper: Create test function
fn create_function(name: &str, params: Vec<HirParameter>, body: Vec<HirStatement>) -> HirFunction {
    HirFunction::new_with_body(name.to_string(), HirType::Void, params, body)
}

/// Helper: Create void* parameter
fn void_ptr_param(name: &str) -> HirParameter {
    HirParameter::new(name.to_string(), HirType::Pointer(Box::new(HirType::Void)))
}

// ============================================================================
// TEST 1: Detect void* parameter
// ============================================================================

#[test]
fn test_detect_void_ptr_parameter() {
    // void process(void* data) { ... }
    let func = create_function("process", vec![void_ptr_param("data")], vec![]);

    let analyzer = VoidPtrAnalyzer::new();
    let patterns = analyzer.analyze(&func);

    assert!(!patterns.is_empty(), "Should detect void* parameter");
    assert_eq!(patterns[0].param_name, "data");
}

// ============================================================================
// TEST 2: Detect cast to specific type
// ============================================================================

#[test]
fn test_detect_cast_to_type() {
    // void process(void* data) { int* p = (int*)data; }
    let func = create_function(
        "process",
        vec![void_ptr_param("data")],
        vec![HirStatement::VariableDeclaration {
            name: "p".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::Cast {
                expr: Box::new(HirExpression::Variable("data".to_string())),
                target_type: HirType::Pointer(Box::new(HirType::Int)),
            }),
        }],
    );

    let analyzer = VoidPtrAnalyzer::new();
    let patterns = analyzer.analyze(&func);

    assert!(!patterns.is_empty());
    assert!(
        patterns[0].inferred_types.contains(&HirType::Int),
        "Should infer int from cast"
    );
}

// ============================================================================
// TEST 3: Detect swap pattern
// ============================================================================

#[test]
fn test_detect_swap_pattern() {
    // void swap(void* a, void* b, size_t size) - classic generic swap
    let func = create_function(
        "swap",
        vec![
            void_ptr_param("a"),
            void_ptr_param("b"),
            HirParameter::new("size".to_string(), HirType::Int), // size_t → i32
        ],
        vec![],
    );

    let analyzer = VoidPtrAnalyzer::new();
    let patterns = analyzer.analyze(&func);

    // Should detect both a and b as void* params
    assert!(patterns.len() >= 2, "Should detect multiple void* params");

    // Should recognize swap pattern (two void* + size_t)
    assert!(
        patterns.iter().any(|p| p.pattern == VoidPtrPattern::Swap),
        "Should detect swap pattern"
    );
}

// ============================================================================
// TEST 4: Detect compare callback pattern
// ============================================================================

#[test]
fn test_detect_compare_pattern() {
    // int compare(const void* a, const void* b) - qsort-style comparator
    let func = HirFunction::new_with_body(
        "compare".to_string(),
        HirType::Int,
        vec![void_ptr_param("a"), void_ptr_param("b")],
        vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
    );

    let analyzer = VoidPtrAnalyzer::new();
    let patterns = analyzer.analyze(&func);

    assert!(
        patterns
            .iter()
            .any(|p| p.pattern == VoidPtrPattern::Compare),
        "Should detect compare pattern"
    );
}

// ============================================================================
// TEST 5: Infer constraint from dereference
// ============================================================================

#[test]
fn test_infer_constraint_from_deref() {
    // void process(void* data) { *(int*)data = 42; }
    let func = create_function(
        "process",
        vec![void_ptr_param("data")],
        vec![HirStatement::DerefAssignment {
            target: HirExpression::Cast {
                expr: Box::new(HirExpression::Variable("data".to_string())),
                target_type: HirType::Pointer(Box::new(HirType::Int)),
            },
            value: HirExpression::IntLiteral(42),
        }],
    );

    let analyzer = VoidPtrAnalyzer::new();
    let patterns = analyzer.analyze(&func);

    assert!(!patterns.is_empty());
    // Should infer that T must be assignable (mutable)
    assert!(
        patterns[0].constraints.contains(&TypeConstraint::Mutable),
        "Should infer mutable constraint from write"
    );
}

// ============================================================================
// TEST 6: Multiple void* with different uses
// ============================================================================

#[test]
fn test_multiple_void_ptr_different_types() {
    // void copy(void* dest, void* src, size_t n)
    // where dest is written, src is read
    let func = create_function(
        "copy",
        vec![
            void_ptr_param("dest"),
            void_ptr_param("src"),
            HirParameter::new("n".to_string(), HirType::Int), // size_t → i32
        ],
        vec![],
    );

    let analyzer = VoidPtrAnalyzer::new();
    let patterns = analyzer.analyze(&func);

    let dest_pattern = patterns.iter().find(|p| p.param_name == "dest");
    let src_pattern = patterns.iter().find(|p| p.param_name == "src");

    assert!(dest_pattern.is_some(), "Should have dest pattern");
    assert!(src_pattern.is_some(), "Should have src pattern");
}

// ============================================================================
// TEST 7: No void* - empty result
// ============================================================================

#[test]
fn test_no_void_ptr_empty_result() {
    let func = create_function(
        "add",
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
        vec![],
    );

    let analyzer = VoidPtrAnalyzer::new();
    let patterns = analyzer.analyze(&func);

    assert!(
        patterns.is_empty(),
        "Should be empty for non-void* functions"
    );
}

// ============================================================================
// TEST 8: Detect cmp function pattern
// ============================================================================

#[test]
fn test_detect_cmp_function_pattern() {
    // int my_cmp(const void* a, const void* b) - function with "cmp" in name
    let func = HirFunction::new_with_body(
        "my_cmp".to_string(),
        HirType::Int,
        vec![void_ptr_param("a"), void_ptr_param("b")],
        vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
    );

    let analyzer = VoidPtrAnalyzer::new();
    let patterns = analyzer.analyze(&func);

    assert!(
        patterns
            .iter()
            .any(|p| p.pattern == VoidPtrPattern::Compare),
        "Should detect compare pattern for cmp function"
    );
}

// ============================================================================
// TEST 9: Detect copy pattern
// ============================================================================

#[test]
fn test_detect_copy_pattern() {
    // void memcopy(void* dest, void* src, int size)
    let func = create_function(
        "memcopy",
        vec![
            void_ptr_param("dest"),
            void_ptr_param("src"),
            HirParameter::new("size".to_string(), HirType::Int),
        ],
        vec![],
    );

    let analyzer = VoidPtrAnalyzer::new();
    let patterns = analyzer.analyze(&func);

    assert!(
        patterns.iter().any(|p| p.pattern == VoidPtrPattern::Copy),
        "Should detect copy pattern for dest/src params"
    );
}

// ============================================================================
// TEST 10: Detect PartialOrd constraint from comparison
// ============================================================================

#[test]
fn test_detect_partial_ord_constraint() {
    // void process(void* data) { if (*(int*)data < 10) ... }
    let func = create_function(
        "process",
        vec![void_ptr_param("data")],
        vec![HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::LessThan,
                left: Box::new(HirExpression::Dereference(Box::new(HirExpression::Cast {
                    expr: Box::new(HirExpression::Variable("data".to_string())),
                    target_type: HirType::Pointer(Box::new(HirType::Int)),
                }))),
                right: Box::new(HirExpression::IntLiteral(10)),
            },
            then_block: vec![],
            else_block: None,
        }],
    );

    let analyzer = VoidPtrAnalyzer::new();
    let patterns = analyzer.analyze(&func);

    assert!(!patterns.is_empty());
    assert!(
        patterns[0]
            .constraints
            .contains(&TypeConstraint::PartialOrd),
        "Should infer PartialOrd from < comparison"
    );
}

// ============================================================================
// TEST 11: Detect PartialEq constraint from equality
// ============================================================================

#[test]
fn test_detect_partial_eq_constraint() {
    // void process(void* data) { if (*(int*)data == 0) ... }
    let func = create_function(
        "process",
        vec![void_ptr_param("data")],
        vec![HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Equal,
                left: Box::new(HirExpression::Dereference(Box::new(HirExpression::Cast {
                    expr: Box::new(HirExpression::Variable("data".to_string())),
                    target_type: HirType::Pointer(Box::new(HirType::Int)),
                }))),
                right: Box::new(HirExpression::IntLiteral(0)),
            },
            then_block: vec![],
            else_block: None,
        }],
    );

    let analyzer = VoidPtrAnalyzer::new();
    let patterns = analyzer.analyze(&func);

    assert!(!patterns.is_empty());
    assert!(
        patterns[0].constraints.contains(&TypeConstraint::PartialEq),
        "Should infer PartialEq from == comparison"
    );
}

// ============================================================================
// TEST 12: Clone constraint from deref assignment with deref value
// ============================================================================

#[test]
fn test_detect_clone_constraint() {
    // void copy_val(void* dest, void* src) { *(int*)dest = *(int*)src; }
    let func = create_function(
        "copy_val",
        vec![void_ptr_param("dest"), void_ptr_param("src")],
        vec![HirStatement::DerefAssignment {
            target: HirExpression::Cast {
                expr: Box::new(HirExpression::Variable("dest".to_string())),
                target_type: HirType::Pointer(Box::new(HirType::Int)),
            },
            value: HirExpression::Dereference(Box::new(HirExpression::Cast {
                expr: Box::new(HirExpression::Variable("src".to_string())),
                target_type: HirType::Pointer(Box::new(HirType::Int)),
            })),
        }],
    );

    let analyzer = VoidPtrAnalyzer::new();
    let patterns = analyzer.analyze(&func);

    let dest_info = patterns.iter().find(|p| p.param_name == "dest");
    assert!(dest_info.is_some());
    assert!(
        dest_info
            .unwrap()
            .constraints
            .contains(&TypeConstraint::Clone),
        "Should infer Clone constraint from copying through deref"
    );
}

// ============================================================================
// TEST 13: While loop analysis
// ============================================================================

#[test]
fn test_while_loop_analysis() {
    // void process(void* data) { while (*data) { ... } }
    let func = create_function(
        "process",
        vec![void_ptr_param("data")],
        vec![HirStatement::While {
            condition: HirExpression::Dereference(Box::new(HirExpression::Cast {
                expr: Box::new(HirExpression::Variable("data".to_string())),
                target_type: HirType::Pointer(Box::new(HirType::Int)),
            })),
            body: vec![],
        }],
    );

    let analyzer = VoidPtrAnalyzer::new();
    let patterns = analyzer.analyze(&func);

    assert!(!patterns.is_empty());
    assert!(
        patterns[0].inferred_types.contains(&HirType::Int),
        "Should infer type from cast in while condition"
    );
}

// ============================================================================
// TEST 14: For loop analysis
// ============================================================================

#[test]
fn test_for_loop_analysis() {
    let func = create_function(
        "process",
        vec![void_ptr_param("data")],
        vec![HirStatement::For {
            init: vec![],
            condition: HirExpression::IntLiteral(1),
            increment: vec![],
            body: vec![HirStatement::VariableDeclaration {
                name: "val".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::Cast {
                    expr: Box::new(HirExpression::Variable("data".to_string())),
                    target_type: HirType::Pointer(Box::new(HirType::Float)),
                }),
            }],
        }],
    );

    let analyzer = VoidPtrAnalyzer::new();
    let patterns = analyzer.analyze(&func);

    assert!(!patterns.is_empty());
    assert!(
        patterns[0].inferred_types.contains(&HirType::Float),
        "Should infer type from cast in for body"
    );
}

// ============================================================================
// TEST 15: Expression statement analysis
// ============================================================================

#[test]
fn test_expression_statement_analysis() {
    let func = create_function(
        "process",
        vec![void_ptr_param("data")],
        vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "use_data".to_string(),
            arguments: vec![HirExpression::Cast {
                expr: Box::new(HirExpression::Variable("data".to_string())),
                target_type: HirType::Pointer(Box::new(HirType::Double)),
            }],
        })],
    );

    let analyzer = VoidPtrAnalyzer::new();
    let patterns = analyzer.analyze(&func);

    assert!(!patterns.is_empty());
    assert!(
        patterns[0].inferred_types.contains(&HirType::Double),
        "Should infer type from cast in function call"
    );
}

// ============================================================================
// TEST 16: If-else analysis
// ============================================================================

#[test]
fn test_if_else_analysis() {
    let func = create_function(
        "process",
        vec![void_ptr_param("data")],
        vec![HirStatement::If {
            condition: HirExpression::IntLiteral(1),
            then_block: vec![HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::Cast {
                    expr: Box::new(HirExpression::Variable("data".to_string())),
                    target_type: HirType::Pointer(Box::new(HirType::Char)),
                }),
            }],
            else_block: Some(vec![HirStatement::VariableDeclaration {
                name: "y".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::Cast {
                    expr: Box::new(HirExpression::Variable("data".to_string())),
                    target_type: HirType::Pointer(Box::new(HirType::Int)),
                }),
            }]),
        }],
    );

    let analyzer = VoidPtrAnalyzer::new();
    let patterns = analyzer.analyze(&func);

    assert!(!patterns.is_empty());
    // Should find types from both branches
    assert!(
        patterns[0].inferred_types.contains(&HirType::Char)
            || patterns[0].inferred_types.contains(&HirType::Int),
        "Should infer types from both if branches"
    );
}

// ============================================================================
// TEST 17: Return statement analysis
// ============================================================================

#[test]
fn test_return_statement_analysis() {
    let func = HirFunction::new_with_body(
        "get_data".to_string(),
        HirType::Int,
        vec![void_ptr_param("data")],
        vec![HirStatement::Return(Some(HirExpression::Dereference(
            Box::new(HirExpression::Cast {
                expr: Box::new(HirExpression::Variable("data".to_string())),
                target_type: HirType::Pointer(Box::new(HirType::Int)),
            }),
        )))],
    );

    let analyzer = VoidPtrAnalyzer::new();
    let patterns = analyzer.analyze(&func);

    assert!(!patterns.is_empty());
    assert!(
        patterns[0].inferred_types.contains(&HirType::Int),
        "Should infer type from cast in return"
    );
}

// ============================================================================
// TEST 18: Greater than comparison
// ============================================================================

#[test]
fn test_greater_than_constraint() {
    let func = create_function(
        "process",
        vec![void_ptr_param("data")],
        vec![HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::GreaterThan,
                left: Box::new(HirExpression::Variable("data".to_string())),
                right: Box::new(HirExpression::IntLiteral(10)),
            },
            then_block: vec![],
            else_block: None,
        }],
    );

    let analyzer = VoidPtrAnalyzer::new();
    let patterns = analyzer.analyze(&func);

    assert!(!patterns.is_empty());
    assert!(
        patterns[0]
            .constraints
            .contains(&TypeConstraint::PartialOrd),
        "Should infer PartialOrd from > comparison"
    );
}

// ============================================================================
// TEST 19: NotEqual comparison
// ============================================================================

#[test]
fn test_not_equal_constraint() {
    let func = create_function(
        "process",
        vec![void_ptr_param("data")],
        vec![HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::NotEqual,
                left: Box::new(HirExpression::Variable("data".to_string())),
                right: Box::new(HirExpression::IntLiteral(0)),
            },
            then_block: vec![],
            else_block: None,
        }],
    );

    let analyzer = VoidPtrAnalyzer::new();
    let patterns = analyzer.analyze(&func);

    assert!(!patterns.is_empty());
    assert!(
        patterns[0].constraints.contains(&TypeConstraint::PartialEq),
        "Should infer PartialEq from != comparison"
    );
}

// ============================================================================
// TEST 20: LessEqual comparison
// ============================================================================

#[test]
fn test_less_equal_constraint() {
    let func = create_function(
        "process",
        vec![void_ptr_param("data")],
        vec![HirStatement::While {
            condition: HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::LessEqual,
                left: Box::new(HirExpression::Variable("data".to_string())),
                right: Box::new(HirExpression::IntLiteral(100)),
            },
            body: vec![],
        }],
    );

    let analyzer = VoidPtrAnalyzer::new();
    let patterns = analyzer.analyze(&func);

    assert!(!patterns.is_empty());
    assert!(
        patterns[0]
            .constraints
            .contains(&TypeConstraint::PartialOrd),
        "Should infer PartialOrd from <= comparison"
    );
}

// ============================================================================
// TEST 21: GreaterEqual comparison
// ============================================================================

#[test]
fn test_greater_equal_constraint() {
    let func = create_function(
        "process",
        vec![void_ptr_param("data")],
        vec![HirStatement::While {
            condition: HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::GreaterEqual,
                left: Box::new(HirExpression::Variable("data".to_string())),
                right: Box::new(HirExpression::IntLiteral(0)),
            },
            body: vec![],
        }],
    );

    let analyzer = VoidPtrAnalyzer::new();
    let patterns = analyzer.analyze(&func);

    assert!(!patterns.is_empty());
    assert!(
        patterns[0]
            .constraints
            .contains(&TypeConstraint::PartialOrd),
        "Should infer PartialOrd from >= comparison"
    );
}

// ============================================================================
// TEST 22: Generic pattern (fallback)
// ============================================================================

#[test]
fn test_generic_pattern_fallback() {
    // Function that doesn't match specific patterns
    let func = create_function("process_data", vec![void_ptr_param("ptr")], vec![]);

    let analyzer = VoidPtrAnalyzer::new();
    let patterns = analyzer.analyze(&func);

    assert!(!patterns.is_empty());
    assert_eq!(
        patterns[0].pattern,
        VoidPtrPattern::Generic,
        "Should fallback to Generic pattern"
    );
}

// ============================================================================
// TEST 23: VoidPtrInfo fields
// ============================================================================

#[test]
fn test_void_ptr_info_debug() {
    use decy_analyzer::void_ptr_analysis::VoidPtrInfo;

    let info = VoidPtrInfo {
        param_name: "test".to_string(),
        pattern: VoidPtrPattern::Generic,
        inferred_types: vec![HirType::Int],
        constraints: vec![TypeConstraint::Mutable],
    };

    let debug = format!("{:?}", info);
    assert!(debug.contains("test"));
    assert!(debug.contains("Generic"));
}

// ============================================================================
// TEST 24: Pattern enum traits
// ============================================================================

#[test]
fn test_void_ptr_pattern_clone_eq() {
    let p1 = VoidPtrPattern::Swap;
    let p2 = p1.clone();
    assert_eq!(p1, p2);

    let p3 = VoidPtrPattern::Compare;
    assert_ne!(p1, p3);
}

#[test]
fn test_type_constraint_clone_eq() {
    let c1 = TypeConstraint::Readable;
    let c2 = c1.clone();
    assert_eq!(c1, c2);

    let c3 = TypeConstraint::Mutable;
    assert_ne!(c1, c3);
}
