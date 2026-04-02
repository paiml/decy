// --- Sizeof: known variable → size_of_val ---
#[test]
fn expr_target_sizeof_variable() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::Sizeof { type_name: "x".to_string() };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("size_of_val(&x)"), "Got: {}", result);
}

// --- NullLiteral → None ---
#[test]
fn expr_target_null_literal() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::NullLiteral;
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "None");
}

// --- Calloc HIR node → vec![default; count] ---
#[test]
fn expr_target_calloc_hir_node_int() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(10)),
        element_type: Box::new(HirType::Int),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("vec![0i32; 10]"), "Got: {}", result);
}

// --- Calloc HIR node unsigned int ---
#[test]
fn expr_target_calloc_hir_node_unsigned_int() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(5)),
        element_type: Box::new(HirType::UnsignedInt),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("vec![0u32; 5]"), "Got: {}", result);
}

// --- Malloc HIR node with multiply → Vec::with_capacity ---
#[test]
fn expr_target_malloc_hir_node_multiply() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Malloc {
        size: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("n".to_string())),
            right: Box::new(HirExpression::Sizeof { type_name: "int".to_string() }),
        }),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("Vec::with_capacity("), "Got: {}", result);
}

// --- Malloc HIR node without multiply → Box::new ---
#[test]
fn expr_target_malloc_hir_node_single() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Malloc {
        size: Box::new(HirExpression::IntLiteral(4)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("Box::new(0i32)"), "Got: {}", result);
}

// ============================================================================
// BATCH 19: PostIncrement/PreIncrement/PreDecrement/PostDecrement HIR variants,
// Realloc HIR, StringMethodCall, Cast, CompoundLiteral, Ternary, IsNotNull
// ============================================================================

// --- PostIncrement: string iteration → as_bytes()[0] + slice advance ---
#[test]
fn expr_target_post_increment_string_iter() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("key".to_string(), HirType::StringReference);
    let expr = HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Variable("key".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("as_bytes()[0]"), "Got: {}", result);
    assert!(result.contains("&key[1..]"), "Got: {}", result);
}

// --- PostIncrement: dereference pointer (*p)++ → unsafe ---
#[test]
fn expr_target_post_increment_deref_pointer() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Dereference(
            Box::new(HirExpression::Variable("p".to_string())),
        )),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains("*p += 1"), "Got: {}", result);
}

// --- PostIncrement: pointer type → wrapping_add ---
#[test]
fn expr_target_post_increment_hir_pointer() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Variable("p".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("wrapping_add(1)"), "Got: {}", result);
}

// --- PostIncrement: non-pointer → += 1 ---
#[test]
fn expr_target_post_increment_hir_int() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("i".to_string(), HirType::Int);
    let expr = HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Variable("i".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("+= 1"), "Got: {}", result);
    assert!(result.contains("__tmp"), "Got: {}", result);
}

// --- PreIncrement: dereference pointer ++(*p) → unsafe ---
#[test]
fn expr_target_pre_increment_deref_pointer() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PreIncrement {
        operand: Box::new(HirExpression::Dereference(
            Box::new(HirExpression::Variable("p".to_string())),
        )),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains("*p += 1"), "Got: {}", result);
}

// --- PreIncrement: pointer type → wrapping_add ---
#[test]
fn expr_target_pre_increment_hir_pointer() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PreIncrement {
        operand: Box::new(HirExpression::Variable("p".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("wrapping_add(1)"), "Got: {}", result);
}

// --- PreIncrement: non-pointer → += 1 ---
#[test]
fn expr_target_pre_increment_hir_int() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("i".to_string(), HirType::Int);
    let expr = HirExpression::PreIncrement {
        operand: Box::new(HirExpression::Variable("i".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("+= 1"), "Got: {}", result);
    assert!(!result.contains("__tmp"), "Got: {}", result);
}

// --- PostDecrement: dereference pointer (*p)-- → unsafe ---
#[test]
fn expr_target_post_decrement_deref_pointer() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PostDecrement {
        operand: Box::new(HirExpression::Dereference(
            Box::new(HirExpression::Variable("p".to_string())),
        )),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains("*p -= 1"), "Got: {}", result);
}

// --- PostDecrement: pointer → wrapping_sub ---
#[test]
fn expr_target_post_decrement_hir_pointer() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PostDecrement {
        operand: Box::new(HirExpression::Variable("p".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("wrapping_sub(1)"), "Got: {}", result);
}

// --- PostDecrement: non-pointer → -= 1 ---
#[test]
fn expr_target_post_decrement_hir_int() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("i".to_string(), HirType::Int);
    let expr = HirExpression::PostDecrement {
        operand: Box::new(HirExpression::Variable("i".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("-= 1"), "Got: {}", result);
}

// --- PreDecrement: dereference pointer --(*p) → unsafe ---
#[test]
fn expr_target_pre_decrement_deref_pointer() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PreDecrement {
        operand: Box::new(HirExpression::Dereference(
            Box::new(HirExpression::Variable("p".to_string())),
        )),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains("*p -= 1"), "Got: {}", result);
}

// --- PreDecrement: pointer → wrapping_sub ---
#[test]
fn expr_target_pre_decrement_hir_pointer() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PreDecrement {
        operand: Box::new(HirExpression::Variable("p".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("wrapping_sub(1)"), "Got: {}", result);
}

// --- PreDecrement: non-pointer → -= 1 ---
#[test]
fn expr_target_pre_decrement_hir_int() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("i".to_string(), HirType::Int);
    let expr = HirExpression::PreDecrement {
        operand: Box::new(HirExpression::Variable("i".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("-= 1"), "Got: {}", result);
}

// --- Realloc HIR: NULL pointer + multiply → vec ---
#[test]
fn expr_target_realloc_hir_null_multiply() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Realloc {
        pointer: Box::new(HirExpression::NullLiteral),
        new_size: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("n".to_string())),
            right: Box::new(HirExpression::Sizeof { type_name: "int".to_string() }),
        }),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("vec![0i32;"), "Got: {}", result);
}

