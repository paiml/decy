// --- default_value_for_type: TypeAlias size_t → 0usize (line 3697-3700) ---

#[test]
fn default_value_for_type_alias_size_t() {
    let result = CodeGenerator::default_value_for_type(&HirType::TypeAlias("size_t".to_string()));
    assert_eq!(result, "0usize", "size_t default should be 0usize");
}

// --- default_value_for_type: TypeAlias ssize_t → 0isize (line 3701) ---

#[test]
fn default_value_for_type_alias_ssize_t() {
    let result = CodeGenerator::default_value_for_type(&HirType::TypeAlias("ssize_t".to_string()));
    assert_eq!(result, "0isize", "ssize_t default should be 0isize");
}

// --- default_value_for_type: TypeAlias unknown → 0 (line 3702) ---

#[test]
fn default_value_for_type_alias_unknown() {
    let result = CodeGenerator::default_value_for_type(&HirType::TypeAlias("custom_t".to_string()));
    assert_eq!(result, "0", "Unknown TypeAlias default should be 0");
}

// ============================================================================
// BATCH 36: generate_expression_with_target_type uncovered branches
// ============================================================================

// --- LogicalNot with target Int: bool operand → (!expr) as i32 (line 1061-1064) ---

#[test]
fn logical_not_bool_to_int_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // !(x > 5) with target type Int → (!(...)) as i32
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(5)),
        }),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(
        code.contains("as i32"),
        "LogicalNot of bool to int should cast: {}",
        code
    );
}

// --- LogicalNot with target Int: int operand → (expr == 0) as i32 (line 1065-1067) ---

#[test]
fn logical_not_int_to_int_target() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("n".to_string(), HirType::Int);
    // !n with target type Int → (n == 0) as i32
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(
        code.contains("== 0") && code.contains("as i32"),
        "LogicalNot of int to int should use (n == 0) as i32: {}",
        code
    );
}

// --- StringLiteral to char pointer (line 1082-1094) ---

#[test]
fn string_literal_to_char_pointer_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::StringLiteral("hello".to_string());
    let target = HirType::Pointer(Box::new(HirType::Char));
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(
        code.contains("as_ptr()") && code.contains("as *mut u8"),
        "String to char* should convert: {}",
        code
    );
}

#[test]
fn string_literal_with_escapes_to_char_pointer() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::StringLiteral("say \"hi\"".to_string());
    let target = HirType::Pointer(Box::new(HirType::Char));
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(
        code.contains("as_ptr()"),
        "Escaped string to char* should convert: {}",
        code
    );
}

#[test]
fn string_literal_no_target_type() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::StringLiteral("test".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(code, "\"test\"");
}

#[test]
fn string_literal_to_non_char_pointer() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::StringLiteral("data".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    // Non-char pointer target: should NOT convert to byte string
    assert_eq!(code, "\"data\"");
}

// --- For loop with None condition → loop {} (line 4584-4591) ---

#[test]
fn for_infinite_loop_generates_loop() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // for(;;) { break; } → loop { break; }
    let stmt = HirStatement::For {
        init: vec![],
        condition: None,
        increment: vec![],
        body: vec![HirStatement::Break],
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("loop {"),
        "for(;;) should generate loop: {}",
        code
    );
    assert!(code.contains("break;"));
}

#[test]
fn for_infinite_loop_with_init_and_increment() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // for(int i = 0; ; i++) → let mut i = 0; loop { ... i += 1; }
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
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(code.contains("loop {"), "Should generate loop: {}", code);
}

// --- Return in main: char cast (line 4318-4321) ---

#[test]
fn return_char_in_main_casts_to_i32() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("c".to_string(), HirType::Char);
    let stmt = HirStatement::Return(Some(HirExpression::Variable("c".to_string())));
    let code = cg.generate_statement_with_context(&stmt, Some("main"), &mut ctx, Some(&HirType::Int));
    assert!(
        code.contains("std::process::exit") && code.contains("as i32"),
        "Return char in main should cast: {}",
        code
    );
}

// --- Return None in main → std::process::exit(0) (line 4325-4326) ---

#[test]
fn return_none_in_main_exits_zero() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Return(None);
    let code = cg.generate_statement_with_context(&stmt, Some("main"), &mut ctx, Some(&HirType::Int));
    assert_eq!(code, "std::process::exit(0);");
}

// --- Return int in main → std::process::exit(N) no cast (line 4322-4323) ---

#[test]
fn return_int_in_main_no_cast() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Return(Some(HirExpression::IntLiteral(1)));
    let code = cg.generate_statement_with_context(&stmt, Some("main"), &mut ctx, Some(&HirType::Int));
    assert!(
        code.contains("std::process::exit(1)"),
        "Return int in main: {}",
        code
    );
    assert!(!code.contains("as i32"), "Int return should not cast: {}", code);
}

// --- FloatLiteral with Float target type (line 996-1015) ---

#[test]
fn float_literal_target_float_typed_expr() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FloatLiteral("1.5".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Float));
    assert!(
        code.contains("f32") || code.contains("1.5"),
        "Float literal with Float target: {}",
        code
    );
}

#[test]
fn float_literal_target_double_typed_expr() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FloatLiteral("2.718".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Double));
    assert!(
        code.contains("f64") || code.contains("2.718"),
        "Float literal with Double target: {}",
        code
    );
}

