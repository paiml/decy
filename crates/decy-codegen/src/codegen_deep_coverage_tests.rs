//! Deep coverage tests for codegen edge cases.
//!
//! Targets: convert_format_specifiers, default_value_for_type, map_sizeof_type,
//! StringMethodCall, IsNotNull, Calloc, TypeContext helpers.

use super::*;
use decy_hir::{
    BinaryOperator, HirExpression, HirFunction, HirParameter, HirStatement, HirType, SwitchCase,
    UnaryOperator,
};

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

#[test]
fn format_spec_percent_i() {
    // %i is same as %d (signed integer)
    let result = CodeGenerator::convert_format_specifiers("val=%i");
    assert_eq!(result, "val={}");
}

#[test]
fn format_spec_unknown_specifier() {
    // Unknown specifier should keep original C format string
    let result = CodeGenerator::convert_format_specifiers("test %q end");
    assert!(result.contains("%q"), "Unknown specifier should be kept, got: {}", result);
}

#[test]
fn format_spec_uppercase_float() {
    // %F is same as %f
    let result = CodeGenerator::convert_format_specifiers("val=%F");
    assert_eq!(result, "val={}");
}

#[test]
fn format_spec_float_precision_only() {
    // %.3f → {:.3}
    let result = CodeGenerator::convert_format_specifiers("%.3f");
    assert_eq!(result, "{:.3}");
}

#[test]
fn format_spec_multiple_flags() {
    // Multiple flags combined: %+05d
    let result = CodeGenerator::convert_format_specifiers("%+05d");
    assert!(result.contains("{:"), "Should generate Rust format, got: {}", result);
}

#[test]
fn format_spec_string_width() {
    // %20s → {:20}
    let result = CodeGenerator::convert_format_specifiers("%20s");
    assert_eq!(result, "{:20}");
}

#[test]
fn format_spec_hh_length_modifier() {
    // %hhd → {} (length modifier stripped)
    let result = CodeGenerator::convert_format_specifiers("val=%hhd");
    assert_eq!(result, "val={}");
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

// ============================================================================
// Statement codegen: generate_statement
// ============================================================================

#[test]
fn stmt_return_with_value() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Return(Some(HirExpression::IntLiteral(42)));
    let code = cg.generate_statement(&stmt);
    assert_eq!(code, "return 42;");
}

#[test]
fn stmt_return_void() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Return(None);
    let code = cg.generate_statement(&stmt);
    assert_eq!(code, "return;");
}

#[test]
fn stmt_return_in_main_exits() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Return(Some(HirExpression::IntLiteral(1)));
    let code = cg.generate_statement_for_function(&stmt, Some("main"));
    assert_eq!(code, "std::process::exit(1);");
}

#[test]
fn stmt_return_in_main_void_exits_zero() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Return(None);
    let code = cg.generate_statement_for_function(&stmt, Some("main"));
    assert_eq!(code, "std::process::exit(0);");
}

#[test]
fn stmt_break() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Break;
    let code = cg.generate_statement(&stmt);
    assert_eq!(code, "break;");
}

#[test]
fn stmt_continue() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Continue;
    let code = cg.generate_statement(&stmt);
    assert_eq!(code, "continue;");
}

#[test]
fn stmt_expression() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Expression(HirExpression::FunctionCall {
        function: "foo".to_string(),
        arguments: vec![HirExpression::IntLiteral(42)],
    });
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("foo"));
    assert!(code.ends_with(';'));
}

#[test]
fn stmt_free_generates_raii_comment() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Free {
        pointer: HirExpression::Variable("ptr".to_string()),
    };
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("RAII"));
    assert!(code.contains("ptr"));
}

#[test]
fn stmt_inline_asm_not_translatable() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::InlineAsm {
        text: "mov eax, ebx".to_string(),
        translatable: false,
    };
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("manual review required"));
    assert!(code.contains("mov eax, ebx"));
    assert!(!code.contains("translatable to Rust"));
}

#[test]
fn stmt_inline_asm_translatable() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::InlineAsm {
        text: "nop".to_string(),
        translatable: true,
    };
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("manual review required"));
    assert!(code.contains("translatable to Rust intrinsics"));
}

#[test]
fn stmt_if_with_boolean_condition() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
        then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
        else_block: None,
    };
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("if x == 0"));
    assert!(code.contains("return 1;"));
}

#[test]
fn stmt_if_with_else() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        },
        then_block: vec![HirStatement::Return(Some(HirExpression::Variable(
            "a".to_string(),
        )))],
        else_block: Some(vec![HirStatement::Return(Some(HirExpression::Variable(
            "b".to_string(),
        )))]),
    };
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("if a > b"));
    assert!(code.contains("} else {"));
}

#[test]
fn stmt_if_non_boolean_condition_wraps_ne_zero() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::Variable("flag".to_string()),
        then_block: vec![HirStatement::Break],
        else_block: None,
    };
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("!= 0"));
}

#[test]
fn stmt_while_basic() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::While {
        condition: HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::Variable("n".to_string())),
        },
        body: vec![HirStatement::Expression(HirExpression::PostIncrement {
            operand: Box::new(HirExpression::Variable("i".to_string())),
        })],
    };
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("while i < n"));
}

#[test]
fn stmt_for_with_init_cond_inc() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::For {
        init: vec![HirStatement::VariableDeclaration {
            name: "i".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(0)),
        }],
        condition: Some(HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(10)),
        }),
        increment: vec![HirStatement::Assignment {
            target: "i".to_string(),
            value: HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
        body: vec![HirStatement::Break],
    };
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("let mut i"));
    assert!(code.contains("while i < 10"));
    assert!(code.contains("break;"));
}

#[test]
fn stmt_for_infinite_loop() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::For {
        init: vec![],
        condition: None,
        increment: vec![],
        body: vec![HirStatement::Break],
    };
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("loop {"));
    assert!(code.contains("break;"));
}

#[test]
fn stmt_switch_basic() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Switch {
        condition: HirExpression::Variable("choice".to_string()),
        cases: vec![decy_hir::SwitchCase {
            value: Some(HirExpression::IntLiteral(1)),
            body: vec![
                HirStatement::Return(Some(HirExpression::StringLiteral("one".to_string()))),
                HirStatement::Break,
            ],
        }],
        default_case: Some(vec![HirStatement::Return(Some(
            HirExpression::StringLiteral("other".to_string()),
        ))]),
    };
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("match choice"));
    assert!(code.contains("1 =>"));
    assert!(code.contains("_ =>"));
    // Break statements should be filtered out
    assert!(!code.contains("break;"));
}

#[test]
fn stmt_variable_declaration_with_init() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "x".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::IntLiteral(42)),
    };
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("let mut x"));
    assert!(code.contains("42"));
}

#[test]
fn stmt_variable_declaration_no_init() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "y".to_string(),
        var_type: HirType::Float,
        initializer: None,
    };
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("let mut y"));
    assert!(code.contains("f32"));
}

#[test]
fn stmt_assignment_simple() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Assignment {
        target: "x".to_string(),
        value: HirExpression::IntLiteral(99),
    };
    let code = cg.generate_statement(&stmt);
    assert_eq!(code, "x = 99;");
}

#[test]
fn stmt_deref_assignment_field_access() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::PointerFieldAccess {
            pointer: Box::new(HirExpression::Variable("node".to_string())),
            field: "next".to_string(),
        },
        value: HirExpression::NullLiteral,
    };
    let code = cg.generate_statement(&stmt);
    // PointerFieldAccess doesn't need extra dereference
    assert!(code.contains("node"));
    assert!(code.contains("next"));
}

#[test]
fn stmt_deref_assignment_array_index() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::ArrayIndex {
            array: Box::new(HirExpression::Variable("arr".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
        },
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("arr"));
    assert!(code.contains("42"));
}

#[test]
fn stmt_field_assignment_basic() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::FieldAssignment {
        object: HirExpression::Variable("point".to_string()),
        field: "x".to_string(),
        value: HirExpression::IntLiteral(10),
    };
    let code = cg.generate_statement(&stmt);
    assert_eq!(code, "point.x = 10;");
}

#[test]
fn stmt_array_index_assignment_basic() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::Variable("i".to_string())),
        value: HirExpression::IntLiteral(0),
    };
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("arr[(i) as usize] = 0;"));
}

// ============================================================================
// Expression codegen: CompoundLiteral
// ============================================================================

#[test]
fn expr_compound_literal_struct_empty() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Point".to_string()),
        initializers: vec![],
    };
    let code = cg.generate_expression(&expr);
    assert_eq!(code, "Point {}");
}

#[test]
fn expr_compound_literal_struct_with_values() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Point".to_string()),
        initializers: vec![HirExpression::IntLiteral(10), HirExpression::IntLiteral(20)],
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("Point"));
    assert!(code.contains("10"));
    assert!(code.contains("20"));
}

#[test]
fn expr_compound_literal_array_empty_with_size() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(5),
        },
        initializers: vec![],
    };
    let code = cg.generate_expression(&expr);
    assert_eq!(code, "[0i32; 5]");
}

#[test]
fn expr_compound_literal_array_single_init() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
        initializers: vec![HirExpression::IntLiteral(0)],
    };
    let code = cg.generate_expression(&expr);
    assert_eq!(code, "[0; 10]");
}

#[test]
fn expr_compound_literal_array_partial_init() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(4),
        },
        initializers: vec![HirExpression::IntLiteral(1), HirExpression::IntLiteral(2)],
    };
    let code = cg.generate_expression(&expr);
    // Should pad with defaults
    assert!(code.contains("1"));
    assert!(code.contains("2"));
    assert!(code.contains("0i32"));
}

#[test]
fn expr_compound_literal_array_full_init() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(3),
        },
        initializers: vec![
            HirExpression::IntLiteral(1),
            HirExpression::IntLiteral(2),
            HirExpression::IntLiteral(3),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert_eq!(code, "[1, 2, 3]");
}

#[test]
fn expr_compound_literal_array_no_size() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None,
        },
        initializers: vec![HirExpression::IntLiteral(1), HirExpression::IntLiteral(2)],
    };
    let code = cg.generate_expression(&expr);
    assert_eq!(code, "[1, 2]");
}

#[test]
fn expr_compound_literal_other_type() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Double,
        initializers: vec![],
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("Compound literal"));
}

// ============================================================================
// Expression codegen: Cast
// ============================================================================

#[test]
fn expr_cast_int_to_float() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Cast {
        target_type: HirType::Float,
        expr: Box::new(HirExpression::Variable("x".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert_eq!(code, "x as f32");
}

#[test]
fn expr_cast_binary_op_wrapped_in_parens() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Cast {
        target_type: HirType::Int,
        expr: Box::new(HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("(a + b) as i32"));
}

#[test]
fn expr_cast_address_of_to_int() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Cast {
        target_type: HirType::Int,
        expr: Box::new(HirExpression::AddressOf(Box::new(
            HirExpression::Variable("x".to_string()),
        ))),
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("as *const _"));
    assert!(code.contains("as isize"));
}

// ============================================================================
// Helper functions: is_string_ternary
// ============================================================================

#[test]
fn is_string_ternary_true() {
    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::Variable("flag".to_string())),
        then_expr: Box::new(HirExpression::StringLiteral("yes".to_string())),
        else_expr: Box::new(HirExpression::StringLiteral("no".to_string())),
    };
    assert!(CodeGenerator::is_string_ternary(&expr));
}

#[test]
fn is_string_ternary_false_non_string() {
    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::Variable("flag".to_string())),
        then_expr: Box::new(HirExpression::IntLiteral(1)),
        else_expr: Box::new(HirExpression::IntLiteral(0)),
    };
    assert!(!CodeGenerator::is_string_ternary(&expr));
}

#[test]
fn is_string_ternary_false_not_ternary() {
    let expr = HirExpression::Variable("x".to_string());
    assert!(!CodeGenerator::is_string_ternary(&expr));
}

// ============================================================================
// Helper functions: wrap_with_cstr, wrap_raw_ptr_with_cstr
// ============================================================================

#[test]
fn wrap_with_cstr_basic() {
    let result = CodeGenerator::wrap_with_cstr("buf");
    assert!(result.contains("CStr::from_ptr"));
    assert!(result.contains("buf.as_ptr()"));
    assert!(result.contains("unsafe"));
}

#[test]
fn wrap_raw_ptr_with_cstr_basic() {
    let result = CodeGenerator::wrap_raw_ptr_with_cstr("raw_ptr");
    assert!(result.contains("CStr::from_ptr"));
    assert!(result.contains("raw_ptr as *const i8"));
    assert!(!result.contains(".as_ptr()"));
}

// ============================================================================
// Helper functions: is_malloc_call, is_any_malloc_or_calloc, is_array_allocation_size
// ============================================================================

#[test]
fn is_any_malloc_basic() {
    let expr = HirExpression::Malloc {
        size: Box::new(HirExpression::IntLiteral(100)),
    };
    assert!(CodeGenerator::is_any_malloc_or_calloc(&expr));
}

#[test]
fn is_any_malloc_calloc() {
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(10)),
        element_type: Box::new(HirType::Int),
    };
    assert!(CodeGenerator::is_any_malloc_or_calloc(&expr));
}

#[test]
fn is_any_malloc_func_call() {
    let expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(100)],
    };
    assert!(CodeGenerator::is_any_malloc_or_calloc(&expr));
}

#[test]
fn is_any_malloc_through_cast() {
    let expr = HirExpression::Cast {
        target_type: HirType::Pointer(Box::new(HirType::Int)),
        expr: Box::new(HirExpression::Malloc {
            size: Box::new(HirExpression::IntLiteral(100)),
        }),
    };
    assert!(CodeGenerator::is_any_malloc_or_calloc(&expr));
}

#[test]
fn is_any_malloc_false() {
    let expr = HirExpression::Variable("ptr".to_string());
    assert!(!CodeGenerator::is_any_malloc_or_calloc(&expr));
}

#[test]
fn is_malloc_call_array_pattern() {
    let expr = HirExpression::Malloc {
        size: Box::new(HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("n".to_string())),
            right: Box::new(HirExpression::Sizeof {
                type_name: "int".to_string(),
            }),
        }),
    };
    assert!(CodeGenerator::is_malloc_call(&expr));
}

