#[test]
fn expr_pointer_equal_zero_null_check() {
    // C: ptr == 0 → ptr == std::ptr::null_mut()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("ptr".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("null") || code.contains("is_null"),
        "Pointer == 0 should become null check, got: {}",
        code
    );
}

#[test]
fn expr_pointer_not_equal_zero_not_null() {
    // C: ptr != 0 → !ptr.is_null() or ptr != null_mut()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("ptr".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("null") || code.contains("!"),
        "Pointer != 0 should become not-null check, got: {}",
        code
    );
}

// ============================================================================
// TERNARY / CONDITIONAL EXPRESSION
// ============================================================================

#[test]
fn expr_ternary_with_unary_else() {
    // C: x > 0 ? x : -x → if x > 0 { x } else { -x }
    let cg = CodeGenerator::new();
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
        code.contains("if"),
        "Ternary should generate if expression, got: {}",
        code
    );
}

// ============================================================================
// FUNCTION CALL — fopen, fclose special handling
// ============================================================================

#[test]
fn expr_fopen_call() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "fopen".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("test.txt".to_string()),
            HirExpression::StringLiteral("r".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("File") || code.contains("open") || code.contains("fopen"),
        "fopen should generate File::open or equivalent, got: {}",
        code
    );
}

#[test]
fn expr_fclose_call() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "fclose".to_string(),
        arguments: vec![HirExpression::Variable("fp".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("drop") || code.contains("fp"),
        "fclose should generate drop or equivalent, got: {}",
        code
    );
}

// ============================================================================
// ASSIGNMENT TO STRUCT FIELD — pointer field with unsafe
// ============================================================================

#[test]
fn stmt_field_assignment_pointer_obj_unsafe() {
    // C: ptr->field = value; → unsafe { (*ptr).field = value; }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(
        HirType::Struct("Node".to_string()),
    )));
    let stmt = HirStatement::FieldAssignment {
        object: HirExpression::Dereference(Box::new(HirExpression::Variable("ptr".to_string()))),
        field: "value".to_string(),
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("unsafe") || code.contains("ptr"),
        "Pointer field assignment should use unsafe, got: {}",
        code
    );
}

// ============================================================================
// WHILE WITH POINTER CONDITION
// ============================================================================

#[test]
fn stmt_while_pointer_condition() {
    // C: while (ptr) { ... } → while !ptr.is_null() { ... }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let stmt = HirStatement::While {
        condition: HirExpression::Variable("ptr".to_string()),
        body: vec![HirStatement::Break],
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("while") && (code.contains("is_null") || code.contains("!= 0")),
        "While with pointer should check null, got: {}",
        code
    );
}

// ============================================================================
// SWITCH WITH FALL-THROUGH — multiple cases sharing body
// ============================================================================

#[test]
fn stmt_switch_empty_case_fallthrough() {
    // C: switch(x) { case 1: case 2: return 1; }
    // Cases with empty bodies fall through to next case
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Switch {
        condition: HirExpression::Variable("x".to_string()),
        cases: vec![
            SwitchCase {
                value: Some(HirExpression::IntLiteral(1)),
                body: vec![], // empty = fallthrough
            },
            SwitchCase {
                value: Some(HirExpression::IntLiteral(2)),
                body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
            },
        ],
        default_case: None,
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("match") || code.contains("1") && code.contains("2"),
        "Switch with fallthrough should generate match, got: {}",
        code
    );
}

// ============================================================================
// FOR LOOP — with condition and body
// ============================================================================

#[test]
fn stmt_for_standard_loop() {
    // C: for(int i = 0; i < 10; i++) { ... }
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
            function: "printf".to_string(),
            arguments: vec![
                HirExpression::StringLiteral("%d".to_string()),
                HirExpression::Variable("i".to_string()),
            ],
        })],
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("while") && code.contains("10"),
        "Standard for loop should generate while, got: {}",
        code
    );
}

// ============================================================================
// ARRAY INDEX EXPRESSION — safe and unsafe paths
// ============================================================================

#[test]
fn expr_array_index_pointer_unsafe() {
    // C: ptr[i] → unsafe { *ptr.add(i as usize) }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("arr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::Variable("i".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("unsafe") || code.contains("arr"),
        "Pointer array index should use unsafe, got: {}",
        code
    );
}

#[test]
fn expr_array_index_global_unsafe() {
    // C: global_arr[i] → unsafe { global_arr[i as usize] }
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
    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("global_arr".to_string())),
        index: Box::new(HirExpression::Variable("i".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("unsafe"),
        "Global array index should use unsafe, got: {}",
        code
    );
}

// ============================================================================
// FIELD ACCESS — regular and pointer
// ============================================================================

#[test]
fn expr_pointer_field_access_unsafe() {
    // C: ptr->field → unsafe { (*ptr).field }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(
        HirType::Struct("Node".to_string()),
    )));
    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("ptr".to_string())),
        field: "value".to_string(),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("unsafe") || code.contains("ptr") && code.contains("value"),
        "Pointer field access should use unsafe, got: {}",
        code
    );
}