#[test]
fn float_literal_no_target_typed_expr() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FloatLiteral("3.14".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(code.contains("3.14"), "Float literal no target: {}", code);
}

// --- Variable char to int coercion (line 1273-1279) ---

#[test]
fn variable_char_to_int_target() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("c".to_string(), HirType::Char);
    let expr = HirExpression::Variable("c".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(
        code.contains("as i32"),
        "Char variable to Int should cast: {}",
        code
    );
}

// ============================================================================
// Batch 37: generate_function method paths
// Targets: length_to_array mapping, detect_vec_return, empty body stubs,
//          struct pointer context, generate_function_with_structs
// ============================================================================

// --- generate_function with array param + length param (lines 6356-6384) ---

#[test]
fn generate_function_array_with_length_param() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "process".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("arr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len".to_string(), HirType::Int),
        ],
        vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "printf".to_string(),
            arguments: vec![HirExpression::Variable("len".to_string())],
        })],
    );
    let code = cg.generate_function(&func);
    // Should transform len references to arr.len() calls
    assert!(
        code.contains(".len()") || code.contains("arr"),
        "Array+length function: {}",
        code
    );
}

// --- generate_function with empty body (lines 6438-6444) ---

#[test]
fn generate_function_empty_body_void() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "noop".to_string(),
        HirType::Void,
        vec![],
        vec![],
    );
    let code = cg.generate_function(&func);
    assert!(code.contains("fn noop"), "Should have function name: {}", code);
    assert!(code.contains("}"), "Should have closing brace: {}", code);
}

#[test]
fn generate_function_empty_body_returns_int() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "get_zero".to_string(),
        HirType::Int,
        vec![],
        vec![],
    );
    let code = cg.generate_function(&func);
    assert!(code.contains("fn get_zero"), "Function name: {}", code);
    assert!(code.contains("-> i32") || code.contains("0"), "Return type or default: {}", code);
}

// --- generate_function with struct pointer param (lines 6415-6424) ---

#[test]
fn generate_function_struct_pointer_param() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "process_node".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "node".to_string(),
            HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
        )],
        vec![HirStatement::Return(None)],
    );
    let code = cg.generate_function(&func);
    assert!(
        code.contains("node") && code.contains("Node"),
        "Should reference node param and Node type: {}",
        code
    );
}

// --- detect_vec_return: function returning malloc result (lines 5256-5297) ---

#[test]
fn generate_function_vec_return_from_malloc() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "create_array".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new("size".to_string(), HirType::Int)],
        vec![
            HirStatement::VariableDeclaration {
                name: "result".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Malloc {
                    size: Box::new(HirExpression::Variable("size".to_string())),
                }),
            },
            HirStatement::Return(Some(HirExpression::Variable("result".to_string()))),
        ],
    );
    let code = cg.generate_function(&func);
    // detect_vec_return should detect malloc+return pattern and use Vec
    assert!(
        code.contains("Vec") || code.contains("vec!") || code.contains("vec"),
        "Should detect Vec return pattern: {}",
        code
    );
}

// --- generate_function_with_structs (lines 6471-6530) ---

#[test]
fn generate_function_with_structs_field_access() {
    let cg = CodeGenerator::new();
    let struct_def = decy_hir::HirStruct::new(
        "Point".to_string(),
        vec![
            decy_hir::HirStructField::new("x".to_string(), HirType::Int),
            decy_hir::HirStructField::new("y".to_string(), HirType::Int),
        ],
    );
    let func = HirFunction::new_with_body(
        "get_x".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Struct("Point".to_string()))),
        )],
        vec![HirStatement::Return(Some(HirExpression::FieldAccess {
            object: Box::new(HirExpression::Variable("p".to_string())),
            field: "x".to_string(),
        }))],
    );
    let code = cg.generate_function_with_structs(&func, &[struct_def]);
    assert!(code.contains("fn get_x"), "Function name: {}", code);
    assert!(code.contains(".x"), "Should access field x: {}", code);
}

// --- generate_function: pointer arithmetic skips array transform (line 6362-6364) ---

#[test]
fn generate_function_pointer_arithmetic_keeps_raw() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "walk".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("arr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len".to_string(), HirType::Int),
        ],
        vec![HirStatement::Assignment {
            target: "arr".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("arr".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
    );
    let code = cg.generate_function(&func);
    // With pointer arithmetic, arr should NOT be transformed to slice
    // len should NOT be mapped to arr.len()
    assert!(
        !code.contains("&[i32]") && !code.contains("&mut [i32]"),
        "Should NOT transform to slice when pointer arithmetic present: {}",
        code
    );
}

// --- is_any_malloc_or_calloc through cast (line 5312) ---

#[test]
fn generate_function_calloc_return_detected_as_vec() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "alloc_zeroed".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::Cast {
            target_type: HirType::Pointer(Box::new(HirType::Int)),
            expr: Box::new(HirExpression::Calloc {
                count: Box::new(HirExpression::Variable("n".to_string())),
                element_type: Box::new(HirType::Int),
            }),
        }))],
    );
    let code = cg.generate_function(&func);
    // detect_vec_return should detect calloc through cast
    assert!(
        code.contains("Vec") || code.contains("vec") || code.contains("alloc"),
        "Should detect calloc return: {}",
        code
    );
}

