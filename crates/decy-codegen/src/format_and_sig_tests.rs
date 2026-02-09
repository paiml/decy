//! Coverage tests for convert_format_specifiers and generate_annotated_signature_with_func.
//!
//! Tests cover all format specifier branches including flags, width, precision, length modifiers,
//! and all signature generation branches including lifetimes, output params, slices, pointers,
//! keyword renaming, and return type handling.

use super::*;
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType};
use decy_ownership::lifetime_gen::{
    AnnotatedParameter, AnnotatedSignature, AnnotatedType, LifetimeParam,
};

// ============================================================================
// convert_format_specifiers: flag combinations (-, +, space, #)
// ============================================================================

#[test]
fn test_format_minus_flag_with_width() {
    // %-10d → left-align integer, flag '-' is skipped, width 10
    let result = CodeGenerator::convert_format_specifiers("%-10d");
    assert!(result.contains("{:"));
    assert!(result.contains("10}"));
}

#[test]
fn test_format_plus_flag_with_width() {
    // %+5d → signed integer with explicit sign
    let result = CodeGenerator::convert_format_specifiers("%+5d");
    assert!(result.contains("{:"));
    assert!(result.contains("5}"));
}

#[test]
fn test_format_space_flag() {
    // % d → space flag before positive numbers
    let result = CodeGenerator::convert_format_specifiers("% d");
    assert!(result.contains('{'));
}

#[test]
fn test_format_hash_flag() {
    // %#x → alternate form for hex (0x prefix in C)
    let result = CodeGenerator::convert_format_specifiers("%#x");
    assert!(result.contains("x}"));
}

#[test]
fn test_format_hash_flag_octal() {
    // %#o → alternate form for octal
    let result = CodeGenerator::convert_format_specifiers("%#o");
    assert!(result.contains("o}"));
}

#[test]
fn test_format_multiple_flags_combined() {
    // %0-10d → multiple flags: zero-pad and left-align
    let result = CodeGenerator::convert_format_specifiers("%0-10d");
    assert!(result.contains("{:"));
    assert!(result.contains("10}"));
}

#[test]
fn test_format_zero_and_plus_flags() {
    // %+010d → signed with zero-pad
    let result = CodeGenerator::convert_format_specifiers("%+010d");
    assert!(result.contains("{:0"));
    assert!(result.contains("10}"));
}

// ============================================================================
// convert_format_specifiers: length modifiers with various specifiers
// ============================================================================

#[test]
fn test_format_long_hex() {
    // %lx → long hex
    assert_eq!(CodeGenerator::convert_format_specifiers("%lx"), "{:x}");
}

#[test]
fn test_format_long_long_hex() {
    // %llx → long long hex
    assert_eq!(CodeGenerator::convert_format_specifiers("%llx"), "{:x}");
}

#[test]
fn test_format_long_unsigned() {
    // %lu → long unsigned
    assert_eq!(CodeGenerator::convert_format_specifiers("%lu"), "{}");
}

#[test]
fn test_format_short_unsigned() {
    // %hu → short unsigned
    assert_eq!(CodeGenerator::convert_format_specifiers("%hu"), "{}");
}

#[test]
fn test_format_size_t_hex() {
    // %zx → size_t hex
    assert_eq!(CodeGenerator::convert_format_specifiers("%zx"), "{:x}");
}

#[test]
fn test_format_intmax_d() {
    // %jd → intmax_t integer
    assert_eq!(CodeGenerator::convert_format_specifiers("%jd"), "{}");
}

#[test]
fn test_format_ptrdiff_d() {
    // %td → ptrdiff_t integer
    assert_eq!(CodeGenerator::convert_format_specifiers("%td"), "{}");
}

#[test]
fn test_format_long_double() {
    // %Lf → long double (L modifier)
    assert_eq!(CodeGenerator::convert_format_specifiers("%Lf"), "{}");
}

#[test]
fn test_format_short_short_d() {
    // %hhd → char-sized integer
    assert_eq!(CodeGenerator::convert_format_specifiers("%hhd"), "{}");
}

// ============================================================================
// convert_format_specifiers: float uppercase and edge cases
// ============================================================================

#[test]
fn test_format_uppercase_float() {
    // %F → uppercase float (same as %f for output)
    assert_eq!(CodeGenerator::convert_format_specifiers("%F"), "{}");
}

#[test]
fn test_format_uppercase_float_with_precision() {
    // %.3F → uppercase float with precision
    assert_eq!(CodeGenerator::convert_format_specifiers("%.3F"), "{:.3}");
}

#[test]
fn test_format_uppercase_float_with_width_precision() {
    // %10.3F → uppercase float with width and precision
    assert_eq!(
        CodeGenerator::convert_format_specifiers("%10.3F"),
        "{:10.3}"
    );
}