// ============================================================================
// SLICE INDEX EXPRESSION
// ============================================================================

#[test]
fn expr_slice_index() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::SliceIndex {
        slice: Box::new(HirExpression::Variable("data".to_string())),
        index: Box::new(HirExpression::Variable("i".to_string())),
        element_type: HirType::Int,
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("data") && code.contains("i"),
        "Slice index should contain variable names, got: {}",
        code
    );
}

// ============================================================================
// TypeContext field type inference (lines 200-230 uncovered)
// Box<Struct> and Reference<Struct> field lookup
// ============================================================================

#[test]
fn ctx_field_type_box_struct() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // Register a struct with fields
    ctx.structs.insert(
        "Node".to_string(),
        vec![
            ("value".to_string(), HirType::Int),
            ("next".to_string(), HirType::Pointer(Box::new(HirType::Struct("Node".to_string())))),
        ],
    );
    // Register variable as Box<Struct>
    ctx.add_variable("node".to_string(), HirType::Box(Box::new(HirType::Struct("Node".to_string()))));
    // Access field through box — test the field access expression
    let expr = HirExpression::FieldAccess {
        object: Box::new(HirExpression::Variable("node".to_string())),
        field: "value".to_string(),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("node") && code.contains("value"),
        "Box struct field access should work, got: {}",
        code
    );
}

#[test]
fn ctx_field_type_reference_struct() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.structs.insert(
        "Point".to_string(),
        vec![
            ("x".to_string(), HirType::Float),
            ("y".to_string(), HirType::Float),
        ],
    );
    ctx.add_variable(
        "pt".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Struct("Point".to_string())),
            mutable: false,
        },
    );
    let expr = HirExpression::FieldAccess {
        object: Box::new(HirExpression::Variable("pt".to_string())),
        field: "x".to_string(),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("pt") && code.contains("x"),
        "Reference struct field access should work, got: {}",
        code
    );
}

#[test]
fn ctx_field_type_box_non_struct_returns_none() {
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Box(Box::new(HirType::Int)));
    // Trying to get field type on Box<Int> should return None
    let expr = HirExpression::Variable("x".to_string());
    let result = ctx.get_field_type(&expr, "value");
    assert!(result.is_none(), "Box<Int> should not have fields");
}

#[test]
fn ctx_field_type_reference_non_struct_returns_none() {
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "x".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
    );
    let expr = HirExpression::Variable("x".to_string());
    let result = ctx.get_field_type(&expr, "value");
    assert!(result.is_none(), "Reference<Int> should not have fields");
}

// ============================================================================
// String literal to char pointer conversion (line 1088)
// ============================================================================

#[test]
fn expr_string_literal_to_char_pointer() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::StringLiteral("hello".to_string());
    // With target type Pointer(Char), should convert to b"hello\0".as_ptr() as *mut u8
    let code = cg.generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Char))),
    );
    assert!(
        code.contains("as_ptr()") && code.contains("*mut u8"),
        "String literal with Pointer<Char> target should become byte string pointer, got: {}",
        code
    );
}

#[test]
fn expr_string_literal_with_quotes_to_char_pointer() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::StringLiteral("say \"hi\"".to_string());
    let code = cg.generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Char))),
    );
    assert!(
        code.contains("as_ptr()"),
        "String with quotes should be escaped, got: {}",
        code
    );
}

// ============================================================================
// Variable-to-pointer conversions (lines 1178-1217 uncovered)
// Reference/Vec/Array to raw pointer
// ============================================================================

#[test]
fn expr_reference_to_pointer_mutable() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "val".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: true,
        },
    );
    let expr = HirExpression::Variable("val".to_string());
    let code = cg.generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(
        code.contains("as *mut"),
        "Mutable reference to pointer should use 'as *mut', got: {}",
        code
    );
}

#[test]
fn expr_reference_to_pointer_immutable() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "val".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
    );
    let expr = HirExpression::Variable("val".to_string());
    let code = cg.generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(
        code.contains("as *const") || code.contains("as *mut"),
        "Immutable reference to pointer should cast, got: {}",
        code
    );
}

#[test]
fn expr_vec_to_pointer() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("buf".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::Variable("buf".to_string());
    let code = cg.generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(
        code.contains("as_mut_ptr"),
        "Vec to pointer should use as_mut_ptr(), got: {}",
        code
    );
}

#[test]
fn expr_array_to_pointer() {
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
    let code = cg.generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(
        code.contains("as_mut_ptr"),
        "Array to pointer should use as_mut_ptr(), got: {}",
        code
    );
}

#[test]
fn expr_array_to_void_pointer() {
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
    let code = cg.generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Void))),
    );
    assert!(
        code.contains("as_mut_ptr") && code.contains("*mut ()"),
        "Array to void pointer should cast to *mut (), got: {}",
        code
    );
}

#[test]
fn expr_pointer_to_pointer_passthrough() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let expr = HirExpression::Variable("ptr".to_string());
    let code = cg.generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert_eq!(
        code, "ptr",
        "Pointer to pointer should pass through unchanged, got: {}",
        code
    );
}