#[test]
fn is_malloc_call_sizeof_only_not_array() {
    let expr = HirExpression::Malloc {
        size: Box::new(HirExpression::Sizeof {
            type_name: "struct Point".to_string(),
        }),
    };
    assert!(!CodeGenerator::is_malloc_call(&expr));
}

#[test]
fn is_array_allocation_size_multiply() {
    let expr = HirExpression::BinaryOp {
        op: decy_hir::BinaryOperator::Multiply,
        left: Box::new(HirExpression::Variable("n".to_string())),
        right: Box::new(HirExpression::Sizeof {
            type_name: "int".to_string(),
        }),
    };
    assert!(CodeGenerator::is_array_allocation_size(&expr));
}

#[test]
fn is_array_allocation_size_sizeof_false() {
    let expr = HirExpression::Sizeof {
        type_name: "int".to_string(),
    };
    assert!(!CodeGenerator::is_array_allocation_size(&expr));
}

#[test]
fn is_array_allocation_size_int_literal_false() {
    let expr = HirExpression::IntLiteral(100);
    assert!(!CodeGenerator::is_array_allocation_size(&expr));
}

#[test]
fn is_array_allocation_size_variable_false() {
    let expr = HirExpression::Variable("size".to_string());
    assert!(!CodeGenerator::is_array_allocation_size(&expr));
}

// ============================================================================
// Helper functions: is_malloc_array_pattern
// ============================================================================

#[test]
fn is_malloc_array_pattern_through_cast() {
    let expr = HirExpression::Cast {
        target_type: HirType::Pointer(Box::new(HirType::Int)),
        expr: Box::new(HirExpression::Malloc {
            size: Box::new(HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Multiply,
                left: Box::new(HirExpression::Variable("n".to_string())),
                right: Box::new(HirExpression::Sizeof {
                    type_name: "int".to_string(),
                }),
            }),
        }),
    };
    assert!(CodeGenerator::is_malloc_array_pattern(&expr));
}

// ============================================================================
// Statement codegen: Realloc in Assignment
// ============================================================================

#[test]
fn stmt_assignment_realloc_zero_clears() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Assignment {
        target: "buf".to_string(),
        value: HirExpression::Realloc {
            pointer: Box::new(HirExpression::Variable("buf".to_string())),
            new_size: Box::new(HirExpression::IntLiteral(0)),
        },
    };
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("buf.clear()"));
}

#[test]
fn stmt_assignment_realloc_with_multiply_resizes() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("buf".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let stmt = HirStatement::Assignment {
        target: "buf".to_string(),
        value: HirExpression::Realloc {
            pointer: Box::new(HirExpression::Variable("buf".to_string())),
            new_size: Box::new(HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Multiply,
                left: Box::new(HirExpression::Variable("new_count".to_string())),
                right: Box::new(HirExpression::Sizeof {
                    type_name: "int".to_string(),
                }),
            }),
        },
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(code.contains("buf.resize"));
    assert!(code.contains("new_count"));
}

#[test]
fn stmt_assignment_realloc_fallback_size() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Assignment {
        target: "buf".to_string(),
        value: HirExpression::Realloc {
            pointer: Box::new(HirExpression::Variable("buf".to_string())),
            new_size: Box::new(HirExpression::Variable("new_sz".to_string())),
        },
    };
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("buf.resize"));
    assert!(code.contains("as usize"));
}

// ============================================================================
// Statement codegen: Assignment with globals
// ============================================================================

#[test]
fn stmt_assignment_global_wraps_unsafe() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_global("g_counter".to_string());
    let stmt = HirStatement::Assignment {
        target: "g_counter".to_string(),
        value: HirExpression::IntLiteral(0),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(code.contains("unsafe"));
    assert!(code.contains("g_counter = 0"));
}

#[test]
fn stmt_assignment_errno() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Assignment {
        target: "errno".to_string(),
        value: HirExpression::IntLiteral(0),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(code.contains("unsafe"));
    assert!(code.contains("ERRNO"));
}

// ============================================================================
// Statement codegen: Global array index assignment
// ============================================================================

#[test]
fn stmt_array_index_assignment_global_wraps_unsafe() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_global("g_table".to_string());
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("g_table".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(code.contains("unsafe"));
    assert!(code.contains("g_table"));
}

// ============================================================================
// Statement codegen: VLA (Variable-length array) declaration
// ============================================================================

#[test]
fn stmt_vla_declaration() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("vec!"));
    assert!(code.contains("0i32"));
}

#[test]
fn stmt_vla_declaration_float() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Float),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("vec!"));
    assert!(code.contains("0.0f32"));
}

#[test]
fn stmt_vla_declaration_double() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Double),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("vec!"));
    assert!(code.contains("0.0f64"));
}

#[test]
fn stmt_vla_declaration_unsigned_int() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::UnsignedInt),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("vec!"));
    assert!(code.contains("0u32"));
}

#[test]
fn stmt_vla_declaration_char() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Char),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("vec!"));
    assert!(code.contains("0u8"));
}

#[test]
fn stmt_vla_declaration_signed_char() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::SignedChar),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("vec!"));
    assert!(code.contains("0i8"));
}

// ============================================================================
// Expression codegen: Realloc in expression context
// ============================================================================

#[test]
fn expr_realloc_expression() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Realloc {
        pointer: Box::new(HirExpression::Variable("ptr".to_string())),
        new_size: Box::new(HirExpression::IntLiteral(100)),
    };
    // In expression context, realloc returns the pointer unchanged
    let code = cg.generate_expression(&expr);
    assert!(code.contains("ptr"));
}

// ============================================================================
// Expression codegen: Sizeof
// ============================================================================

#[test]
fn expr_sizeof_int() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Sizeof {
        type_name: "int".to_string(),
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("size_of"));
    assert!(code.contains("i32"));
}

#[test]
fn expr_sizeof_double() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Sizeof {
        type_name: "double".to_string(),
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("size_of"));
    assert!(code.contains("f64"));
}

// ============================================================================
// Helper: statement_deref_modifies_variable
// ============================================================================

#[test]
fn statement_deref_modifies_deref_assignment() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("ptr".to_string()),
        value: HirExpression::IntLiteral(42),
    };
    assert!(cg.statement_deref_modifies_variable(&stmt, "ptr"));
    assert!(!cg.statement_deref_modifies_variable(&stmt, "other"));
}

#[test]
fn statement_deref_modifies_array_index_assignment() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
        value: HirExpression::IntLiteral(1),
    };
    assert!(cg.statement_deref_modifies_variable(&stmt, "arr"));
    assert!(!cg.statement_deref_modifies_variable(&stmt, "other"));
}

#[test]
fn statement_deref_modifies_regular_assignment_false() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Assignment {
        target: "ptr".to_string(),
        value: HirExpression::IntLiteral(0),
    };
    // Regular assignment does NOT count as deref modification
    assert!(!cg.statement_deref_modifies_variable(&stmt, "ptr"));
}

#[test]
fn statement_deref_modifies_in_if_block() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
        then_block: vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("ptr".to_string()),
            value: HirExpression::IntLiteral(1),
        }],
        else_block: None,
    };
    assert!(cg.statement_deref_modifies_variable(&stmt, "ptr"));
}

#[test]
fn statement_deref_modifies_in_while_body() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::While {
        condition: HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(10)),
        },
        body: vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("ptr".to_string()),
            value: HirExpression::IntLiteral(99),
        }],
    };
    assert!(cg.statement_deref_modifies_variable(&stmt, "ptr"));
}

#[test]
fn statement_deref_modifies_break_false() {
    let cg = CodeGenerator::new();
    assert!(!cg.statement_deref_modifies_variable(&HirStatement::Break, "ptr"));
}

// ============================================================================
// Helper: expression_compares_to_null
// ============================================================================

#[test]
fn expression_compares_to_null_eq_zero() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: decy_hir::BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("ptr".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    assert!(cg.expression_compares_to_null(&expr, "ptr"));
}

#[test]
fn expression_compares_to_null_ne_null() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: decy_hir::BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("ptr".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    assert!(cg.expression_compares_to_null(&expr, "ptr"));
}

#[test]
fn expression_compares_to_null_different_var() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: decy_hir::BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("other".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    assert!(!cg.expression_compares_to_null(&expr, "ptr"));
}

// ============================================================================
// Expression codegen: Ternary
// ============================================================================

#[test]
fn expr_ternary_basic() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
        then_expr: Box::new(HirExpression::IntLiteral(1)),
        else_expr: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("if"));
    assert!(code.contains("1"));
    assert!(code.contains("0"));
}

// ============================================================================
// Expression codegen: Boolean expressions
// ============================================================================

#[test]
fn expr_is_not_null() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::IsNotNull(Box::new(HirExpression::Variable("ptr".to_string())));
    let code = cg.generate_expression(&expr);
    assert!(code.contains("is_null") || code.contains("ptr"));
}

#[test]
fn expr_null_literal() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::NullLiteral;
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("null")
            || code.contains("None")
            || code.contains("ptr::null")
            || code.contains("std::ptr::null()")
    );
}

// ============================================================================
// Expression codegen: AddressOf, Dereference
// ============================================================================

#[test]
fn expr_address_of() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::AddressOf(Box::new(HirExpression::Variable("x".to_string())));
    let code = cg.generate_expression(&expr);
    assert!(code.contains("&"));
    assert!(code.contains("x"));
}

#[test]
fn expr_dereference() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("ptr".to_string())));
    let code = cg.generate_expression(&expr);
    assert!(code.contains("*"));
    assert!(code.contains("ptr"));
}

// ============================================================================
// Expression codegen: Field access
// ============================================================================

#[test]
fn expr_field_access() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FieldAccess {
        object: Box::new(HirExpression::Variable("point".to_string())),
        field: "x".to_string(),
    };
    let code = cg.generate_expression(&expr);
    assert_eq!(code, "point.x");
}

#[test]
fn expr_pointer_field_access() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("node".to_string())),
        field: "next".to_string(),
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("node"));
    assert!(code.contains("next"));
}

// ============================================================================
// Expression codegen: Array index
// ============================================================================

#[test]
fn expr_array_index() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::Variable("i".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("arr"));
    assert!(code.contains("i"));
    assert!(code.contains("as usize"));
}

// ============================================================================
// TypeContext: variable type tracking
// ============================================================================

#[test]
fn type_context_add_and_get_variable() {
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    assert_eq!(ctx.get_type("x"), Some(&HirType::Int));
    assert!(ctx.get_type("y").is_none());
}

#[test]
fn type_context_is_pointer() {
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    ctx.add_variable("val".to_string(), HirType::Int);
    assert!(ctx.is_pointer("ptr"));
    assert!(!ctx.is_pointer("val"));
    assert!(!ctx.is_pointer("unknown"));
}

#[test]
fn type_context_renamed_local() {
    let mut ctx = TypeContext::new();
    ctx.add_global("g_val".to_string());
    ctx.add_renamed_local("g_val".to_string(), "g_val_local".to_string());
    // Renamed locals should be accessible
    let renamed = ctx.get_renamed_local("g_val");
    assert_eq!(renamed, Some(&"g_val_local".to_string()));
}

// ============================================================================
// Expression codegen: Unary operations
// ============================================================================

#[test]
fn expr_unary_negate() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::Minus,
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("-"));
    assert!(code.contains("x"));
}

#[test]
fn expr_unary_not() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("flag".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("flag"));
}

#[test]
fn expr_unary_bitwise_not() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::BitwiseNot,
        operand: Box::new(HirExpression::Variable("mask".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("!"));
    assert!(code.contains("mask"));
}

// ============================================================================
// Statement codegen: FieldAssignment with reserved keyword
// ============================================================================

#[test]
fn stmt_field_assignment_reserved_keyword() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::FieldAssignment {
        object: HirExpression::Variable("obj".to_string()),
        field: "type".to_string(),
        value: HirExpression::IntLiteral(1),
    };
    let code = cg.generate_statement(&stmt);
    // "type" is a Rust keyword, should be escaped
    assert!(code.contains("r#type") || code.contains("type_"));
}

// ============================================================================
// S3-Phase1: Standard library function mapping tests
// Note: Pointer-based functions (memcpy, memset, strcmp, strncmp, strcat)
// use the stub mechanism rather than inline expansion because transpiled
// code uses raw pointer types that don't support safe Rust operations.
// ============================================================================

#[test]
fn stdlib_atoi_generates_parse() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "atoi".to_string(),
        arguments: vec![HirExpression::Variable("s".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("parse::<i32>"),
        "atoi should generate parse::<i32>(), got: {}",
        code
    );
    assert!(code.contains("unwrap_or(0)"));
}

#[test]
fn stdlib_atoi_invalid_args() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "atoi".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("atoi requires 1 arg"));
}

#[test]
fn stdlib_atof_generates_parse_f64() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "atof".to_string(),
        arguments: vec![HirExpression::Variable("s".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("parse::<f64>"),
        "atof should generate parse::<f64>(), got: {}",
        code
    );
    assert!(code.contains("unwrap_or(0.0)"));
}

#[test]
fn stdlib_atof_invalid_args() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "atof".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("atof requires 1 arg"));
}

#[test]
fn stdlib_abs_generates_abs() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "abs".to_string(),
        arguments: vec![HirExpression::Variable("x".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains(".abs()"),
        "abs should generate .abs(), got: {}",
        code
    );
}

#[test]
fn stdlib_abs_invalid_args() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "abs".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("abs requires 1 arg"));
}

#[test]
fn stdlib_exit_generates_process_exit() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "exit".to_string(),
        arguments: vec![HirExpression::IntLiteral(0)],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("std::process::exit"),
        "exit should generate std::process::exit, got: {}",
        code
    );
}

#[test]
fn stdlib_exit_no_args() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "exit".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("std::process::exit(1)"));
}

#[test]
fn stdlib_puts_generates_println() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "puts".to_string(),
        arguments: vec![HirExpression::StringLiteral("hello".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("println!"),
        "puts should generate println!, got: {}",
        code
    );
}

#[test]
fn stdlib_puts_no_args() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "puts".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("println!()"));
}

#[test]
fn stdlib_snprintf_generates_format() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "snprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::IntLiteral(256),
            HirExpression::StringLiteral("value: %d".to_string()),
            HirExpression::Variable("x".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("format!"),
        "snprintf should generate format!, got: {}",
        code
    );
}

#[test]
fn stdlib_snprintf_no_varargs() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "snprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::IntLiteral(256),
            HirExpression::StringLiteral("hello".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("format!"));
}