// --- Realloc HIR: NULL pointer no multiply → Vec::new ---
#[test]
fn expr_target_realloc_hir_null_no_multiply() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Realloc {
        pointer: Box::new(HirExpression::NullLiteral),
        new_size: Box::new(HirExpression::IntLiteral(100)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("Vec::new()"), "Got: {}", result);
}

// --- Realloc HIR: non-NULL pointer → passthrough ---
#[test]
fn expr_target_realloc_hir_non_null() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("buf".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::Realloc {
        pointer: Box::new(HirExpression::Variable("buf".to_string())),
        new_size: Box::new(HirExpression::IntLiteral(200)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("buf"), "Got: {}", result);
}

// --- StringMethodCall: len → .len() as i32 ---
#[test]
fn expr_target_string_method_call_len() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("s".to_string())),
        method: "len".to_string(),
        arguments: vec![],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains(".len() as i32"), "Got: {}", result);
}

// --- StringMethodCall: other no-arg method ---
#[test]
fn expr_target_string_method_call_is_empty() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("s".to_string())),
        method: "is_empty".to_string(),
        arguments: vec![],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("s.is_empty()"), "Got: {}", result);
}

// --- StringMethodCall: clone_into → &mut prefix ---
#[test]
fn expr_target_string_method_call_clone_into() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("src".to_string())),
        method: "clone_into".to_string(),
        arguments: vec![HirExpression::Variable("dest".to_string())],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("clone_into(&mut dest)"), "Got: {}", result);
}

// --- StringMethodCall: method with args ---
#[test]
fn expr_target_string_method_call_with_args() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("s".to_string())),
        method: "push_str".to_string(),
        arguments: vec![HirExpression::StringLiteral("hello".to_string())],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("push_str("), "Got: {}", result);
}

// --- Cast: Vec target + malloc inner → unwrap cast, generate vec ---
#[test]
fn expr_target_cast_vec_target_malloc() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Cast {
        target_type: HirType::Pointer(Box::new(HirType::Int)),
        expr: Box::new(HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(HirExpression::Variable("n".to_string())),
                right: Box::new(HirExpression::Sizeof { type_name: "int".to_string() }),
            }],
        }),
    };
    let result = cg.generate_expression_with_target_type(
        &expr, &ctx, Some(&HirType::Vec(Box::new(HirType::Int))),
    );
    assert!(result.contains("vec![0i32;"), "Got: {}", result);
}

// --- Cast: address-of + integer target → pointer as isize ---
#[test]
fn expr_target_cast_address_of_to_int() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Cast {
        target_type: HirType::Int,
        expr: Box::new(HirExpression::AddressOf(
            Box::new(HirExpression::Variable("x".to_string())),
        )),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("as *const _"), "Got: {}", result);
    assert!(result.contains("as isize"), "Got: {}", result);
}

// --- Cast: regular type → expr as type ---
#[test]
fn expr_target_cast_regular() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Cast {
        target_type: HirType::Float,
        expr: Box::new(HirExpression::Variable("x".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("x as f32"), "Got: {}", result);
}

// --- Cast: binary op wrapped in parens ---
#[test]
fn expr_target_cast_binary_op_parens() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Cast {
        target_type: HirType::Double,
        expr: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("(a + b) as f64"), "Got: {}", result);
}

// --- CompoundLiteral: struct with fields ---
#[test]
fn expr_target_compound_literal_struct() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let s = HirStruct::new("Point".to_string(), vec![
        HirStructField::new("x".to_string(), HirType::Int),
        HirStructField::new("y".to_string(), HirType::Int),
    ]);
    ctx.add_struct(&s);
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Point".to_string()),
        initializers: vec![
            HirExpression::IntLiteral(10),
            HirExpression::IntLiteral(20),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("Point"), "Got: {}", result);
    assert!(result.contains("x: 10"), "Got: {}", result);
    assert!(result.contains("y: 20"), "Got: {}", result);
}

// --- CompoundLiteral: struct partial init → ..Default::default() ---
#[test]
fn expr_target_compound_literal_struct_partial() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let s = HirStruct::new("Point".to_string(), vec![
        HirStructField::new("x".to_string(), HirType::Int),
        HirStructField::new("y".to_string(), HirType::Int),
        HirStructField::new("z".to_string(), HirType::Int),
    ]);
    ctx.add_struct(&s);
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Point".to_string()),
        initializers: vec![HirExpression::IntLiteral(10)],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("..Default::default()"), "Got: {}", result);
}

// --- CompoundLiteral: empty struct ---
#[test]
fn expr_target_compound_literal_empty_struct() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Empty".to_string()),
        initializers: vec![],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("Empty {}"), "Got: {}", result);
}

// --- CompoundLiteral: array with elements ---
#[test]
fn expr_target_compound_literal_array() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
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
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("[1, 2, 3]"), "Got: {}", result);
}