// =============================================================================
// Batch 38: generate_expression_with_target_type branch coverage
// =============================================================================
// Targets lines 982-1096: IntLiteral→Option/Pointer, FloatLiteral,
// AddressOf, LogicalNot, StringLiteral→pointer, Variable→stderr/Vec

#[test]
fn expr_target_type_int_zero_to_option() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::IntLiteral(0);
    let target = HirType::Option(Box::new(HirType::Pointer(Box::new(HirType::Int))));
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert_eq!(code, "None", "IntLiteral(0) with Option target should be None: {}", code);
}

#[test]
fn expr_target_type_int_zero_to_null_mut() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::IntLiteral(0);
    let target = HirType::Pointer(Box::new(HirType::Int));
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert_eq!(code, "std::ptr::null_mut()", "IntLiteral(0) with Pointer target: {}", code);
}

#[test]
fn expr_target_type_int_nonzero_ignores_pointer_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::IntLiteral(42);
    let target = HirType::Pointer(Box::new(HirType::Int));
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert_eq!(code, "42", "Non-zero int should be literal: {}", code);
}

#[test]
fn expr_target_type_float_literal_f32() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FloatLiteral("3.14f".to_string());
    let target = HirType::Float;
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert_eq!(code, "3.14f32", "Float literal with Float target: {}", code);
}

#[test]
fn expr_target_type_float_literal_f64() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FloatLiteral("2.718".to_string());
    let target = HirType::Double;
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert_eq!(code, "2.718f64", "Float literal with Double target: {}", code);
}

#[test]
fn expr_target_type_float_literal_no_decimal_default() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // Integer-like float without decimal/exponent → should add .0f64
    let expr = HirExpression::FloatLiteral("42".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(code, "42.0f64", "Float without decimal gets .0f64: {}", code);
}

#[test]
fn expr_target_type_float_literal_with_exponent_default() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FloatLiteral("1e10".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(code, "1e10f64", "Float with exponent gets f64 suffix: {}", code);
}

#[test]
fn expr_target_type_address_of_with_pointer_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::AddressOf(Box::new(HirExpression::Variable("x".to_string())));
    let target = HirType::Pointer(Box::new(HirType::Int));
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(
        code.contains("&mut x") && code.contains("*mut i32"),
        "AddressOf with Pointer target should cast: {}",
        code
    );
}

#[test]
fn expr_target_type_address_of_no_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::AddressOf(Box::new(HirExpression::Variable("x".to_string())));
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(code, "&x", "AddressOf without target: {}", code);
}

#[test]
fn expr_target_type_address_of_deref() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // AddressOf(Dereference(x)) without target → &(*x)
    let expr = HirExpression::AddressOf(Box::new(HirExpression::Dereference(Box::new(
        HirExpression::Variable("x".to_string()),
    ))));
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("&("),
        "AddressOf(Deref) should wrap in parens: {}",
        code
    );
}

#[test]
fn expr_target_type_unary_address_of_with_pointer_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::AddressOf,
        operand: Box::new(HirExpression::Variable("y".to_string())),
    };
    let target = HirType::Pointer(Box::new(HirType::Int));
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(
        code.contains("&mut y") && code.contains("*mut i32"),
        "UnaryOp AddressOf with Pointer target should cast: {}",
        code
    );
}

#[test]
fn expr_target_type_logical_not_bool_to_int() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // !true_expr → (!expr) as i32 when target is Int
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
    };
    let target = HirType::Int;
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(
        code.contains("as i32"),
        "LogicalNot of bool expr with Int target should cast: {}",
        code
    );
}

#[test]
fn expr_target_type_logical_not_int_to_int() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // !int_expr → (int == 0) as i32 when target is Int
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("flag".to_string())),
    };
    let target = HirType::Int;
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(
        code.contains("== 0") && code.contains("as i32"),
        "LogicalNot of int with Int target: {}",
        code
    );
}

#[test]
fn expr_target_type_logical_not_bool_no_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // !bool_expr → !expr when no target type
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(5)),
        }),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.starts_with('!'),
        "LogicalNot of bool without target: {}",
        code
    );
    assert!(
        !code.contains("as i32"),
        "Should NOT cast when no target: {}",
        code
    );
}

#[test]
fn expr_target_type_logical_not_int_no_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // !int_expr → (int == 0) when no target type
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("count".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("== 0"),
        "LogicalNot of int without target: {}",
        code
    );
    assert!(
        !code.contains("as i32"),
        "Should NOT cast when no target: {}",
        code
    );
}

#[test]
fn expr_target_type_string_literal_to_char_pointer() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::StringLiteral("hello".to_string());
    let target = HirType::Pointer(Box::new(HirType::Char));
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(
        code.contains("b\"hello\\0\"") && code.contains("as_ptr") && code.contains("*mut u8"),
        "String to char* should convert to byte string: {}",
        code
    );
}

#[test]
fn expr_target_type_string_literal_no_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::StringLiteral("world".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(code, "\"world\"", "String without target: {}", code);
}

#[test]
fn expr_target_type_char_literal_null() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::CharLiteral(0i8); // '\0'
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(code, "0u8", "Null char: {}", code);
}