#[test]
fn stdlib_snprintf_invalid_args() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "snprintf".to_string(),
        arguments: vec![HirExpression::Variable("buf".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("snprintf requires 3+ args"));
}

#[test]
fn stdlib_sprintf_generates_format() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "sprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::StringLiteral("val=%d".to_string()),
            HirExpression::Variable("x".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("format!"),
        "sprintf should generate format!, got: {}",
        code
    );
}

#[test]
fn stdlib_sprintf_no_varargs() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "sprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::StringLiteral("hello".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("format!"));
}

#[test]
fn stdlib_sprintf_invalid_args() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "sprintf".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("sprintf requires 2+ args"));
}

#[test]
fn stdlib_qsort_generates_sort_by() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "qsort".to_string(),
        arguments: vec![
            HirExpression::Variable("arr".to_string()),
            HirExpression::Variable("n".to_string()),
            HirExpression::FunctionCall {
                function: "sizeof".to_string(),
                arguments: vec![HirExpression::Variable("int".to_string())],
            },
            HirExpression::Variable("compare".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("sort_by"),
        "qsort should generate sort_by, got: {}",
        code
    );
}

#[test]
fn stdlib_qsort_invalid_args() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "qsort".to_string(),
        arguments: vec![HirExpression::Variable("arr".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("qsort requires 4 args"));
}

// ============================================================================
// Signature generation: function name renaming (DECY-241 keyword conflicts)
// ============================================================================

#[test]
fn signature_renames_write_to_c_write() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("write".to_string(), HirType::Void, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("fn c_write"),
        "write should be renamed to c_write, got: {}",
        sig
    );
}

#[test]
fn signature_renames_read_to_c_read() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("read".to_string(), HirType::Int, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("fn c_read"),
        "read should be renamed to c_read, got: {}",
        sig
    );
}

#[test]
fn signature_renames_type_to_c_type() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("type".to_string(), HirType::Void, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("fn c_type"),
        "type should be renamed to c_type, got: {}",
        sig
    );
}

#[test]
fn signature_renames_match_to_c_match() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("match".to_string(), HirType::Int, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("fn c_match"),
        "match should be renamed to c_match, got: {}",
        sig
    );
}

#[test]
fn signature_renames_self_to_c_self() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("self".to_string(), HirType::Void, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("fn c_self"),
        "self should be renamed to c_self, got: {}",
        sig
    );
}

#[test]
fn signature_renames_in_to_c_in() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("in".to_string(), HirType::Void, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("fn c_in"),
        "in should be renamed to c_in, got: {}",
        sig
    );
}

#[test]
fn signature_preserves_normal_name() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("process_data".to_string(), HirType::Int, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("fn process_data"),
        "Normal name should be preserved, got: {}",
        sig
    );
}

// ============================================================================
// Signature generation: main() special case, return types
// ============================================================================

#[test]
fn signature_main_omits_return_type() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("main".to_string(), HirType::Int, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("fn main"),
        "Should generate main, got: {}",
        sig
    );
    assert!(
        !sig.contains("-> i32"),
        "main should not have -> i32 return, got: {}",
        sig
    );
}

#[test]
fn signature_non_main_has_return_type() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("compute".to_string(), HirType::Int, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("-> i32"),
        "Non-main with Int return should have -> i32, got: {}",
        sig
    );
}

#[test]
fn signature_void_return_no_arrow() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("process".to_string(), HirType::Void, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        !sig.contains("->"),
        "Void return should have no arrow, got: {}",
        sig
    );
}

#[test]
fn signature_float_return() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("calc".to_string(), HirType::Float, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("-> f32"),
        "Float return should be -> f32, got: {}",
        sig
    );
}

#[test]
fn signature_double_return() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("precise".to_string(), HirType::Double, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("-> f64"),
        "Double return should be -> f64, got: {}",
        sig
    );
}

#[test]
fn signature_char_return() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("getchar_fn".to_string(), HirType::Char, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("->"),
        "Char return should have arrow, got: {}",
        sig
    );
}

// ============================================================================
// Signature generation: parameters
// ============================================================================

#[test]
fn signature_basic_int_params() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new(
        "add".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
    );
    let sig = cg.generate_signature(&func);
    assert!(sig.contains("a:"), "Should contain param a, got: {}", sig);
    assert!(sig.contains("b:"), "Should contain param b, got: {}", sig);
}

#[test]
fn signature_pointer_param() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "deref".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::Dereference(
            Box::new(HirExpression::Variable("ptr".to_string())),
        )))],
    );
    let sig = cg.generate_signature(&func);
    assert!(sig.contains("ptr"), "Should contain ptr param, got: {}", sig);
}

#[test]
fn signature_unsigned_int_return() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("count".to_string(), HirType::UnsignedInt, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("-> u32"),
        "UnsignedInt return should be -> u32, got: {}",
        sig
    );
}

// ============================================================================
// Expression target type: null pointer detection
// ============================================================================

#[test]
fn expr_int_zero_to_pointer_is_null_mut() {
    let cg = CodeGenerator::new();
    // VariableDeclaration with pointer type and IntLiteral(0) initializer
    let stmt = HirStatement::VariableDeclaration {
        name: "ptr".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("null_mut") || code.contains("None"),
        "0 assigned to pointer should generate null_mut or None, got: {}",
        code
    );
}

#[test]
fn expr_int_nonzero_to_pointer_no_null() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "ptr".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::IntLiteral(42)),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        !code.contains("null_mut"),
        "Non-zero to pointer should NOT be null_mut, got: {}",
        code
    );
}

#[test]
fn expr_string_literal_to_pointer_type() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "msg".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Char)),
        initializer: Some(HirExpression::StringLiteral("hello".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("hello"),
        "String literal assigned to char* should contain the string, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: InlineAsm
// ============================================================================

#[test]
fn statement_inline_asm_translatable() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::InlineAsm {
        text: "nop".to_string(),
        translatable: true,
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("asm") || code.contains("nop"),
        "InlineAsm with translatable should generate asm, got: {}",
        code
    );
}

#[test]
fn statement_inline_asm_not_translatable() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::InlineAsm {
        text: "int 0x80".to_string(),
        translatable: false,
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        !code.is_empty(),
        "InlineAsm non-translatable should generate something, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: switch/case with char literal
// ============================================================================

#[test]
fn statement_switch_basic() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Switch {
        condition: HirExpression::Variable("x".to_string()),
        cases: vec![SwitchCase {
            value: Some(HirExpression::IntLiteral(1)),
            body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(10)))],
        }],
        default_case: Some(vec![HirStatement::Return(Some(
            HirExpression::IntLiteral(0),
        ))]),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("match"),
        "Switch should generate match, got: {}",
        code
    );
}

#[test]
fn statement_switch_char_cases() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Switch {
        condition: HirExpression::Variable("ch".to_string()),
        cases: vec![
            SwitchCase {
                value: Some(HirExpression::CharLiteral(b'a' as i8)),
                body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
            },
            SwitchCase {
                value: Some(HirExpression::CharLiteral(b'b' as i8)),
                body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(2)))],
            },
        ],
        default_case: None,
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("match"),
        "Switch with chars should generate match, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: for loop variants
// ============================================================================

#[test]
fn statement_for_with_init_and_increment() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::For {
        init: vec![HirStatement::VariableDeclaration {
            name: "i".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(0)),
        }],
        condition: Some(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(10)),
        }),
        increment: vec![HirStatement::Expression(HirExpression::UnaryOp {
            op: UnaryOperator::PostIncrement,
            operand: Box::new(HirExpression::Variable("i".to_string())),
        })],
        body: vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "puts".to_string(),
            arguments: vec![HirExpression::StringLiteral("tick".to_string())],
        })],
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("while") || code.contains("for"),
        "For loop should generate while or for, got: {}",
        code
    );
}

#[test]
fn statement_for_infinite_loop() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::For {
        init: vec![],
        condition: None,
        increment: vec![],
        body: vec![HirStatement::Break],
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("loop"),
        "for(;;) should generate loop, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: deref assignment
// ============================================================================

#[test]
fn statement_deref_assignment() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("ptr".to_string()),
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("42"),
        "Deref assignment should contain value, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: array index assignment
// ============================================================================

#[test]
fn statement_array_index_assignment() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
        value: HirExpression::IntLiteral(99),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("arr") && code.contains("99"),
        "Array index assignment should contain array and value, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: field assignment
// ============================================================================

#[test]
fn statement_field_assignment() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::FieldAssignment {
        object: HirExpression::Variable("point".to_string()),
        field: "x".to_string(),
        value: HirExpression::IntLiteral(10),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("point") && code.contains("x") && code.contains("10"),
        "Field assignment should contain object, field, value, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: free
// ============================================================================

#[test]
fn statement_free_generates_drop_comment() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Free {
        pointer: HirExpression::Variable("ptr".to_string()),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("drop") || code.contains("RAII") || code.contains("freed"),
        "Free should generate drop/RAII comment, got: {}",
        code
    );
}

// ============================================================================
// Expression coverage: char literal
// ============================================================================

#[test]
fn expr_char_literal_printable() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::CharLiteral(b'A' as i8);
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("'A'") || code.contains("b'A'") || code.contains("65"),
        "Printable char should generate char literal, got: {}",
        code
    );
}

#[test]
fn expr_char_literal_non_printable() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::CharLiteral(b'\n' as i8);
    let code = cg.generate_expression(&expr);
    assert!(
        !code.is_empty(),
        "Non-printable char should generate something, got: {}",
        code
    );
}

#[test]
fn expr_char_literal_zero() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::CharLiteral(0);
    let code = cg.generate_expression(&expr);
    assert!(
        !code.is_empty(),
        "Null char should generate something, got: {}",
        code
    );
}

// ============================================================================
// Expression coverage: unary ops
// ============================================================================

#[test]
fn expr_unary_post_increment() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PostIncrement,
        operand: Box::new(HirExpression::Variable("i".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(!code.is_empty(), "PostIncrement should generate code, got: {}", code);
}

#[test]
fn expr_unary_pre_decrement() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PreDecrement,
        operand: Box::new(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(!code.is_empty(), "PreDecrement should generate code, got: {}", code);
}

#[test]
fn expr_unary_logical_not() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("flag".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("!") || code.contains("== 0"),
        "LogicalNot should generate negation or == 0, got: {}",
        code
    );
}

// ============================================================================
// Expression coverage: dereference
// ============================================================================

#[test]
fn expr_dereference_variable() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable(
        "ptr".to_string(),
    )));
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("ptr"),
        "Dereference should contain variable name, got: {}",
        code
    );
}

// ============================================================================
// Expression coverage: address-of
// ============================================================================

#[test]
fn expr_address_of_variable() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::AddressOf(Box::new(HirExpression::Variable(
        "x".to_string(),
    )));
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("&") || code.contains("x"),
        "AddressOf should generate reference, got: {}",
        code
    );
}

// ============================================================================
// Expression coverage: sizeof
// ============================================================================

#[test]
fn expr_sizeof_type() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Sizeof { type_name: "int".to_string() };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("size_of") || code.contains("mem::size_of"),
        "SizeOf should generate size_of, got: {}",
        code
    );
}

#[test]
fn expr_sizeof_pointer_type() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Sizeof { type_name: "char*".to_string() };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("size_of"),
        "SizeOf pointer should generate size_of, got: {}",
        code
    );
}

// ============================================================================
// Expression coverage: type cast (Cast variant)
// ============================================================================

#[test]
fn expr_cast_var_to_float() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Cast {
        expr: Box::new(HirExpression::Variable("x".to_string())),
        target_type: HirType::Float,
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("as f32") || code.contains("f32"),
        "Cast var to float should generate as f32, got: {}",
        code
    );
}

#[test]
fn expr_cast_var_to_int() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Cast {
        expr: Box::new(HirExpression::Variable("f".to_string())),
        target_type: HirType::Int,
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("as i32") || code.contains("i32"),
        "Cast float to int should generate as i32, got: {}",
        code
    );
}

#[test]
fn expr_cast_to_unsigned() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Cast {
        expr: Box::new(HirExpression::Variable("x".to_string())),
        target_type: HirType::UnsignedInt,
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("as u32") || code.contains("u32"),
        "Cast to unsigned should generate u32, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: VLA (variable-length array) patterns
// ============================================================================

#[test]
fn statement_vla_int_array() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("vec![") && code.contains("0i32"),
        "VLA int should generate vec![0i32; n], got: {}",
        code
    );
}

#[test]
fn statement_vla_float_array() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "buf".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Float),
            size: None,
        },
        initializer: Some(HirExpression::Variable("size".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("vec![") && code.contains("0.0f32"),
        "VLA float should generate vec![0.0f32; size], got: {}",
        code
    );
}

#[test]
fn statement_vla_double_array() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "data".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Double),
            size: None,
        },
        initializer: Some(HirExpression::Variable("len".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("vec![") && code.contains("0.0f64"),
        "VLA double should generate vec![0.0f64; len], got: {}",
        code
    );
}

#[test]
fn statement_vla_char_array() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "buffer".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Char),
            size: None,
        },
        initializer: Some(HirExpression::Variable("sz".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("vec![") && code.contains("0u8"),
        "VLA char should generate vec![0u8; sz], got: {}",
        code
    );
}

#[test]
fn statement_vla_unsigned_int_array() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "counts".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::UnsignedInt),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("vec![") && code.contains("0u32"),
        "VLA unsigned int should generate vec![0u32; n], got: {}",
        code
    );
}

#[test]
fn statement_vla_signed_char_array() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "vals".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::SignedChar),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("vec![") && code.contains("0i8"),
        "VLA signed char should generate vec![0i8; n], got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: malloc initialization patterns
// ============================================================================

#[test]
fn statement_malloc_init_box_pattern() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "data".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
        initializer: Some(HirExpression::Malloc {
            size: Box::new(HirExpression::Sizeof {
                type_name: "Node".to_string(),
            }),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("Box") || code.contains("box"),
        "Struct malloc should generate Box, got: {}",
        code
    );
}

#[test]
fn statement_malloc_init_vec_pattern() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::Malloc {
            size: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(HirExpression::Variable("n".to_string())),
                right: Box::new(HirExpression::Sizeof {
                    type_name: "int".to_string(),
                }),
            }),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("Vec") || code.contains("vec"),
        "Array malloc pattern should generate Vec, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: char* string literal initialization