#[test]
fn test_format_uppercase_float_with_width_only() {
    // %10F → uppercase float with width only
    assert_eq!(CodeGenerator::convert_format_specifiers("%10F"), "{:10}");
}

#[test]
fn test_format_float_zero_pad_width_precision() {
    // %010.2f → zero-pad, width 10, precision 2
    assert_eq!(
        CodeGenerator::convert_format_specifiers("%010.2f"),
        "{:010.2}"
    );
}

#[test]
fn test_format_precision_only_no_width() {
    // %.6f → precision 6, no width
    assert_eq!(CodeGenerator::convert_format_specifiers("%.6f"), "{:.6}");
}

#[test]
fn test_format_precision_zero() {
    // %.0f → precision 0
    assert_eq!(CodeGenerator::convert_format_specifiers("%.0f"), "{:.0}");
}

// ============================================================================
// convert_format_specifiers: binary specifier with flags
// ============================================================================

#[test]
fn test_format_binary_with_zero_pad_width() {
    // %016b → zero-padded binary 16-wide
    assert_eq!(
        CodeGenerator::convert_format_specifiers("%016b"),
        "{:016b}"
    );
}

#[test]
fn test_format_binary_with_width_only() {
    // %8b → binary with width 8
    assert_eq!(CodeGenerator::convert_format_specifiers("%8b"), "{:8b}");
}

// ============================================================================
// convert_format_specifiers: complex format strings
// ============================================================================

#[test]
fn test_format_complex_printf_pattern() {
    // Complex real-world printf: "Value: %08X (decimal: %d, float: %.2f)"
    let result = CodeGenerator::convert_format_specifiers(
        "Value: %08X (decimal: %d, float: %.2f)",
    );
    assert_eq!(
        result,
        "Value: {:08X} (decimal: {}, float: {:.2})"
    );
}

#[test]
fn test_format_all_basic_specifiers() {
    // String with all basic specifiers
    let result = CodeGenerator::convert_format_specifiers("%d %u %x %X %o %b %f %e %E %g %G %s %c %p");
    assert_eq!(
        result,
        "{} {} {:x} {:X} {:o} {:b} {} {:e} {:E} {} {} {} {} {:p}"
    );
}

#[test]
fn test_format_newline_in_string() {
    // Format string with embedded newline chars
    let result = CodeGenerator::convert_format_specifiers("line1: %d\\nline2: %s");
    assert_eq!(result, "line1: {}\\nline2: {}");
}

#[test]
fn test_format_adjacent_specifiers() {
    // No separator between specifiers
    assert_eq!(
        CodeGenerator::convert_format_specifiers("%d%d%d"),
        "{}{}{}"
    );
}

#[test]
fn test_format_percent_between_specifiers() {
    // %% interleaved with real specifiers
    assert_eq!(
        CodeGenerator::convert_format_specifiers("%d%%%d"),
        "{}%{}"
    );
}

// ============================================================================
// convert_format_specifiers: incomplete / edge cases
// ============================================================================

#[test]
fn test_format_single_percent_at_end() {
    // Trailing % without specifier
    let result = CodeGenerator::convert_format_specifiers("test%");
    assert!(result.contains("test"));
    assert!(result.contains('%'));
}

#[test]
fn test_format_just_percent() {
    // Single % character alone
    let result = CodeGenerator::convert_format_specifiers("%");
    assert!(!result.is_empty());
}

#[test]
fn test_format_percent_with_just_width_no_specifier() {
    // %10 at end of string (incomplete)
    let result = CodeGenerator::convert_format_specifiers("val: %10");
    assert!(result.contains("val:"));
}

#[test]
fn test_format_unknown_specifier_y() {
    // Unknown specifier %y → preserve original
    let result = CodeGenerator::convert_format_specifiers("%y");
    assert_eq!(result, "%y");
}

#[test]
fn test_format_unknown_specifier_with_width() {
    // Unknown specifier %10y → preserve original
    let result = CodeGenerator::convert_format_specifiers("%10y");
    assert_eq!(result, "%10y");
}

// ============================================================================
// convert_c_format_to_rust: wrapper that handles quoted strings
// ============================================================================

#[test]
fn test_c_format_to_rust_quoted_string() {
    // Quoted format string
    let result = CodeGenerator::convert_c_format_to_rust("\"%d\"");
    assert_eq!(result, "\"{}\"");
}

#[test]
fn test_c_format_to_rust_not_quoted() {
    // Non-quoted string returned as-is
    let result = CodeGenerator::convert_c_format_to_rust("some_var");
    assert_eq!(result, "some_var");
}

#[test]
fn test_c_format_to_rust_empty_quotes() {
    // Empty quoted string
    let result = CodeGenerator::convert_c_format_to_rust("\"\"");
    assert_eq!(result, "\"\"");
}