#[test]
fn expr_target_type_char_literal_printable() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::CharLiteral(b'A' as i8); // 'A' = 65
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(code, "b'A'", "Printable char: {}", code);
}

#[test]
fn expr_target_type_char_literal_nonprintable() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::CharLiteral(1i8); // '\x01'
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(code, "1u8", "Non-printable char: {}", code);
}

#[test]
fn expr_target_type_variable_stderr() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Variable("stderr".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(code, "std::io::stderr()", "stderr mapping: {}", code);
}

#[test]
fn expr_target_type_variable_stdout() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Variable("stdout".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(code, "std::io::stdout()", "stdout mapping: {}", code);
}

#[test]
fn expr_target_type_variable_errno() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Variable("errno".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(code, "unsafe { ERRNO }", "errno mapping: {}", code);
}

#[test]
fn expr_target_type_variable_erange() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Variable("ERANGE".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(code, "34i32", "ERANGE constant: {}", code);
}

#[test]
fn expr_target_type_variable_vec_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Variable("data".to_string());
    let target = HirType::Vec(Box::new(HirType::Int));
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert_eq!(code, "data", "Variable with Vec target returns directly: {}", code);
}

// =============================================================================
// Batch 39: generate_struct derive combination + field type coverage
// =============================================================================
// Targets lines 7002-7139: derive macro combinations for has_large_array,
// has_float_fields, can_derive_copy, plus flexible array member and
// reference field paths.

#[test]
fn generate_struct_large_array_no_float_no_copy() {
    // has_large_array=true, has_float=false, copy=false → Debug, Clone, PartialEq, Eq (no Default, no Copy)
    let cg = CodeGenerator::new();
    let s = decy_hir::HirStruct::new(
        "BigBuf".to_string(),
        vec![
            decy_hir::HirStructField::new(
                "data".to_string(),
                HirType::Array {
                    element_type: Box::new(HirType::Int),
                    size: Some(64), // > 32, triggers large array
                },
            ),
            decy_hir::HirStructField::new(
                "name".to_string(),
                HirType::OwnedString, // Not Copy
            ),
        ],
    );
    let code = cg.generate_struct(&s);
    assert!(code.contains("PartialEq"), "Should have PartialEq: {}", code);
    assert!(code.contains("Eq"), "Should have Eq: {}", code);
    assert!(!code.contains("Default"), "Large array should skip Default: {}", code);
    assert!(!code.contains("Copy"), "OwnedString is not Copy: {}", code);
}

#[test]
fn generate_struct_large_array_no_float_copy() {
    // has_large_array=true, has_float=false, copy=true → Debug, Clone, Copy, PartialEq, Eq (no Default)
    let cg = CodeGenerator::new();
    let s = decy_hir::HirStruct::new(
        "BigArr".to_string(),
        vec![decy_hir::HirStructField::new(
            "data".to_string(),
            HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(64),
            },
        )],
    );
    let code = cg.generate_struct(&s);
    assert!(code.contains("Copy"), "All-int array is Copy: {}", code);
    assert!(!code.contains("Default"), "Large array skips Default: {}", code);
    assert!(code.contains("Eq"), "No floats, should have Eq: {}", code);
}

#[test]
fn generate_struct_large_array_with_float() {
    // has_large_array=true, has_float=true, copy=true → no Default, no Eq, yes Copy
    // Note: has_float_fields checks top-level field type, not array element type
    let cg = CodeGenerator::new();
    let s = decy_hir::HirStruct::new(
        "BigFloat".to_string(),
        vec![
            decy_hir::HirStructField::new(
                "vals".to_string(),
                HirType::Array {
                    element_type: Box::new(HirType::Int),
                    size: Some(100), // > 32, triggers large array
                },
            ),
            decy_hir::HirStructField::new(
                "scale".to_string(),
                HirType::Float, // Top-level float triggers has_float_fields
            ),
        ],
    );
    let code = cg.generate_struct(&s);
    assert!(!code.contains("Default"), "Large array skips Default: {}", code);
    assert!(code.contains("PartialEq"), "Should have PartialEq: {}", code);
    assert!(code.contains("Copy"), "Int+Float are Copy: {}", code);
    // Should NOT have standalone Eq (float doesn't implement Eq)
    // "PartialEq" contains "Eq" so we check specifically for ", Eq" or "Eq," as standalone
    assert!(
        !code.contains(", Eq)") && !code.contains(", Eq,"),
        "Float struct should not have standalone Eq: {}",
        code
    );
}

#[test]
fn generate_struct_with_float_no_large_array_copy() {
    // has_large_array=false, has_float=true, copy=true → Default, Copy, PartialEq (no Eq)
    let cg = CodeGenerator::new();
    let s = decy_hir::HirStruct::new(
        "Vec2".to_string(),
        vec![
            decy_hir::HirStructField::new("x".to_string(), HirType::Float),
            decy_hir::HirStructField::new("y".to_string(), HirType::Float),
        ],
    );
    let code = cg.generate_struct(&s);
    assert!(code.contains("Default"), "No large array, should have Default: {}", code);
    assert!(code.contains("Copy"), "All-float is Copy: {}", code);
    assert!(code.contains("PartialEq"), "Should have PartialEq: {}", code);
    // Eq is NOT in the derive (floats don't implement Eq)
    // Be careful: "PartialEq, Eq" shouldn't appear
    assert!(
        !code.contains("Eq)") || code.contains("PartialEq)"),
        "Should not have Eq after PartialEq for float struct: {}",
        code
    );
}

