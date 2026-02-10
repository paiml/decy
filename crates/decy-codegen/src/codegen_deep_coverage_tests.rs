//! Deep coverage tests for codegen edge cases.
//!
//! Targets: convert_format_specifiers, default_value_for_type, map_sizeof_type,
//! StringMethodCall, IsNotNull, Calloc, TypeContext helpers.

use super::*;
use decy_hir::{HirExpression, HirType};

// ============================================================================
// convert_format_specifiers edge cases
// ============================================================================

#[test]
fn format_spec_basic_percent_d() {
    let result = CodeGenerator::convert_format_specifiers("hello %d world");
    assert_eq!(result, "hello {} world");
}

#[test]
fn format_spec_percent_percent() {
    let result = CodeGenerator::convert_format_specifiers("100%%");
    assert_eq!(result, "100%");
}

#[test]
fn format_spec_zero_padded_int() {
    let result = CodeGenerator::convert_format_specifiers("%02d");
    assert_eq!(result, "{:02}");
}

#[test]
fn format_spec_hex_lower() {
    let result = CodeGenerator::convert_format_specifiers("%x");
    assert_eq!(result, "{:x}");
}

#[test]
fn format_spec_hex_upper() {
    let result = CodeGenerator::convert_format_specifiers("%X");
    assert_eq!(result, "{:X}");
}

#[test]
fn format_spec_hex_with_width() {
    let result = CodeGenerator::convert_format_specifiers("%08X");
    assert_eq!(result, "{:08X}");
}

#[test]
fn format_spec_octal() {
    let result = CodeGenerator::convert_format_specifiers("%o");
    assert_eq!(result, "{:o}");
}

#[test]
fn format_spec_octal_with_width() {
    let result = CodeGenerator::convert_format_specifiers("%04o");
    assert_eq!(result, "{:04o}");
}

#[test]
fn format_spec_binary() {
    let result = CodeGenerator::convert_format_specifiers("%b");
    assert_eq!(result, "{:b}");
}

#[test]
fn format_spec_binary_with_width() {
    let result = CodeGenerator::convert_format_specifiers("%08b");
    assert_eq!(result, "{:08b}");
}

#[test]
fn format_spec_float_precision() {
    let result = CodeGenerator::convert_format_specifiers("%.3f");
    assert_eq!(result, "{:.3}");
}

#[test]
fn format_spec_float_width_and_precision() {
    let result = CodeGenerator::convert_format_specifiers("%10.3f");
    assert_eq!(result, "{:10.3}");
}

#[test]
fn format_spec_float_width_only() {
    let result = CodeGenerator::convert_format_specifiers("%10f");
    assert_eq!(result, "{:10}");
}

#[test]
fn format_spec_scientific_lower() {
    let result = CodeGenerator::convert_format_specifiers("%e");
    assert_eq!(result, "{:e}");
}

#[test]
fn format_spec_scientific_upper() {
    let result = CodeGenerator::convert_format_specifiers("%E");
    assert_eq!(result, "{:E}");
}

#[test]
fn format_spec_g_general() {
    let result = CodeGenerator::convert_format_specifiers("%g");
    assert_eq!(result, "{}");
}

#[test]
fn format_spec_string_basic() {
    let result = CodeGenerator::convert_format_specifiers("%s");
    assert_eq!(result, "{}");
}

#[test]
fn format_spec_string_with_width() {
    let result = CodeGenerator::convert_format_specifiers("%20s");
    assert_eq!(result, "{:20}");
}

#[test]
fn format_spec_char() {
    let result = CodeGenerator::convert_format_specifiers("%c");
    assert_eq!(result, "{}");
}

#[test]
fn format_spec_pointer() {
    let result = CodeGenerator::convert_format_specifiers("%p");
    assert_eq!(result, "{:p}");
}

#[test]
fn format_spec_unsigned() {
    let result = CodeGenerator::convert_format_specifiers("%u");
    assert_eq!(result, "{}");
}

#[test]
fn format_spec_length_modifier_long() {
    let result = CodeGenerator::convert_format_specifiers("%ld");
    assert_eq!(result, "{}");
}