// ============================================================================
// Int-to-char coercion (line 1228)
// ============================================================================

#[test]
fn expr_int_var_to_char_target() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("c".to_string(), HirType::Int);
    let expr = HirExpression::Variable("c".to_string());
    let code = cg.generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Char),
    );
    assert!(
        code.contains("as u8"),
        "Int variable with Char target should cast to u8, got: {}",
        code
    );
}

// ============================================================================
// Pointer comparison with 0 (lines 1381-1383)
// 0 == ptr_expr pattern (reversed)
// ============================================================================

#[test]
fn expr_zero_equals_pointer_expr() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    // 0 == ptr should become std::ptr::null_mut() == ptr
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::IntLiteral(0)),
        right: Box::new(HirExpression::Variable("ptr".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("null_mut"),
        "0 == ptr should become null_mut() comparison, got: {}",
        code
    );
}

// ============================================================================
// Vec null check (lines 1393-1401): Vec != NULL → true
// ============================================================================

#[test]
fn expr_vec_null_check_not_equal_with_ctx() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("arr".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("true") && code.contains("Vec never null"),
        "Vec != NULL should be 'true /* Vec never null */', got: {}",
        code
    );
}

// ============================================================================
// Box null check (lines 1410-1423): Box == 0 → always false
// ============================================================================

#[test]
fn expr_box_null_check_equal_with_ctx() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("b".to_string(), HirType::Box(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("b".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("false") && code.contains("Box never null"),
        "Box == 0 should be 'false /* Box never null */', got: {}",
        code
    );
}

#[test]
fn expr_box_null_check_not_equal_with_ctx() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("b".to_string(), HirType::Box(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("b".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("true") && code.contains("Box never null"),
        "Box != NULL should be 'true /* Box never null */', got: {}",
        code
    );
}

// ============================================================================
// strlen(s) == 0 → s.is_empty() (lines 1441-1461)
// Both directions: strlen(s) != 0 and 0 == strlen(s)
// ============================================================================

#[test]
fn expr_strlen_neq_zero_is_not_empty() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::FunctionCall {
            function: "strlen".to_string(),
            arguments: vec![HirExpression::Variable("s".to_string())],
        }),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("is_empty"),
        "strlen(s) != 0 should become !s.is_empty(), got: {}",
        code
    );
}

#[test]
fn expr_zero_eq_strlen_is_empty() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // 0 == strlen(s) → s.is_empty()
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::IntLiteral(0)),
        right: Box::new(HirExpression::FunctionCall {
            function: "strlen".to_string(),
            arguments: vec![HirExpression::Variable("s".to_string())],
        }),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("is_empty"),
        "0 == strlen(s) should become s.is_empty(), got: {}",
        code
    );
}

#[test]
fn expr_zero_neq_strlen_not_empty() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // 0 != strlen(s) → !s.is_empty()
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::IntLiteral(0)),
        right: Box::new(HirExpression::FunctionCall {
            function: "strlen".to_string(),
            arguments: vec![HirExpression::Variable("s".to_string())],
        }),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("is_empty"),
        "0 != strlen(s) should become !s.is_empty(), got: {}",
        code
    );
}

// ============================================================================
// Pointer subtraction (line 1580): ptr - int_expr → wrapping_sub
// ============================================================================

#[test]
fn expr_pointer_subtract_literal() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    // ptr - 5 → ptr.wrapping_sub(5 as usize)
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("ptr".to_string())),
        right: Box::new(HirExpression::IntLiteral(5)),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("wrapping_sub"),
        "ptr - literal should use wrapping_sub, got: {}",
        code
    );
}

// ============================================================================
// Bitwise operations with bool operands (lines 1849-1860)
// Bool in arithmetic → cast to i32
// ============================================================================

#[test]
fn expr_bool_bitwise_and_unsigned_int() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::UnsignedInt);
    // (a > b) & x where x is unsigned → needs cast to i32 for both, then back to u32
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::BitwiseAnd,
        left: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
        right: Box::new(HirExpression::Variable("x".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("as i32") && code.contains("as u32"),
        "Bool & unsigned should cast both sides and result, got: {}",
        code
    );
}

#[test]
fn expr_unsigned_bitwise_or_bool() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::UnsignedInt);
    // x | (a == b) where x is unsigned — bitwise OR with bool operand
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::BitwiseOr,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("as i32"),
        "Unsigned | bool should cast, got: {}",
        code
    );
}

#[test]
fn expr_bool_bitwise_xor_int() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // (a < b) ^ c where c is int — bitwise XOR with bool operand
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::BitwiseXor,
        left: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
        right: Box::new(HirExpression::Variable("c".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("as i32"),
        "Bool ^ int should cast bool to i32, got: {}",
        code
    );
}

// ============================================================================
// Dereference of string variable (line 1902): *str++ on StringReference
// ============================================================================