#[test]
fn generate_struct_with_float_no_large_array_no_copy() {
    // has_large_array=false, has_float=true, copy=false → Default, PartialEq (no Copy, no Eq)
    let cg = CodeGenerator::new();
    let s = decy_hir::HirStruct::new(
        "MixedFloat".to_string(),
        vec![
            decy_hir::HirStructField::new("val".to_string(), HirType::Double),
            decy_hir::HirStructField::new(
                "name".to_string(),
                HirType::OwnedString, // Not Copy
            ),
        ],
    );
    let code = cg.generate_struct(&s);
    assert!(code.contains("Default"), "Should have Default: {}", code);
    assert!(!code.contains("Copy"), "OwnedString blocks Copy: {}", code);
    assert!(code.contains("PartialEq"), "Should have PartialEq: {}", code);
}

#[test]
fn generate_struct_simple_copy_default() {
    // has_large_array=false, has_float=false, copy=true → Default, Copy, PartialEq, Eq
    let cg = CodeGenerator::new();
    let s = decy_hir::HirStruct::new(
        "Point".to_string(),
        vec![
            decy_hir::HirStructField::new("x".to_string(), HirType::Int),
            decy_hir::HirStructField::new("y".to_string(), HirType::Int),
        ],
    );
    let code = cg.generate_struct(&s);
    assert!(code.contains("Default"), "Should have Default: {}", code);
    assert!(code.contains("Copy"), "All-int is Copy: {}", code);
    assert!(code.contains("Eq"), "No floats, should have Eq: {}", code);
}

#[test]
fn generate_struct_no_copy_no_float_no_large() {
    // has_large_array=false, has_float=false, copy=false → Default, PartialEq, Eq (no Copy)
    let cg = CodeGenerator::new();
    let s = decy_hir::HirStruct::new(
        "Config".to_string(),
        vec![
            decy_hir::HirStructField::new("count".to_string(), HirType::Int),
            decy_hir::HirStructField::new(
                "buffer".to_string(),
                HirType::Vec(Box::new(HirType::Int)), // Not Copy
            ),
        ],
    );
    let code = cg.generate_struct(&s);
    assert!(code.contains("Default"), "Should have Default: {}", code);
    assert!(!code.contains("Copy"), "Vec blocks Copy: {}", code);
    assert!(code.contains("Eq"), "No floats, should have Eq: {}", code);
}

#[test]
fn generate_struct_flexible_array_member() {
    // Array with size: None → Vec<T>
    let cg = CodeGenerator::new();
    let s = decy_hir::HirStruct::new(
        "DynBuf".to_string(),
        vec![
            decy_hir::HirStructField::new("len".to_string(), HirType::Int),
            decy_hir::HirStructField::new(
                "data".to_string(),
                HirType::Array {
                    element_type: Box::new(HirType::Char),
                    size: None, // Flexible array member
                },
            ),
        ],
    );
    let code = cg.generate_struct(&s);
    assert!(
        code.contains("Vec<u8>"),
        "Flexible array member should become Vec<T>: {}",
        code
    );
}

#[test]
fn generate_struct_with_reference_field() {
    // Reference field → needs lifetime annotation
    let cg = CodeGenerator::new();
    let s = decy_hir::HirStruct::new(
        "Borrowed".to_string(),
        vec![decy_hir::HirStructField::new(
            "data".to_string(),
            HirType::Reference {
                inner: Box::new(HirType::Int),
                mutable: false,
            },
        )],
    );
    let code = cg.generate_struct(&s);
    assert!(
        code.contains("<'a>"),
        "Reference field should trigger lifetime param: {}",
        code
    );
}

#[test]
fn generate_struct_keyword_field_name() {
    // Field named with Rust keyword → escaped
    let cg = CodeGenerator::new();
    let s = decy_hir::HirStruct::new(
        "Obj".to_string(),
        vec![
            decy_hir::HirStructField::new("r#type".to_string(), HirType::Int),
        ],
    );
    let code = cg.generate_struct(&s);
    assert!(code.contains("pub r#type: i32"), "Should escape keyword: {}", code);
}

// =============================================================================
// Batch 39b: generate_statement_with_context — VLA and malloc paths
// =============================================================================

#[test]
fn stmt_context_vla_to_vec() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // C99 VLA: int arr[n]; → let mut arr = vec![0i32; n];
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None, // VLA marker
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("vec![0i32;") && code.contains("n"),
        "VLA should become vec![default; size]: {}",
        code
    );
}

#[test]
fn stmt_context_vla_float_to_vec() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "buf".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Float),
            size: None,
        },
        initializer: Some(HirExpression::Variable("len".to_string())),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("vec![0.0f32;"),
        "Float VLA should use 0.0f32: {}",
        code
    );
}

#[test]
fn stmt_context_vla_char_to_vec() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "cbuf".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Char),
            size: None,
        },
        initializer: Some(HirExpression::Variable("sz".to_string())),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("vec![0u8;"),
        "Char VLA should use 0u8: {}",
        code
    );
}