#[test]
fn format_spec_length_modifier_long_long() {
    let result = CodeGenerator::convert_format_specifiers("%lld");
    assert_eq!(result, "{}");
}

#[test]
fn format_spec_length_modifier_size_t() {
    let result = CodeGenerator::convert_format_specifiers("%zu");
    assert_eq!(result, "{}");
}

#[test]
fn format_spec_multiple_specifiers() {
    let result = CodeGenerator::convert_format_specifiers("name=%s age=%d rate=%.2f");
    assert_eq!(result, "name={} age={} rate={:.2}");
}

#[test]
fn format_spec_no_specifiers() {
    let result = CodeGenerator::convert_format_specifiers("hello world");
    assert_eq!(result, "hello world");
}

#[test]
fn format_spec_incomplete_at_end() {
    let result = CodeGenerator::convert_format_specifiers("trailing %");
    assert_eq!(result, "trailing %");
}

#[test]
fn format_spec_zero_padded_hex() {
    let result = CodeGenerator::convert_format_specifiers("%02x");
    assert_eq!(result, "{:02x}");
}

// ============================================================================
// find_string_format_positions
// ============================================================================

#[test]
fn find_string_format_positions_basic() {
    let positions = CodeGenerator::find_string_format_positions("%d %s %d");
    assert_eq!(positions, vec![1]); // %s is at arg position 1
}

#[test]
fn find_string_format_positions_no_strings() {
    let positions = CodeGenerator::find_string_format_positions("%d %f %x");
    assert!(positions.is_empty());
}

#[test]
fn find_string_format_positions_multiple_strings() {
    let positions = CodeGenerator::find_string_format_positions("%s %d %s");
    assert_eq!(positions, vec![0, 2]);
}

#[test]
fn find_string_format_positions_percent_literal() {
    let positions = CodeGenerator::find_string_format_positions("%% %s");
    assert_eq!(positions, vec![0]); // %% is literal, %s is arg 0
}

#[test]
fn find_string_format_positions_quoted() {
    let positions = CodeGenerator::find_string_format_positions("\"%d %s\"");
    assert_eq!(positions, vec![1]);
}

// ============================================================================
// default_value_for_type
// ============================================================================

#[test]
fn default_value_void() {
    assert_eq!(CodeGenerator::default_value_for_type(&HirType::Void), "()");
}

#[test]
fn default_value_int() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::Int),
        "0i32"
    );
}

#[test]
fn default_value_unsigned_int() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::UnsignedInt),
        "0u32"
    );
}

#[test]
fn default_value_float() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::Float),
        "0.0f32"
    );
}

#[test]
fn default_value_double() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::Double),
        "0.0f64"
    );
}

#[test]
fn default_value_char() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::Char),
        "0u8"
    );
}

#[test]
fn default_value_signed_char() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::SignedChar),
        "0i8"
    );
}

#[test]
fn default_value_pointer() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::Pointer(Box::new(HirType::Int))),
        "std::ptr::null_mut()"
    );
}

#[test]
fn default_value_box() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::Box(Box::new(HirType::Int))),
        "Box::new(0i32)"
    );
}

#[test]
fn default_value_vec() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::Vec(Box::new(HirType::Int))),
        "Vec::new()"
    );
}

#[test]
fn default_value_option() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::Option(Box::new(HirType::Int))),
        "None"
    );
}

#[test]
fn default_value_struct() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::Struct("MyStruct".to_string())),
        "MyStruct::default()"
    );
}

#[test]
fn default_value_enum() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::Enum("MyEnum".to_string())),
        "MyEnum::default()"
    );
}

#[test]
fn default_value_array_sized() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        }),
        "[0i32; 10]"
    );
}

#[test]
fn default_value_function_pointer() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::FunctionPointer {
            param_types: vec![HirType::Int],
            return_type: Box::new(HirType::Int),
        }),
        "None"
    );
}

#[test]
fn default_value_string_literal() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::StringLiteral),
        "\"\""
    );
}

#[test]
fn default_value_owned_string() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::OwnedString),
        "String::new()"
    );
}

#[test]
fn default_value_string_reference() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::StringReference),
        "\"\""
    );
}