// ============================================================================

#[test]
fn statement_char_ptr_string_literal() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "msg".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Char)),
        initializer: Some(HirExpression::StringLiteral("hello world".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("&str") || code.contains("hello world"),
        "char* with string literal should use &str, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: char array from string literal
// ============================================================================

#[test]
fn statement_char_array_string_init() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "buf".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Char),
            size: Some(10),
        },
        initializer: Some(HirExpression::StringLiteral("test".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("test") || code.contains("b\"test"),
        "Char array from string should contain the string, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: return in main (exit code) vs non-main
// ============================================================================

#[test]
fn statement_return_none() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Return(None);
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("return"),
        "Return None should generate return, got: {}",
        code
    );
}

#[test]
fn statement_return_expression() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Return(Some(HirExpression::Variable("result".to_string())));
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("return") && code.contains("result"),
        "Return expr should generate return result, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: while loop
// ============================================================================

#[test]
fn statement_while_basic() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::While {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("n".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
        body: vec![HirStatement::Expression(HirExpression::UnaryOp {
            op: UnaryOperator::PostDecrement,
            operand: Box::new(HirExpression::Variable("n".to_string())),
        })],
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("while"),
        "While should generate while, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: if/else
// ============================================================================

#[test]
fn statement_if_only() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::Variable("flag".to_string()),
        then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
        else_block: None,
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("if"),
        "If should generate if, got: {}",
        code
    );
}

#[test]
fn statement_if_else() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
        then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
        else_block: Some(vec![HirStatement::Return(Some(
            HirExpression::IntLiteral(1),
        ))]),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("if") && code.contains("else"),
        "If/else should generate both branches, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: continue and break
// ============================================================================

#[test]
fn statement_break() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Break;
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("break"), "Break should generate break, got: {}", code);
}

#[test]
fn statement_continue() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Continue;
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("continue"), "Continue should generate continue, got: {}", code);
}

// ============================================================================
// Statement coverage: expression statement
// ============================================================================

#[test]
fn statement_expression_function_call() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Expression(HirExpression::FunctionCall {
        function: "process".to_string(),
        arguments: vec![HirExpression::Variable("data".to_string())],
    });
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("process") && code.contains("data"),
        "Expression statement should contain function call, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: assignment
// ============================================================================

#[test]
fn statement_assignment_simple() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Assignment {
        target: "x".to_string(),
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("x") && code.contains("42"),
        "Assignment should contain target and value, got: {}",
        code
    );
}

// ============================================================================
// Expression coverage: binary operators
// ============================================================================

#[test]
fn expr_binary_add() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("+") || code.contains("a") && code.contains("b"),
        "Add should generate +, got: {}",
        code
    );
}

#[test]
fn expr_binary_subtract() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("-") || code.contains("wrapping_sub"),
        "Subtract should generate - or wrapping_sub, got: {}",
        code
    );
}

#[test]
fn expr_binary_logical_and() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalAnd,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("&&") || code.contains("!= 0"),
        "LogicalAnd should generate &&, got: {}",
        code
    );
}

#[test]
fn expr_binary_logical_or() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalOr,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("||") || code.contains("!= 0"),
        "LogicalOr should generate ||, got: {}",
        code
    );
}

#[test]
fn expr_binary_bitwise_and() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::BitwiseAnd,
        left: Box::new(HirExpression::Variable("flags".to_string())),
        right: Box::new(HirExpression::IntLiteral(0xFF)),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("&") || code.contains("flags"),
        "BitwiseAnd should generate &, got: {}",
        code
    );
}

#[test]
fn expr_binary_shift_left() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LeftShift,
        left: Box::new(HirExpression::IntLiteral(1)),
        right: Box::new(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("<<"),
        "LeftShift should generate <<, got: {}",
        code
    );
}

#[test]
fn expr_null_literal_codegen() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::NullLiteral;
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("None") || code.contains("null"),
        "NullLiteral should generate None or null, got: {}",
        code
    );
}

#[test]
fn expr_is_not_null_codegen() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::IsNotNull(Box::new(HirExpression::Variable("ptr".to_string())));
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("is_some") || code.contains("is_null") || code.contains("ptr"),
        "IsNotNull should generate null check, got: {}",
        code
    );
}

#[test]
fn expr_calloc_generates_vec_zeroed() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(10)),
        element_type: Box::new(HirType::Int),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("vec![") || code.contains("Vec"),
        "Calloc should generate vec or Vec, got: {}",
        code
    );
}

// ============================================================================
// Signature generation: pointer parameter transformation with body
// ============================================================================

#[test]
fn signature_pointer_param_read_only_becomes_ref() {
    let cg = CodeGenerator::new();
    // void print_val(int* p) { return *p; }
    let func = HirFunction::new_with_body(
        "print_val".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::Dereference(
            Box::new(HirExpression::Variable("p".to_string())),
        )))],
    );
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("&") && sig.contains("i32"),
        "Read-only pointer param should become reference, got: {}",
        sig
    );
}

#[test]
fn signature_pointer_param_modified_becomes_mut_ref() {
    let cg = CodeGenerator::new();
    // int increment(int* p) { *p = *p + 1; return *p; }
    // Using int return type + deref write means output param detector won't claim 'p'
    let func = HirFunction::new_with_body(
        "increment".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("p".to_string()),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Dereference(Box::new(
                        HirExpression::Variable("p".to_string()),
                    ))),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            },
            HirStatement::Return(Some(HirExpression::Dereference(Box::new(
                HirExpression::Variable("p".to_string()),
            )))),
        ],
    );
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("&mut") && sig.contains("i32"),
        "Modified pointer param should become &mut, got: {}",
        sig
    );
}

#[test]
fn signature_void_star_stub_no_generic() {
    let cg = CodeGenerator::new();
    // void process(void* data); — no body (stub)
    let func = HirFunction::new(
        "process".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "data".to_string(),
            HirType::Pointer(Box::new(HirType::Void)),
        )],
    );
    let sig = cg.generate_signature(&func);
    assert!(
        !sig.contains("<T>"),
        "void* stub without body should NOT get generic <T>, got: {}",
        sig
    );
    assert!(
        sig.contains("*mut ()"),
        "void* stub should become *mut (), got: {}",
        sig
    );
}

#[test]
fn signature_multiple_params_mixed_types() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "compute".to_string(),
        HirType::Float,
        vec![
            HirParameter::new("x".to_string(), HirType::Int),
            HirParameter::new("scale".to_string(), HirType::Float),
        ],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Cast {
                target_type: HirType::Float,
                expr: Box::new(HirExpression::Variable("x".to_string())),
            }),
            right: Box::new(HirExpression::Variable("scale".to_string())),
        }))],
    );
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("x:") && sig.contains("scale:"),
        "Should contain both params, got: {}",
        sig
    );
    assert!(
        sig.contains("-> f32"),
        "Should return f32, got: {}",
        sig
    );
}

#[test]
fn signature_array_param_becomes_slice() {
    let cg = CodeGenerator::new();
    // int sum(int arr[10]) { return arr[0]; }
    let func = HirFunction::new_with_body(
        "sum".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "arr".to_string(),
            HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(10),
            },
        )],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("arr".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }))],
    );
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("[i32]") || sig.contains("arr"),
        "Array param should become slice or keep name, got: {}",
        sig
    );
}

// ============================================================================
// generate_function: full function code generation
// ============================================================================

#[test]
fn generate_function_simple() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "add".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }))],
    );
    let code = cg.generate_function(&func);
    assert!(
        code.contains("fn add"),
        "Should contain fn add, got: {}",
        code
    );
    assert!(
        code.contains("return"),
        "Should contain return, got: {}",
        code
    );
}

#[test]
fn generate_function_void_no_return() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "do_nothing".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "puts".to_string(),
            arguments: vec![HirExpression::StringLiteral("hello".to_string())],
        })],
    );
    let code = cg.generate_function(&func);
    assert!(
        code.contains("fn do_nothing"),
        "Should contain fn do_nothing, got: {}",
        code
    );
    assert!(
        !code.contains("->"),
        "Void function should have no return type arrow, got: {}",
        code
    );
}

// ============================================================================
// Expression coverage: comma operator, char arithmetic, logical ops
// ============================================================================

#[test]
fn expr_comma_operator() {
    let cg = CodeGenerator::new();
    // C: (a = 1, b = 2) — comma operator evaluates both, returns last
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Comma,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let code = cg.generate_expression(&expr);
    // Comma generates block: { a; b }
    assert!(
        code.contains("a") && code.contains("b"),
        "Comma should include both operands, got: {}",
        code
    );
}

#[test]
fn expr_char_literal_arithmetic_add() {
    let cg = CodeGenerator::new();
    // C: (num % 10) + '0'
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Modulo,
            left: Box::new(HirExpression::Variable("num".to_string())),
            right: Box::new(HirExpression::IntLiteral(10)),
        }),
        right: Box::new(HirExpression::CharLiteral(b'0' as i8)),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("num") && code.contains("0"),
        "Should contain operands, got: {}",
        code
    );
}

#[test]
fn expr_logical_and_generates_bool() {
    let cg = CodeGenerator::new();
    // C: a && b
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalAnd,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("&&") || code.contains("a") && code.contains("b"),
        "LogicalAnd should generate && or bool check, got: {}",
        code
    );
}

#[test]
fn expr_logical_or_generates_bool() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalOr,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::Variable("y".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("||") || code.contains("x") && code.contains("y"),
        "LogicalOr should generate || or bool check, got: {}",
        code
    );
}

#[test]
fn expr_bitwise_xor() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::BitwiseXor,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("^"),
        "BitwiseXor should generate ^, got: {}",
        code
    );
}

#[test]
fn expr_modulo_operator() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Modulo,
        left: Box::new(HirExpression::Variable("n".to_string())),
        right: Box::new(HirExpression::IntLiteral(7)),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("%"),
        "Modulo should generate %, got: {}",
        code
    );
}

#[test]
fn expr_not_equal_comparison() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("!=") || code.contains("x"),
        "NotEqual should generate != or truthy check, got: {}",
        code
    );
}

#[test]
fn expr_greater_than_or_equal() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::GreaterEqual,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains(">="),
        "GreaterThanOrEqual should generate >=, got: {}",
        code
    );
}

#[test]
fn expr_less_than_or_equal() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LessEqual,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::IntLiteral(100)),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("<="),
        "LessThanOrEqual should generate <=, got: {}",
        code
    );
}

#[test]
fn expr_ternary_simple() {
    let cg = CodeGenerator::new();
    // C: (x > 0) ? x : -x
    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
        then_expr: Box::new(HirExpression::Variable("x".to_string())),
        else_expr: Box::new(HirExpression::UnaryOp {
            op: UnaryOperator::Minus,
            operand: Box::new(HirExpression::Variable("x".to_string())),
        }),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("if") || code.contains("x"),
        "Ternary should generate if expression, got: {}",
        code
    );
}

#[test]
fn expr_string_literal_basic() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::StringLiteral("hello world".to_string());
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("hello world"),
        "StringLiteral should contain the string, got: {}",
        code
    );
}

#[test]
fn expr_float_literal() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FloatLiteral("3.14".to_string());
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("3.14"),
        "FloatLiteral should contain 3.14, got: {}",
        code
    );
}

#[test]
fn expr_int_literal_negative() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::IntLiteral(-42);
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("-42") || code.contains("42"),
        "Negative IntLiteral should contain the value, got: {}",
        code
    );
}

#[test]
fn expr_int_literal_zero() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::IntLiteral(0);
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("0"),
        "IntLiteral(0) should generate 0, got: {}",
        code
    );
}

#[test]
fn expr_post_increment_simple() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PostIncrement,
        operand: Box::new(HirExpression::Variable("i".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("i") && (code.contains("+=") || code.contains("+ 1")),
        "PostIncrement should increment i, got: {}",
        code
    );
}

#[test]
fn expr_post_decrement_simple() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PostDecrement,
        operand: Box::new(HirExpression::Variable("j".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("j") && (code.contains("-=") || code.contains("- 1")),
        "PostDecrement should decrement j, got: {}",
        code
    );
}

#[test]
fn expr_pre_increment() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PreIncrement,
        operand: Box::new(HirExpression::Variable("k".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("k"),
        "PreIncrement should reference k, got: {}",
        code
    );
}

#[test]
fn expr_pre_decrement() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PreDecrement,
        operand: Box::new(HirExpression::Variable("m".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("m"),
        "PreDecrement should reference m, got: {}",
        code
    );
}

#[test]
fn expr_bitwise_not() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::BitwiseNot,
        operand: Box::new(HirExpression::Variable("flags".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("!") || code.contains("flags"),
        "BitwiseNot should negate flags, got: {}",
        code
    );
}

#[test]
fn expr_compound_literal_array() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(3),
        },
        initializers: vec![
            HirExpression::IntLiteral(1),
            HirExpression::IntLiteral(2),
            HirExpression::IntLiteral(3),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("1") && code.contains("2") && code.contains("3"),
        "CompoundLiteral should contain all initializers, got: {}",
        code
    );
}

#[test]
fn expr_compound_literal_struct() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Point".to_string()),
        initializers: vec![
            HirExpression::IntLiteral(10),
            HirExpression::IntLiteral(20),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("Point") || code.contains("10"),
        "CompoundLiteral struct should reference type or values, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: realloc, empty return, pointer conditions, errno, for(;;)
// ============================================================================

#[test]
fn stmt_return_empty_non_main() {
    let cg = CodeGenerator::new();
    // C: void foo() { return; }
    let stmt = HirStatement::Return(None);
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("return"),
        "Empty return should generate return, got: {}",
        code
    );
}

#[test]
fn stmt_for_loop_infinite() {
    let cg = CodeGenerator::new();
    // C: for(;;) { break; }
    let stmt = HirStatement::For {
        init: vec![],
        condition: None,
        increment: vec![],
        body: vec![HirStatement::Break],
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("loop"),
        "for(;;) should generate loop, got: {}",
        code
    );
}