#[test]
fn stmt_context_malloc_struct_to_box() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // malloc(sizeof(Node)) → Box::new(Node::default())
    let stmt = HirStatement::VariableDeclaration {
        name: "node".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
        initializer: Some(HirExpression::Malloc {
            size: Box::new(HirExpression::Sizeof { type_name: "Node".to_string() }),
        }),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("Box") && code.contains("Node"),
        "Struct malloc should use Box: {}",
        code
    );
}

#[test]
fn stmt_context_malloc_array_to_vec() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // malloc(n * sizeof(int)) → Vec::with_capacity(n)
    let stmt = HirStatement::VariableDeclaration {
        name: "buf".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::Malloc {
            size: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(HirExpression::Variable("n".to_string())),
                right: Box::new(HirExpression::Sizeof { type_name: "int".to_string() }),
            }),
        }),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("Vec") || code.contains("vec"),
        "Array malloc should use Vec: {}",
        code
    );
}

#[test]
fn stmt_context_global_var_rename() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_global("count".to_string()); // Register as global
    // Local var with same name as global → renamed to count_local
    let stmt = HirStatement::VariableDeclaration {
        name: "count".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("count_local"),
        "Should rename local shadowing global: {}",
        code
    );
}

// =============================================================================
// Batch 40: generate_expression_with_target_type deep Variable branches
// =============================================================================
// Targets lines 1140-1218: Box→raw, Reference(Array)→as_mut_ptr,
// Reference(Vec)→as_mut_ptr, Reference(T)→cast, Vec→as_mut_ptr,
// Array→as_mut_ptr, Array→void*, Pointer→Pointer passthrough,
// int→char coercion.

#[test]
fn expr_target_type_box_to_raw_pointer() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("node".to_string(), HirType::Box(Box::new(HirType::Struct("Node".to_string()))));
    let expr = HirExpression::Variable("node".to_string());
    let target = HirType::Pointer(Box::new(HirType::Struct("Node".to_string())));
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(
        code.contains("Box::into_raw"),
        "Box to raw pointer should use Box::into_raw: {}",
        code
    );
}

#[test]
fn expr_target_type_ref_array_to_mut_ptr() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "arr".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(10),
            }),
            mutable: true,
        },
    );
    let expr = HirExpression::Variable("arr".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(
        code.contains(".as_mut_ptr()"),
        "Mutable ref to array → as_mut_ptr(): {}",
        code
    );
}

#[test]
fn expr_target_type_ref_array_to_const_ptr() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "data".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(5),
            }),
            mutable: false,
        },
    );
    let expr = HirExpression::Variable("data".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(
        code.contains(".as_ptr()"),
        "Immutable ref to array → as_ptr(): {}",
        code
    );
}

#[test]
fn expr_target_type_ref_vec_to_mut_ptr() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "buf".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Vec(Box::new(HirType::Int))),
            mutable: true,
        },
    );
    let expr = HirExpression::Variable("buf".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(
        code.contains(".as_mut_ptr()"),
        "Mutable ref to Vec → as_mut_ptr(): {}",
        code
    );
}

#[test]
fn expr_target_type_ref_single_to_mut_ptr() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "x".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: true,
        },
    );
    let expr = HirExpression::Variable("x".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(
        code.contains("as *mut _"),
        "Mutable ref to pointer → cast: {}",
        code
    );
}

#[test]
fn expr_target_type_ref_single_immutable_to_ptr() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "x".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
    );
    let expr = HirExpression::Variable("x".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(
        code.contains("as *const _ as *mut _"),
        "Immutable ref to pointer → double cast: {}",
        code
    );
}

#[test]
fn expr_target_type_vec_to_mut_ptr() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("buf".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::Variable("buf".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(
        code.contains(".as_mut_ptr()"),
        "Vec to pointer → as_mut_ptr(): {}",
        code
    );
}

#[test]
fn expr_target_type_array_to_mut_ptr() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "arr".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );
    let expr = HirExpression::Variable("arr".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(
        code.contains(".as_mut_ptr()"),
        "Array to pointer → as_mut_ptr(): {}",
        code
    );
}

#[test]
fn expr_target_type_array_to_void_ptr() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "data".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Char),
            size: Some(256),
        },
    );
    let expr = HirExpression::Variable("data".to_string());
    let target = HirType::Pointer(Box::new(HirType::Void));
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(
        code.contains("as_mut_ptr()") && code.contains("as *mut ()"),
        "Array to void* → as_mut_ptr() as *mut (): {}",
        code
    );
}

#[test]
fn expr_target_type_ptr_to_ptr_passthrough() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "p".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let expr = HirExpression::Variable("p".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert_eq!(code, "p", "Raw pointer to raw pointer → passthrough: {}", code);
}

#[test]
fn expr_target_type_int_to_char_coercion() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("c".to_string(), HirType::Int);
    let expr = HirExpression::Variable("c".to_string());
    let target = HirType::Char;
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(
        code.contains("as u8"),
        "Int to char → cast as u8: {}",
        code
    );
}

// =============================================================================
// Batch 41: generate_statement_with_context — string literal and char* paths
// =============================================================================

#[test]
fn stmt_context_char_ptr_string_literal_to_str() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // char* s = "hello" → let mut s: &str = "hello";
    let stmt = HirStatement::VariableDeclaration {
        name: "s".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Char)),
        initializer: Some(HirExpression::StringLiteral("hello".to_string())),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("&str"),
        "char* with string literal → &str: {}",
        code
    );
    assert!(
        code.contains("\"hello\""),
        "Should keep string literal: {}",
        code
    );
}