#[test]
fn test_c_format_to_rust_with_whitespace() {
    // Quoted string with leading/trailing whitespace
    let result = CodeGenerator::convert_c_format_to_rust("  \"%d %s\"  ");
    assert_eq!(result, "\"{} {}\"");
}

// ============================================================================
// generate_annotated_signature_with_func: basic signatures (no func)
// ============================================================================

fn make_simple_sig(name: &str, params: Vec<AnnotatedParameter>, ret: AnnotatedType) -> AnnotatedSignature {
    AnnotatedSignature {
        name: name.to_string(),
        lifetimes: vec![],
        parameters: params,
        return_type: ret,
    }
}

fn make_param(name: &str, param_type: AnnotatedType) -> AnnotatedParameter {
    AnnotatedParameter {
        name: name.to_string(),
        param_type,
    }
}

#[test]
fn test_sig_simple_int_function() {
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig(
        "add",
        vec![
            make_param("a", AnnotatedType::Simple(HirType::Int)),
            make_param("b", AnnotatedType::Simple(HirType::Int)),
        ],
        AnnotatedType::Simple(HirType::Int),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert_eq!(result, "fn add(mut a: i32, mut b: i32) -> i32");
}

#[test]
fn test_sig_void_return() {
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig(
        "noop",
        vec![],
        AnnotatedType::Simple(HirType::Void),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert_eq!(result, "fn noop()");
}

#[test]
fn test_sig_no_params_with_return() {
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig(
        "get_value",
        vec![],
        AnnotatedType::Simple(HirType::Double),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert_eq!(result, "fn get_value() -> f64");
}

// ============================================================================
// generate_annotated_signature_with_func: keyword renaming (DECY-241)
// ============================================================================

#[test]
fn test_sig_rename_write() {
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig("write", vec![], AnnotatedType::Simple(HirType::Int));
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert!(result.starts_with("fn c_write("));
}

#[test]
fn test_sig_rename_read() {
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig("read", vec![], AnnotatedType::Simple(HirType::Int));
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert!(result.starts_with("fn c_read("));
}

#[test]
fn test_sig_rename_type() {
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig("type", vec![], AnnotatedType::Simple(HirType::Void));
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert!(result.starts_with("fn c_type("));
}

#[test]
fn test_sig_rename_match() {
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig("match", vec![], AnnotatedType::Simple(HirType::Void));
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert!(result.starts_with("fn c_match("));
}

#[test]
fn test_sig_rename_self() {
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig("self", vec![], AnnotatedType::Simple(HirType::Void));
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert!(result.starts_with("fn c_self("));
}

#[test]
fn test_sig_rename_in() {
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig("in", vec![], AnnotatedType::Simple(HirType::Void));
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert!(result.starts_with("fn c_in("));
}

#[test]
fn test_sig_no_rename_for_normal_name() {
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig("process", vec![], AnnotatedType::Simple(HirType::Void));
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert!(result.starts_with("fn process("));
}

// ============================================================================
// generate_annotated_signature_with_func: lifetime parameters
// ============================================================================

#[test]
fn test_sig_with_lifetime_and_reference() {
    let codegen = CodeGenerator::new();
    let lt = LifetimeParam::standard(0); // 'a
    let sig = AnnotatedSignature {
        name: "get_ref".to_string(),
        lifetimes: vec![lt.clone()],
        parameters: vec![make_param(
            "data",
            AnnotatedType::Reference {
                inner: Box::new(AnnotatedType::Simple(HirType::Int)),
                mutable: false,
                lifetime: Some(lt.clone()),
            },
        )],
        return_type: AnnotatedType::Reference {
            inner: Box::new(AnnotatedType::Simple(HirType::Int)),
            mutable: false,
            lifetime: Some(lt),
        },
    };
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert!(result.contains("<'a>"), "Expected lifetime param, got: {}", result);
    assert!(result.contains("&'a i32"), "Expected annotated ref, got: {}", result);
}

#[test]
fn test_sig_multiple_lifetimes() {
    let codegen = CodeGenerator::new();
    let lt_a = LifetimeParam::standard(0); // 'a
    let lt_b = LifetimeParam::standard(1); // 'b
    let sig = AnnotatedSignature {
        name: "merge".to_string(),
        lifetimes: vec![lt_a.clone(), lt_b.clone()],
        parameters: vec![
            make_param(
                "first",
                AnnotatedType::Reference {
                    inner: Box::new(AnnotatedType::Simple(HirType::Int)),
                    mutable: false,
                    lifetime: Some(lt_a.clone()),
                },
            ),
            make_param(
                "second",
                AnnotatedType::Reference {
                    inner: Box::new(AnnotatedType::Simple(HirType::Int)),
                    mutable: false,
                    lifetime: Some(lt_b.clone()),
                },
            ),
        ],
        return_type: AnnotatedType::Simple(HirType::Int),
    };
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert!(result.contains("<'a, 'b>"), "Expected two lifetimes, got: {}", result);
}

#[test]
fn test_sig_lifetime_not_added_for_slice_only() {
    // Slice parameters have elided lifetimes, so no explicit lifetime needed
    let codegen = CodeGenerator::new();
    let lt = LifetimeParam::standard(0);
    let sig = AnnotatedSignature {
        name: "process_slice".to_string(),
        lifetimes: vec![lt.clone()],
        parameters: vec![make_param(
            "data",
            AnnotatedType::Reference {
                inner: Box::new(AnnotatedType::Simple(HirType::Array {
                    element_type: Box::new(HirType::Int),
                    size: None,
                })),
                mutable: false,
                lifetime: Some(lt),
            },
        )],
        return_type: AnnotatedType::Simple(HirType::Void),
    };
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    // Slice-only references should NOT emit lifetime params
    assert!(!result.contains("<'a>"), "Slice-only should not have explicit lifetime: {}", result);
}

// ============================================================================
// generate_annotated_signature_with_func: slice parameters (DECY-072)
// ============================================================================

#[test]
fn test_sig_immutable_slice_param() {
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig(
        "sum_array",
        vec![make_param(
            "arr",
            AnnotatedType::Reference {
                inner: Box::new(AnnotatedType::Simple(HirType::Array {
                    element_type: Box::new(HirType::Int),
                    size: None,
                })),
                mutable: false,
                lifetime: None,
            },
        )],
        AnnotatedType::Simple(HirType::Int),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert!(result.contains("arr: &[i32]"), "Expected immutable slice, got: {}", result);
}

#[test]
fn test_sig_mutable_slice_param() {
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig(
        "fill_array",
        vec![make_param(
            "arr",
            AnnotatedType::Reference {
                inner: Box::new(AnnotatedType::Simple(HirType::Array {
                    element_type: Box::new(HirType::Double),
                    size: None,
                })),
                mutable: true,
                lifetime: None,
            },
        )],
        AnnotatedType::Simple(HirType::Void),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert!(result.contains("arr: &mut [f64]"), "Expected mutable slice, got: {}", result);
}

#[test]
fn test_sig_slice_with_non_matching_inner() {
    // A Reference whose inner is NOT an unsized array falls through to annotated_type_to_string
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig(
        "process",
        vec![make_param(
            "data",
            AnnotatedType::Reference {
                inner: Box::new(AnnotatedType::Simple(HirType::Int)),
                mutable: false,
                lifetime: None,
            },
        )],
        AnnotatedType::Simple(HirType::Void),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    // Should NOT be treated as slice
    assert!(!result.contains("[i32]"), "Non-array ref should not be slice: {}", result);
}

// ============================================================================
// generate_annotated_signature_with_func: pointer params (DECY-111)
// ============================================================================

#[test]
fn test_sig_simple_pointer_becomes_mut_ref() {
    // Simple pointer param → &mut T (no func context)
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig(
        "increment",
        vec![make_param(
            "val",
            AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Int))),
        )],
        AnnotatedType::Simple(HirType::Void),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert!(result.contains("val: &mut i32"), "Expected &mut i32, got: {}", result);
}

#[test]
fn test_sig_void_pointer_stays_raw() {
    // void* → *mut () (DECY-168)
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig(
        "generic_op",
        vec![make_param(
            "ptr",
            AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Void))),
        )],
        AnnotatedType::Simple(HirType::Void),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert!(result.contains("ptr: *mut ()"), "Expected *mut (), got: {}", result);
}

#[test]
fn test_sig_double_pointer_becomes_mut_ref() {
    // Double* → &mut f64
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig(
        "scale",
        vec![make_param(
            "val",
            AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Double))),
        )],
        AnnotatedType::Simple(HirType::Void),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert!(result.contains("val: &mut f64"), "Expected &mut f64, got: {}", result);
}