// --- CompoundLiteral: array single init → repeat ---
#[test]
fn expr_target_compound_literal_array_single_init() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
        initializers: vec![HirExpression::IntLiteral(0)],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("[0; 10]"), "Got: {}", result);
}

// --- CompoundLiteral: empty array with size → default fill ---
#[test]
fn expr_target_compound_literal_array_empty_sized() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(5),
        },
        initializers: vec![],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("[0i32; 5]"), "Got: {}", result);
}

// --- CompoundLiteral: empty array no size → [] ---
#[test]
fn expr_target_compound_literal_array_empty_unsized() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None,
        },
        initializers: vec![],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "[]");
}

// --- CompoundLiteral: other type → comment ---
#[test]
fn expr_target_compound_literal_other_type() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Int,
        initializers: vec![HirExpression::IntLiteral(42)],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("Compound literal"), "Got: {}", result);
}

// --- Ternary: boolean condition ---
#[test]
fn expr_target_ternary_bool_condition() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
        then_expr: Box::new(HirExpression::Variable("a".to_string())),
        else_expr: Box::new(HirExpression::Variable("b".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("if a > b"), "Got: {}", result);
    assert!(result.contains("{ a }"), "Got: {}", result);
    assert!(result.contains("{ b }"), "Got: {}", result);
}

// --- Ternary: non-boolean condition → != 0 ---
#[test]
fn expr_target_ternary_int_condition() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("flag".to_string(), HirType::Int);
    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::Variable("flag".to_string())),
        then_expr: Box::new(HirExpression::IntLiteral(1)),
        else_expr: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("!= 0"), "Got: {}", result);
}

// --- IsNotNull → if let Some ---
#[test]
fn expr_target_is_not_null() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::IsNotNull(
        Box::new(HirExpression::Variable("ptr".to_string())),
    );
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("if let Some(_)"), "Got: {}", result);
}

// --- Calloc HIR: float type → 0.0f32 ---
#[test]
fn expr_target_calloc_hir_float() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(5)),
        element_type: Box::new(HirType::Float),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("0.0f32"), "Got: {}", result);
}

// --- Calloc HIR: double → 0.0f64 ---
#[test]
fn expr_target_calloc_hir_double() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(5)),
        element_type: Box::new(HirType::Double),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("0.0f64"), "Got: {}", result);
}

// --- Calloc HIR: char → 0u8 ---
#[test]
fn expr_target_calloc_hir_char() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(256)),
        element_type: Box::new(HirType::Char),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("0u8"), "Got: {}", result);
}

// --- Calloc HIR: signed char → 0i8 ---
#[test]
fn expr_target_calloc_hir_signed_char() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(128)),
        element_type: Box::new(HirType::SignedChar),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("0i8"), "Got: {}", result);
}

// --- SliceIndex → arr[i as usize] ---
#[test]
fn expr_target_slice_index() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::SliceIndex {
        slice: Box::new(HirExpression::Variable("data".to_string())),
        index: Box::new(HirExpression::Variable("i".to_string())),
        element_type: HirType::Int,
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("data[(i) as usize]"), "Got: {}", result);
}

// --- FieldAccess → object.field ---
#[test]
fn expr_target_field_access() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FieldAccess {
        object: Box::new(HirExpression::Variable("point".to_string())),
        field: "x".to_string(),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("point.x"), "Got: {}", result);
}

// --- PointerFieldAccess: chained → no explicit deref ---
#[test]
fn expr_target_pointer_field_access_chained() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::PointerFieldAccess {
            pointer: Box::new(HirExpression::Variable("a".to_string())),
            field: "b".to_string(),
        }),
        field: "c".to_string(),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    // Chained: a->b->c → (*a).b.c (no double deref)
    assert!(result.contains(".c"), "Got: {}", result);
    assert!(result.contains(".b"), "Got: {}", result);
}

// --- PointerFieldAccess: raw pointer → unsafe ---
#[test]
fn expr_target_pointer_field_access_raw_pointer_unsafe() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("node".to_string(), HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))));
    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("node".to_string())),
        field: "data".to_string(),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains("(*node).data"), "Got: {}", result);
}

// --- CompoundLiteral: array partial init → pad with defaults ---
#[test]
fn expr_target_compound_literal_array_partial_init() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(5),
        },
        initializers: vec![
            HirExpression::IntLiteral(1),
            HirExpression::IntLiteral(2),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    // Should pad remaining 3 elements with 0i32
    assert!(result.contains("1, 2, 0i32, 0i32, 0i32"), "Got: {}", result);
}

// ============================================================================
// BATCH 20: Default function call path (slice/string_iter/raw_ptr/ref params),
// Variable→Pointer coercion, malloc in statement context
// ============================================================================

// --- FunctionCall default: AddressOf arg → &mut ---
#[test]
fn expr_target_func_call_address_of_arg_mut() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // Function expects &mut for param 0
    ctx.add_function("modify".to_string(), vec![
        HirType::Reference { inner: Box::new(HirType::Int), mutable: true },
    ]);
    let expr = HirExpression::FunctionCall {
        function: "modify".to_string(),
        arguments: vec![HirExpression::AddressOf(
            Box::new(HirExpression::Variable("x".to_string())),
        )],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("&mut x"), "Got: {}", result);
}