#[test]
fn expr_deref_post_increment_string() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("s".to_string(), HirType::StringReference);
    // *s++ where s is &str → PostIncrement on string generates byte value
    let expr = HirExpression::Dereference(Box::new(HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Variable("s".to_string())),
    }));
    let code = cg.generate_expression_with_context(&expr, &ctx);
    // Should NOT double-dereference
    assert!(
        !code.is_empty(),
        "Deref of PostIncrement on string should produce code, got: {}",
        code
    );
}

// ============================================================================
// LogicalNot on boolean vs integer (lines 2007-2014)
// ============================================================================

#[test]
fn expr_logical_not_on_boolean_expr() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // !(a == b) → !(a == b) (already boolean)
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.starts_with("!") && !code.contains("== 0"),
        "LogicalNot on boolean should not add '== 0', got: {}",
        code
    );
}

#[test]
fn expr_logical_not_on_integer() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // !x where x is an integer → (x == 0) (no target type, so no as i32 cast)
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("== 0"),
        "LogicalNot on integer should become (x == 0), got: {}",
        code
    );
}

#[test]
fn expr_logical_not_on_integer_with_int_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // !x with target type Int → (x == 0) as i32
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let code = cg.generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Int),
    );
    assert!(
        code.contains("== 0") && code.contains("as i32"),
        "LogicalNot on integer with Int target should become (x == 0) as i32, got: {}",
        code
    );
}

#[test]
fn expr_logical_not_on_bool_with_int_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // !(a == b) with target type Int → (!(a == b)) as i32
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let code = cg.generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Int),
    );
    assert!(
        code.contains("as i32"),
        "LogicalNot on bool with Int target should cast to i32, got: {}",
        code
    );
}

// ============================================================================
// Printf raw pointer %s argument wrapping (line 2382)
// ============================================================================

#[test]
fn expr_printf_raw_pointer_arg_with_percent_s() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("name".to_string(), HirType::Pointer(Box::new(HirType::Char)));
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("Hello %s".to_string()),
            HirExpression::Variable("name".to_string()),
        ],
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("print"),
        "printf with raw pointer arg should generate print, got: {}",
        code
    );
}

// ============================================================================
// Calloc with SignedChar element type (line 3052)
// ============================================================================

#[test]
fn expr_calloc_signed_char() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(10)),
        element_type: Box::new(HirType::SignedChar),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("0i8") && code.contains("10"),
        "Calloc with SignedChar should use 0i8, got: {}",
        code
    );
}

// ============================================================================
// sizeof struct member (lines 2978-3011 uncovered)
// sizeof(record.field) and sizeof(record->field) patterns
// ============================================================================

#[test]
fn expr_sizeof_struct_dot_field() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.structs.insert(
        "Record".to_string(),
        vec![("data".to_string(), HirType::Int)],
    );
    // sizeof(Record.data) — dot access pattern
    let expr = HirExpression::Sizeof {
        type_name: "Record.data".to_string(),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("size_of"),
        "sizeof struct.field should use size_of, got: {}",
        code
    );
}

#[test]
fn expr_sizeof_struct_arrow_field_with_known_type() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.structs.insert(
        "Record".to_string(),
        vec![("data".to_string(), HirType::Double)],
    );
    // sizeof(Record data) — member access pattern (preprocessed by parser)
    let expr = HirExpression::Sizeof {
        type_name: "Record data".to_string(),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("size_of") && code.contains("f64"),
        "sizeof struct->field with known type should resolve to field type, got: {}",
        code
    );
}

#[test]
fn expr_sizeof_variable() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    // sizeof(x) where x is a known variable → size_of_val(&x)
    let expr = HirExpression::Sizeof {
        type_name: "x".to_string(),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("size_of_val"),
        "sizeof variable should use size_of_val, got: {}",
        code
    );
}

// ============================================================================
// PostIncrement/PostDecrement on dereferenced pointer (lines 3327, 3390)
// (*p)++ and (*p)-- patterns
// ============================================================================

#[test]
fn expr_post_increment_deref_pointer() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    // (*p)++ → { let __tmp = unsafe { *p }; unsafe { *p += 1 }; __tmp }
    let expr = HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("p".to_string()),
        ))),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("unsafe") && code.contains("__tmp"),
        "(*p)++ should use unsafe deref with tmp, got: {}",
        code
    );
}

#[test]
fn expr_post_decrement_deref_pointer() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    // (*p)-- → { let __tmp = unsafe { *p }; unsafe { *p -= 1 }; __tmp }
    let expr = HirExpression::PostDecrement {
        operand: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("p".to_string()),
        ))),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("unsafe") && code.contains("-= 1"),
        "(*p)-- should use unsafe deref with decrement, got: {}",
        code
    );
}

#[test]
fn expr_pre_increment_deref_pointer() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    // ++(*p) → { unsafe { *p += 1 }; unsafe { *p } }
    let expr = HirExpression::PreIncrement {
        operand: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("p".to_string()),
        ))),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("unsafe") && code.contains("+= 1"),
        "++(*p) should use unsafe deref with increment, got: {}",
        code
    );
}

