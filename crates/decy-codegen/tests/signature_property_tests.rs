//! Property-based tests for array parameter signature transformation
//!
//! DECY-073 RED: These tests verify that array parameters are correctly
//! transformed to safe Rust slice signatures.
//!
//! Target: Additional properties to reach 20K total test cases

use decy_core::transpile;
use proptest::prelude::*;

// Helper to check if a name is a reserved keyword
fn is_reserved_keyword(name: &str) -> bool {
    // Exact reserved keywords
    let exact_reserved = [
        "asm", "auto", "break", "case", "char", "const", "continue", "default", "do", "double",
        "else", "enum", "extern", "float", "for", "goto", "if", "int", "long", "register",
        "return", "short", "signed", "sizeof", "static", "struct", "switch", "typedef", "union",
        "unsigned", "void", "volatile", "while", "fn", "let", "mut", "use", "mod", "pub", "crate",
        "self", "super", "impl", "trait", "type", "where", "async", "await", "dyn", "move", "ref",
        "match", "loop", "unsafe", "box",
        // DECY-111: Common parameter names that conflict with test patterns
        "len", "size", "count", "idx",
        // C predefined macros that cause syntax errors (POSIX/system macros)
        "unix", "linux", "i386", "i686", "x86_64", "amd64", "arm", "aarch64",
        "null", "true", "false", "main", "exit", "errno", "stdin", "stdout", "stderr",
    ];

    if exact_reserved.contains(&name) {
        return true;
    }

    // Names starting with "len" will trigger false positives in tests
    // because "< lena" contains "< len" as substring
    if name.starts_with("len") {
        return true;
    }

    false
}

// ============================================================================
// PROPERTY 1: Array parameters always produce slice syntax
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_array_param_produces_slice(
        func_name in "[a-z]{3,10}",
        arr_suffix in "[a-z]{0,5}",
        len_suffix in "[a-z]{0,5}",
    ) {
        // Filter out reserved keywords
        prop_assume!(!is_reserved_keyword(&func_name));

        // Use common names to trigger detection heuristics
        let arr_name = format!("arr{}", arr_suffix);
        let len_name = format!("len{}", len_suffix);

        prop_assume!(arr_name != len_name);
        prop_assume!(func_name != arr_name && func_name != len_name);

        let c_code = format!(
            "void {}(int* {}, int {}) {{ }}",
            func_name, arr_name, len_name
        );

        let result = transpile(&c_code);
        assert!(result.is_ok(), "Transpilation should succeed");

        let rust_code = result.unwrap();

        // Should contain slice syntax
        assert!(
            rust_code.contains(&format!("{}: &[i32]", arr_name)),
            "Should transform to slice syntax\nGenerated:\n{}",
            rust_code
        );

        // Should NOT contain length parameter
        assert!(
            !rust_code.contains(&format!("{}: i32", len_name)) &&
            !rust_code.contains(&format!("{}: usize", len_name)),
            "Should remove length parameter\nGenerated:\n{}",
            rust_code
        );
    }
}

// ============================================================================
// PROPERTY 2: Mutable arrays produce mutable slices
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_mutable_array_produces_mut_slice(
        func_name in "[a-z]{3,10}",
        arr_name in "[a-z]{3,8}",
    ) {
        prop_assume!(!is_reserved_keyword(&func_name));
        prop_assume!(!is_reserved_keyword(&arr_name));
        prop_assume!(func_name != arr_name);

        let c_code = format!(
            "void {}(int* {}, int len) {{ {}[0] = 1; }}",
            func_name, arr_name, arr_name
        );

        let result = transpile(&c_code);
        assert!(result.is_ok(), "Transpilation should succeed");

        let rust_code = result.unwrap();

        // Should contain mutable slice syntax
        assert!(
            rust_code.contains(&format!("{}: &mut [i32]", arr_name)),
            "Should transform to mutable slice\nGenerated:\n{}",
            rust_code
        );
    }
}