// --- FunctionCall default: AddressOf arg → & (immutable) ---
#[test]
fn expr_target_func_call_address_of_arg_immut() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // Function expects & for param 0
    ctx.add_function("read_val".to_string(), vec![
        HirType::Reference { inner: Box::new(HirType::Int), mutable: false },
    ]);
    let expr = HirExpression::FunctionCall {
        function: "read_val".to_string(),
        arguments: vec![HirExpression::AddressOf(
            Box::new(HirExpression::Variable("x".to_string())),
        )],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("&x"), "Got: {}", result);
    assert!(!result.contains("&mut"), "Got: {}", result);
}

// --- FunctionCall default: slice mapping — skip len arg ---
#[test]
fn expr_target_func_call_slice_mapping_skip_len() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // Array param at index 0, length param at index 1 → skip len
    ctx.add_slice_func_args("process".to_string(), vec![(0, 1)]);
    let expr = HirExpression::FunctionCall {
        function: "process".to_string(),
        arguments: vec![
            HirExpression::Variable("arr".to_string()),
            HirExpression::Variable("len".to_string()),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("&arr"), "Got: {}", result);
    assert!(!result.contains("len"), "Got: {}", result);
}

// --- FunctionCall default: string iter mutable array → &mut arr ---
#[test]
fn expr_target_func_call_string_iter_mutable_array() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("buf".to_string(), HirType::Array {
        element_type: Box::new(HirType::Char),
        size: Some(256),
    });
    ctx.add_string_iter_func("fill".to_string(), vec![(0, true)]); // param 0 is mutable
    let expr = HirExpression::FunctionCall {
        function: "fill".to_string(),
        arguments: vec![HirExpression::Variable("buf".to_string())],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("&mut buf"), "Got: {}", result);
}

// --- FunctionCall default: string iter immutable array → &arr ---
#[test]
fn expr_target_func_call_string_iter_immut_array() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("buf".to_string(), HirType::Array {
        element_type: Box::new(HirType::Char),
        size: Some(256),
    });
    ctx.add_string_iter_func("scan".to_string(), vec![(0, false)]); // param 0 is immutable
    let expr = HirExpression::FunctionCall {
        function: "scan".to_string(),
        arguments: vec![HirExpression::Variable("buf".to_string())],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("&buf"), "Got: {}", result);
    assert!(!result.contains("&mut"), "Got: {}", result);
}

// --- FunctionCall default: string iter string literal → byte string ---
#[test]
fn expr_target_func_call_string_iter_str_literal() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_string_iter_func("parse".to_string(), vec![(0, false)]);
    let expr = HirExpression::FunctionCall {
        function: "parse".to_string(),
        arguments: vec![HirExpression::StringLiteral("hello".to_string())],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("b\"hello\""), "Got: {}", result);
}

// --- FunctionCall default: raw pointer param + array arg → as_mut_ptr ---
#[test]
fn expr_target_func_call_raw_ptr_param_array_arg() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("data".to_string(), HirType::Array {
        element_type: Box::new(HirType::Int),
        size: Some(100),
    });
    ctx.add_function("process_raw".to_string(), vec![
        HirType::Pointer(Box::new(HirType::Int)),
    ]);
    let expr = HirExpression::FunctionCall {
        function: "process_raw".to_string(),
        arguments: vec![HirExpression::Variable("data".to_string())],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("data.as_mut_ptr()"), "Got: {}", result);
}

// --- FunctionCall default: raw pointer param + string literal → as_ptr ---
#[test]
fn expr_target_func_call_raw_ptr_param_str_literal() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_function("process_raw".to_string(), vec![
        HirType::Pointer(Box::new(HirType::Char)),
    ]);
    let expr = HirExpression::FunctionCall {
        function: "process_raw".to_string(),
        arguments: vec![HirExpression::StringLiteral("test".to_string())],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("as_ptr() as *mut u8"), "Got: {}", result);
}

// --- FunctionCall default: ref param + pointer arg → unsafe &mut *ptr ---
#[test]
fn expr_target_func_call_ref_param_pointer_arg() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    ctx.add_function("take_ref".to_string(), vec![
        HirType::Reference { inner: Box::new(HirType::Int), mutable: true },
    ]);
    let expr = HirExpression::FunctionCall {
        function: "take_ref".to_string(),
        arguments: vec![HirExpression::Variable("ptr".to_string())],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains("&mut *ptr"), "Got: {}", result);
}

// --- FunctionCall default: slice param + sized array → &mut arr ---
#[test]
fn expr_target_func_call_slice_param_sized_array() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("arr".to_string(), HirType::Array {
        element_type: Box::new(HirType::Int),
        size: Some(10),
    });
    ctx.add_function("take_slice".to_string(), vec![
        HirType::Array { element_type: Box::new(HirType::Int), size: None },
    ]);
    let expr = HirExpression::FunctionCall {
        function: "take_slice".to_string(),
        arguments: vec![HirExpression::Variable("arr".to_string())],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("&mut arr"), "Got: {}", result);
}

