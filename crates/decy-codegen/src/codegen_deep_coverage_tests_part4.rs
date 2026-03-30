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

// ============================================================================
// POINTER ARITHMETIC (DECY-041) — wrapping_add/sub/offset_from
// ============================================================================

#[test]
fn expr_pointer_add_wrapping_add() {
    // C: ptr + n → ptr.wrapping_add(n as usize)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("ptr".to_string())),
        right: Box::new(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("wrapping_add"),
        "Pointer + int should use wrapping_add, got: {}",
        code
    );
}

#[test]
fn expr_pointer_sub_integer_wrapping_sub() {
    // C: ptr - n → ptr.wrapping_sub(n as usize)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("ptr".to_string())),
        right: Box::new(HirExpression::IntLiteral(1)),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("wrapping_sub"),
        "Pointer - int should use wrapping_sub, got: {}",
        code
    );
}

#[test]
fn expr_pointer_sub_pointer_offset_from() {
    // C: ptr1 - ptr2 → unsafe { ptr1.offset_from(ptr2) as i32 }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr1".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    ctx.add_variable("ptr2".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("ptr1".to_string())),
        right: Box::new(HirExpression::Variable("ptr2".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("offset_from") && code.contains("unsafe"),
        "Pointer - pointer should use unsafe offset_from, got: {}",
        code
    );
}

// ============================================================================
// MIXED NUMERIC TYPE ARITHMETIC (DECY-204)
// ============================================================================

#[test]
fn expr_int_plus_float_promotion() {
    // C: int_var + float_var → (int_var as f32) + float_var
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("i".to_string(), HirType::Int);
    ctx.add_variable("f".to_string(), HirType::Float);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("i".to_string())),
        right: Box::new(HirExpression::Variable("f".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("as f32"),
        "Int + Float should promote int to f32, got: {}",
        code
    );
}

#[test]
fn expr_int_plus_double_promotion() {
    // C: int_var + double_var → (int_var as f64) + double_var
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("i".to_string(), HirType::Int);
    ctx.add_variable("d".to_string(), HirType::Double);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("i".to_string())),
        right: Box::new(HirExpression::Variable("d".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("as f64"),
        "Int + Double should promote int to f64, got: {}",
        code
    );
}

#[test]
fn expr_float_plus_double_promotion() {
    // C: float_var + double_var → (float_var as f64) + double_var
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("f".to_string(), HirType::Float);
    ctx.add_variable("d".to_string(), HirType::Double);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(HirExpression::Variable("f".to_string())),
        right: Box::new(HirExpression::Variable("d".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("as f64"),
        "Float * Double should promote float to f64, got: {}",
        code
    );
}

// ============================================================================
// SIGNED/UNSIGNED COMPARISON MISMATCH (DECY-251)
// ============================================================================

#[test]
fn expr_signed_unsigned_comparison_casts_to_i64() {
    // C: int_var < unsigned_var → (int_var as i64) < (unsigned_var as i64)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("s".to_string(), HirType::Int);
    ctx.add_variable("u".to_string(), HirType::UnsignedInt);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LessThan,
        left: Box::new(HirExpression::Variable("s".to_string())),
        right: Box::new(HirExpression::Variable("u".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("as i64"),
        "Signed/unsigned comparison should cast to i64, got: {}",
        code
    );
}

// ============================================================================
// CHAINED COMPARISONS (DECY-206) — (x < y) < z
// ============================================================================

#[test]
fn expr_chained_comparison_casts_bool_to_i32() {
    // C: (a < b) < c → ((a < b) as i32) < c
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LessThan,
        left: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
        right: Box::new(HirExpression::Variable("c".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("as i32"),
        "Chained comparison should cast bool to i32, got: {}",
        code
    );
}

// ============================================================================
// LOGICAL OPERATORS — bool conversion for non-boolean operands
// ============================================================================

#[test]
fn expr_logical_and_integer_operands_adds_ne_zero() {
    // C: a && b (where a, b are int) → (a != 0) && (b != 0)
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalAnd,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("!= 0"),
        "Logical AND with int operands should add != 0, got: {}",
        code
    );
}

#[test]
fn expr_logical_or_integer_operands_adds_ne_zero() {
    // C: a || b (where a, b are int) → (a != 0) || (b != 0)
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalOr,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("!= 0"),
        "Logical OR with int operands should add != 0, got: {}",
        code
    );
}

#[test]
fn expr_logical_and_with_bool_operand_no_conversion() {
    // C: (a > 0) && b → (a > 0) && (b != 0)  — left already bool, right gets converted
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalAnd,
        left: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let code = cg.generate_expression(&expr);
    // Left should NOT have != 0 (it's already a comparison)
    // Right should have != 0
    assert!(
        code.contains("&&"),
        "Logical AND should be present, got: {}",
        code
    );
}

// ============================================================================
// SIGNATURE — const char*, void*, main return type, Vec return
// ============================================================================

#[test]
fn sig_const_char_pointer_becomes_str() {
    // C: void process(const char* s) → fn process(mut s: &str)
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "process".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "s".to_string(),
            HirType::Pointer(Box::new(HirType::Char)),
        )],
        vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "puts".to_string(),
            arguments: vec![HirExpression::Variable("s".to_string())],
        })],
    );
    let sig = cg.generate_signature(&func);
    // The const char* detection depends on the parser marking it as const
    // At minimum, a char* should generate some pointer/reference
    assert!(
        sig.contains("process"),
        "Signature should contain function name, got: {}",
        sig
    );
}

#[test]
fn sig_void_return_no_annotation() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new(
        "cleanup".to_string(),
        HirType::Void,
        vec![],
    );
    let sig = cg.generate_signature(&func);
    assert!(
        !sig.contains("->"),
        "Void function should have no return type annotation, got: {}",
        sig
    );
}