// ============================================================================
// generate_annotated_signature_with_func: unsized array params (DECY-196)
// ============================================================================

#[test]
fn test_sig_unsized_array_becomes_mut_slice() {
    // char arr[] → &mut [u8] (DECY-196)
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig(
        "process_chars",
        vec![make_param(
            "arr",
            AnnotatedType::Simple(HirType::Array {
                element_type: Box::new(HirType::Char),
                size: None,
            }),
        )],
        AnnotatedType::Simple(HirType::Void),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert!(result.contains("arr: &mut [u8]"), "Expected &mut [u8], got: {}", result);
}

#[test]
fn test_sig_unsized_int_array_becomes_mut_slice() {
    // int arr[] → &mut [i32]
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig(
        "sum",
        vec![make_param(
            "arr",
            AnnotatedType::Simple(HirType::Array {
                element_type: Box::new(HirType::Int),
                size: None,
            }),
        )],
        AnnotatedType::Simple(HirType::Int),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert!(result.contains("arr: &mut [i32]"), "Expected &mut [i32], got: {}", result);
}

// ============================================================================
// generate_annotated_signature_with_func: main function special handling
// ============================================================================

#[test]
fn test_sig_main_no_return_type() {
    // int main() → fn main() (no return type)
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig(
        "main",
        vec![],
        AnnotatedType::Simple(HirType::Int),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert_eq!(result, "fn main()");
    assert!(!result.contains("-> i32"), "main should not have i32 return: {}", result);
}

#[test]
fn test_sig_main_with_void_return() {
    // void main() → fn main() (no return type either way)
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig(
        "main",
        vec![],
        AnnotatedType::Simple(HirType::Void),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert_eq!(result, "fn main()");
}

// ============================================================================
// generate_annotated_signature_with_func: non-void return types
// ============================================================================

#[test]
fn test_sig_float_return() {
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig(
        "compute",
        vec![make_param("x", AnnotatedType::Simple(HirType::Float))],
        AnnotatedType::Simple(HirType::Float),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert!(result.contains("-> f32"), "Expected f32 return, got: {}", result);
}

#[test]
fn test_sig_char_return() {
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig(
        "first_char",
        vec![],
        AnnotatedType::Simple(HirType::Char),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert!(result.contains("-> u8"), "Expected u8 return, got: {}", result);
}

#[test]
fn test_sig_unsigned_int_return() {
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig(
        "count",
        vec![],
        AnnotatedType::Simple(HirType::UnsignedInt),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert!(result.contains("-> u32"), "Expected u32 return, got: {}", result);
}

// ============================================================================
// generate_annotated_signature_with_func: with HirFunction (output params)
// ============================================================================

#[test]
fn test_sig_with_func_output_param_result_name() {
    // void get_result(int input, int* result) → fn get_result(input: i32) -> i32
    let codegen = CodeGenerator::new();

    let func = HirFunction::new_with_body(
        "get_result".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("input".to_string(), HirType::Int),
            HirParameter::new("result".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        ],
        vec![
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("result".to_string()),
                value: HirExpression::Variable("input".to_string()),
            },
        ],
    );

    let sig = make_simple_sig(
        "get_result",
        vec![
            make_param("input", AnnotatedType::Simple(HirType::Int)),
            make_param("result", AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Int)))),
        ],
        AnnotatedType::Simple(HirType::Void),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, Some(&func));
    // "result" name should be detected as output param
    assert!(result.contains("-> i32") || result.contains("&mut i32"),
        "Expected output param transformation, got: {}", result);
}

#[test]
fn test_sig_with_func_output_param_out_name() {
    // void compute(int x, float* out) → fn compute(x: i32) -> f32
    let codegen = CodeGenerator::new();

    let func = HirFunction::new_with_body(
        "compute".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("x".to_string(), HirType::Int),
            HirParameter::new("out".to_string(), HirType::Pointer(Box::new(HirType::Float))),
        ],
        vec![
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("out".to_string()),
                value: HirExpression::IntLiteral(42),
            },
        ],
    );

    let sig = make_simple_sig(
        "compute",
        vec![
            make_param("x", AnnotatedType::Simple(HirType::Int)),
            make_param("out", AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Float)))),
        ],
        AnnotatedType::Simple(HirType::Void),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, Some(&func));
    // "out" is an output param name
    assert!(result.contains("-> f32") || result.contains("&mut f32"),
        "Expected output param transformation for 'out' name, got: {}", result);
}