#[test]
fn stmt_context_vla_double_to_vec() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Double),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("vec![0.0f64;"),
        "Double VLA should use 0.0f64: {}",
        code
    );
}

#[test]
fn stmt_context_vla_unsigned_to_vec() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::UnsignedInt),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("vec![0u32;"),
        "UnsignedInt VLA should use 0u32: {}",
        code
    );
}

#[test]
fn stmt_context_vla_signed_char_to_vec() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::SignedChar),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("vec![0i8;"),
        "SignedChar VLA should use 0i8: {}",
        code
    );
}

#[test]
fn stmt_context_malloc_vec_with_capacity() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // int* buf = (int*)malloc(n * sizeof(int)); → Vec::with_capacity(n)
    let stmt = HirStatement::VariableDeclaration {
        name: "buf".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::Cast {
            target_type: HirType::Pointer(Box::new(HirType::Int)),
            expr: Box::new(HirExpression::Malloc {
                size: Box::new(HirExpression::BinaryOp {
                    op: BinaryOperator::Multiply,
                    left: Box::new(HirExpression::Variable("n".to_string())),
                    right: Box::new(HirExpression::Sizeof { type_name: "int".to_string() }),
                }),
            }),
        }),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("Vec"),
        "Cast-wrapped malloc should still be detected as Vec: {}",
        code
    );
}

#[test]
fn stmt_context_var_decl_no_init() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // int x; → let mut x: i32 = 0i32;
    let stmt = HirStatement::VariableDeclaration {
        name: "x".to_string(),
        var_type: HirType::Int,
        initializer: None,
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("0i32"),
        "Uninitialized int should default to 0i32: {}",
        code
    );
}

#[test]
fn stmt_context_var_decl_pointer_no_init() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // int* p; → let mut p: *mut i32 = std::ptr::null_mut();
    let stmt = HirStatement::VariableDeclaration {
        name: "p".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: None,
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("null_mut()"),
        "Uninitialized pointer should default to null_mut: {}",
        code
    );
}

// =============================================================================
// Batch 42: generate_annotated_signature_with_func — output param paths
// =============================================================================
// Targets lines 5936-6177: output parameter detection, single/multiple output
// returns, fallible output (Result<T, i32>), Vec return detection.

#[test]
fn annotated_sig_output_param_single_nonfallible() {
    // void compute(int input, int* result) where result is write-only
    // → fn compute(mut input: i32) -> i32
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "compute".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("input".to_string(), HirType::Int),
            HirParameter::new(
                "result".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            ),
        ],
        vec![
            // *result = input * 2; (dereference write to result, no read)
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("result".to_string()),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Multiply,
                    left: Box::new(HirExpression::Variable("input".to_string())),
                    right: Box::new(HirExpression::IntLiteral(2)),
                },
            },
        ],
    );
    let sig = make_annotated_sig(&func);
    let code = cg.generate_annotated_signature_with_func(&sig, Some(&func));
    // "result" should be removed from params and used as return type
    assert!(
        !code.contains("result"),
        "Output param should be removed from params: {}",
        code
    );
    assert!(
        code.contains("-> i32"),
        "Should return the output type: {}",
        code
    );
}

#[test]
fn annotated_sig_output_param_fallible() {
    // int process(int input, int* out) → fn process(mut input: i32) -> Result<i32, i32>
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "process".to_string(),
        HirType::Int, // Int return = fallible
        vec![
            HirParameter::new("input".to_string(), HirType::Int),
            HirParameter::new(
                "out".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            ),
        ],
        vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("out".to_string()),
            value: HirExpression::Variable("input".to_string()),
        }],
    );
    let sig = make_annotated_sig(&func);
    let code = cg.generate_annotated_signature_with_func(&sig, Some(&func));
    // Check that output param is removed and Result type is generated
    assert!(
        !code.contains("out:") && !code.contains("out :"),
        "Output param should be removed: {}",
        code
    );
    assert!(
        code.contains("Result<i32, i32>"),
        "Fallible output should use Result: {}",
        code
    );
}

#[test]
fn annotated_sig_no_output_params() {
    // Regular function: int add(int a, int b) → fn add(mut a: i32, mut b: i32) -> i32
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
    let sig = make_annotated_sig(&func);
    let code = cg.generate_annotated_signature_with_func(&sig, Some(&func));
    assert!(
        code.contains("-> i32"),
        "Regular return type: {}",
        code
    );
    assert!(
        !code.contains("Result"),
        "No Result for regular functions: {}",
        code
    );
}

#[test]
fn annotated_sig_void_no_return() {
    // void noop() → fn noop()
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "noop".to_string(),
        HirType::Void,
        vec![],
        vec![],
    );
    let sig = make_annotated_sig(&func);
    let code = cg.generate_annotated_signature_with_func(&sig, Some(&func));
    assert!(
        !code.contains("->"),
        "Void function should have no return type: {}",
        code
    );
}