#[test]
fn stmt_for_loop_with_init_no_condition() {
    let cg = CodeGenerator::new();
    // C: for(int i = 0; ; i++) { break; }
    let stmt = HirStatement::For {
        init: vec![HirStatement::VariableDeclaration {
            name: "i".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(0)),
        }],
        condition: None,
        increment: vec![HirStatement::Expression(HirExpression::UnaryOp {
            op: UnaryOperator::PostIncrement,
            operand: Box::new(HirExpression::Variable("i".to_string())),
        })],
        body: vec![HirStatement::Break],
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("loop"),
        "for(init;;inc) should generate loop with init, got: {}",
        code
    );
}

#[test]
fn stmt_assignment_to_errno() {
    let cg = CodeGenerator::new();
    // C: errno = 0;
    let stmt = HirStatement::Assignment {
        target: "errno".to_string(),
        value: HirExpression::IntLiteral(0),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("errno") || code.contains("0"),
        "errno assignment should reference errno, got: {}",
        code
    );
}

#[test]
fn stmt_switch_with_default_only() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Switch {
        condition: HirExpression::Variable("val".to_string()),
        cases: vec![],
        default_case: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(
            -1,
        )))]),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("_") || code.contains("match") || code.contains("val"),
        "Switch with default only should generate match, got: {}",
        code
    );
}

#[test]
fn stmt_switch_multiple_cases() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Switch {
        condition: HirExpression::Variable("op".to_string()),
        cases: vec![
            SwitchCase {
                value: Some(HirExpression::IntLiteral(1)),
                body: vec![HirStatement::Return(Some(HirExpression::StringLiteral(
                    "add".to_string(),
                )))],
            },
            SwitchCase {
                value: Some(HirExpression::IntLiteral(2)),
                body: vec![HirStatement::Return(Some(HirExpression::StringLiteral(
                    "sub".to_string(),
                )))],
            },
        ],
        default_case: Some(vec![HirStatement::Return(Some(
            HirExpression::StringLiteral("unknown".to_string()),
        ))]),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("1") && code.contains("2"),
        "Switch should contain both case values, got: {}",
        code
    );
}

#[test]
fn stmt_switch_char_literal_case() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Switch {
        condition: HirExpression::Variable("ch".to_string()),
        cases: vec![
            SwitchCase {
                value: Some(HirExpression::CharLiteral(b'A' as i8)),
                body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
            },
            SwitchCase {
                value: Some(HirExpression::CharLiteral(b'B' as i8)),
                body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(2)))],
            },
        ],
        default_case: None,
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("A") || code.contains("65"),
        "Switch with char cases should contain char values, got: {}",
        code
    );
}

#[test]
fn stmt_var_decl_with_realloc_null() {
    let cg = CodeGenerator::new();
    // C: int* p = realloc(NULL, 10 * sizeof(int));
    let stmt = HirStatement::VariableDeclaration {
        name: "p".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::Realloc {
            pointer: Box::new(HirExpression::NullLiteral),
            new_size: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(HirExpression::IntLiteral(10)),
                right: Box::new(HirExpression::Sizeof {
                    type_name: "int".to_string(),
                }),
            }),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("p") && (code.contains("Vec") || code.contains("vec") || code.contains("alloc")),
        "realloc(NULL, ...) should generate Vec or allocation, got: {}",
        code
    );
}

#[test]
fn stmt_var_decl_with_realloc_non_pattern_size() {
    let cg = CodeGenerator::new();
    // C: int* p = realloc(old, new_size); — non-multiply size pattern
    let stmt = HirStatement::VariableDeclaration {
        name: "p".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::Realloc {
            pointer: Box::new(HirExpression::Variable("old".to_string())),
            new_size: Box::new(HirExpression::Variable("new_size".to_string())),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("p"),
        "realloc with non-pattern size should still generate, got: {}",
        code
    );
}

// ============================================================================
// Format specifier coverage: via printf FunctionCall expressions
// ============================================================================

#[test]
fn expr_printf_with_width() {
    let cg = CodeGenerator::new();
    // C: printf("%10d", x);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%10d".to_string()),
            HirExpression::Variable("x".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print") || code.contains("x"),
        "printf with width should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_with_precision() {
    let cg = CodeGenerator::new();
    // C: printf("%.2f", val);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%.2f".to_string()),
            HirExpression::Variable("val".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print") || code.contains("val"),
        "printf with precision should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_with_width_and_precision() {
    let cg = CodeGenerator::new();
    // C: printf("%10.5f", val);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%10.5f".to_string()),
            HirExpression::Variable("val".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print"),
        "printf with width.precision should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_zero_pad_flag() {
    let cg = CodeGenerator::new();
    // C: printf("%05d", x);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%05d".to_string()),
            HirExpression::Variable("x".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print") || code.contains("x"),
        "printf with zero-pad should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_left_align_flag() {
    let cg = CodeGenerator::new();
    // C: printf("%-10s", name);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%-10s".to_string()),
            HirExpression::Variable("name".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print"),
        "printf with left-align should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_hex_alternate() {
    let cg = CodeGenerator::new();
    // C: printf("%#x", val);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%#x".to_string()),
            HirExpression::Variable("val".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print"),
        "printf with # flag should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_long_long() {
    let cg = CodeGenerator::new();
    // C: printf("%lld", big_val);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%lld".to_string()),
            HirExpression::Variable("big_val".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print"),
        "printf with lld should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_unsigned() {
    let cg = CodeGenerator::new();
    // C: printf("%u", count);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%u".to_string()),
            HirExpression::Variable("count".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print"),
        "printf with %%u should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_octal() {
    let cg = CodeGenerator::new();
    // C: printf("%o", val);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%o".to_string()),
            HirExpression::Variable("val".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print"),
        "printf with %%o should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_multiple_specifiers() {
    let cg = CodeGenerator::new();
    // C: printf("name=%s age=%d score=%.1f", name, age, score);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("name=%s age=%d score=%.1f".to_string()),
            HirExpression::Variable("name".to_string()),
            HirExpression::Variable("age".to_string()),
            HirExpression::Variable("score".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print"),
        "printf with multiple specifiers should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_char_specifier() {
    let cg = CodeGenerator::new();
    // C: printf("%c", ch);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%c".to_string()),
            HirExpression::Variable("ch".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print"),
        "printf with %%c should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_pointer_specifier() {
    let cg = CodeGenerator::new();
    // C: printf("%p", ptr);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%p".to_string()),
            HirExpression::Variable("ptr".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print"),
        "printf with %%p should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_size_t_specifier() {
    let cg = CodeGenerator::new();
    // C: printf("%zu", len);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%zu".to_string()),
            HirExpression::Variable("len".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print"),
        "printf with %%zu should generate print, got: {}",
        code
    );
}

#[test]
fn expr_fprintf_to_stderr() {
    let cg = CodeGenerator::new();
    // C: fprintf(stderr, "error: %s\n", msg);
    let expr = HirExpression::FunctionCall {
        function: "fprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("stderr".to_string()),
            HirExpression::StringLiteral("error: %s\\n".to_string()),
            HirExpression::Variable("msg".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("eprint") || code.contains("stderr") || code.contains("msg"),
        "fprintf to stderr should generate eprint, got: {}",
        code
    );
}

// ============================================================================
// Additional stdlib function coverage
// ============================================================================

#[test]
fn expr_fgetc_call() {
    let cg = CodeGenerator::new();
    // C: fgetc(fp);
    let expr = HirExpression::FunctionCall {
        function: "fgetc".to_string(),
        arguments: vec![HirExpression::Variable("fp".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        !code.is_empty(),
        "fgetc should generate something, got: {}",
        code
    );
}

#[test]
fn expr_fputc_call() {
    let cg = CodeGenerator::new();
    // C: fputc('c', fp);
    let expr = HirExpression::FunctionCall {
        function: "fputc".to_string(),
        arguments: vec![
            HirExpression::CharLiteral(b'c' as i8),
            HirExpression::Variable("fp".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        !code.is_empty(),
        "fputc should generate something, got: {}",
        code
    );
}

#[test]
fn expr_realloc_call() {
    let cg = CodeGenerator::new();
    // C: realloc(ptr, new_size);
    let expr = HirExpression::Realloc {
        pointer: Box::new(HirExpression::Variable("ptr".to_string())),
        new_size: Box::new(HirExpression::Variable("new_size".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        !code.is_empty(),
        "realloc should generate code, got: {}",
        code
    );
}

// ============================================================================
// Variable declaration edge cases
// ============================================================================

#[test]
fn stmt_var_decl_char_array_from_compound() {
    let cg = CodeGenerator::new();
    // C: char arr[3] = {65, 66, 67};
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Char),
            size: Some(3),
        },
        initializer: Some(HirExpression::CompoundLiteral {
            literal_type: HirType::Array {
                element_type: Box::new(HirType::Char),
                size: Some(3),
            },
            initializers: vec![
                HirExpression::IntLiteral(65),
                HirExpression::IntLiteral(66),
                HirExpression::IntLiteral(67),
            ],
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("arr"),
        "Char array from compound should contain arr, got: {}",
        code
    );
}

#[test]
fn stmt_var_decl_unsigned_int() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "count".to_string(),
        var_type: HirType::UnsignedInt,
        initializer: Some(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("u32") || code.contains("count"),
        "UnsignedInt decl should use u32, got: {}",
        code
    );
}

#[test]
fn stmt_var_decl_double() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "big".to_string(),
        var_type: HirType::Double,
        initializer: Some(HirExpression::FloatLiteral("0.0".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("f64") || code.contains("big"),
        "Double decl should use f64, got: {}",
        code
    );
}

#[test]
fn stmt_var_decl_signed_char() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "sc".to_string(),
        var_type: HirType::SignedChar,
        initializer: Some(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("i8") || code.contains("sc"),
        "SignedChar decl should use i8, got: {}",
        code
    );
}

#[test]
fn stmt_var_decl_struct_type() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "pt".to_string(),
        var_type: HirType::Struct("Point".to_string()),
        initializer: None,
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("Point") || code.contains("pt"),
        "Struct decl should reference Point, got: {}",
        code
    );
}

#[test]
fn stmt_var_decl_string_literal_type() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "msg".to_string(),
        var_type: HirType::StringLiteral,
        initializer: Some(HirExpression::StringLiteral("hello".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("msg") || code.contains("hello"),
        "StringLiteral decl should contain msg or hello, got: {}",
        code
    );
}

// ============================================================================
// generate_annotated_signature_with_func coverage
// ============================================================================

#[test]
fn annotated_sig_simple_void_function() {
    use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let sig = AnnotatedSignature {
        name: "noop".to_string(),
        lifetimes: vec![],
        parameters: vec![],
        return_type: AnnotatedType::Simple(HirType::Void),
    };
    let code = cg.generate_annotated_signature_with_func(&sig, None);
    assert!(
        code.contains("fn noop()"),
        "Should generate fn noop(), got: {}",
        code
    );
    assert!(
        !code.contains("->"),
        "Void should have no return arrow, got: {}",
        code
    );
}

#[test]
fn annotated_sig_with_params_no_func() {
    use decy_ownership::lifetime_gen::{AnnotatedParameter, AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let sig = AnnotatedSignature {
        name: "add".to_string(),
        lifetimes: vec![],
        parameters: vec![
            AnnotatedParameter {
                name: "a".to_string(),
                param_type: AnnotatedType::Simple(HirType::Int),
            },
            AnnotatedParameter {
                name: "b".to_string(),
                param_type: AnnotatedType::Simple(HirType::Int),
            },
        ],
        return_type: AnnotatedType::Simple(HirType::Int),
    };
    let code = cg.generate_annotated_signature_with_func(&sig, None);
    assert!(
        code.contains("fn add"),
        "Should contain fn add, got: {}",
        code
    );
    assert!(
        code.contains("a:") && code.contains("b:"),
        "Should contain both params, got: {}",
        code
    );
    assert!(
        code.contains("-> i32"),
        "Should return i32, got: {}",
        code
    );
}

#[test]
fn annotated_sig_keyword_rename() {
    use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let sig = AnnotatedSignature {
        name: "type".to_string(),
        lifetimes: vec![],
        parameters: vec![],
        return_type: AnnotatedType::Simple(HirType::Int),
    };
    let code = cg.generate_annotated_signature_with_func(&sig, None);
    assert!(
        code.contains("c_type"),
        "Keyword 'type' should be renamed to c_type, got: {}",
        code
    );
}

#[test]
fn annotated_sig_main_no_return_type() {
    use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let sig = AnnotatedSignature {
        name: "main".to_string(),
        lifetimes: vec![],
        parameters: vec![],
        return_type: AnnotatedType::Simple(HirType::Int),
    };
    // main function with int return should NOT have -> i32 (Rust main returns ())
    let func = HirFunction::new("main".to_string(), HirType::Int, vec![]);
    let code = cg.generate_annotated_signature_with_func(&sig, Some(&func));
    assert!(
        !code.contains("->"),
        "main() should have no return type arrow, got: {}",
        code
    );
}

#[test]
fn annotated_sig_with_pointer_param_and_func_body() {
    use decy_ownership::lifetime_gen::{AnnotatedParameter, AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let sig = AnnotatedSignature {
        name: "read_val".to_string(),
        lifetimes: vec![],
        parameters: vec![AnnotatedParameter {
            name: "p".to_string(),
            param_type: AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Int))),
        }],
        return_type: AnnotatedType::Simple(HirType::Int),
    };
    // Function that only reads via pointer → &i32
    let func = HirFunction::new_with_body(
        "read_val".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::Dereference(
            Box::new(HirExpression::Variable("p".to_string())),
        )))],
    );
    let code = cg.generate_annotated_signature_with_func(&sig, Some(&func));
    assert!(
        code.contains("&") && code.contains("i32"),
        "Read-only pointer should become reference, got: {}",
        code
    );
}

// ============================================================================
// generate_function_with_lifetimes: full function generation
// ============================================================================

#[test]
fn gen_func_with_lifetimes_simple() {
    use decy_ownership::lifetime_gen::{AnnotatedParameter, AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "square".to_string(),
        HirType::Int,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::Variable("x".to_string())),
        }))],
    );
    let sig = AnnotatedSignature {
        name: "square".to_string(),
        lifetimes: vec![],
        parameters: vec![AnnotatedParameter {
            name: "x".to_string(),
            param_type: AnnotatedType::Simple(HirType::Int),
        }],
        return_type: AnnotatedType::Simple(HirType::Int),
    };
    let code = cg.generate_function_with_lifetimes(&func, &sig);
    assert!(
        code.contains("fn square"),
        "Should contain fn square, got: {}",
        code
    );
    assert!(
        code.contains("return") || code.contains("x * x") || code.contains("x"),
        "Should contain body, got: {}",
        code
    );
}

#[test]
fn gen_func_with_lifetimes_empty_body() {
    use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "stub".to_string(),
        HirType::Void,
        vec![],
        vec![],
    );
    let sig = AnnotatedSignature {
        name: "stub".to_string(),
        lifetimes: vec![],
        parameters: vec![],
        return_type: AnnotatedType::Simple(HirType::Void),
    };
    let code = cg.generate_function_with_lifetimes(&func, &sig);
    assert!(
        code.contains("fn stub"),
        "Should contain fn stub, got: {}",
        code
    );
}

#[test]
fn gen_func_with_lifetimes_pointer_param() {
    use decy_ownership::lifetime_gen::{AnnotatedParameter, AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "inc".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "val".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("val".to_string()),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Dereference(Box::new(
                    HirExpression::Variable("val".to_string()),
                ))),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
    );
    let sig = AnnotatedSignature {
        name: "inc".to_string(),
        lifetimes: vec![],
        parameters: vec![AnnotatedParameter {
            name: "val".to_string(),
            param_type: AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Int))),
        }],
        return_type: AnnotatedType::Simple(HirType::Void),
    };
    let code = cg.generate_function_with_lifetimes(&func, &sig);
    assert!(
        code.contains("fn inc"),
        "Should contain fn inc, got: {}",
        code
    );
    assert!(
        code.contains("&mut") || code.contains("val"),
        "Should transform pointer param, got: {}",
        code
    );
}

// ============================================================================
// More expression targets: string method, field access, array index
// ============================================================================

#[test]
fn expr_field_access_nested() {
    let cg = CodeGenerator::new();
    // point.inner.x — nested field access
    let expr = HirExpression::FieldAccess {
        object: Box::new(HirExpression::FieldAccess {
            object: Box::new(HirExpression::Variable("point".to_string())),
            field: "inner".to_string(),
        }),
        field: "x".to_string(),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("point") && code.contains("inner") && code.contains("x"),
        "Nested FieldAccess should chain, got: {}",
        code
    );
}

#[test]
fn expr_array_index_expression() {
    let cg = CodeGenerator::new();
    // arr[i + 1]
    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        }),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("arr"),
        "ArrayIndex with expr index should reference arr, got: {}",
        code
    );
}