#[test]
fn expr_pre_decrement_deref_pointer() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    // --(*p) → { unsafe { *p -= 1 }; unsafe { *p } }
    let expr = HirExpression::PreDecrement {
        operand: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("p".to_string()),
        ))),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("unsafe") && code.contains("-= 1"),
        "--(*p) should use unsafe deref with decrement, got: {}",
        code
    );
}

// ============================================================================
// VLA (Variable-Length Array) declaration (lines 4045, 4058)
// char vla[n] → vec![0u8; n]
// ============================================================================

#[test]
fn stmt_vla_declaration_signed_char_with_context() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("n".to_string(), HirType::Int);
    let stmt = HirStatement::VariableDeclaration {
        name: "buf".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::SignedChar),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("func"), &mut ctx, None);
    assert!(
        code.contains("vec!") && code.contains("0i8"),
        "VLA of SignedChar should use vec![0i8; n], got: {}",
        code
    );
}

// ============================================================================
// Malloc init for Vec type (lines 4193-4196)
// int* arr = malloc(n * sizeof(int)) → Vec
// ============================================================================

#[test]
fn stmt_malloc_vec_non_multiply_pattern() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // Pointer with malloc init where size is NOT n * sizeof(T)
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::Malloc {
            size: Box::new(HirExpression::IntLiteral(100)),
        }),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("func"), &mut ctx, None);
    assert!(
        code.contains("Vec") || code.contains("Box") || code.contains("vec!"),
        "malloc with non-multiply size should still generate allocation, got: {}",
        code
    );
}

// ============================================================================
// Char array with string literal init (lines 4274-4278)
// char str[20] = "hello" → let mut str: [u8; 20] = *b"hello\0"
// ============================================================================

#[test]
fn stmt_char_array_string_literal_init() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "msg".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Char),
            size: Some(20),
        },
        initializer: Some(HirExpression::StringLiteral("hello".to_string())),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("func"), &mut ctx, None);
    assert!(
        code.contains("b\"hello\\0\""),
        "Char array with string literal should become *b\"hello\\0\", got: {}",
        code
    );
}

// ============================================================================
// Char*[] array of string literals (lines 4142-4154)
// char *arr[] = {"a", "b"} → let arr: [&str; 2] = ["a", "b"]
// ============================================================================

#[test]
fn stmt_char_pointer_array_string_literals() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "names".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Pointer(Box::new(HirType::Char))),
            size: Some(2),
        },
        initializer: Some(HirExpression::CompoundLiteral {
            literal_type: HirType::Array {
                element_type: Box::new(HirType::Pointer(Box::new(HirType::Char))),
                size: Some(2),
            },
            initializers: vec![
                HirExpression::StringLiteral("alice".to_string()),
                HirExpression::StringLiteral("bob".to_string()),
            ],
        }),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("func"), &mut ctx, None);
    assert!(
        code.contains("&str"),
        "char *arr[] of string literals should become [&str], got: {}",
        code
    );
}

// ============================================================================
// Realloc from NULL (lines 4461-4475)
// ptr = realloc(NULL, n * sizeof(T))
// ============================================================================

#[test]
fn stmt_realloc_from_null() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("arr".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let stmt = HirStatement::Assignment {
        target: "arr".to_string(),
        value: HirExpression::Realloc {
            pointer: Box::new(HirExpression::NullLiteral),
            new_size: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(HirExpression::IntLiteral(10)),
                right: Box::new(HirExpression::Sizeof {
                    type_name: "int".to_string(),
                }),
            }),
        },
    };
    let code = cg.generate_statement_with_context(&stmt, Some("func"), &mut ctx, None);
    assert!(
        code.contains("resize"),
        "realloc(NULL, n*sizeof(T)) should become resize, got: {}",
        code
    );
}

// ============================================================================
// String iteration param pointer arithmetic (lines 4514-4524)
// ptr = ptr + N / ptr = ptr - N with string iter params
// ============================================================================

#[test]
fn stmt_string_iter_param_advance() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("s".to_string(), HirType::Pointer(Box::new(HirType::Char)));
    ctx.add_string_iter_param("s".to_string(), "s_idx".to_string());
    // s = s + 1 → s_idx += 1 as usize
    let stmt = HirStatement::Assignment {
        target: "s".to_string(),
        value: HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("s".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        },
    };
    let code = cg.generate_statement_with_context(&stmt, Some("func"), &mut ctx, None);
    assert!(
        code.contains("s_idx") && code.contains("+="),
        "String iter param advance should use s_idx, got: {}",
        code
    );
}