// ============================================================================
// PROPERTY 3: Char arrays transform to u8 slices
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_char_array_to_u8_slice(
        func_name in "[a-z]{3,10}",
        buf_name in "[a-z]{3,8}",
    ) {
        prop_assume!(!is_reserved_keyword(&func_name));
        prop_assume!(!is_reserved_keyword(&buf_name));
        prop_assume!(func_name != buf_name);

        let c_code = format!(
            "void {}(char* {}, int size) {{ {}[0] = 65; }}",
            func_name, buf_name, buf_name
        );

        let result = transpile(&c_code);
        assert!(result.is_ok(), "Transpilation should succeed");

        let rust_code = result.unwrap();

        // Should contain u8 slice
        assert!(
            rust_code.contains(&format!("{}: &mut [u8]", buf_name)) ||
            rust_code.contains(&format!("{}: &[u8]", buf_name)),
            "Should transform char* to u8 slice\nGenerated:\n{}",
            rust_code
        );
    }
}

// ============================================================================
// PROPERTY 4: Multiple array parameters all transformed
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_multiple_arrays_all_slices(
        arr1 in "[a-z]{3,6}",
        arr2 in "[a-z]{3,6}",
    ) {
        prop_assume!(!is_reserved_keyword(&arr1));
        prop_assume!(!is_reserved_keyword(&arr2));
        prop_assume!(arr1 != arr2);

        let c_code = format!(
            "void merge(int* {}, int len1, int* {}, int len2) {{ }}",
            arr1, arr2
        );

        let result = transpile(&c_code);
        assert!(result.is_ok(), "Transpilation should succeed");

        let rust_code = result.unwrap();

        // Both should be slices
        assert!(
            rust_code.contains(&format!("{}: &[i32]", arr1)),
            "First array should be slice\nGenerated:\n{}",
            rust_code
        );
        assert!(
            rust_code.contains(&format!("{}: &[i32]", arr2)),
            "Second array should be slice\nGenerated:\n{}",
            rust_code
        );

        // No length parameters
        assert!(
            !rust_code.contains("len1") && !rust_code.contains("len2"),
            "Length parameters should be removed\nGenerated:\n{}",
            rust_code
        );
    }
}

// ============================================================================
// PROPERTY 5: Length usage transformed to .len() calls
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_length_usage_becomes_method_call(
        arr_name in "[a-z]{3,8}",
    ) {
        prop_assume!(!is_reserved_keyword(&arr_name));

        let c_code = format!(
            r#"
            int sum(int* {}, int len) {{
                int total = 0;
                for (int i = 0; i < len; i++) {{
                    total = total + {}[i];
                }}
                return total;
            }}
            "#,
            arr_name, arr_name
        );

        let result = transpile(&c_code);
        assert!(result.is_ok(), "Transpilation should succeed");

        let rust_code = result.unwrap();

        // Should use .len() method
        assert!(
            rust_code.contains(&format!("{}.len()", arr_name)),
            "Should use .len() method\nGenerated:\n{}",
            rust_code
        );

        // Should NOT have standalone 'len' variable (not part of array name)
        // Use more specific patterns to avoid false positives like "alen:" containing "len:"
        assert!(
            !rust_code.contains("< len") && !rust_code.contains(", len:") && !rust_code.contains("(len:"),
            "Should not have length variable\nGenerated:\n{}",
            rust_code
        );
    }
}