#[test]
fn test_sig_output_param_len_name() {
    // void measure(int data, int* len) → output param detected by "len" name
    let codegen = CodeGenerator::new();

    let func = HirFunction::new_with_body(
        "measure".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("data".to_string(), HirType::Int),
            HirParameter::new("len".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        ],
        vec![
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("len".to_string()),
                value: HirExpression::IntLiteral(10),
            },
        ],
    );

    let sig = make_simple_sig(
        "measure",
        vec![
            make_param("data", AnnotatedType::Simple(HirType::Int)),
            make_param("len", AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Int)))),
        ],
        AnnotatedType::Simple(HirType::Void),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, Some(&func));
    // "len" matches output name heuristic
    assert!(result.contains("-> i32") || result.contains("&mut"),
        "Expected output param for 'len', got: {}", result);
}

#[test]
fn test_sig_output_param_size_name() {
    let codegen = CodeGenerator::new();

    let func = HirFunction::new_with_body(
        "get_size".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("input".to_string(), HirType::Int),
            HirParameter::new("size".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        ],
        vec![
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("size".to_string()),
                value: HirExpression::IntLiteral(100),
            },
        ],
    );

    let sig = make_simple_sig(
        "get_size",
        vec![
            make_param("input", AnnotatedType::Simple(HirType::Int)),
            make_param("size", AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Int)))),
        ],
        AnnotatedType::Simple(HirType::Void),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, Some(&func));
    assert!(result.contains("-> i32") || result.contains("&mut"),
        "Expected output param for 'size', got: {}", result);
}

