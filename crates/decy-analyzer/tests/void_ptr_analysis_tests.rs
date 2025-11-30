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