// ============================================================================
// PROPERTY 6: Return values preserved correctly
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_return_type_preserved(
        arr_name in "[a-z]{3,8}",
        return_type in prop::sample::select(vec!["int", "float", "double", "char"]),
    ) {
        prop_assume!(!is_reserved_keyword(&arr_name));

        let c_code = format!(
            "{} first(int* {}, int len) {{ return {}[0]; }}",
            return_type, arr_name, arr_name
        );

        let result = transpile(&c_code);
        assert!(result.is_ok(), "Transpilation should succeed");

        let rust_code = result.unwrap();

        // Should preserve return type
        let expected_rust_type = match return_type {
            "int" => "i32",
            "float" => "f32",
            "double" => "f64",
            "char" => "u8",
            _ => unreachable!(),
        };

        assert!(
            rust_code.contains(&format!("-> {}", expected_rust_type)),
            "Should preserve return type\nGenerated:\n{}",
            rust_code
        );
    }
}

// ============================================================================
// PROPERTY 7: No unsafe blocks generated
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_no_unsafe_blocks(
        func_name in "[a-z]{3,10}",
        arr_name in "[a-z]{3,8}",
    ) {
        prop_assume!(!is_reserved_keyword(&func_name));
        prop_assume!(!is_reserved_keyword(&arr_name));
        prop_assume!(func_name != arr_name);

        let c_code = format!(
            "void {}(int* {}, int len) {{ {}[0] = 42; }}",
            func_name, arr_name, arr_name
        );

        let result = transpile(&c_code);
        assert!(result.is_ok(), "Transpilation should succeed");

        let rust_code = result.unwrap();

        // Must NOT contain unsafe keyword
        assert!(
            !rust_code.contains("unsafe"),
            "Should not generate unsafe blocks\nGenerated:\n{}",
            rust_code
        );
    }
}

// ============================================================================
// PROPERTY 8: Float array parameters work
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_float_arrays_work(
        func_name in "[a-z]{3,10}",
        arr_name in "[a-z]{3,8}",
    ) {
        prop_assume!(!is_reserved_keyword(&func_name));
        prop_assume!(!is_reserved_keyword(&arr_name));
        prop_assume!(func_name != arr_name);

        let c_code = format!(
            "void {}(float* {}, int len) {{ }}",
            func_name, arr_name
        );

        let result = transpile(&c_code);
        assert!(result.is_ok(), "Transpilation should succeed");

        let rust_code = result.unwrap();

        // Should be f32 slice
        assert!(
            rust_code.contains(&format!("{}: &[f32]", arr_name)),
            "Should transform float* to &[f32]\nGenerated:\n{}",
            rust_code
        );
    }
}

// ============================================================================
// PROPERTY 9: Double array parameters work
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_double_arrays_work(
        func_name in "[a-z]{3,10}",
        arr_name in "[a-z]{3,8}",
    ) {
        prop_assume!(!is_reserved_keyword(&func_name));
        prop_assume!(!is_reserved_keyword(&arr_name));
        prop_assume!(func_name != arr_name);

        let c_code = format!(
            "void {}(double* {}, int len) {{ }}",
            func_name, arr_name
        );

        let result = transpile(&c_code);
        assert!(result.is_ok(), "Transpilation should succeed");

        let rust_code = result.unwrap();

        // Should be f64 slice
        assert!(
            rust_code.contains(&format!("{}: &[f64]", arr_name)),
            "Should transform double* to &[f64]\nGenerated:\n{}",
            rust_code
        );
    }
}

// ============================================================================
// PROPERTY 10: Empty body functions work
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_empty_body_transformation(
        func_name in "[a-z]{3,10}",
        arr_name in "[a-z]{3,8}",
    ) {
        prop_assume!(!is_reserved_keyword(&func_name));
        prop_assume!(!is_reserved_keyword(&arr_name));
        prop_assume!(func_name != arr_name);

        let c_code = format!(
            "void {}(int* {}, int len) {{ }}",
            func_name, arr_name
        );

        let result = transpile(&c_code);
        assert!(result.is_ok(), "Transpilation should succeed");

        let rust_code = result.unwrap();

        // Should still transform signature
        assert!(
            rust_code.contains(&format!("{}: &[i32]", arr_name)),
            "Empty body should still transform signature\nGenerated:\n{}",
            rust_code
        );
    }
}