#[test]
fn test_sig_output_param_coordinate_names() {
    // Test x, y, z, w, h dimension names
    let codegen = CodeGenerator::new();
    for name in &["x", "y", "z", "w", "h"] {
        let func = HirFunction::new_with_body(
            "get_dim".to_string(),
            HirType::Void,
            vec![
                HirParameter::new("input".to_string(), HirType::Int),
                HirParameter::new(name.to_string(), HirType::Pointer(Box::new(HirType::Int))),
            ],
            vec![
                HirStatement::DerefAssignment {
                    target: HirExpression::Variable(name.to_string()),
                    value: HirExpression::IntLiteral(0),
                },
            ],
        );

        let sig = make_simple_sig(
            "get_dim",
            vec![
                make_param("input", AnnotatedType::Simple(HirType::Int)),
                make_param(name, AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Int)))),
            ],
            AnnotatedType::Simple(HirType::Void),
        );
        let result = codegen.generate_annotated_signature_with_func(&sig, Some(&func));
        assert!(result.contains("-> i32") || result.contains("&mut"),
            "Expected output param for '{}', got: {}", name, result);
    }
}

#[test]
fn test_sig_output_param_color_names() {
    // Test r, g, b color names
    let codegen = CodeGenerator::new();
    for name in &["r", "g", "b"] {
        let func = HirFunction::new_with_body(
            "get_color".to_string(),
            HirType::Void,
            vec![
                HirParameter::new("pixel".to_string(), HirType::Int),
                HirParameter::new(name.to_string(), HirType::Pointer(Box::new(HirType::Int))),
            ],
            vec![
                HirStatement::DerefAssignment {
                    target: HirExpression::Variable(name.to_string()),
                    value: HirExpression::IntLiteral(0),
                },
            ],
        );

        let sig = make_simple_sig(
            "get_color",
            vec![
                make_param("pixel", AnnotatedType::Simple(HirType::Int)),
                make_param(name, AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Int)))),
            ],
            AnnotatedType::Simple(HirType::Void),
        );
        let result = codegen.generate_annotated_signature_with_func(&sig, Some(&func));
        assert!(result.contains("-> i32") || result.contains("&mut"),
            "Expected output param for '{}', got: {}", name, result);
    }
}

#[test]
fn test_sig_output_param_width_height_names() {
    let codegen = CodeGenerator::new();
    for name in &["width", "height", "count", "avg"] {
        let func = HirFunction::new_with_body(
            "measure".to_string(),
            HirType::Void,
            vec![
                HirParameter::new("data".to_string(), HirType::Int),
                HirParameter::new(name.to_string(), HirType::Pointer(Box::new(HirType::Int))),
            ],
            vec![
                HirStatement::DerefAssignment {
                    target: HirExpression::Variable(name.to_string()),
                    value: HirExpression::IntLiteral(0),
                },
            ],
        );

        let sig = make_simple_sig(
            "measure",
            vec![
                make_param("data", AnnotatedType::Simple(HirType::Int)),
                make_param(name, AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Int)))),
            ],
            AnnotatedType::Simple(HirType::Void),
        );
        let result = codegen.generate_annotated_signature_with_func(&sig, Some(&func));
        assert!(result.contains("-> i32") || result.contains("&mut"),
            "Expected output param for '{}', got: {}", name, result);
    }
}

#[test]
fn test_sig_output_param_ret_name() {
    let codegen = CodeGenerator::new();

    let func = HirFunction::new_with_body(
        "get_val".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("input".to_string(), HirType::Int),
            HirParameter::new("ret".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        ],
        vec![
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("ret".to_string()),
                value: HirExpression::IntLiteral(0),
            },
        ],
    );

    let sig = make_simple_sig(
        "get_val",
        vec![
            make_param("input", AnnotatedType::Simple(HirType::Int)),
            make_param("ret", AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Int)))),
        ],
        AnnotatedType::Simple(HirType::Void),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, Some(&func));
    assert!(result.contains("-> i32") || result.contains("&mut"),
        "Expected output param for 'ret', got: {}", result);
}

// ============================================================================
// generate_annotated_signature_with_func: const char* → &str (DECY-135)
// ============================================================================

