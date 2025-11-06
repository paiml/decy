//! Integration tests for array parameter to slice transformation (DECY-072).
//!
//! Tests transformation of C array parameters to safe Rust slices:
//! - `void process(int* arr, int len)` → `fn process(arr: &mut [i32])`
//! - Remove redundant length parameter
//! - Use `arr.len()` instead of length parameter in function body

use decy_codegen::CodeGenerator;
use decy_hir::{HirFunction, HirParameter, HirStatement, HirType};
use decy_ownership::dataflow::DataflowAnalyzer;

/// Test helper: Create a simple array parameter function
fn create_array_param_function(name: &str, param_name: &str, len_param_name: &str) -> HirFunction {
    let params = vec![
        HirParameter::new(
            param_name.to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        ),
        HirParameter::new(len_param_name.to_string(), HirType::Int),
    ];

    HirFunction::new(name.to_string(), HirType::Void, params)
}

#[test]
fn test_int_array_parameter_transformed_to_slice() {
    // RED: This test will FAIL because current implementation generates raw pointers
    // C: void process(int* arr, int len)
    // Expected Rust: fn process(arr: &mut [i32])

    let func = create_array_param_function("process", "arr", "len");
    let codegen = CodeGenerator::new();

    let rust_code = codegen.generate_function(&func);

    // Should transform to slice parameter
    assert!(
        rust_code.contains("arr: &[i32]") || rust_code.contains("arr: &mut [i32]"),
        "Expected slice parameter, got: {}",
        rust_code
    );

    // Should NOT contain raw pointer
    assert!(
        !rust_code.contains("*mut i32") && !rust_code.contains("*const i32"),
        "Should not use raw pointers for array parameters, got: {}",
        rust_code
    );

    // Should NOT include redundant length parameter
    assert!(
        !rust_code.contains("len: i32"),
        "Should not include redundant length parameter, got: {}",
        rust_code
    );
}

#[test]
fn test_char_buffer_parameter_transformed_to_u8_slice() {
    // RED: This test will FAIL
    // C: void read_buffer(char* buf, size_t size)
    // Expected Rust: fn read_buffer(buf: &[u8])

    let params = vec![
        HirParameter::new("buf".to_string(), HirType::Pointer(Box::new(HirType::Char))),
        HirParameter::new("size".to_string(), HirType::Int),
    ];

    let func = HirFunction::new("read_buffer".to_string(), HirType::Void, params);
    let codegen = CodeGenerator::new();

    let rust_code = codegen.generate_function(&func);

    // Should transform to u8 slice
    assert!(
        rust_code.contains("buf: &[u8]") || rust_code.contains("buf: &mut [u8]"),
        "Expected u8 slice parameter, got: {}",
        rust_code
    );

    // Should NOT include size parameter
    assert!(
        !rust_code.contains("size: i32"),
        "Should not include redundant size parameter, got: {}",
        rust_code
    );
}

#[test]
fn test_mutable_array_parameter_uses_mut_slice() {
    // RED: This test will FAIL
    // C: void modify_array(int* arr, int len) { arr[0] = 42; }
    // Expected Rust: fn modify_array(arr: &mut [i32]) { arr[0] = 42; }

    use decy_hir::HirExpression;

    let params = vec![
        HirParameter::new("arr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        HirParameter::new("len".to_string(), HirType::Int),
    ];

    // Function body: arr[0] = 42;
    let body = vec![HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
        value: HirExpression::IntLiteral(42),
    }];

    let func = HirFunction::new_with_body("modify_array".to_string(), HirType::Void, params, body);

    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_function(&func);

    // Should use mutable slice for arrays that are modified
    assert!(
        rust_code.contains("arr: &mut [i32]"),
        "Expected mutable slice for modified array, got: {}",
        rust_code
    );
}

