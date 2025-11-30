//! Tests for array params missing `&` bug (P0-ARRAY-REF-001).
//!
//! Verifies that array params generate `arr: &[i32]` NOT `arr: [i32]`

use decy_codegen::CodeGenerator;
use decy_hir::{HirFunction, HirParameter, HirStatement, HirType};

/// Helper: Create test function
fn create_function(
    name: &str,
    params: Vec<HirParameter>,
    return_type: HirType,
    body: Vec<HirStatement>,
) -> HirFunction {
    HirFunction::new_with_body(name.to_string(), return_type, params, body)
}

// ============================================================================
// TEST 1: Array param has & reference
// ============================================================================

#[test]
fn test_array_param_has_reference() {
    // void process(int arr[]) → fn process(arr: &[i32])
    let func = create_function(
        "process",
        vec![HirParameter::new(
            "arr".to_string(),
            HirType::Array {
                element_type: Box::new(HirType::Int),
                size: None, // Unsized array
            },
        )],
        HirType::Void,
        vec![],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    // Must have &[i32] not just [i32]
    assert!(
        code.contains("&[i32]") || code.contains("&mut [i32]"),
        "Array param should be &[i32] or &mut [i32], got:\n{}",
        code
    );
    // Must NOT have bare [i32] without &
    let has_bare_array = code.contains(": [i32]") && !code.contains("&[i32]");
    assert!(
        !has_bare_array,
        "Should NOT have bare [i32] without &:\n{}",
        code
    );
}

// ============================================================================
// TEST 2: Fixed-size array param has & reference
// ============================================================================

#[test]
fn test_fixed_array_param_has_reference() {
    // void process(int arr[10]) → fn process(arr: &[i32])
    let func = create_function(
        "process_fixed",
        vec![HirParameter::new(
            "arr".to_string(),
            HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(10),
            },
        )],
        HirType::Void,
        vec![],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    // Should have slice reference (arrays decay to pointers in C)
    assert!(
        code.contains("&[i32]") || code.contains("&mut [i32]"),
        "Fixed array param should decay to slice reference:\n{}",
        code
    );
}

// ============================================================================
// TEST 3: Char array param has & reference
// ============================================================================

#[test]
fn test_char_array_param_has_reference() {
    // void process(char str[]) → fn process(str: &[u8])
    let func = create_function(
        "process_str",
        vec![HirParameter::new(
            "str".to_string(),
            HirType::Array {
                element_type: Box::new(HirType::Char),
                size: None,
            },
        )],
        HirType::Void,
        vec![],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    assert!(
        code.contains("&[u8]") || code.contains("&mut [u8]"),
        "Char array param should be &[u8]:\n{}",
        code
    );
}