// --- Variable → Pointer: Vec to *mut T ---
#[test]
fn expr_target_variable_vec_to_raw_ptr() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("buf".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::Variable("buf".to_string());
    let result = cg.generate_expression_with_target_type(
        &expr, &ctx, Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(result.contains("as_mut_ptr()"), "Got: {}", result);
}

// --- Variable → Pointer: Array to *mut T ---
#[test]
fn expr_target_variable_array_to_raw_ptr() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("arr".to_string(), HirType::Array {
        element_type: Box::new(HirType::Int),
        size: Some(10),
    });
    let expr = HirExpression::Variable("arr".to_string());
    let result = cg.generate_expression_with_target_type(
        &expr, &ctx, Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(result.contains("as_mut_ptr()"), "Got: {}", result);
}

// --- Variable → Pointer: Array to *mut () (void pointer) ---
#[test]
fn expr_target_variable_array_to_void_ptr() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("arr".to_string(), HirType::Array {
        element_type: Box::new(HirType::Int),
        size: Some(10),
    });
    let expr = HirExpression::Variable("arr".to_string());
    let result = cg.generate_expression_with_target_type(
        &expr, &ctx, Some(&HirType::Pointer(Box::new(HirType::Void))),
    );
    assert!(result.contains("as_mut_ptr() as *mut ()"), "Got: {}", result);
}

// --- Variable → Pointer: Pointer to Pointer (no conversion) ---
#[test]
fn expr_target_variable_ptr_to_ptr() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::Variable("p".to_string());
    let result = cg.generate_expression_with_target_type(
        &expr, &ctx, Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    // Should just return "p" without conversion
    assert_eq!(result, "p");
}

// --- Variable: int to char coercion → as u8 ---
#[test]
fn expr_target_variable_int_to_char_coercion() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("c".to_string(), HirType::Int);
    let expr = HirExpression::Variable("c".to_string());
    let result = cg.generate_expression_with_target_type(
        &expr, &ctx, Some(&HirType::Char),
    );
    assert!(result.contains("as u8"), "Got: {}", result);
}

// --- Statement: malloc init with Box(struct with default) → Box::default() ---
#[test]
fn stmt_ctx_malloc_box_struct_default() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let s = HirStruct::new("Node".to_string(), vec![
        HirStructField::new("val".to_string(), HirType::Int),
    ]);
    ctx.add_struct(&s);
    // Mark Node as having Default
    // struct_has_default is auto-derived when no arrays > 32 elements (already the case)
    let stmt = HirStatement::VariableDeclaration {
        name: "node".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
        initializer: Some(HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::Sizeof { type_name: "Node".to_string() }],
        }),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("Box::default()") || result.contains("Box::new"), "Got: {}", result);
}

// --- FunctionCall default: int param + char literal → cast as i32 ---
#[test]
fn expr_target_func_call_int_param_char_literal() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_function("putchar".to_string(), vec![HirType::Int]);
    let expr = HirExpression::FunctionCall {
        function: "putchar".to_string(),
        arguments: vec![HirExpression::CharLiteral(32)], // space = 32
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("i32"), "Got: {}", result);
}

// --- FunctionCall default: string func (strcmp) with PointerFieldAccess → CStr ---
#[test]
fn expr_target_func_call_strcmp_pointer_field_access() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("entry".to_string(), HirType::Pointer(Box::new(HirType::Struct("Entry".to_string()))));
    ctx.add_function("strcmp".to_string(), vec![
        HirType::StringReference,
        HirType::StringReference,
    ]);
    let expr = HirExpression::FunctionCall {
        function: "strcmp".to_string(),
        arguments: vec![
            HirExpression::PointerFieldAccess {
                pointer: Box::new(HirExpression::Variable("entry".to_string())),
                field: "key".to_string(),
            },
            HirExpression::StringLiteral("test".to_string()),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("CStr") || result.contains("unsafe"), "Got: {}", result);
}

// --- FunctionCall default: WIFSIGNALED → .signal().is_some() ---
#[test]
fn expr_target_wifsignaled() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "WIFSIGNALED".to_string(),
        arguments: vec![HirExpression::Variable("status".to_string())],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains(".signal().is_some()"), "Got: {}", result);
}

// --- FunctionCall default: WTERMSIG → .signal().unwrap_or(0) ---
#[test]
fn expr_target_wtermsig() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "WTERMSIG".to_string(),
        arguments: vec![HirExpression::Variable("status".to_string())],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains(".signal().unwrap_or(0)"), "Got: {}", result);
}

// --- FunctionCall default: waitpid → child.wait() ---
#[test]
fn expr_target_waitpid() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "waitpid".to_string(),
        arguments: vec![],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("child.wait()"), "Got: {}", result);
}

// --- FunctionCall: fopen append mode → File::create ---
#[test]
fn expr_target_fopen_append_mode() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "fopen".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("log.txt".to_string()),
            HirExpression::StringLiteral("a".to_string()),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("File::create"), "Got: {}", result);
}

// ============================================================================
// BATCH 21: malloc FunctionCall in statement context, Malloc HIR in statement,
// char pointer string literal init, literal targets, address-of targets
// ============================================================================

// --- Statement: FunctionCall malloc with struct pointer → Box::default (struct has default) ---
#[test]
fn stmt_ctx_func_call_malloc_struct_box_default() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let s = HirStruct::new("Node".to_string(), vec![
        HirStructField::new("val".to_string(), HirType::Int),
        HirStructField::new("next".to_string(), HirType::Pointer(Box::new(HirType::Struct("Node".to_string())))),
    ]);
    ctx.add_struct(&s);
    let stmt = HirStatement::VariableDeclaration {
        name: "node".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
        initializer: Some(HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::Sizeof { type_name: "Node".to_string() }],
        }),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("Box::default()"), "Got: {}", result);
}