#[test]
fn test_array_length_usage_transformed_to_len_method() {
    // RED: This test will FAIL
    // C: void print_len(int* arr, int len) { printf("%d", len); }
    // Expected Rust: fn print_len(arr: &[i32]) { println!("{}", arr.len()); }

    use decy_hir::HirExpression;

    let params = vec![
        HirParameter::new("arr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        HirParameter::new("len".to_string(), HirType::Int),
    ];

    // Function body: return len;
    let body = vec![HirStatement::Return(Some(HirExpression::Variable(
        "len".to_string(),
    )))];

    let func = HirFunction::new_with_body("get_length".to_string(), HirType::Int, params, body);

    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_function(&func);

    // Should use arr.len() instead of len parameter
    assert!(
        rust_code.contains("arr.len()"),
        "Expected arr.len() call, got: {}",
        rust_code
    );

    // Should NOT reference the len parameter (it shouldn't exist)
    assert!(
        !rust_code.contains("len") || rust_code.contains(".len()"),
        "Should not use len parameter, got: {}",
        rust_code
    );
}

#[test]
fn test_pointer_without_length_not_transformed() {
    // RED: This should PASS even in RED phase (negative test)
    // C: void process(int* ptr)
    // Expected Rust: fn process(ptr: *mut i32) - raw pointer (no length)

    let params = vec![HirParameter::new(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    )];

    let func = HirFunction::new("process".to_string(), HirType::Void, params);
    let codegen = CodeGenerator::new();

    let rust_code = codegen.generate_function(&func);

    // Should NOT transform to slice (no length parameter)
    assert!(
        !rust_code.contains("&[i32]") && !rust_code.contains("&mut [i32]"),
        "Should not transform pointer without length to slice, got: {}",
        rust_code
    );
}

#[test]
fn test_array_parameter_detection_by_name_pattern() {
    // RED: This test will FAIL
    // C: void sum_array(int* array, int count)
    // Expected: Detected as array due to "array" name

    let params = vec![
        HirParameter::new(
            "array".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        ),
        HirParameter::new("count".to_string(), HirType::Int),
    ];

    let func = HirFunction::new("sum_array".to_string(), HirType::Int, params);
    let codegen = CodeGenerator::new();

    let rust_code = codegen.generate_function(&func);

    // Should detect as array due to naming pattern
    assert!(
        rust_code.contains("array: &[i32]") || rust_code.contains("array: &mut [i32]"),
        "Expected slice parameter for 'array' name, got: {}",
        rust_code
    );
}

#[test]
fn test_buffer_parameter_detection_by_name_pattern() {
    // RED: This test will FAIL
    // C: void process_buffer(char* buffer, int length)
    // Expected: Detected as array due to "buffer" name

    let params = vec![
        HirParameter::new(
            "buffer".to_string(),
            HirType::Pointer(Box::new(HirType::Char)),
        ),
        HirParameter::new("length".to_string(), HirType::Int),
    ];

    let func = HirFunction::new("process_buffer".to_string(), HirType::Void, params);
    let codegen = CodeGenerator::new();

    let rust_code = codegen.generate_function(&func);

    // Should detect as array due to naming pattern
    assert!(
        rust_code.contains("buffer: &[u8]") || rust_code.contains("buffer: &mut [u8]"),
        "Expected slice parameter for 'buffer' name, got: {}",
        rust_code
    );
}

#[test]
fn test_dataflow_analysis_detects_array_parameters() {
    // RED: This test will FAIL
    // Test that DataflowAnalyzer correctly identifies array parameters

    let params = vec![
        HirParameter::new("data".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        HirParameter::new("size".to_string(), HirType::Int),
    ];

    let func = HirFunction::new("process_data".to_string(), HirType::Void, params);

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    // Should detect "data" as an array parameter
    let is_array = graph.is_array_parameter("data");

    assert_eq!(
        is_array,
        Some(true),
        "DataflowAnalyzer should detect 'data' as array parameter"
    );
}

#[test]
fn test_multiple_array_parameters_all_transformed() {
    // RED: This test will FAIL
    // C: void merge(int* a, int len_a, int* b, int len_b)
    // Expected: Both arrays transformed to slices

    let params = vec![
        HirParameter::new("a".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        HirParameter::new("len_a".to_string(), HirType::Int),
        HirParameter::new("b".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        HirParameter::new("len_b".to_string(), HirType::Int),
    ];

    let func = HirFunction::new("merge".to_string(), HirType::Void, params);
    let codegen = CodeGenerator::new();

    let rust_code = codegen.generate_function(&func);

    // Both parameters should be slices
    assert!(
        rust_code.contains("a: &[i32]") || rust_code.contains("a: &mut [i32]"),
        "Expected 'a' to be slice, got: {}",
        rust_code
    );

    assert!(
        rust_code.contains("b: &[i32]") || rust_code.contains("b: &mut [i32]"),
        "Expected 'b' to be slice, got: {}",
        rust_code
    );

    // Neither length parameter should exist
    assert!(
        !rust_code.contains("len_a") && !rust_code.contains("len_b"),
        "Should not have length parameters, got: {}",
        rust_code
    );
}

#[test]
fn test_float_array_parameter_transformed() {
    // RED: This test will FAIL
    // C: void process_floats(float* values, int num_values)
    // Expected Rust: fn process_floats(values: &[f32])

    let params = vec![
        HirParameter::new(
            "values".to_string(),
            HirType::Pointer(Box::new(HirType::Float)),
        ),
        HirParameter::new("num_values".to_string(), HirType::Int),
    ];

    let func = HirFunction::new("process_floats".to_string(), HirType::Void, params);
    let codegen = CodeGenerator::new();

    let rust_code = codegen.generate_function(&func);

    // Should transform float array to f32 slice
    assert!(
        rust_code.contains("values: &[f32]") || rust_code.contains("values: &mut [f32]"),
        "Expected f32 slice parameter, got: {}",
        rust_code
    );
}