#[test]
fn default_value_type_alias_size_t() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::TypeAlias("size_t".to_string())),
        "0usize"
    );
}

#[test]
fn default_value_type_alias_ssize_t() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::TypeAlias("ssize_t".to_string())),
        "0isize"
    );
}

#[test]
fn default_value_type_alias_ptrdiff_t() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::TypeAlias("ptrdiff_t".to_string())),
        "0isize"
    );
}

#[test]
fn default_value_type_alias_custom() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::TypeAlias("my_type_t".to_string())),
        "0"
    );
}

// ============================================================================
// map_sizeof_type
// ============================================================================

#[test]
fn map_sizeof_type_basic_types() {
    let cg = CodeGenerator::new();
    assert_eq!(cg.map_sizeof_type("int"), "i32");
    assert_eq!(cg.map_sizeof_type("short"), "i16");
    assert_eq!(cg.map_sizeof_type("long"), "i64");
    assert_eq!(cg.map_sizeof_type("long long"), "i64");
    assert_eq!(cg.map_sizeof_type("float"), "f32");
    assert_eq!(cg.map_sizeof_type("double"), "f64");
    assert_eq!(cg.map_sizeof_type("char"), "u8");
    assert_eq!(cg.map_sizeof_type("void"), "()");
}

#[test]
fn map_sizeof_type_unsigned() {
    let cg = CodeGenerator::new();
    assert_eq!(cg.map_sizeof_type("unsigned int"), "u32");
    assert_eq!(cg.map_sizeof_type("unsigned"), "u32");
    assert_eq!(cg.map_sizeof_type("unsigned short"), "u16");
    assert_eq!(cg.map_sizeof_type("unsigned long"), "u64");
    assert_eq!(cg.map_sizeof_type("unsigned long long"), "u64");
    assert_eq!(cg.map_sizeof_type("unsigned char"), "u8");
}

#[test]
fn map_sizeof_type_signed_char() {
    let cg = CodeGenerator::new();
    assert_eq!(cg.map_sizeof_type("signed char"), "i8");
}

#[test]
fn map_sizeof_type_pointers() {
    let cg = CodeGenerator::new();
    assert_eq!(cg.map_sizeof_type("char*"), "*mut u8");
    assert_eq!(cg.map_sizeof_type("int*"), "*mut i32");
    assert_eq!(cg.map_sizeof_type("void*"), "*mut ()");
    assert_eq!(cg.map_sizeof_type("char *"), "*mut u8");
    assert_eq!(cg.map_sizeof_type("int *"), "*mut i32");
    assert_eq!(cg.map_sizeof_type("void *"), "*mut ()");
}

#[test]
fn map_sizeof_type_struct() {
    let cg = CodeGenerator::new();
    assert_eq!(cg.map_sizeof_type("struct Point"), "Point");
}

#[test]
fn map_sizeof_type_custom() {
    let cg = CodeGenerator::new();
    assert_eq!(cg.map_sizeof_type("MyType"), "MyType");
}

#[test]
fn map_sizeof_type_trimming() {
    let cg = CodeGenerator::new();
    assert_eq!(cg.map_sizeof_type("  int  "), "i32");
}

// ============================================================================
// StringMethodCall codegen
// ============================================================================

#[test]
fn string_method_call_len() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("arr".to_string())),
        method: "len".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression(&expr);
    assert_eq!(code, "arr.len() as i32");
}

#[test]
fn string_method_call_non_len() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("s".to_string())),
        method: "is_empty".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression(&expr);
    assert_eq!(code, "s.is_empty()");
}

#[test]
fn string_method_call_clone_into() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("src".to_string())),
        method: "clone_into".to_string(),
        arguments: vec![HirExpression::Variable("dst".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert_eq!(code, "src.clone_into(&mut dst)");
}

#[test]
fn string_method_call_with_args() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("v".to_string())),
        method: "push".to_string(),
        arguments: vec![HirExpression::IntLiteral(42)],
    };
    let code = cg.generate_expression(&expr);
    assert_eq!(code, "v.push(42)");
}

// ============================================================================
// IsNotNull codegen
// ============================================================================