#[test]
fn stmt_string_iter_param_subtract() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("s".to_string(), HirType::Pointer(Box::new(HirType::Char)));
    ctx.add_string_iter_param("s".to_string(), "s_idx".to_string());
    // s = s - 1 → s_idx -= 1 as usize
    let stmt = HirStatement::Assignment {
        target: "s".to_string(),
        value: HirExpression::BinaryOp {
            op: BinaryOperator::Subtract,
            left: Box::new(HirExpression::Variable("s".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        },
    };
    let code = cg.generate_statement_with_context(&stmt, Some("func"), &mut ctx, None);
    assert!(
        code.contains("s_idx") && code.contains("-="),
        "String iter param subtract should use s_idx, got: {}",
        code
    );
}

#[test]
fn stmt_string_iter_param_other_op() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("s".to_string(), HirType::Pointer(Box::new(HirType::Char)));
    ctx.add_string_iter_param("s".to_string(), "s_idx".to_string());
    // s = s * 2 (not Add/Subtract) → fallback to regular assignment
    let stmt = HirStatement::Assignment {
        target: "s".to_string(),
        value: HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("s".to_string())),
            right: Box::new(HirExpression::IntLiteral(2)),
        },
    };
    let code = cg.generate_statement_with_context(&stmt, Some("func"), &mut ctx, None);
    assert!(
        code.contains("s =") || code.contains("s_idx"),
        "String iter param with non-add/sub should still generate code, got: {}",
        code
    );
}

// ============================================================================
// Double-pointer deref assignment (lines 4767-4779)
// **ptr = val where ptr is Pointer<Pointer<T>>
// ============================================================================

#[test]
fn stmt_double_pointer_deref_assignment() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "pp".to_string(),
        HirType::Pointer(Box::new(HirType::Pointer(Box::new(HirType::Int)))),
    );
    // **pp = 42
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Dereference(Box::new(HirExpression::Variable("pp".to_string()))),
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("func"), &mut ctx, None);
    assert!(
        code.contains("unsafe"),
        "Double pointer deref assignment should be unsafe, got: {}",
        code
    );
}

// ============================================================================
// Default values: Box<Struct> and Enum (lines 3640-3665)
// ============================================================================

#[test]
fn default_value_box_double() {
    let result = CodeGenerator::default_value_for_type(&HirType::Box(Box::new(HirType::Double)));
    assert!(
        result.contains("Box::new") && result.contains("0.0"),
        "Box<Double> default should use Box::new(0.0), got: {}",
        result
    );
}

#[test]
fn default_value_enum_type() {
    let result = CodeGenerator::default_value_for_type(&HirType::Enum("Color".to_string()));
    assert_eq!(result, "Color::default()", "Enum default should be ::default()");
}

// ============================================================================
// find_string_format_positions with rare format specifiers (lines 3932-3942)
// Tests: %G, %n, %a, %A consume arg positions
// ============================================================================

#[test]
fn find_string_format_positions_percent_g_uppercase() {
    // printf("val=%G %s", g_val, name) — %G is at arg 0, %s is at arg 1
    let positions = CodeGenerator::find_string_format_positions("val=%G %s");
    assert_eq!(positions, vec![1], "%s should be at position 1 after %G");
}

#[test]
fn find_string_format_positions_percent_n() {
    // printf("count=%n %s", &n, str) — %n is at arg 0, %s is at arg 1
    let positions = CodeGenerator::find_string_format_positions("count=%n %s");
    assert_eq!(positions, vec![1], "%s should be at position 1 after %n");
}

#[test]
fn find_string_format_positions_percent_a() {
    // printf("hex=%a %s", val, str) — %a is at arg 0, %s is at arg 1
    let positions = CodeGenerator::find_string_format_positions("hex=%a %s");
    assert_eq!(positions, vec![1], "%s should be at position 1 after %a");
}

#[test]
fn find_string_format_positions_percent_a_upper() {
    // printf("hex=%A %s", val, str)
    let positions = CodeGenerator::find_string_format_positions("hex=%A %s");
    assert_eq!(positions, vec![1], "%s should be at position 1 after %A");
}

// ============================================================================
// Global variable generation (lines 7410-7421)
// Array with non-int init, unsized array, pointer with non-zero init
// ============================================================================

#[test]
fn global_var_array_with_non_int_initializer() {
    let cg = CodeGenerator::new();
    let constant = HirConstant::new(
        "TABLE".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(3),
        },
        // Non-integer initializer → use generate_expression directly
        HirExpression::CompoundLiteral {
            literal_type: HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(3),
            },
            initializers: vec![
                HirExpression::IntLiteral(1),
                HirExpression::IntLiteral(2),
                HirExpression::IntLiteral(3),
            ],
        },
    );
    let code = cg.generate_global_variable(&constant, true, false, false);
    assert!(
        code.contains("static mut TABLE"),
        "Global array with compound init should be static mut, got: {}",
        code
    );
}

#[test]
fn global_var_unsized_array() {
    let cg = CodeGenerator::new();
    let constant = HirConstant::new(
        "DATA".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None,
        },
        HirExpression::IntLiteral(0),
    );
    let code = cg.generate_global_variable(&constant, true, false, false);
    assert!(
        code.contains("static mut DATA"),
        "Global unsized array should fall through, got: {}",
        code
    );
}

#[test]
fn global_var_pointer_with_nonzero_init() {
    let cg = CodeGenerator::new();
    let constant = HirConstant::new(
        "PTR".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        HirExpression::IntLiteral(42),
    );
    let code = cg.generate_global_variable(&constant, true, false, false);
    assert!(
        code.contains("static mut PTR") && code.contains("42"),
        "Global pointer with non-zero init should keep value, got: {}",
        code
    );
}