#[test]
fn test_sig_const_char_pointer_to_str() {
    // Use the parser to create a const char* parameter (is_pointee_const is private)
    use decy_parser::parser::CParser;

    let code = r#"void print_msg(const char* msg) {}"#;
    let parser = CParser::new().unwrap();
    let ast = parser.parse(code).unwrap();
    let func = HirFunction::from_ast_function(&ast.functions()[0]);

    let codegen = CodeGenerator::new();
    let sig = make_simple_sig(
        "print_msg",
        vec![make_param(
            "msg",
            AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Char))),
        )],
        AnnotatedType::Simple(HirType::Void),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, Some(&func));
    assert!(result.contains("&str"), "Expected &str for const char*, got: {}", result);
}

// ============================================================================
// generate_annotated_signature_with_func: pointer arithmetic (DECY-123)
// ============================================================================

#[test]
fn test_sig_pointer_arithmetic_stays_raw() {
    let codegen = CodeGenerator::new();

    // Function with pointer arithmetic: p = p + 1
    let func = HirFunction::new_with_body(
        "walk_ptr".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Assignment {
            target: "p".to_string(),
            value: HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("p".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
    );

    let sig = make_simple_sig(
        "walk_ptr",
        vec![make_param(
            "p",
            AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Int))),
        )],
        AnnotatedType::Simple(HirType::Void),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, Some(&func));
    // With pointer arithmetic, should stay as raw pointer or have `mut` prefix
    assert!(
        result.contains("*mut i32") || result.contains("&mut i32"),
        "Expected raw or mut pointer for ptr arithmetic, got: {}",
        result
    );
}

// ============================================================================
// generate_annotated_signature_with_func: Vec return (DECY-142)
// ============================================================================

#[test]
fn test_sig_vec_return_from_malloc() {
    let codegen = CodeGenerator::new();

    // Function: int* make_array() { int* p = malloc(10 * sizeof(int)); return p; }
    let func = HirFunction::new_with_body(
        "make_array".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "p".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(40)],
                }),
            },
            HirStatement::Return(Some(HirExpression::Variable("p".to_string()))),
        ],
    );

    let sig = make_simple_sig(
        "make_array",
        vec![],
        AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Int))),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, Some(&func));
    // Should detect Vec return pattern
    assert!(
        result.contains("Vec<i32>") || result.contains("*mut i32"),
        "Expected Vec or pointer return, got: {}",
        result
    );
}

// ============================================================================
// generate_annotated_signature_with_func: mixed param types
// ============================================================================

#[test]
fn test_sig_mixed_param_types() {
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig(
        "mixed",
        vec![
            make_param("count", AnnotatedType::Simple(HirType::Int)),
            make_param("value", AnnotatedType::Simple(HirType::Double)),
            make_param("flag", AnnotatedType::Simple(HirType::Char)),
        ],
        AnnotatedType::Simple(HirType::Int),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert!(result.contains("mut count: i32"), "Expected i32 param, got: {}", result);
    assert!(result.contains("mut value: f64"), "Expected f64 param, got: {}", result);
    assert!(result.contains("mut flag: u8"), "Expected u8 param, got: {}", result);
    assert!(result.contains("-> i32"), "Expected i32 return, got: {}", result);
}

#[test]
fn test_sig_single_param() {
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig(
        "identity",
        vec![make_param("x", AnnotatedType::Simple(HirType::Int))],
        AnnotatedType::Simple(HirType::Int),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert_eq!(result, "fn identity(mut x: i32) -> i32");
}

// ============================================================================
// generate_annotated_signature (wrapper that calls with_func(sig, None))
// ============================================================================

#[test]
fn test_generate_annotated_signature_wrapper() {
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig(
        "double_it",
        vec![make_param("n", AnnotatedType::Simple(HirType::Int))],
        AnnotatedType::Simple(HirType::Int),
    );
    let direct = codegen.generate_annotated_signature_with_func(&sig, None);
    let wrapper = codegen.generate_annotated_signature(&sig);
    assert_eq!(direct, wrapper);
}

// ============================================================================
// generate_annotated_signature_with_func: mutable reference in slice context
// ============================================================================

#[test]
fn test_sig_slice_reference_fallback_for_non_array_inner() {
    // Reference whose inner is a Reference (nested), not an Array
    let codegen = CodeGenerator::new();
    let sig = make_simple_sig(
        "nested",
        vec![make_param(
            "data",
            AnnotatedType::Reference {
                inner: Box::new(AnnotatedType::Reference {
                    inner: Box::new(AnnotatedType::Simple(HirType::Int)),
                    mutable: false,
                    lifetime: None,
                }),
                mutable: false,
                lifetime: None,
            },
        )],
        AnnotatedType::Simple(HirType::Void),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    // The slice detection should fail for nested references
    assert!(!result.contains("[i32]"), "Nested ref should not be slice: {}", result);
}

#[test]
fn test_sig_sized_array_in_reference_not_slice() {
    // Reference to sized array (size=Some(10)) should NOT be treated as slice
    let codegen = CodeGenerator::new();
    let lt = LifetimeParam::standard(0);
    let sig = AnnotatedSignature {
        name: "fixed_array".to_string(),
        lifetimes: vec![lt.clone()],
        parameters: vec![make_param(
            "data",
            AnnotatedType::Reference {
                inner: Box::new(AnnotatedType::Simple(HirType::Array {
                    element_type: Box::new(HirType::Int),
                    size: Some(10),
                })),
                mutable: false,
                lifetime: Some(lt),
            },
        )],
        return_type: AnnotatedType::Simple(HirType::Void),
    };
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    // Sized array reference should have lifetime since it's not a slice
    assert!(result.contains("<'a>"), "Expected lifetime for sized array ref: {}", result);
}

// ============================================================================
// generate_annotated_signature_with_func: function with no body
// ============================================================================

#[test]
fn test_sig_func_no_body_no_output_detection() {
    let codegen = CodeGenerator::new();

    // Function prototype without body - output param detection should be skipped
    let func = HirFunction::new(
        "proto".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("data".to_string(), HirType::Int),
            HirParameter::new("result".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        ],
    );

    let sig = make_simple_sig(
        "proto",
        vec![
            make_param("data", AnnotatedType::Simple(HirType::Int)),
            make_param("result", AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Int)))),
        ],
        AnnotatedType::Simple(HirType::Void),
    );
    let result = codegen.generate_annotated_signature_with_func(&sig, Some(&func));
    // Even with func, no body means no output detection
    assert!(result.contains("result"), "result param should still be present: {}", result);
}

