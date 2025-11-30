//! Tests for generic function signature generation (DECY-096).
//!
//! Transforms void* functions to generic fn<T>.

use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType};

/// Helper: Create void* parameter
fn void_ptr_param(name: &str) -> HirParameter {
    HirParameter::new(
        name.to_string(),
        HirType::Pointer(Box::new(HirType::Void)),
    )
}

/// Helper: Create function with void* params
fn create_void_ptr_function(
    name: &str,
    params: Vec<HirParameter>,
    body: Vec<HirStatement>,
) -> HirFunction {
    HirFunction::new_with_body(name.to_string(), HirType::Void, params, body)
}

// ============================================================================
// TEST 1: Single void* becomes generic T
// ============================================================================

#[test]
fn test_single_void_ptr_becomes_generic() {
    // void process(void* data) → fn process<T>(data: &T)
    let func = create_void_ptr_function(
        "process",
        vec![void_ptr_param("data")],
        vec![],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    assert!(
        code.contains("<T>"),
        "Should generate generic type parameter:\n{}",
        code
    );
    assert!(
        code.contains("&T") || code.contains("&mut T"),
        "Should replace void* with reference to T:\n{}",
        code
    );
}

// ============================================================================
// TEST 2: Two void* params share same generic T
// ============================================================================

#[test]
fn test_two_void_ptr_same_generic() {
    // void swap(void* a, void* b) → fn swap<T>(a: &mut T, b: &mut T)
    let func = create_void_ptr_function(
        "swap",
        vec![void_ptr_param("a"), void_ptr_param("b")],
        vec![],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    assert!(
        code.contains("<T>"),
        "Should have single generic T:\n{}",
        code
    );
    // Both params should use same T
    let t_count = code.matches("&T").count() + code.matches("&mut T").count();
    assert!(
        t_count >= 2,
        "Both params should use T:\n{}",
        code
    );
}

// ============================================================================
// TEST 3: void* with write becomes &mut T
// ============================================================================

#[test]
fn test_void_ptr_write_becomes_mut_ref() {
    // void set(void* data) { *(int*)data = 42; } → fn set<T>(data: &mut T)
    let func = create_void_ptr_function(
        "set_value",
        vec![void_ptr_param("data")],
        vec![HirStatement::DerefAssignment {
            target: HirExpression::Cast {
                expr: Box::new(HirExpression::Variable("data".to_string())),
                target_type: HirType::Pointer(Box::new(HirType::Int)),
            },
            value: HirExpression::IntLiteral(42),
        }],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    assert!(
        code.contains("&mut T"),
        "Should generate &mut T for written param:\n{}",
        code
    );
}

// ============================================================================
// TEST 4: Read-only void* becomes &T
// ============================================================================

#[test]
fn test_void_ptr_read_becomes_ref() {
    // int get(void* data) { return *(int*)data; } → fn get<T>(data: &T) -> i32
    let func = HirFunction::new_with_body(
        "get_value".to_string(),
        HirType::Int,
        vec![void_ptr_param("data")],
        vec![HirStatement::Return(Some(HirExpression::Dereference(
            Box::new(HirExpression::Cast {
                expr: Box::new(HirExpression::Variable("data".to_string())),
                target_type: HirType::Pointer(Box::new(HirType::Int)),
            }),
        )))],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    // Should prefer &T over &mut T for read-only
    assert!(
        code.contains("<T>"),
        "Should have generic T:\n{}",
        code
    );
}

// ============================================================================
// TEST 5: No void* - no generic
// ============================================================================

#[test]
fn test_no_void_ptr_no_generic() {
    let func = HirFunction::new_with_body(
        "add".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
        vec![],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    assert!(
        !code.contains("<T>"),
        "Should NOT generate generic for non-void* function:\n{}",
        code
    );
}

// ============================================================================
// TEST 6: Removes unsafe casts
// ============================================================================

#[test]
fn test_removes_unsafe_casts() {
    // The (int*)data cast should be removed in generic version
    let func = create_void_ptr_function(
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

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    // Should not have "as *const" or "as *mut" casts
    assert!(
        !code.contains("as *const") && !code.contains("as *mut"),
        "Should remove unsafe pointer casts:\n{}",
        code
    );
}