// --- Statement: FunctionCall malloc with struct pointer (large array → no default) ---
#[test]
fn stmt_ctx_func_call_malloc_struct_box_zeroed() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let s = HirStruct::new("BigStruct".to_string(), vec![
        HirStructField::new("data".to_string(), HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(100), // > 32, so no Default
        }),
    ]);
    ctx.add_struct(&s);
    let stmt = HirStatement::VariableDeclaration {
        name: "big".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Struct("BigStruct".to_string()))),
        initializer: Some(HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::Sizeof { type_name: "BigStruct".to_string() }],
        }),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("zeroed"), "Got: {}", result);
}

// --- Statement: FunctionCall malloc with int pointer + multiply → Vec ---
#[test]
fn stmt_ctx_func_call_malloc_vec_multiply() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(HirExpression::Variable("n".to_string())),
                right: Box::new(HirExpression::Sizeof { type_name: "int".to_string() }),
            }],
        }),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("Vec<i32>") || result.contains("vec!["), "Got: {}", result);
}

// --- Statement: Malloc HIR with Box type → Box::new(default) ---
#[test]
fn stmt_ctx_malloc_hir_box_type() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "p".to_string(),
        var_type: HirType::Box(Box::new(HirType::Int)),
        initializer: Some(HirExpression::Malloc {
            size: Box::new(HirExpression::IntLiteral(4)),
        }),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("Box::new("), "Got: {}", result);
}

// --- Statement: Malloc HIR with Vec type + multiply → Vec::with_capacity ---
#[test]
fn stmt_ctx_malloc_hir_vec_multiply() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "v".to_string(),
        var_type: HirType::Vec(Box::new(HirType::Int)),
        initializer: Some(HirExpression::Malloc {
            size: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(HirExpression::Variable("n".to_string())),
                right: Box::new(HirExpression::Sizeof { type_name: "int".to_string() }),
            }),
        }),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("Vec::with_capacity("), "Got: {}", result);
}

// --- Statement: Malloc HIR with Vec type no multiply → Vec::new ---
#[test]
fn stmt_ctx_malloc_hir_vec_no_multiply() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "v".to_string(),
        var_type: HirType::Vec(Box::new(HirType::Int)),
        initializer: Some(HirExpression::Malloc {
            size: Box::new(HirExpression::IntLiteral(100)),
        }),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("Vec::new()"), "Got: {}", result);
}

// --- Statement: Malloc HIR with other type → Box::new(0i32) ---
#[test]
fn stmt_ctx_malloc_hir_other_type() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "x".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::Malloc {
            size: Box::new(HirExpression::IntLiteral(4)),
        }),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("Box::new(0i32)"), "Got: {}", result);
}

// --- Statement: char* with string literal → &str ---
#[test]
fn stmt_ctx_char_ptr_string_literal_to_str() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "msg".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Char)),
        initializer: Some(HirExpression::StringLiteral("hello".to_string())),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("&str"), "Got: {}", result);
}

// --- Statement: char* array with string literals → [&str; N] ---
#[test]
fn stmt_ctx_char_ptr_array_string_literals() {
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
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("[&str; 2]"), "Got: {}", result);
}

// --- StringLiteral with Pointer(Char) target → byte string pointer ---
#[test]
fn expr_target_string_literal_to_char_ptr_batch21() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::StringLiteral("world".to_string());
    let result = cg.generate_expression_with_target_type(
        &expr, &ctx, Some(&HirType::Pointer(Box::new(HirType::Char))),
    );
    assert!(result.contains("as_ptr() as *mut u8") || result.contains("b\""), "Got: {}", result);
}

// ============================================================================
// BATCH 22: BinaryOp expression paths (assignment, null checks, strlen, char coercion)
// Target: lines 1291-1462 (assignment, option/pointer/Vec/Box null, strlen optimization)
// ============================================================================

// --- DECY-195: Embedded assignment expression → block ---
#[test]
fn expr_target_binary_assign_block() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Assign,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::IntLiteral(42)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("__assign_tmp"), "Got: {}", result);
    assert!(result.contains("x = __assign_tmp"), "Got: {}", result);
}

// --- DECY-223: Global array index assignment in expression → unsafe block ---
#[test]
fn expr_target_binary_assign_global_array_index() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_global("buf".to_string());
    ctx.add_variable("buf".to_string(), HirType::Array {
        element_type: Box::new(HirType::Int),
        size: Some(256),
    });
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Assign,
        left: Box::new(HirExpression::ArrayIndex {
            array: Box::new(HirExpression::Variable("buf".to_string())),
            index: Box::new(HirExpression::Variable("i".to_string())),
        }),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains("buf["), "Got: {}", result);
    assert!(result.contains("__assign_tmp"), "Got: {}", result);
}

// --- Option == NULL → .is_none() ---
#[test]
fn expr_target_binary_option_eq_null_is_none() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("node".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("node".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("is_none()"), "Got: {}", result);
}

// --- Option != NULL → .is_some() ---
#[test]
fn expr_target_binary_option_ne_null_is_some() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("node".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("node".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("is_some()"), "Got: {}", result);
}

// --- NULL == Option → .is_none() (reverse) ---
#[test]
fn expr_target_binary_null_eq_option_reverse() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::NullLiteral),
        right: Box::new(HirExpression::Variable("ptr".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("is_none()"), "Got: {}", result);
}

// --- NULL != Option → .is_some() (reverse) ---
#[test]
fn expr_target_binary_null_ne_option_reverse() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::NullLiteral),
        right: Box::new(HirExpression::Variable("ptr".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("is_some()"), "Got: {}", result);
}