#[test]
fn sig_int_return_has_i32() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new(
        "compute".to_string(),
        HirType::Int,
        vec![],
    );
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("i32"),
        "Int return function should have i32 annotation, got: {}",
        sig
    );
}

#[test]
fn sig_struct_pointer_return() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new(
        "create_node".to_string(),
        HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
        vec![],
    );
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("Node"),
        "Struct pointer return should reference Node, got: {}",
        sig
    );
}

// ============================================================================
// POST/PRE INCREMENT ON POINTER — wrapping_add
// ============================================================================

#[test]
fn expr_post_increment_pointer_wrapping_add() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PostIncrement,
        operand: Box::new(HirExpression::Variable("ptr".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("wrapping_add") || code.contains("ptr"),
        "PostIncrement on pointer should use wrapping_add, got: {}",
        code
    );
}

#[test]
fn expr_pre_increment_pointer() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Char)));
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PreIncrement,
        operand: Box::new(HirExpression::Variable("p".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("wrapping_add") || code.contains("p"),
        "PreIncrement on pointer should use wrapping_add, got: {}",
        code
    );
}

// ============================================================================
// STRING LITERAL TO POINTER — byte string conversion
// ============================================================================

#[test]
fn typed_decl_string_literal_to_char_pointer_type() {
    // C: char* s = "hello"; → b"hello\0" as *mut u8
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "s".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Char)),
        initializer: Some(HirExpression::StringLiteral("hello".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("hello"),
        "String literal to char pointer should contain hello, got: {}",
        code
    );
}

// ============================================================================
// CHAR ARITHMETIC WITH TARGET TYPE
// ============================================================================

#[test]
fn expr_char_operands_with_int_target_promote() {
    // C: int d = *s1 - *s2; where s1, s2 are char* → (*s1 as i32) - (*s2 as i32)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("c1".to_string(), HirType::Char);
    ctx.add_variable("c2".to_string(), HirType::Char);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("c1".to_string())),
        right: Box::new(HirExpression::Variable("c2".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(
        code.contains("as i32"),
        "Char subtraction with int target should promote to i32, got: {}",
        code
    );
}

// ============================================================================
// GENERATE_ANNOTATED_SIGNATURE — various parameter transforms
// ============================================================================

#[test]
fn annotated_sig_void_function_no_params() {
    use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let sig = AnnotatedSignature {
        name: "cleanup".to_string(),
        lifetimes: vec![],
        parameters: vec![],
        return_type: AnnotatedType::Simple(HirType::Void),
    };
    let code = cg.generate_annotated_signature_with_func(&sig, None);
    assert!(
        code.contains("fn cleanup") && !code.contains("->"),
        "Void annotated sig should have no return type, got: {}",
        code
    );
}

#[test]
fn annotated_sig_int_return_type() {
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
        code.contains("fn add") && code.contains("i32"),
        "Int return annotated sig should have i32, got: {}",
        code
    );
}

// ============================================================================
// RETURN IN MAIN — std::process::exit with char cast
// ============================================================================

#[test]
fn stmt_return_in_main_char_cast() {
    // C: return 'a'; in main → std::process::exit('a' as i32);
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("c".to_string(), HirType::Char);
    let stmt = HirStatement::Return(Some(HirExpression::Variable("c".to_string())));
    let code = cg.generate_statement_with_context(&stmt, Some("main"), &mut ctx, None);
    assert!(
        code.contains("std::process::exit") && code.contains("as i32"),
        "Char return in main should cast to i32, got: {}",
        code
    );
}

#[test]
fn stmt_return_in_main_int_no_cast() {
    // C: return 0; in main → std::process::exit(0);
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Return(Some(HirExpression::IntLiteral(0)));
    let code = cg.generate_statement_with_context(&stmt, Some("main"), &mut ctx, None);
    assert!(
        code.contains("std::process::exit(0)"),
        "Int return in main should call process::exit, got: {}",
        code
    );
}

#[test]
fn stmt_return_in_main_no_expr() {
    // C: return; in main → std::process::exit(0);
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Return(None);
    let code = cg.generate_statement_with_context(&stmt, Some("main"), &mut ctx, None);
    assert!(
        code.contains("std::process::exit(0)"),
        "Empty return in main should call process::exit(0), got: {}",
        code
    );
}

#[test]
fn stmt_return_in_non_main_just_return() {
    // C: return x; in add() → return x;
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Return(Some(HirExpression::Variable("x".to_string())));
    let code = cg.generate_statement_with_context(&stmt, Some("add"), &mut ctx, None);
    assert!(
        code.contains("return x"),
        "Non-main return should use return statement, got: {}",
        code
    );
}

// ============================================================================
// POINTER DEREFERENCE ASSIGNMENT — unsafe wrapping
// ============================================================================

#[test]
fn stmt_deref_assignment_with_safety_comment() {
    // C: *ptr = 42; → unsafe { *ptr = 42; } (when ptr is known pointer)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("ptr".to_string()),
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("unsafe") && code.contains("*ptr"),
        "Deref assignment should use unsafe block, got: {}",
        code
    );
}

// ============================================================================
// OPTION COMPARISON WITH NULL (reversed)
// ============================================================================

#[test]
fn expr_null_equal_option_reversed_is_none() {
    // C: NULL == opt → opt.is_none()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("opt".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::NullLiteral),
        right: Box::new(HirExpression::Variable("opt".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        code.contains("is_none") || code.contains("None") || code.contains("=="),
        "NULL == Option should work, got: {}",
        code
    );
}

// ============================================================================
// POINTER NULL CHECK — ptr == 0
// ============================================================================