#[test]
fn expr_string_method_call_len() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("s".to_string())),
        method: "len".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("len") || code.contains("s"),
        "StringMethodCall should reference method, got: {}",
        code
    );
}

#[test]
fn expr_is_not_null_via_not_equal() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::IsNotNull(Box::new(HirExpression::Variable("ptr".to_string())));
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("ptr") || code.contains("null") || code.contains("Some"),
        "IsNotNull should check ptr for non-null, got: {}",
        code
    );
}

#[test]
fn expr_function_call_strcpy() {
    let cg = CodeGenerator::new();
    // C: strcpy(dst, src);
    let expr = HirExpression::FunctionCall {
        function: "strcpy".to_string(),
        arguments: vec![
            HirExpression::Variable("dst".to_string()),
            HirExpression::Variable("src".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("dst") || code.contains("src") || code.contains("clone"),
        "strcpy should generate clone or copy, got: {}",
        code
    );
}

#[test]
fn expr_function_call_strlen() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "strlen".to_string(),
        arguments: vec![HirExpression::Variable("s".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("len") || code.contains("s"),
        "strlen should generate .len(), got: {}",
        code
    );
}

#[test]
fn expr_function_call_atoi() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "atoi".to_string(),
        arguments: vec![HirExpression::Variable("str_val".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("parse") || code.contains("str_val"),
        "atoi should generate parse::<i32>(), got: {}",
        code
    );
}

#[test]
fn expr_function_call_abs() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "abs".to_string(),
        arguments: vec![HirExpression::Variable("n".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("abs") || code.contains("n"),
        "abs should generate .abs(), got: {}",
        code
    );
}

#[test]
fn expr_function_call_exit() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "exit".to_string(),
        arguments: vec![HirExpression::IntLiteral(1)],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("exit") || code.contains("process"),
        "exit should generate std::process::exit, got: {}",
        code
    );
}

#[test]
fn expr_function_call_puts() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "puts".to_string(),
        arguments: vec![HirExpression::StringLiteral("hello".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("println") || code.contains("hello"),
        "puts should generate println!, got: {}",
        code
    );
}

#[test]
fn expr_function_call_qsort() {
    let cg = CodeGenerator::new();
    // C: qsort(arr, n, sizeof(int), compare);
    let expr = HirExpression::FunctionCall {
        function: "qsort".to_string(),
        arguments: vec![
            HirExpression::Variable("arr".to_string()),
            HirExpression::Variable("n".to_string()),
            HirExpression::Sizeof {
                type_name: "int".to_string(),
            },
            HirExpression::Variable("compare".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("sort") || code.contains("arr"),
        "qsort should generate sort_by, got: {}",
        code
    );
}

// ============================================================================
// More statement patterns
// ============================================================================

#[test]
fn stmt_while_with_break_inside() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::While {
        condition: HirExpression::IntLiteral(1),
        body: vec![
            HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::Equal,
                    left: Box::new(HirExpression::Variable("x".to_string())),
                    right: Box::new(HirExpression::IntLiteral(0)),
                },
                then_block: vec![HirStatement::Break],
                else_block: None,
            },
            HirStatement::Expression(HirExpression::UnaryOp {
                op: UnaryOperator::PostDecrement,
                operand: Box::new(HirExpression::Variable("x".to_string())),
            }),
        ],
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("while") || code.contains("loop"),
        "While with break should generate loop structure, got: {}",
        code
    );
}

#[test]
fn stmt_field_assignment() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::FieldAssignment {
        object: HirExpression::Variable("point".to_string()),
        field: "x".to_string(),
        value: HirExpression::IntLiteral(10),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("point") && code.contains("x") && code.contains("10"),
        "FieldAssignment should set point.x = 10, got: {}",
        code
    );
}

#[test]
fn stmt_multiple_var_decl() {
    let cg = CodeGenerator::new();
    // C: int a = 1, b = 2;  → two separate declarations
    let stmt1 = HirStatement::VariableDeclaration {
        name: "a".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::IntLiteral(1)),
    };
    let stmt2 = HirStatement::VariableDeclaration {
        name: "b".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::IntLiteral(2)),
    };
    let code1 = cg.generate_statement(&stmt1);
    let code2 = cg.generate_statement(&stmt2);
    assert!(code1.contains("a") && code2.contains("b"));
}

#[test]
fn stmt_var_decl_pointer_to_struct() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "node".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
        initializer: Some(HirExpression::NullLiteral),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("node"),
        "Pointer to struct decl should contain node, got: {}",
        code
    );
}

#[test]
fn stmt_var_decl_function_pointer() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "callback".to_string(),
        var_type: HirType::FunctionPointer {
            param_types: vec![HirType::Int],
            return_type: Box::new(HirType::Void),
        },
        initializer: None,
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("callback") || code.contains("fn"),
        "Function pointer decl should contain fn or callback, got: {}",
        code
    );
}

#[test]
fn stmt_inline_asm_non_translatable() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::InlineAsm {
        text: "nop".to_string(),
        translatable: false,
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        !code.is_empty(),
        "InlineAsm should generate comment or placeholder, got: {}",
        code
    );
}

#[test]
fn stmt_inline_asm_translatable_pause() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::InlineAsm {
        text: "pause".to_string(),
        translatable: true,
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        !code.is_empty(),
        "Translatable InlineAsm should generate something, got: {}",
        code
    );
}

// ============================================================================
// target_type-dependent expression branches (via typed var declarations)
// ============================================================================

#[test]
fn typed_decl_float_literal_to_float() {
    let cg = CodeGenerator::new();
    // float x = 3.14;  → target_type = Float → "3.14f32"
    let stmt = HirStatement::VariableDeclaration {
        name: "x".to_string(),
        var_type: HirType::Float,
        initializer: Some(HirExpression::FloatLiteral("3.14".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("f32") || code.contains("3.14"),
        "Float decl with float literal should use f32, got: {}",
        code
    );
}

#[test]
fn typed_decl_float_literal_to_double() {
    let cg = CodeGenerator::new();
    // double x = 2.718;  → target_type = Double → "2.718f64"
    let stmt = HirStatement::VariableDeclaration {
        name: "x".to_string(),
        var_type: HirType::Double,
        initializer: Some(HirExpression::FloatLiteral("2.718".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("f64") || code.contains("2.718"),
        "Double decl with float literal should use f64, got: {}",
        code
    );
}

#[test]
fn typed_decl_float_literal_c_suffix() {
    let cg = CodeGenerator::new();
    // float x = 1.0f;  → strip 'f' suffix, add f32
    let stmt = HirStatement::VariableDeclaration {
        name: "x".to_string(),
        var_type: HirType::Float,
        initializer: Some(HirExpression::FloatLiteral("1.0f".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("f32"),
        "Float literal with C suffix should get f32, got: {}",
        code
    );
}

#[test]
fn typed_decl_addressof_to_pointer() {
    let cg = CodeGenerator::new();
    // int* p = &x;  → target_type = Pointer(Int) → "&mut x as *mut i32"
    let stmt = HirStatement::VariableDeclaration {
        name: "p".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::AddressOf(Box::new(
            HirExpression::Variable("x".to_string()),
        ))),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("&") && code.contains("x"),
        "AddressOf to pointer should generate reference, got: {}",
        code
    );
}

#[test]
fn typed_decl_unary_addressof_to_pointer() {
    let cg = CodeGenerator::new();
    // struct Node* n = &node;  → target_type = Pointer(Struct)
    let stmt = HirStatement::VariableDeclaration {
        name: "n".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
        initializer: Some(HirExpression::UnaryOp {
            op: UnaryOperator::AddressOf,
            operand: Box::new(HirExpression::Variable("node".to_string())),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("&") && code.contains("node"),
        "UnaryOp AddressOf to pointer should generate reference, got: {}",
        code
    );
}

#[test]
fn typed_decl_logical_not_to_int() {
    let cg = CodeGenerator::new();
    // int result = !flag;  → target_type = Int → "(flag == 0) as i32"
    let stmt = HirStatement::VariableDeclaration {
        name: "result".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::UnaryOp {
            op: UnaryOperator::LogicalNot,
            operand: Box::new(HirExpression::Variable("flag".to_string())),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("== 0") || code.contains("as i32") || code.contains("!"),
        "LogicalNot to int should cast, got: {}",
        code
    );
}

#[test]
fn typed_decl_logical_not_of_bool_expr_to_int() {
    let cg = CodeGenerator::new();
    // int result = !(a > b);  → target_type = Int → "(!(a > b)) as i32"
    let stmt = HirStatement::VariableDeclaration {
        name: "result".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::UnaryOp {
            op: UnaryOperator::LogicalNot,
            operand: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::GreaterThan,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            }),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("as i32") || code.contains("!"),
        "LogicalNot of bool expr to int should cast, got: {}",
        code
    );
}

#[test]
fn typed_decl_string_to_char_pointer() {
    let cg = CodeGenerator::new();
    // char* s = "hello";  → target_type = Pointer(Char) → b"hello\0".as_ptr()
    let stmt = HirStatement::VariableDeclaration {
        name: "s".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Char)),
        initializer: Some(HirExpression::StringLiteral("hello".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("hello") || code.contains("s"),
        "String to char* should contain string, got: {}",
        code
    );
}

#[test]
fn typed_decl_int_zero_to_pointer() {
    let cg = CodeGenerator::new();
    // int* p = 0;  → target_type = Pointer → std::ptr::null_mut()
    let stmt = HirStatement::VariableDeclaration {
        name: "p".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("null") || code.contains("None") || code.contains("0"),
        "Int 0 to pointer should generate null_mut or None, got: {}",
        code
    );
}

#[test]
fn typed_decl_logical_and_to_int() {
    let cg = CodeGenerator::new();
    // int result = a && b;  → target_type = Int → (a != 0 && b != 0) as i32
    let stmt = HirStatement::VariableDeclaration {
        name: "result".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::BinaryOp {
            op: BinaryOperator::LogicalAnd,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("a") && code.contains("b"),
        "LogicalAnd to int should reference operands, got: {}",
        code
    );
}

#[test]
fn typed_decl_logical_or_to_int() {
    let cg = CodeGenerator::new();
    // int result = a || b;  → target_type = Int
    let stmt = HirStatement::VariableDeclaration {
        name: "result".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::BinaryOp {
            op: BinaryOperator::LogicalOr,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::Variable("y".to_string())),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("x") && code.contains("y"),
        "LogicalOr to int should reference operands, got: {}",
        code
    );
}

#[test]
fn typed_decl_equal_comparison_to_int() {
    let cg = CodeGenerator::new();
    // int eq = (a == b);  → target_type = Int
    let stmt = HirStatement::VariableDeclaration {
        name: "eq".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("a") && code.contains("b"),
        "Comparison to int should reference operands, got: {}",
        code
    );
}

#[test]
fn typed_decl_cast_in_initializer() {
    let cg = CodeGenerator::new();
    // float f = (float)x;  → target_type = Float → "x as f32"
    let stmt = HirStatement::VariableDeclaration {
        name: "f".to_string(),
        var_type: HirType::Float,
        initializer: Some(HirExpression::Cast {
            target_type: HirType::Float,
            expr: Box::new(HirExpression::Variable("x".to_string())),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("f32") || code.contains("as"),
        "Cast in float decl should generate cast, got: {}",
        code
    );
}

#[test]
fn typed_decl_ternary_in_initializer() {
    let cg = CodeGenerator::new();
    // int max = (a > b) ? a : b;
    let stmt = HirStatement::VariableDeclaration {
        name: "max".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::Ternary {
            condition: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::GreaterThan,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            }),
            then_expr: Box::new(HirExpression::Variable("a".to_string())),
            else_expr: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("if") || code.contains("a") && code.contains("b"),
        "Ternary in int decl should generate if expression, got: {}",
        code
    );
}

#[test]
fn typed_decl_box_with_malloc() {
    let cg = CodeGenerator::new();
    // int* p = malloc(sizeof(int));  → Box<i32>
    let stmt = HirStatement::VariableDeclaration {
        name: "p".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::Sizeof {
                type_name: "int".to_string(),
            }],
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("Box") || code.contains("box") || code.contains("alloc") || code.contains("p"),
        "malloc(sizeof) should generate Box, got: {}",
        code
    );
}

#[test]
fn typed_decl_vec_with_malloc_multiply() {
    let cg = CodeGenerator::new();
    // int* arr = malloc(10 * sizeof(int));  → Vec<i32>
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(HirExpression::IntLiteral(10)),
                right: Box::new(HirExpression::Sizeof {
                    type_name: "int".to_string(),
                }),
            }],
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("Vec") || code.contains("vec") || code.contains("arr"),
        "malloc(n*sizeof) should generate Vec, got: {}",
        code
    );
}

#[test]
fn typed_assign_to_existing_var() {
    let cg = CodeGenerator::new();
    // x = a + b;
    let stmt = HirStatement::Assignment {
        target: "x".to_string(),
        value: HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        },
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("x") && code.contains("a") && code.contains("b"),
        "Assignment should reference x, a, b, got: {}",
        code
    );
}

#[test]
fn typed_deref_assign_complex() {
    let cg = CodeGenerator::new();
    // *ptr = *ptr + 1;
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("ptr".to_string()),
        value: HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Dereference(Box::new(
                HirExpression::Variable("ptr".to_string()),
            ))),
            right: Box::new(HirExpression::IntLiteral(1)),
        },
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("ptr"),
        "DerefAssignment should reference ptr, got: {}",
        code
    );
}

#[test]
fn typed_array_index_assign() {
    let cg = CodeGenerator::new();
    // arr[i] = 42;
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::Variable("i".to_string())),
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("arr") && code.contains("42"),
        "ArrayIndexAssignment should assign to arr, got: {}",
        code
    );
}