// --- Pointer == 0 → std::ptr::null_mut() ---
#[test]
fn expr_target_binary_ptr_eq_zero_null_mut() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("p".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("std::ptr::null_mut()"), "Got: {}", result);
}

// --- Pointer != 0 → != std::ptr::null_mut() ---
#[test]
fn expr_target_binary_ptr_ne_zero_null_mut() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("p".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("!= std::ptr::null_mut()"), "Got: {}", result);
}

// --- 0 == ptr → reverse null check ---
#[test]
fn expr_target_binary_zero_eq_ptr_reverse() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::IntLiteral(0)),
        right: Box::new(HirExpression::Variable("p".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("std::ptr::null_mut()"), "Got: {}", result);
}

// --- DECY-235: Pointer field access == 0 → null_mut() ---
#[test]
fn expr_target_binary_field_ptr_eq_zero_null_mut() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let s = HirStruct::new("Node".to_string(), vec![
        HirStructField::new("next".to_string(), HirType::Pointer(Box::new(HirType::Struct("Node".to_string())))),
    ]);
    ctx.add_struct(&s);
    ctx.add_variable("node".to_string(), HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::PointerFieldAccess {
            pointer: Box::new(HirExpression::Variable("node".to_string())),
            field: "next".to_string(),
        }),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("std::ptr::null_mut()") || result.contains("null"), "Got: {}", result);
}

// --- 0 == field_ptr (reverse) ---
#[test]
fn expr_target_binary_zero_eq_field_ptr_reverse() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let s = HirStruct::new("Node".to_string(), vec![
        HirStructField::new("next".to_string(), HirType::Pointer(Box::new(HirType::Struct("Node".to_string())))),
    ]);
    ctx.add_struct(&s);
    ctx.add_variable("node".to_string(), HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::IntLiteral(0)),
        right: Box::new(HirExpression::PointerFieldAccess {
            pointer: Box::new(HirExpression::Variable("node".to_string())),
            field: "next".to_string(),
        }),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("std::ptr::null_mut()") || result.contains("null"), "Got: {}", result);
}

// --- Vec == 0 → false (Vec never null) ---
#[test]
fn expr_target_binary_vec_eq_zero_false() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("arr".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("false"), "Got: {}", result);
}

// --- Vec != NULL → true (Vec never null) ---
#[test]
fn expr_target_binary_vec_ne_null_true() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("arr".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("true"), "Got: {}", result);
}

// --- Box == 0 → false (Box never null) ---
#[test]
fn expr_target_binary_box_eq_zero_false() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("node".to_string(), HirType::Box(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("node".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("false"), "Got: {}", result);
}

// --- Box != NULL → true (Box never null) ---
#[test]
fn expr_target_binary_box_ne_null_true() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("node".to_string(), HirType::Box(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("node".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("true"), "Got: {}", result);
}

// --- strlen(s) == 0 → s.is_empty() ---
#[test]
fn expr_target_binary_strlen_eq_zero_is_empty() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::FunctionCall {
            function: "strlen".to_string(),
            arguments: vec![HirExpression::Variable("s".to_string())],
        }),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("is_empty()"), "Got: {}", result);
}

// --- strlen(s) != 0 → !s.is_empty() ---
#[test]
fn expr_target_binary_strlen_ne_zero_not_is_empty() {
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
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("!") && result.contains("is_empty()"), "Got: {}", result);
}

// --- 0 == strlen(s) → s.is_empty() (reverse) ---
#[test]
fn expr_target_binary_zero_eq_strlen_reverse() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::IntLiteral(0)),
        right: Box::new(HirExpression::FunctionCall {
            function: "strlen".to_string(),
            arguments: vec![HirExpression::Variable("s".to_string())],
        }),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("is_empty()"), "Got: {}", result);
}

// --- 0 != strlen(s) → !s.is_empty() (reverse) ---
#[test]
fn expr_target_binary_zero_ne_strlen_reverse() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::IntLiteral(0)),
        right: Box::new(HirExpression::FunctionCall {
            function: "strlen".to_string(),
            arguments: vec![HirExpression::Variable("s".to_string())],
        }),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("!") && result.contains("is_empty()"), "Got: {}", result);
}

// --- Char-to-Int comparison: int_var != CharLiteral ---
#[test]
fn expr_target_binary_int_cmp_char_literal() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("c".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("c".to_string())),
        right: Box::new(HirExpression::CharLiteral(10)), // '\n'
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("10i32"), "Got: {}", result);
}

// --- Char-to-Int comparison: CharLiteral == int_var (reverse) ---
#[test]
fn expr_target_binary_char_literal_cmp_int_reverse() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("c".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::CharLiteral(65)), // 'A'
        right: Box::new(HirExpression::Variable("c".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("65i32"), "Got: {}", result);
}

// --- Int + CharLiteral arithmetic → cast to i32 ---
#[test]
fn expr_target_binary_int_add_char_literal() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("n".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("n".to_string())),
        right: Box::new(HirExpression::CharLiteral(48)), // '0'
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("48i32"), "Got: {}", result);
}

// --- CharLiteral - Int (reverse arithmetic) ---
#[test]
fn expr_target_binary_char_literal_sub_int_reverse() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("n".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::CharLiteral(48)), // '0'
        right: Box::new(HirExpression::Variable("n".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("48i32"), "Got: {}", result);
}