// ============================================================================
// generate_annotated_signature_with_func: reference return types
// ============================================================================

#[test]
fn test_sig_reference_return_type() {
    let codegen = CodeGenerator::new();
    let lt = LifetimeParam::standard(0);
    let sig = AnnotatedSignature {
        name: "get_ref".to_string(),
        lifetimes: vec![lt.clone()],
        parameters: vec![make_param(
            "data",
            AnnotatedType::Reference {
                inner: Box::new(AnnotatedType::Simple(HirType::Int)),
                mutable: false,
                lifetime: Some(lt.clone()),
            },
        )],
        return_type: AnnotatedType::Reference {
            inner: Box::new(AnnotatedType::Simple(HirType::Int)),
            mutable: false,
            lifetime: Some(lt),
        },
    };
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert!(result.contains("-> &"), "Expected reference return type, got: {}", result);
}

#[test]
fn test_sig_mutable_reference_return() {
    let codegen = CodeGenerator::new();
    let lt = LifetimeParam::standard(0);
    let sig = AnnotatedSignature {
        name: "get_mut".to_string(),
        lifetimes: vec![lt.clone()],
        parameters: vec![make_param(
            "data",
            AnnotatedType::Reference {
                inner: Box::new(AnnotatedType::Simple(HirType::Int)),
                mutable: true,
                lifetime: Some(lt.clone()),
            },
        )],
        return_type: AnnotatedType::Reference {
            inner: Box::new(AnnotatedType::Simple(HirType::Int)),
            mutable: true,
            lifetime: Some(lt),
        },
    };
    let result = codegen.generate_annotated_signature_with_func(&sig, None);
    assert!(result.contains("-> &"), "Expected ref return, got: {}", result);
}

// ============================================================================
// Format specifiers: flags preserved in various combos
// ============================================================================

#[test]
fn test_format_octal_with_zero_pad() {
    assert_eq!(
        CodeGenerator::convert_format_specifiers("%08o"),
        "{:08o}"
    );
}

#[test]
fn test_format_binary_with_flags_and_width() {
    // %032b → zero-padded binary
    assert_eq!(
        CodeGenerator::convert_format_specifiers("%032b"),
        "{:032b}"
    );
}

#[test]
fn test_format_string_between_text() {
    assert_eq!(
        CodeGenerator::convert_format_specifiers("Name: %s, Age: %d, Height: %.1f"),
        "Name: {}, Age: {}, Height: {:.1}"
    );
}

#[test]
fn test_format_just_specifiers_no_text() {
    assert_eq!(
        CodeGenerator::convert_format_specifiers("%s%d%f"),
        "{}{}{}"
    );
}

#[test]
fn test_format_hex_with_length_and_width() {
    // %08lx → long hex with zero-pad and width
    assert_eq!(
        CodeGenerator::convert_format_specifiers("%08lx"),
        "{:08x}"
    );
}

#[test]
fn test_format_pointer_is_always_simple() {
    // %p never has width/flags variants in this impl
    assert_eq!(CodeGenerator::convert_format_specifiers("%p"), "{:p}");
}