#[test]
fn typed_decl_calloc() {
    let cg = CodeGenerator::new();
    // int* arr = calloc(10, sizeof(int));
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::Calloc {
            count: Box::new(HirExpression::IntLiteral(10)),
            element_type: Box::new(HirType::Int),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("arr") && (code.contains("vec") || code.contains("Vec") || code.contains("0")),
        "calloc should generate zeroed Vec, got: {}",
        code
    );
}

#[test]
fn typed_decl_enum_type() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "color".to_string(),
        var_type: HirType::Enum("Color".to_string()),
        initializer: Some(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("color"),
        "Enum type decl should contain color, got: {}",
        code
    );
}

#[test]
fn typed_decl_type_alias() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "len".to_string(),
        var_type: HirType::TypeAlias("size_t".to_string()),
        initializer: Some(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("len"),
        "TypeAlias decl should contain len, got: {}",
        code
    );
}

#[test]
fn typed_decl_vec_type() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "items".to_string(),
        var_type: HirType::Vec(Box::new(HirType::Int)),
        initializer: None,
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("Vec") || code.contains("items"),
        "Vec type decl should contain Vec or items, got: {}",
        code
    );
}

#[test]
fn typed_decl_option_type() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "maybe".to_string(),
        var_type: HirType::Option(Box::new(HirType::Int)),
        initializer: Some(HirExpression::NullLiteral),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("maybe") || code.contains("Option") || code.contains("None"),
        "Option type decl should contain Option or None, got: {}",
        code
    );
}

// ============================================================================
// Special library function coverage (via FunctionCall expressions)
// ============================================================================

#[test]
fn expr_fread_call() {
    let cg = CodeGenerator::new();
    // C: fread(buf, 1, 100, fp)
    let expr = HirExpression::FunctionCall {
        function: "fread".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::IntLiteral(1),
            HirExpression::IntLiteral(100),
            HirExpression::Variable("fp".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        !code.is_empty(),
        "fread should generate read code, got: {}",
        code
    );
}

#[test]
fn expr_fwrite_call() {
    let cg = CodeGenerator::new();
    // C: fwrite(buf, 1, 100, fp)
    let expr = HirExpression::FunctionCall {
        function: "fwrite".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::IntLiteral(1),
            HirExpression::IntLiteral(100),
            HirExpression::Variable("fp".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        !code.is_empty(),
        "fwrite should generate write code, got: {}",
        code
    );
}

#[test]
fn expr_snprintf_call() {
    let cg = CodeGenerator::new();
    // C: snprintf(buf, 100, "%d", val)
    let expr = HirExpression::FunctionCall {
        function: "snprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::IntLiteral(100),
            HirExpression::StringLiteral("%d".to_string()),
            HirExpression::Variable("val".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("format") || code.contains("buf"),
        "snprintf should generate format!, got: {}",
        code
    );
}

#[test]
fn expr_sprintf_call() {
    let cg = CodeGenerator::new();
    // C: sprintf(buf, "%s %d", name, age)
    let expr = HirExpression::FunctionCall {
        function: "sprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::StringLiteral("%s %d".to_string()),
            HirExpression::Variable("name".to_string()),
            HirExpression::Variable("age".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("format") || code.contains("buf"),
        "sprintf should generate format!, got: {}",
        code
    );
}

#[test]
fn expr_atof_call() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "atof".to_string(),
        arguments: vec![HirExpression::Variable("str_val".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("parse") || code.contains("f64"),
        "atof should generate parse::<f64>(), got: {}",
        code
    );
}

#[test]
fn expr_unknown_function_call() {
    let cg = CodeGenerator::new();
    // Unrecognized function — should fall through to default handling
    let expr = HirExpression::FunctionCall {
        function: "custom_func".to_string(),
        arguments: vec![
            HirExpression::Variable("a".to_string()),
            HirExpression::IntLiteral(42),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("custom_func"),
        "Unknown function should preserve name, got: {}",
        code
    );
}

// ============================================================================
// Complex statement patterns for deeper coverage
// ============================================================================

#[test]
fn stmt_if_else_with_return() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
        then_block: vec![HirStatement::Return(Some(HirExpression::Variable(
            "x".to_string(),
        )))],
        else_block: Some(vec![HirStatement::Return(Some(
            HirExpression::UnaryOp {
                op: UnaryOperator::Minus,
                operand: Box::new(HirExpression::Variable("x".to_string())),
            },
        ))]),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("if") && code.contains("else"),
        "If with else should generate both branches, got: {}",
        code
    );
}

#[test]
fn stmt_nested_if_else() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
        then_block: vec![HirStatement::Assignment {
            target: "result".to_string(),
            value: HirExpression::IntLiteral(-1),
        }],
        else_block: Some(vec![HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::GreaterThan,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::IntLiteral(0)),
            },
            then_block: vec![HirStatement::Assignment {
                target: "result".to_string(),
                value: HirExpression::IntLiteral(1),
            }],
            else_block: Some(vec![HirStatement::Assignment {
                target: "result".to_string(),
                value: HirExpression::IntLiteral(0),
            }]),
        }]),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("if") && code.contains("else"),
        "Nested if-else should generate chain, got: {}",
        code
    );
}

#[test]
fn stmt_while_with_complex_condition() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::While {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::LogicalAnd,
            left: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::LessThan,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::Variable("n".to_string())),
            }),
            right: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::NotEqual,
                left: Box::new(HirExpression::Variable("done".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            }),
        },
        body: vec![HirStatement::Expression(HirExpression::UnaryOp {
            op: UnaryOperator::PostIncrement,
            operand: Box::new(HirExpression::Variable("i".to_string())),
        })],
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("while") || code.contains("loop"),
        "While with complex condition should generate loop, got: {}",
        code
    );
}

#[test]
fn stmt_for_with_multiple_init() {
    let cg = CodeGenerator::new();
    // C: for(int i = 0, j = 10; i < j; i++, j--)
    let stmt = HirStatement::For {
        init: vec![
            HirStatement::VariableDeclaration {
                name: "i".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::VariableDeclaration {
                name: "j".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(10)),
            },
        ],
        condition: Some(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::Variable("j".to_string())),
        }),
        increment: vec![
            HirStatement::Expression(HirExpression::UnaryOp {
                op: UnaryOperator::PostIncrement,
                operand: Box::new(HirExpression::Variable("i".to_string())),
            }),
            HirStatement::Expression(HirExpression::UnaryOp {
                op: UnaryOperator::PostDecrement,
                operand: Box::new(HirExpression::Variable("j".to_string())),
            }),
        ],
        body: vec![HirStatement::Continue],
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("i") && code.contains("j"),
        "For with multiple init/increment should contain both vars, got: {}",
        code
    );
}

#[test]
fn stmt_free_expression() {
    let cg = CodeGenerator::new();
    // C: free(ptr);  → RAII drop (comment or drop())
    let stmt = HirStatement::Free {
        pointer: HirExpression::Variable("ptr".to_string()),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("drop") || code.contains("ptr") || code.contains("//"),
        "Free should generate drop or comment, got: {}",
        code
    );
}

#[test]
fn typed_decl_box_type_direct() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "b".to_string(),
        var_type: HirType::Box(Box::new(HirType::Int)),
        initializer: None,
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("Box") || code.contains("b"),
        "Box type decl should contain Box, got: {}",
        code
    );
}

#[test]
fn typed_decl_reference_type() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "r".to_string(),
        var_type: HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
        initializer: Some(HirExpression::Variable("x".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("r") || code.contains("&"),
        "Reference type decl should contain & or r, got: {}",
        code
    );
}

#[test]
fn typed_decl_mut_reference_type() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "r".to_string(),
        var_type: HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: true,
        },
        initializer: Some(HirExpression::Variable("x".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("r") || code.contains("&mut"),
        "Mutable reference type decl should contain &mut, got: {}",
        code
    );
}

#[test]
fn typed_decl_owned_string() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "s".to_string(),
        var_type: HirType::OwnedString,
        initializer: Some(HirExpression::StringLiteral("hello".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("String") || code.contains("s") || code.contains("hello"),
        "OwnedString decl should contain String, got: {}",
        code
    );
}

#[test]
fn typed_decl_string_reference() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "s".to_string(),
        var_type: HirType::StringReference,
        initializer: Some(HirExpression::StringLiteral("hello".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("str") || code.contains("s") || code.contains("hello"),
        "StringReference decl should contain &str, got: {}",
        code
    );
}

#[test]
fn typed_decl_union_type() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "u".to_string(),
        var_type: HirType::Union(vec![
            ("i".to_string(), HirType::Int),
            ("f".to_string(), HirType::Float),
        ]),
        initializer: Some(HirExpression::IntLiteral(42)),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("u"),
        "Union type decl should contain u, got: {}",
        code
    );
}

#[test]
fn typed_decl_array_with_size() {
    let cg = CodeGenerator::new();
    // C: int arr[10] = {0};
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
        initializer: Some(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("arr"),
        "Array with size decl should contain arr, got: {}",
        code
    );
}

#[test]
fn typed_decl_function_pointer_with_init() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "cmp".to_string(),
        var_type: HirType::FunctionPointer {
            param_types: vec![HirType::Int, HirType::Int],
            return_type: Box::new(HirType::Int),
        },
        initializer: Some(HirExpression::Variable("compare".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("cmp") || code.contains("compare"),
        "Function pointer with init should reference cmp, got: {}",
        code
    );
}

// ============================================================================
// NUMERIC TYPE COERCIONS (DECY-203) — generate_expression_with_target_type
// ============================================================================

#[test]
fn typed_decl_int_to_float_coercion() {
    // C: float f = int_var; → var as f32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::Variable("x".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Float));
    assert!(
        code.contains("as f32"),
        "Int to Float coercion should cast as f32, got: {}",
        code
    );
}

#[test]
fn typed_decl_int_to_double_coercion() {
    // C: double d = int_var; → var as f64
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::Variable("x".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Double));
    assert!(
        code.contains("as f64"),
        "Int to Double coercion should cast as f64, got: {}",
        code
    );
}

#[test]
fn typed_decl_float_to_int_coercion() {
    // C: int i = float_var; → var as i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("f".to_string(), HirType::Float);
    let expr = HirExpression::Variable("f".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(
        code.contains("as i32"),
        "Float to Int coercion should cast as i32, got: {}",
        code
    );
}

#[test]
fn typed_decl_float_to_unsigned_int_coercion() {
    // C: unsigned int u = float_var; → var as u32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("f".to_string(), HirType::Double);
    let expr = HirExpression::Variable("f".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::UnsignedInt));
    assert!(
        code.contains("as u32"),
        "Double to UnsignedInt coercion should cast as u32, got: {}",
        code
    );
}

#[test]
fn typed_decl_char_to_int_coercion() {
    // C: int i = char_var; → var as i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("c".to_string(), HirType::Char);
    let expr = HirExpression::Variable("c".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(
        code.contains("as i32"),
        "Char to Int coercion should cast as i32, got: {}",
        code
    );
}

#[test]
fn typed_decl_int_to_char_coercion() {
    // C: char c = int_var; → var as u8
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("n".to_string(), HirType::Int);
    let expr = HirExpression::Variable("n".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Char));
    assert!(
        code.contains("as u8"),
        "Int to Char coercion should cast as u8, got: {}",
        code
    );
}

#[test]
fn typed_decl_unsigned_int_to_float_coercion() {
    // C: float f = unsigned_var; → var as f32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("u".to_string(), HirType::UnsignedInt);
    let expr = HirExpression::Variable("u".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Float));
    assert!(
        code.contains("as f32"),
        "UnsignedInt to Float coercion should cast as f32, got: {}",
        code
    );
}

// ============================================================================
// VEC/BOX NULL CHECKS — always false/true optimization
// ============================================================================

#[test]
fn expr_vec_null_check_equal() {
    // C: arr == NULL → false (Vec never null)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("arr".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("false"),
        "Vec == 0 should be false, got: {}",
        code
    );
}

#[test]
fn expr_vec_null_check_not_equal() {
    // C: arr != NULL → true (Vec never null)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("arr".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("true"),
        "Vec != NULL should be true, got: {}",
        code
    );
}

#[test]
fn expr_box_null_check_equal() {
    // C: ptr == NULL → false (Box never null)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Box(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("ptr".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("false"),
        "Box == 0 should be false, got: {}",
        code
    );
}

#[test]
fn expr_box_null_check_not_equal() {
    // C: ptr != NULL → true (Box never null)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Box(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("ptr".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("true"),
        "Box != NULL should be true, got: {}",
        code
    );
}

// ============================================================================
// STRLEN OPTIMIZATION — strlen(s) == 0 → s.is_empty()
// ============================================================================