#[test]
fn is_not_null_codegen() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::IsNotNull(Box::new(HirExpression::Variable("ptr".to_string())));
    let code = cg.generate_expression(&expr);
    assert_eq!(code, "if let Some(_) = ptr");
}

// ============================================================================
// NullLiteral codegen
// ============================================================================

#[test]
fn null_literal_codegen() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::NullLiteral;
    let code = cg.generate_expression(&expr);
    assert_eq!(code, "None");
}

// ============================================================================
// Calloc codegen
// ============================================================================

#[test]
fn calloc_int_codegen() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(10)),
        element_type: Box::new(HirType::Int),
    };
    let code = cg.generate_expression(&expr);
    assert_eq!(code, "vec![0i32; 10]");
}

#[test]
fn calloc_unsigned_int_codegen() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(5)),
        element_type: Box::new(HirType::UnsignedInt),
    };
    let code = cg.generate_expression(&expr);
    assert_eq!(code, "vec![0u32; 5]");
}

#[test]
fn calloc_float_codegen() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(3)),
        element_type: Box::new(HirType::Float),
    };
    let code = cg.generate_expression(&expr);
    assert_eq!(code, "vec![0.0f32; 3]");
}

#[test]
fn calloc_double_codegen() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(3)),
        element_type: Box::new(HirType::Double),
    };
    let code = cg.generate_expression(&expr);
    assert_eq!(code, "vec![0.0f64; 3]");
}

#[test]
fn calloc_char_codegen() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::Variable("n".to_string())),
        element_type: Box::new(HirType::Char),
    };
    let code = cg.generate_expression(&expr);
    assert_eq!(code, "vec![0u8; n]");
}

#[test]
fn calloc_signed_char_codegen() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(4)),
        element_type: Box::new(HirType::SignedChar),
    };
    let code = cg.generate_expression(&expr);
    assert_eq!(code, "vec![0i8; 4]");
}

// ============================================================================
// TypeContext helpers
// ============================================================================

#[test]
fn type_context_string_iter_func() {
    let mut ctx = TypeContext::new();
    ctx.add_string_iter_func("process_str".to_string(), vec![(0, true)]);
    assert!(ctx.get_string_iter_func("process_str").is_some());
    let params = ctx.get_string_iter_func("process_str").unwrap();
    assert_eq!(params.len(), 1);
}

#[test]
fn type_context_slice_func_args() {
    let mut ctx = TypeContext::new();
    ctx.add_slice_func_args("sum_array".to_string(), vec![(0, 1)]);
    let indices = ctx.get_slice_func_len_indices("sum_array");
    assert!(indices.is_some());
    assert_eq!(indices.unwrap(), &[(0, 1)]);
}

#[test]
fn type_context_slice_func_args_missing() {
    let ctx = TypeContext::new();
    assert!(ctx.get_slice_func_len_indices("nonexistent").is_none());
}

#[test]
fn type_context_string_iter_func_none() {
    let ctx = TypeContext::new();
    assert!(ctx.get_string_iter_func("nonexistent").is_none());
}

#[test]
fn type_context_string_iter_param() {
    let mut ctx = TypeContext::new();
    ctx.add_string_iter_param("str_param".to_string(), "i".to_string());
    assert!(ctx.is_string_iter_param("str_param"));
    assert!(!ctx.is_string_iter_param("other"));
    assert_eq!(ctx.get_string_iter_index("str_param"), Some(&"i".to_string()));
}

#[test]
fn type_context_global_tracking() {
    let mut ctx = TypeContext::new();
    ctx.add_global("g_counter".to_string());
    assert!(ctx.is_global("g_counter"));
    assert!(!ctx.is_global("local_var"));
}

#[test]
fn type_context_function_param_type() {
    let mut ctx = TypeContext::new();
    ctx.add_function("process".to_string(), vec![HirType::Int, HirType::Float]);
    assert_eq!(
        ctx.get_function_param_type("process", 0),
        Some(&HirType::Int)
    );
    assert_eq!(
        ctx.get_function_param_type("process", 1),
        Some(&HirType::Float)
    );
    assert!(ctx.get_function_param_type("process", 2).is_none());
    assert!(ctx.get_function_param_type("unknown", 0).is_none());
}