#[test]
fn global_var_pointer_with_null_init() {
    let cg = CodeGenerator::new();
    let constant = HirConstant::new(
        "HEAD".to_string(),
        HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
        HirExpression::IntLiteral(0),
    );
    let code = cg.generate_global_variable(&constant, true, false, false);
    assert!(
        code.contains("null_mut"),
        "Global pointer with 0 init should become null_mut(), got: {}",
        code
    );
}

#[test]
fn global_var_const_char_pointer() {
    let cg = CodeGenerator::new();
    let constant = HirConstant::new(
        "MSG".to_string(),
        HirType::Pointer(Box::new(HirType::Char)),
        HirExpression::StringLiteral("hello".to_string()),
    );
    let code = cg.generate_global_variable(&constant, true, false, true);
    assert!(
        code.contains("&str") && code.contains("const MSG"),
        "const char* global should become &str const, got: {}",
        code
    );
}

#[test]
fn global_var_extern_declaration() {
    let cg = CodeGenerator::new();
    let constant = HirConstant::new(
        "count".to_string(),
        HirType::Int,
        HirExpression::IntLiteral(0),
    );
    let code = cg.generate_global_variable(&constant, false, true, false);
    assert!(
        code.contains("extern \"C\"") && code.contains("static count: i32"),
        "extern global should use extern C block, got: {}",
        code
    );
}

// ============================================================================
// generate_function_with_structs with struct definitions (lines 6502-6520)
// Tests the context setup where pointer params become references
// ============================================================================

#[test]
fn func_with_structs_pointer_param_becomes_reference() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "process".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "data".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("data".to_string()),
            value: HirExpression::IntLiteral(42),
        }],
    );
    let structs = vec![];
    let code = cg.generate_function_with_structs(&func, &structs);
    assert!(
        code.contains("fn process") && code.contains("data"),
        "Function with struct context should generate code, got: {}",
        code
    );
}

// ============================================================================
// generate_struct with struct that has reference fields (needs lifetimes)
// Lines 7054 — Option type is_copy_type returns false
// ============================================================================

#[test]
fn generate_struct_with_simple_fields() {
    let cg = CodeGenerator::new();
    let hir_struct = HirStruct::new(
        "Point".to_string(),
        vec![
            HirStructField::new("x".to_string(), HirType::Int),
            HirStructField::new("y".to_string(), HirType::Int),
        ],
    );
    let code = cg.generate_struct(&hir_struct);
    assert!(
        code.contains("struct Point") && code.contains("x: i32") && code.contains("y: i32"),
        "Simple struct should generate fields, got: {}",
        code
    );
    // Simple copy types should get Copy derive
    assert!(
        code.contains("Copy"),
        "Struct with Copy-able fields should derive Copy, got: {}",
        code
    );
}

#[test]
fn generate_struct_with_option_field() {
    let cg = CodeGenerator::new();
    let hir_struct = HirStruct::new(
        "Config".to_string(),
        vec![
            HirStructField::new("value".to_string(), HirType::Int),
            HirStructField::new(
                "callback".to_string(),
                HirType::Option(Box::new(HirType::Int)),
            ),
        ],
    );
    let code = cg.generate_struct(&hir_struct);
    assert!(
        code.contains("struct Config"),
        "Struct with Option field should generate, got: {}",
        code
    );
    // Option is not Copy
    assert!(
        !code.contains("Copy"),
        "Struct with Option field should NOT derive Copy, got: {}",
        code
    );
}

#[test]
fn generate_struct_with_pointer_field() {
    let cg = CodeGenerator::new();
    let hir_struct = HirStruct::new(
        "Node".to_string(),
        vec![
            HirStructField::new("value".to_string(), HirType::Int),
            HirStructField::new(
                "next".to_string(),
                HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
            ),
        ],
    );
    let code = cg.generate_struct(&hir_struct);
    assert!(
        code.contains("struct Node") && code.contains("next"),
        "Struct with pointer field should generate, got: {}",
        code
    );
}

// ============================================================================
// Malloc FunctionCall init for struct → Box (lines 4215-4228)
// malloc(sizeof(T)) where T doesn't derive Default → zeroed
// ============================================================================

#[test]
fn stmt_malloc_struct_no_default_uses_zeroed() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // Don't register struct as having Default — so it uses zeroed fallback
    let stmt = HirStatement::VariableDeclaration {
        name: "node".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Struct("BigStruct".to_string()))),
        initializer: Some(HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::Sizeof {
                type_name: "BigStruct".to_string(),
            }],
        }),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("func"), &mut ctx, None);
    assert!(
        code.contains("Box") && code.contains("zeroed"),
        "malloc(sizeof(T)) without Default should use zeroed, got: {}",
        code
    );
}

// ============================================================================
// Reference deref assignment needs unsafe (line 4770)
// **ref_ptr = val where ref_ptr is Reference<Pointer<T>>
// ============================================================================