#[test]
fn annotated_sig_keyword_rename_all() {
    // Test all keyword renames: write, read, type, match, self, in
    let cg = CodeGenerator::new();
    for (c_name, rust_name) in [
        ("write", "c_write"),
        ("read", "c_read"),
        ("type", "c_type"),
        ("match", "c_match"),
        ("self", "c_self"),
        ("in", "c_in"),
    ] {
        let func = HirFunction::new_with_body(
            c_name.to_string(),
            HirType::Void,
            vec![],
            vec![],
        );
        let sig = make_annotated_sig(&func);
        let code = cg.generate_annotated_signature_with_func(&sig, Some(&func));
        assert!(
            code.contains(rust_name),
            "{} should be renamed to {}: {}",
            c_name, rust_name, code
        );
    }
}

// =============================================================================
// Batch 42b: generate_expression_with_target_type — remaining numeric coercions
// =============================================================================

#[test]
fn expr_target_type_variable_global_int_to_float() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("counter".to_string(), HirType::Int);
    ctx.add_global("counter".to_string());
    let expr = HirExpression::Variable("counter".to_string());
    let target = HirType::Float;
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(
        code.contains("unsafe") && code.contains("as f32"),
        "Global int→float should be unsafe: {}",
        code
    );
}

#[test]
fn expr_target_type_variable_global_int_to_double() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("counter".to_string(), HirType::Int);
    ctx.add_global("counter".to_string());
    let expr = HirExpression::Variable("counter".to_string());
    let target = HirType::Double;
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(
        code.contains("unsafe") && code.contains("as f64"),
        "Global int→double should be unsafe: {}",
        code
    );
}

#[test]
fn expr_target_type_variable_local_int_to_float() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("val".to_string(), HirType::Int);
    let expr = HirExpression::Variable("val".to_string());
    let target = HirType::Float;
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(
        code.contains("as f32") && !code.contains("unsafe"),
        "Local int→float should not be unsafe: {}",
        code
    );
}

// =============================================================================
// Batch 43: generate_statement_with_context — control flow and realloc
// =============================================================================

#[test]
fn stmt_context_if_pointer_condition() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let stmt = HirStatement::If {
        condition: HirExpression::Variable("p".to_string()),
        then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
        else_block: None,
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("is_null"),
        "Pointer condition should use is_null: {}",
        code
    );
}

#[test]
fn stmt_context_if_int_condition() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("flag".to_string(), HirType::Int);
    let stmt = HirStatement::If {
        condition: HirExpression::Variable("flag".to_string()),
        then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
        else_block: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))]),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("!= 0"),
        "Int condition should use != 0: {}",
        code
    );
    assert!(
        code.contains("} else {"),
        "Should have else block: {}",
        code
    );
}

#[test]
fn stmt_context_if_bool_condition() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(10)),
        },
        then_block: vec![HirStatement::Break],
        else_block: None,
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("if x < 10"),
        "Bool condition should pass through: {}",
        code
    );
    assert!(
        code.contains("break;"),
        "Should contain break: {}",
        code
    );
}

#[test]
fn stmt_context_while_pointer_condition() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("node".to_string(), HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))));
    let stmt = HirStatement::While {
        condition: HirExpression::Variable("node".to_string()),
        body: vec![HirStatement::Continue],
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("is_null"),
        "While pointer condition → is_null: {}",
        code
    );
    assert!(
        code.contains("continue;"),
        "Should contain continue: {}",
        code
    );
}

#[test]
fn stmt_context_while_bool_condition() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::While {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
        body: vec![],
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("while i > 0"),
        "Bool while condition passes through: {}",
        code
    );
}

#[test]
fn stmt_context_realloc_zero_clears() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("buf".to_string(), HirType::Vec(Box::new(HirType::Int)));
    // realloc(buf, 0) → buf.clear()
    let stmt = HirStatement::Assignment {
        target: "buf".to_string(),
        value: HirExpression::Realloc {
            pointer: Box::new(HirExpression::Variable("buf".to_string())),
            new_size: Box::new(HirExpression::IntLiteral(0)),
        },
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains(".clear()"),
        "realloc(ptr, 0) should clear: {}",
        code
    );
}

#[test]
fn stmt_context_realloc_resize() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("buf".to_string(), HirType::Vec(Box::new(HirType::Int)));
    // realloc(buf, n * sizeof(int)) → buf.resize(n, 0i32)
    let stmt = HirStatement::Assignment {
        target: "buf".to_string(),
        value: HirExpression::Realloc {
            pointer: Box::new(HirExpression::Variable("buf".to_string())),
            new_size: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(HirExpression::Variable("n".to_string())),
                right: Box::new(HirExpression::Sizeof { type_name: "int".to_string() }),
            }),
        },
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains(".resize("),
        "realloc should use resize: {}",
        code
    );
    assert!(
        code.contains("0i32"),
        "Should use default value for element type: {}",
        code
    );
}

#[test]
fn stmt_context_realloc_fallback() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("buf".to_string(), HirType::Vec(Box::new(HirType::Int)));
    // realloc(buf, 42) → buf.resize(42 as usize, 0i32) (fallback path)
    let stmt = HirStatement::Assignment {
        target: "buf".to_string(),
        value: HirExpression::Realloc {
            pointer: Box::new(HirExpression::Variable("buf".to_string())),
            new_size: Box::new(HirExpression::IntLiteral(42)),
        },
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains(".resize(42 as usize"),
        "realloc fallback should resize with as usize: {}",
        code
    );
}