// --- Char variable with Int target type ---
#[test]
fn expr_target_char_var_to_int_cast() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ch".to_string(), HirType::Char);
    let expr = HirExpression::Variable("ch".to_string());
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("as i32"), "Got: {}", result);
}

// --- Global char variable with Int target → unsafe ---
#[test]
fn expr_target_global_char_var_to_int_unsafe() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_global("ch".to_string());
    ctx.add_variable("ch".to_string(), HirType::Char);
    let expr = HirExpression::Variable("ch".to_string());
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains("as i32"), "Got: {}", result);
}

// ============================================================================
// BATCH 22 continued: Pointer subtraction detection (lines 5710-5760)
// ============================================================================

// --- statement_uses_pointer_subtraction in If then_block ---
#[test]
fn ptr_sub_detect_if_then_block() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "calc_len".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("str".to_string(), HirType::Pointer(Box::new(HirType::Char))),
            HirParameter::new("start".to_string(), HirType::Pointer(Box::new(HirType::Char))),
        ],
        vec![
            HirStatement::If {
                condition: HirExpression::Variable("str".to_string()),
                then_block: vec![
                    HirStatement::Return(Some(HirExpression::BinaryOp {
                        op: BinaryOperator::Subtract,
                        left: Box::new(HirExpression::Variable("str".to_string())),
                        right: Box::new(HirExpression::Variable("start".to_string())),
                    })),
                ],
                else_block: None,
            },
        ],
    );
    let uses = cg.function_uses_pointer_subtraction(&func, "str");
    assert!(uses, "Should detect ptr subtraction in if then_block");
}

// --- statement_uses_pointer_subtraction in If else_block ---
#[test]
fn ptr_sub_detect_if_else_block() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "calc_len".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("str".to_string(), HirType::Pointer(Box::new(HirType::Char))),
            HirParameter::new("start".to_string(), HirType::Pointer(Box::new(HirType::Char))),
        ],
        vec![
            HirStatement::If {
                condition: HirExpression::IntLiteral(1),
                then_block: vec![],
                else_block: Some(vec![
                    HirStatement::Return(Some(HirExpression::BinaryOp {
                        op: BinaryOperator::Subtract,
                        left: Box::new(HirExpression::Variable("str".to_string())),
                        right: Box::new(HirExpression::Variable("start".to_string())),
                    })),
                ]),
            },
        ],
    );
    let uses = cg.function_uses_pointer_subtraction(&func, "str");
    assert!(uses, "Should detect ptr subtraction in if else_block");
}

// --- statement_uses_pointer_subtraction in While loop ---
#[test]
fn ptr_sub_detect_while_body() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "scan".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Char))),
            HirParameter::new("base".to_string(), HirType::Pointer(Box::new(HirType::Char))),
        ],
        vec![
            HirStatement::While {
                condition: HirExpression::IntLiteral(1),
                body: vec![
                    HirStatement::Return(Some(HirExpression::BinaryOp {
                        op: BinaryOperator::Subtract,
                        left: Box::new(HirExpression::Variable("p".to_string())),
                        right: Box::new(HirExpression::Variable("base".to_string())),
                    })),
                ],
            },
        ],
    );
    let uses = cg.function_uses_pointer_subtraction(&func, "p");
    assert!(uses, "Should detect ptr subtraction in while body");
}

// --- statement_uses_pointer_subtraction in While condition ---
#[test]
fn ptr_sub_detect_while_condition() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "check".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Char))),
            HirParameter::new("end".to_string(), HirType::Pointer(Box::new(HirType::Char))),
        ],
        vec![
            HirStatement::While {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::Subtract,
                    left: Box::new(HirExpression::Variable("p".to_string())),
                    right: Box::new(HirExpression::Variable("end".to_string())),
                },
                body: vec![],
            },
        ],
    );
    let uses = cg.function_uses_pointer_subtraction(&func, "p");
    assert!(uses, "Should detect ptr subtraction in while condition");
}

// --- expression_uses_pointer_subtraction in Dereference ---
#[test]
fn ptr_sub_detect_deref_expr() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "get_diff".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("q".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        ],
        vec![
            HirStatement::Return(Some(HirExpression::Dereference(
                Box::new(HirExpression::BinaryOp {
                    op: BinaryOperator::Subtract,
                    left: Box::new(HirExpression::Variable("p".to_string())),
                    right: Box::new(HirExpression::Variable("q".to_string())),
                }),
            ))),
        ],
    );
    let uses = cg.function_uses_pointer_subtraction(&func, "p");
    assert!(uses, "Should detect ptr subtraction inside dereference");
}

// --- expression_uses_pointer_subtraction in Cast ---
#[test]
fn ptr_sub_detect_cast_expr() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "diff_as_int".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("a".to_string(), HirType::Pointer(Box::new(HirType::Char))),
            HirParameter::new("b".to_string(), HirType::Pointer(Box::new(HirType::Char))),
        ],
        vec![
            HirStatement::Return(Some(HirExpression::Cast {
                expr: Box::new(HirExpression::BinaryOp {
                    op: BinaryOperator::Subtract,
                    left: Box::new(HirExpression::Variable("a".to_string())),
                    right: Box::new(HirExpression::Variable("b".to_string())),
                }),
                target_type: HirType::Int,
            })),
        ],
    );
    let uses = cg.function_uses_pointer_subtraction(&func, "a");
    assert!(uses, "Should detect ptr subtraction inside cast");
}