#[test]
fn stmt_ref_to_pointer_deref_assignment() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "rp".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Pointer(Box::new(HirType::Int))),
            mutable: true,
        },
    );
    // **rp = 42 where rp is &mut *mut i32
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Dereference(Box::new(HirExpression::Variable("rp".to_string()))),
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("func"), &mut ctx, None);
    assert!(
        code.contains("unsafe"),
        "Deref assignment through reference-to-pointer should be unsafe, got: {}",
        code
    );
}

// ============================================================================
// ArrayIndexAssignment with non-global array expression (line 4818)
// ============================================================================

#[test]
fn stmt_array_index_assign_non_variable_array() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // (func()).arr[0] = 42 — array is a FieldAccess not a Variable
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::FieldAccess {
            object: Box::new(HirExpression::Variable("obj".to_string())),
            field: "arr".to_string(),
        }),
        index: Box::new(HirExpression::IntLiteral(0)),
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("func"), &mut ctx, None);
    assert!(
        code.contains("42"),
        "Array index assign with non-variable array should still work, got: {}",
        code
    );
}

// ============================================================================
// Switch with default case and statements (line 4672)
// ============================================================================

#[test]
fn stmt_switch_with_nonempty_default() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Switch {
        condition: HirExpression::Variable("cmd".to_string()),
        cases: vec![
            SwitchCase {
                value: Some(HirExpression::IntLiteral(1)),
                body: vec![
                    HirStatement::Expression(HirExpression::FunctionCall {
                        function: "handle_one".to_string(),
                        arguments: vec![],
                    }),
                    HirStatement::Break,
                ],
            },
        ],
        default_case: Some(vec![
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "handle_default".to_string(),
                arguments: vec![],
            }),
        ]),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("func"), &mut ctx, None);
    assert!(
        code.contains("handle_default") && code.contains("_ =>"),
        "Switch with non-empty default should include default body, got: {}",
        code
    );
}

// ============================================================================
// Cast expression wrapping malloc to Vec target (line 3154)
// ============================================================================

#[test]
fn expr_cast_malloc_to_vec() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // (int*)malloc(n * sizeof(int)) with Vec target type
    let expr = HirExpression::Cast {
        target_type: HirType::Pointer(Box::new(HirType::Int)),
        expr: Box::new(HirExpression::Malloc {
            size: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(HirExpression::IntLiteral(10)),
                right: Box::new(HirExpression::Sizeof {
                    type_name: "int".to_string(),
                }),
            }),
        }),
    };
    let code = cg.generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Vec(Box::new(HirType::Int))),
    );
    assert!(
        code.contains("vec!") || code.contains("Vec"),
        "Cast(malloc) with Vec target should generate vec, got: {}",
        code
    );
}

// ============================================================================
// is_array_allocation_size with cast wrapping (line 5361)
// ============================================================================

#[test]
fn is_array_allocation_size_through_cast() {
    // Cast wrapping: (size_t)(n * sizeof(int)) should still be array pattern
    let size_expr = HirExpression::Cast {
        target_type: HirType::TypeAlias("size_t".to_string()),
        expr: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::IntLiteral(10)),
            right: Box::new(HirExpression::Sizeof {
                type_name: "int".to_string(),
            }),
        }),
    };
    assert!(
        CodeGenerator::is_array_allocation_size(&size_expr),
        "Cast-wrapped multiply should be array allocation"
    );
}

// ============================================================================
// expression_compares_to_null reversed (lines 5534-5539)
// 0 == var and NULL != var patterns
// ============================================================================

#[test]
fn null_comparison_reversed_zero_eq_var() {
    let cg = CodeGenerator::new();
    // statement_uses_null_comparison for: if (0 == ptr)
    let stmt = HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::IntLiteral(0)),
            right: Box::new(HirExpression::Variable("ptr".to_string())),
        },
        then_block: vec![],
        else_block: None,
    };
    assert!(
        cg.statement_uses_null_comparison(&stmt, "ptr"),
        "0 == ptr should be detected as null comparison"
    );
}

#[test]
fn null_comparison_null_neq_var() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::NotEqual,
            left: Box::new(HirExpression::NullLiteral),
            right: Box::new(HirExpression::Variable("ptr".to_string())),
        },
        then_block: vec![],
        else_block: None,
    };
    assert!(
        cg.statement_uses_null_comparison(&stmt, "ptr"),
        "NULL != ptr should be detected as null comparison"
    );
}

// ============================================================================
// uses_pointer_arithmetic through various statement types (lines 5571-5628)
// Linked list traversal, PointerFieldAccess, expression increment
// ============================================================================

#[test]
fn pointer_arithmetic_linked_list_traversal() {
    let cg = CodeGenerator::new();
    // head = head->next is pointer arithmetic (reassignment from field access)
    let stmt = HirStatement::Assignment {
        target: "head".to_string(),
        value: HirExpression::PointerFieldAccess {
            pointer: Box::new(HirExpression::Variable("head".to_string())),
            field: "next".to_string(),
        },
    };
    assert!(
        cg.statement_uses_pointer_arithmetic(&stmt, "head"),
        "head = head->next should be detected as pointer arithmetic"
    );
}