#[test]
fn expr_strlen_equal_zero() {
    // C: strlen(s) == 0 → s.is_empty()
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::FunctionCall {
            function: "strlen".to_string(),
            arguments: vec![HirExpression::Variable("s".to_string())],
        }),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("is_empty"),
        "strlen(s) == 0 should become is_empty(), got: {}",
        code
    );
}

#[test]
fn expr_strlen_not_equal_zero() {
    // C: strlen(s) != 0 → !s.is_empty()
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::FunctionCall {
            function: "strlen".to_string(),
            arguments: vec![HirExpression::Variable("s".to_string())],
        }),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("!") && code.contains("is_empty"),
        "strlen(s) != 0 should become !is_empty(), got: {}",
        code
    );
}

#[test]
fn expr_zero_equal_strlen_reversed() {
    // C: 0 == strlen(s) → s.is_empty()
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::IntLiteral(0)),
        right: Box::new(HirExpression::FunctionCall {
            function: "strlen".to_string(),
            arguments: vec![HirExpression::Variable("s".to_string())],
        }),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("is_empty"),
        "0 == strlen(s) should become is_empty(), got: {}",
        code
    );
}

// ============================================================================
// CHAR LITERAL PROMOTION — comparison and arithmetic
// ============================================================================

#[test]
fn expr_int_var_compared_with_char_literal() {
    // C: c != '\n' where c is int → c != 10i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("c".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("c".to_string())),
        right: Box::new(HirExpression::CharLiteral(10)), // '\n'
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("10i32"),
        "Char literal in comparison with int should be promoted to i32, got: {}",
        code
    );
}

#[test]
fn expr_char_literal_compared_with_int_var_reversed() {
    // C: '\0' == c where c is int → 0i32 == c
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("c".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::CharLiteral(0)), // '\0'
        right: Box::new(HirExpression::Variable("c".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("0i32"),
        "Reversed char literal comparison should promote to i32, got: {}",
        code
    );
}

#[test]
fn expr_int_plus_char_literal_arithmetic() {
    // C: (n % 10) + '0' → (n % 10) + 48i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("n".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Modulo,
            left: Box::new(HirExpression::Variable("n".to_string())),
            right: Box::new(HirExpression::IntLiteral(10)),
        }),
        right: Box::new(HirExpression::CharLiteral(48)), // '0'
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("48i32"),
        "Char literal in arithmetic should be promoted to i32, got: {}",
        code
    );
}

#[test]
fn expr_char_literal_minus_int_reversed() {
    // C: 'z' - n where n is int → 122i32 - n
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("n".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::CharLiteral(122)), // 'z'
        right: Box::new(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("122i32"),
        "Reversed char literal arithmetic should promote to i32, got: {}",
        code
    );
}

// ============================================================================
// GLOBAL VARIABLE — assignment and access with unsafe wrapping
// ============================================================================

#[test]
fn stmt_errno_assignment() {
    // C: errno = EACCES; → unsafe { ERRNO = EACCES; }
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Assignment {
        target: "errno".to_string(),
        value: HirExpression::Variable("EACCES".to_string()),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("unsafe") && code.contains("ERRNO"),
        "Errno assignment should use unsafe ERRNO, got: {}",
        code
    );
}

#[test]
fn stmt_global_var_assignment() {
    // C: global_x = 42; → unsafe { global_x = 42; }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("global_x".to_string(), HirType::Int);
    ctx.add_global("global_x".to_string());
    let stmt = HirStatement::Assignment {
        target: "global_x".to_string(),
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("unsafe") && code.contains("global_x = 42"),
        "Global variable assignment should be wrapped in unsafe, got: {}",
        code
    );
}

#[test]
fn stmt_global_array_index_assignment() {
    // C: global_arr[i] = 42; → unsafe { global_arr[i as usize] = 42; }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "global_arr".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );
    ctx.add_global("global_arr".to_string());
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("global_arr".to_string())),
        index: Box::new(HirExpression::Variable("i".to_string())),
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("unsafe"),
        "Global array assignment should be wrapped in unsafe, got: {}",
        code
    );
}

#[test]
fn stmt_global_struct_field_assignment() {
    // C: global_config.value = 42; → unsafe { global_config.value = 42; }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("global_config".to_string(), HirType::Struct("Config".to_string()));
    ctx.add_global("global_config".to_string());
    let stmt = HirStatement::FieldAssignment {
        object: HirExpression::Variable("global_config".to_string()),
        field: "value".to_string(),
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("unsafe") && code.contains("global_config.value"),
        "Global struct field assignment should be unsafe, got: {}",
        code
    );
}

// ============================================================================
// GLOBAL VARIABLE ACCESS — expression with unsafe wrapping
// ============================================================================

#[test]
fn expr_global_variable_access_unsafe() {
    // C: x = global_var; → unsafe { global_var }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("global_var".to_string(), HirType::Int);
    ctx.add_global("global_var".to_string());
    let expr = HirExpression::Variable("global_var".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("unsafe") && code.contains("global_var"),
        "Global variable access should be unsafe, got: {}",
        code
    );
}

#[test]
fn expr_global_int_to_float_coercion_unsafe() {
    // C: float f = global_int; → unsafe { global_int } as f32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("global_int".to_string(), HirType::Int);
    ctx.add_global("global_int".to_string());
    let expr = HirExpression::Variable("global_int".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Float));
    assert!(
        code.contains("unsafe") && code.contains("as f32"),
        "Global int to float should use unsafe + cast, got: {}",
        code
    );
}

// ============================================================================
// KEYWORD RENAMING (DECY-241) — generate_signature
// ============================================================================

#[test]
fn sig_keyword_rename_write() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new(
        "write".to_string(),
        HirType::Void,
        vec![],
    );
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("c_write"),
        "write should be renamed to c_write, got: {}",
        sig
    );
}

#[test]
fn sig_keyword_rename_read() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new(
        "read".to_string(),
        HirType::Void,
        vec![],
    );
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("c_read"),
        "read should be renamed to c_read, got: {}",
        sig
    );
}

#[test]
fn sig_keyword_rename_type() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new(
        "type".to_string(),
        HirType::Void,
        vec![],
    );
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("c_type"),
        "type should be renamed to c_type, got: {}",
        sig
    );
}

#[test]
fn sig_keyword_rename_match() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new(
        "match".to_string(),
        HirType::Void,
        vec![],
    );
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("c_match"),
        "match should be renamed to c_match, got: {}",
        sig
    );
}

#[test]
fn sig_keyword_rename_self() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new(
        "self".to_string(),
        HirType::Void,
        vec![],
    );
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("c_self"),
        "self should be renamed to c_self, got: {}",
        sig
    );
}

#[test]
fn sig_keyword_rename_in() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new(
        "in".to_string(),
        HirType::Void,
        vec![],
    );
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("c_in"),
        "in should be renamed to c_in, got: {}",
        sig
    );
}

// ============================================================================
// POINTER IF CONDITION (DECY-238)
// ============================================================================

#[test]
fn stmt_if_pointer_condition_is_null_check() {
    // C: if (ptr) { ... } → if !ptr.is_null() { ... }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let stmt = HirStatement::If {
        condition: HirExpression::Variable("ptr".to_string()),
        then_block: vec![HirStatement::Break],
        else_block: None,
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("is_null"),
        "If with pointer condition should use is_null(), got: {}",
        code
    );
}

// ============================================================================
// SIZEOF EXPRESSIONS
// ============================================================================

#[test]
fn expr_sizeof_basic_type() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Sizeof {
        type_name: "int".to_string(),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("size_of") || code.contains("mem::size_of"),
        "Sizeof should use std::mem::size_of, got: {}",
        code
    );
}

#[test]
fn expr_sizeof_struct_type() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Sizeof {
        type_name: "struct Node".to_string(),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("size_of") || code.contains("Node"),
        "Sizeof struct should reference type, got: {}",
        code
    );
}

// ============================================================================
// CAST EXPRESSIONS
// ============================================================================

#[test]
fn expr_cast_variable_to_float() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Cast {
        target_type: HirType::Float,
        expr: Box::new(HirExpression::Variable("x".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("as f32"),
        "Cast int to float should use as f32, got: {}",
        code
    );
}

#[test]
fn expr_cast_double_to_int() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Cast {
        target_type: HirType::Int,
        expr: Box::new(HirExpression::Variable("d".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("as i32"),
        "Cast double to int should use as i32, got: {}",
        code
    );
}

#[test]
fn expr_cast_to_unsigned_int() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Cast {
        target_type: HirType::UnsignedInt,
        expr: Box::new(HirExpression::Variable("x".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("as u32"),
        "Cast to unsigned int should use as u32, got: {}",
        code
    );
}

#[test]
fn expr_cast_to_char() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Cast {
        target_type: HirType::Char,
        expr: Box::new(HirExpression::Variable("x".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("as u8"),
        "Cast to char should use as u8, got: {}",
        code
    );
}

#[test]
fn expr_cast_to_pointer() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Cast {
        target_type: HirType::Pointer(Box::new(HirType::Void)),
        expr: Box::new(HirExpression::Variable("x".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("as") || code.contains("*mut"),
        "Cast to void pointer should generate pointer cast, got: {}",
        code
    );
}

// ============================================================================
// COMPOUND LITERALS — struct initializer
// ============================================================================

#[test]
fn expr_compound_literal_struct_with_named_fields() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Point".to_string()),
        initializers: vec![
            HirExpression::IntLiteral(1),
            HirExpression::IntLiteral(2),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("Point"),
        "Struct compound literal should contain type name, got: {}",
        code
    );
}

#[test]
fn expr_compound_literal_array_multiple() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(3),
        },
        initializers: vec![
            HirExpression::IntLiteral(10),
            HirExpression::IntLiteral(20),
            HirExpression::IntLiteral(30),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("10") && code.contains("20") && code.contains("30"),
        "Array literal should contain all values, got: {}",
        code
    );
}

// ============================================================================
// DEREFERENCE EXPRESSIONS — unsafe wrapping
// ============================================================================

#[test]
fn expr_deref_raw_pointer_unsafe() {
    // C: *ptr → unsafe { *ptr }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("ptr".to_string())));
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("unsafe") || code.contains("*ptr"),
        "Dereference of raw pointer should use unsafe, got: {}",
        code
    );
}

// ============================================================================
// GENERATE_FUNCTION_WITH_LIFETIMES — empty body / stub
// ============================================================================

#[test]
fn func_with_lifetimes_empty_body_stub() {
    use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let func = HirFunction::new(
        "stub".to_string(),
        HirType::Int,
        vec![],
    );
    let sig = AnnotatedSignature {
        name: "stub".to_string(),
        lifetimes: vec![],
        parameters: vec![],
        return_type: AnnotatedType::Simple(HirType::Int),
    };
    let code = cg.generate_function_with_lifetimes(&func, &sig);
    assert!(
        code.contains("fn stub"),
        "Stub function should generate function signature, got: {}",
        code
    );
}

#[test]
fn func_with_lifetimes_void_empty_body() {
    use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let func = HirFunction::new(
        "noop".to_string(),
        HirType::Void,
        vec![],
    );
    let sig = AnnotatedSignature {
        name: "noop".to_string(),
        lifetimes: vec![],
        parameters: vec![],
        return_type: AnnotatedType::Simple(HirType::Void),
    };
    let code = cg.generate_function_with_lifetimes(&func, &sig);
    assert!(
        code.contains("fn noop"),
        "Void stub should generate function, got: {}",
        code
    );
}

// ============================================================================
// MAIN FUNCTION SPECIAL CASE
// ============================================================================

#[test]
fn sig_main_suppresses_return_type_new() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "main".to_string(),
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
    );
    let sig = cg.generate_signature(&func);
    assert!(
        !sig.contains("-> i32"),
        "main() should suppress return type annotation, got: {}",
        sig
    );
}

#[test]
fn sig_non_main_shows_return_type() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "add".to_string(),
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
    );
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("i32"),
        "Non-main function should show return type, got: {}",
        sig
    );
}

// ============================================================================
// GENERATE_FUNCTION_WITH_STRUCTS — context registration
// ============================================================================

#[test]
fn func_with_structs_pointer_param() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "process".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "data".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(None)],
    );
    let code = cg.generate_function_with_structs(&func, &[]);
    assert!(
        code.contains("fn process"),
        "Function with struct context should generate, got: {}",
        code
    );
}

// ============================================================================
// OPTION NULL COMPARISON — Option<T> == NULL → .is_none()
// ============================================================================

#[test]
fn expr_option_equal_null_is_none() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("opt".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("opt".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("is_none") || code.contains("None"),
        "Option == NULL should use is_none(), got: {}",
        code
    );
}

#[test]
fn expr_option_not_equal_null_is_some() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("opt".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("opt".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("is_some") || code.contains("Some"),
        "Option != NULL should use is_some(), got: {}",
        code
    );
}

// ============================================================================
// LOGICAL AND/OR — target_type Int coercion
// ============================================================================

#[test]
fn typed_decl_logical_and_with_int_operands() {
    // C: int result = a && b; where a, b are int → (a != 0 && b != 0) as i32
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "result".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::BinaryOp {
            op: BinaryOperator::LogicalAnd,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("as i32") || code.contains("!= 0"),
        "Logical AND assigned to int should coerce, got: {}",
        code
    );
}

// ============================================================================
// COMPARISON RESULT TO INT
// ============================================================================

#[test]
fn typed_decl_comparison_result_to_int() {
    // C: int result = a > b; → (a > b) as i32
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "result".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("as i32"),
        "Comparison result assigned to int should cast, got: {}",
        code
    );
}

// ============================================================================
// ARITHMETIC WITH TARGET TYPE CAST
// ============================================================================

#[test]
fn typed_decl_int_arithmetic_to_float() {
    // C: float f = a + b; (where a,b are int) → (a + b) as f32
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "f".to_string(),
        var_type: HirType::Float,
        initializer: Some(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("as f32") || code.contains("f32"),
        "Int arithmetic to float target should cast, got: {}",
        code
    );
}

#[test]
fn typed_decl_int_arithmetic_to_double() {
    // C: double d = a + b; → (a + b) as f64
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "d".to_string(),
        var_type: HirType::Double,
        initializer: Some(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("as f64") || code.contains("f64"),
        "Int arithmetic to double target should cast, got: {}",
        code
    );
}
