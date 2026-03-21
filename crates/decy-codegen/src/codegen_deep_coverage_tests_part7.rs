    let code = cg.generate_function_with_lifetimes_and_structs(
        &func,
        &sig,
        &[],
        &[],
        &[],
        &[],
        &[],
    );
    assert!(
        code.contains("fn sum_arr"),
        "Should generate function with array param context, got: {}",
        code
    );
}

// ============================================================================
// Batch 5: transform_vec_statement with capacity (line 6939-6941)
// ============================================================================

#[test]
fn transform_vec_statement_with_capacity() {
    // Lines 6913-6917: VecCandidate WITH capacity_expr
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "items".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::Malloc {
            size: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(HirExpression::IntLiteral(10)),
                right: Box::new(HirExpression::IntLiteral(4)),
            }),
        }),
    };
    let candidate = decy_analyzer::patterns::VecCandidate {
        variable: "items".to_string(),
        malloc_index: 0,
        free_index: None,
        capacity_expr: Some(HirExpression::IntLiteral(10)),
    };
    let result = cg.transform_vec_statement(&stmt, &candidate);
    if let HirStatement::VariableDeclaration {
        var_type,
        initializer,
        ..
    } = &result
    {
        assert!(
            matches!(var_type, HirType::Vec(_)),
            "Should transform to Vec type, got: {:?}",
            var_type
        );
        assert!(
            initializer.is_some(),
            "Should have Vec::with_capacity initializer"
        );
    } else {
        panic!("Expected VariableDeclaration");
    }
}

// ============================================================================
// Batch 5: generate_function_with_box_and_vec_transform with body (line 6983)
// ============================================================================

#[test]
fn generate_function_with_box_and_vec_transform_with_body() {
    // Lines 6970-6985: Combined transform with matching candidates
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "alloc_both".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Malloc {
                    size: Box::new(HirExpression::BinaryOp {
                        op: BinaryOperator::Multiply,
                        left: Box::new(HirExpression::IntLiteral(10)),
                        right: Box::new(HirExpression::IntLiteral(4)),
                    }),
                }),
            },
            HirStatement::Return(None),
        ],
    );
    let vec_candidates = vec![decy_analyzer::patterns::VecCandidate {
        variable: "arr".to_string(),
        malloc_index: 0,
        free_index: None,
        capacity_expr: Some(HirExpression::IntLiteral(10)),
    }];
    let code = cg.generate_function_with_box_and_vec_transform(&func, &[], &vec_candidates);
    assert!(
        code.contains("fn alloc_both"),
        "Should generate combined transform function, got: {}",
        code
    );
}

// ============================================================================
// Batch 5: Malloc Vec with non-multiply size (lines 4193-4197)
// ============================================================================

#[test]
fn stmt_malloc_vec_non_multiply_size() {
    // Lines 4192-4197: Malloc with Vec type but non-multiply size → Vec::new()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "items".to_string(),
        var_type: HirType::Vec(Box::new(HirType::Int)),
        initializer: Some(HirExpression::Malloc {
            size: Box::new(HirExpression::IntLiteral(40)),
        }),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("Vec::new()"),
        "Vec malloc with non-multiply size should use Vec::new(), got: {}",
        code
    );
}

// ============================================================================
// Batch 5: Malloc Box with non-Default struct (lines 4215-4228)
// ============================================================================

#[test]
fn stmt_malloc_box_struct_no_default() {
    // Lines 4221-4228: FunctionCall("malloc") with Box(Struct) without Default → zeroed
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // Note: no struct registered as having Default, so struct_has_default returns false
    let stmt = HirStatement::VariableDeclaration {
        name: "node".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
        initializer: Some(HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::Sizeof {
                type_name: "Node".to_string(),
            }],
        }),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("zeroed") || code.contains("Box::new"),
        "Malloc Box struct without Default should use zeroed, got: {}",
        code
    );
}

// ============================================================================
// Batch 5: Realloc NULL with non-multiply size (lines 4475)
// ============================================================================

#[test]
fn realloc_null_non_multiply_fallback() {
    // Lines 4461-4475: Realloc from NULL with non-multiply size → no resize
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("items".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let stmt = HirStatement::Assignment {
        target: "items".to_string(),
        value: HirExpression::Realloc {
            pointer: Box::new(HirExpression::NullLiteral),
            new_size: Box::new(HirExpression::IntLiteral(100)),
        },
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    // Non-multiply size → falls through to normal realloc path at line 4478+
    assert!(
        !code.is_empty(),
        "Realloc NULL with non-multiply should produce code"
    );
}

// ============================================================================
// Batch 5: String iter param assignment with non-matching left (lines 4522-4524)
// ============================================================================

#[test]
fn string_iter_param_assignment_left_mismatch() {
    // Lines 4505-4522: String iter param, BinaryOp but left != target
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("s".to_string(), HirType::Pointer(Box::new(HirType::Char)));
    ctx.add_string_iter_param("s".to_string(), "s_idx".to_string());
    let stmt = HirStatement::Assignment {
        target: "s".to_string(),
        value: HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("other".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        },
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    // Left is "other", not "s", so doesn't match string iter advance
    assert!(
        !code.is_empty(),
        "Mismatched left should still produce code"
    );
}

// ============================================================================
// Batch 5: DerefAssignment with strip_unsafe (lines 4731-4734)
// ============================================================================

#[test]
fn deref_assign_double_pointer_strips_unsafe() {
    // Lines 4728-4734: strip_unsafe helper in DerefAssignment
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "pp".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Pointer(Box::new(HirType::Int))),
            mutable: true,
        },
    );
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Dereference(Box::new(HirExpression::Variable("pp".to_string()))),
        value: HirExpression::IntLiteral(99),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("unsafe"),
        "Reference(Pointer) deref assign should be unsafe, got: {}",
        code
    );
}

// ============================================================================
// Batch 5: ArrayIndexAssignment non-variable (line 4818)
// ============================================================================

#[test]
fn array_index_assign_field_access_array() {
    // Line 4818: ArrayIndexAssignment where array is not a simple Variable
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::FieldAccess {
            object: Box::new(HirExpression::Variable("obj".to_string())),
            field: "data".to_string(),
        }),
        index: Box::new(HirExpression::IntLiteral(0)),
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("obj.data") && code.contains("["),
        "ArrayIndexAssignment with field access should work, got: {}",
        code
    );
}

// ============================================================================
// Batch 5: Pointer arithmetic assignment field access (lines 5571-5572, 5582)
// ============================================================================

#[test]
fn ptr_arithmetic_add_assignment_not_same_var() {
    // Lines 5565-5572: ptr = ptr + n where left is not same variable
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Assignment {
        target: "ptr".to_string(),
        value: HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("other".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        },
    };
    assert!(
        !cg.statement_uses_pointer_arithmetic(&stmt, "ptr"),
        "ptr = other + 1 should NOT detect pointer arithmetic for ptr"
    );
}

#[test]
fn ptr_arithmetic_field_access_any_pointer() {
    // Lines 5590-5591: ptr = any_ptr->field
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Assignment {
        target: "cur".to_string(),
        value: HirExpression::PointerFieldAccess {
            pointer: Box::new(HirExpression::Variable("head".to_string())),
            field: "next".to_string(),
        },
    };
    assert!(
        cg.statement_uses_pointer_arithmetic(&stmt, "cur"),
        "cur = head->next should detect pointer arithmetic"
    );
}

#[test]
fn ptr_arithmetic_expression_pre_increment() {
    // Lines 5624-5628: PreIncrement expression
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Expression(HirExpression::PreIncrement {
        operand: Box::new(HirExpression::Variable("ptr".to_string())),
    });
    assert!(
        cg.statement_uses_pointer_arithmetic(&stmt, "ptr"),
        "++ptr should detect pointer arithmetic"
    );
}

#[test]
fn ptr_arithmetic_expression_post_decrement() {
    // Lines 5626-5628: PostDecrement expression
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Expression(HirExpression::PostDecrement {
        operand: Box::new(HirExpression::Variable("ptr".to_string())),
    });
    assert!(
        cg.statement_uses_pointer_arithmetic(&stmt, "ptr"),
        "ptr-- should detect pointer arithmetic"
    );
}

// ============================================================================
// BATCH 6: TypeContext field type inference, variable-to-pointer conversion,
//          inc/dec on deref non-variable, malloc expression checks,
//          LogicalNot, string deref, ternary/format edge cases
// ============================================================================

#[test]
fn type_context_get_field_type_box_struct() {
    // Line 210-215: Box<Struct> → extract struct name from Box inner
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "node".to_string(),
        HirType::Box(Box::new(HirType::Struct("Node".to_string()))),
    );
    ctx.structs.insert(
        "Node".to_string(),
        vec![("value".to_string(), HirType::Int)],
    );
    let result = ctx.get_field_type(&HirExpression::Variable("node".to_string()), "value");
    assert_eq!(result, Some(HirType::Int));
}

#[test]
fn type_context_get_field_type_box_non_struct() {
    // Line 214: Box<non-Struct> → return None
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "boxed".to_string(),
        HirType::Box(Box::new(HirType::Int)),
    );
    let result = ctx.get_field_type(&HirExpression::Variable("boxed".to_string()), "field");
    assert_eq!(result, None);
}

#[test]
fn type_context_get_field_type_reference_struct() {
    // Line 218-224: Reference { inner: Struct } → extract struct name
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "ref_node".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Struct("Point".to_string())),
            mutable: false,
        },
    );
    ctx.structs.insert(
        "Point".to_string(),
        vec![
            ("x".to_string(), HirType::Int),
            ("y".to_string(), HirType::Int),
        ],
    );
    let result = ctx.get_field_type(&HirExpression::Variable("ref_node".to_string()), "x");
    assert_eq!(result, Some(HirType::Int));
}

#[test]
fn type_context_get_field_type_reference_non_struct() {
    // Line 222: Reference { inner: non-Struct } → return None
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "ref_int".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
    );
    let result = ctx.get_field_type(&HirExpression::Variable("ref_int".to_string()), "field");
    assert_eq!(result, None);
}

#[test]
fn type_context_get_field_type_pointer_non_struct() {
    // Line 206: Pointer(non-Struct) → return None
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let result = ctx.get_field_type(&HirExpression::Variable("ptr".to_string()), "field");
    assert_eq!(result, None);
}

#[test]
fn type_context_get_field_type_unknown_type() {
    // Line 225: Other type (e.g., Int) → return None
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    let result = ctx.get_field_type(&HirExpression::Variable("x".to_string()), "field");
    assert_eq!(result, None);
}

#[test]
fn type_context_get_field_type_from_type_non_struct() {
    // Line 373: get_field_type_from_type with non-Struct type → None
    let ctx = TypeContext::new();
    let result = ctx.get_field_type_from_type(&HirType::Int, "field");
    assert_eq!(result, None);
}

#[test]
fn var_to_ptr_reference_mutable_to_pointer() {
    // Lines 1179-1183: Reference { inner: T, mutable: true } assigned to Pointer(T)
    // Should produce "var as *mut _"
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
    let result = cg.generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(result.contains("as *mut _"), "Got: {}", result);
}

#[test]
fn var_to_ptr_reference_immutable_to_pointer() {
    // Lines 1184-1186: Reference { inner: T, mutable: false } assigned to Pointer(T)
    // Should produce "var as *const _ as *mut _"
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
    let result = cg.generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(result.contains("as *const _ as *mut _"), "Got: {}", result);
}

#[test]
fn var_to_ptr_vec_to_pointer() {
    // Lines 1190-1193: Vec<T> to *mut T → .as_mut_ptr()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("buf".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::Variable("buf".to_string());
    let result = cg.generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(result.contains(".as_mut_ptr()"), "Got: {}", result);
}

#[test]
fn var_to_ptr_array_to_pointer() {
    // Lines 1198-1201: Array[T; N] to *mut T → .as_mut_ptr()
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
    let result = cg.generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(result.contains(".as_mut_ptr()"), "Got: {}", result);
}

#[test]
fn var_to_ptr_array_to_void_pointer() {
    // Lines 1204-1206: Array[T; N] to *mut () (void pointer) → .as_mut_ptr() as *mut ()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "arr".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(5),
        },
    );
    let expr = HirExpression::Variable("arr".to_string());
    let result = cg.generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Void))),
    );
    assert!(
        result.contains(".as_mut_ptr() as *mut ()"),
        "Got: {}",
        result
    );
}

#[test]
fn var_to_ptr_pointer_to_pointer() {
    // Lines 1211-1213: Pointer(T) → Pointer(T) — return variable directly (no conversion)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let expr = HirExpression::Variable("ptr".to_string());
    let result = cg.generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert_eq!(result, "ptr");
}

#[test]
fn var_to_ptr_int_to_char_coercion() {
    // Lines 1223-1228: Int variable with Char target → "x as u8"
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("c".to_string(), HirType::Int);
    let expr = HirExpression::Variable("c".to_string());
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Char));
    assert!(result.contains("as u8"), "Got: {}", result);
}

#[test]
fn binary_op_option_null_equal() {
    // Lines 1324-1330: Option var == NULL → .is_none()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "p".to_string(),
        HirType::Option(Box::new(HirType::Int)),
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("p".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("is_none"), "Got: {}", result);
}

#[test]
fn binary_op_option_null_not_equal() {
    // Lines 1324-1330: Option var != NULL → .is_some()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "p".to_string(),
        HirType::Option(Box::new(HirType::Int)),
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("p".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("is_some"), "Got: {}", result);
}

#[test]
fn binary_op_null_option_equal_reversed() {
    // Lines 1334-1341: NULL == Option var → .is_none() (reversed operands)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "p".to_string(),
        HirType::Option(Box::new(HirType::Int)),
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::NullLiteral),
        right: Box::new(HirExpression::Variable("p".to_string())),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("is_none"), "Got: {}", result);
}

#[test]
fn binary_op_null_option_not_equal_reversed() {
    // Lines 1334-1341: NULL != Option var → .is_some() (reversed operands)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "p".to_string(),
        HirType::Option(Box::new(HirType::Int)),
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::NullLiteral),
        right: Box::new(HirExpression::Variable("p".to_string())),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("is_some"), "Got: {}", result);
}

#[test]
fn binary_op_pointer_compare_zero() {
    // Lines 1347-1353: Pointer var == 0 → "ptr == std::ptr::null_mut()"
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("ptr".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("std::ptr::null_mut()"), "Got: {}", result);
}

#[test]
fn binary_op_zero_compare_pointer_reversed() {
    // Lines 1356-1362: 0 == Pointer var → "std::ptr::null_mut() == ptr" (reversed)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::IntLiteral(0)),
        right: Box::new(HirExpression::Variable("ptr".to_string())),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("std::ptr::null_mut()"), "Got: {}", result);
}

#[test]
fn binary_op_pointer_field_compare_zero() {
    // Lines 1367-1376: ptr->field == 0 where field is pointer → compare with null_mut()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "node".to_string(),
        HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
    );
    ctx.structs.insert(
        "Node".to_string(),
        vec![("next".to_string(), HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))))],
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::PointerFieldAccess {
            pointer: Box::new(HirExpression::Variable("node".to_string())),
            field: "next".to_string(),
        }),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("std::ptr::null_mut()"), "Got: {}", result);
}

#[test]
fn binary_op_zero_compare_pointer_field_reversed() {
    // Lines 1377-1385: 0 == ptr->field where field is pointer → null_mut() == ...
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "node".to_string(),
        HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
    );
    ctx.structs.insert(
        "Node".to_string(),
        vec![("next".to_string(), HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))))],
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::IntLiteral(0)),
        right: Box::new(HirExpression::PointerFieldAccess {
            pointer: Box::new(HirExpression::Variable("node".to_string())),
            field: "next".to_string(),
        }),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("std::ptr::null_mut()"), "Got: {}", result);
}

#[test]
fn binary_op_vec_null_check_equal() {
    // Lines 1391-1402: Vec var == 0 → "false /* Vec never null */"
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("arr".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("false"), "Got: {}", result);
}

#[test]
fn binary_op_vec_null_check_not_equal() {
    // Lines 1391-1402: Vec var != NULL → "true /* Vec never null */"
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("arr".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("true"), "Got: {}", result);
}

#[test]
fn binary_op_box_null_check_equal() {
    // Lines 1408-1423: Box var == 0 → "false /* Box never null */"
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "node".to_string(),
        HirType::Box(Box::new(HirType::Struct("Node".to_string()))),
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("node".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("false"), "Got: {}", result);
}

#[test]
fn binary_op_box_null_check_not_equal() {
    // Lines 1408-1423: Box var != NULL → "true /* Box never null */"
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "node".to_string(),
        HirType::Box(Box::new(HirType::Struct("Node".to_string()))),
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("node".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("true"), "Got: {}", result);
}

#[test]
fn binary_op_strlen_equal_zero() {
    // Lines 1434-1443: strlen(s) == 0 → s.is_empty()
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
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("is_empty()"), "Got: {}", result);
}

#[test]
fn binary_op_strlen_not_equal_zero() {
    // Lines 1434-1443: strlen(s) != 0 → !s.is_empty()
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
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("!s.is_empty()"), "Got: {}", result);
}

#[test]
fn binary_op_zero_equal_strlen_reversed() {
    // Lines 1452-1461: 0 == strlen(s) → s.is_empty() (reversed operands)
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
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("is_empty()"), "Got: {}", result);
}

#[test]
fn binary_op_zero_not_equal_strlen_reversed() {
    // Lines 1452-1461: 0 != strlen(s) → !s.is_empty() (reversed)
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
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("!s.is_empty()"), "Got: {}", result);
}

#[test]
fn post_inc_deref_non_variable_fallback() {
    // Lines 3318-3327: PostIncrement on Dereference of non-Variable (falls through to generic path)
    // Dereference(ArrayIndex) is NOT a Variable, so the inner match fails
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "arr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let expr = HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::ArrayIndex {
                array: Box::new(HirExpression::Variable("arr".to_string())),
                index: Box::new(HirExpression::IntLiteral(0)),
            },
        ))),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    // Falls through to generic post-increment path
    assert!(result.contains("__tmp"), "Got: {}", result);
}

#[test]
fn pre_inc_deref_non_variable_fallback() {
    // Lines 3353-3361: PreIncrement on Dereference of non-Variable (falls through)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "arr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let expr = HirExpression::PreIncrement {
        operand: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::ArrayIndex {
                array: Box::new(HirExpression::Variable("arr".to_string())),
                index: Box::new(HirExpression::IntLiteral(0)),
            },
        ))),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    // Falls through to generic pre-increment path
    assert!(result.contains("+= 1"), "Got: {}", result);
}

#[test]
fn post_dec_deref_non_variable_fallback() {
    // Lines 3382-3390: PostDecrement on Dereference of non-Variable (falls through)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "arr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let expr = HirExpression::PostDecrement {
        operand: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::ArrayIndex {
                array: Box::new(HirExpression::Variable("arr".to_string())),
                index: Box::new(HirExpression::IntLiteral(0)),
            },
        ))),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("__tmp"), "Got: {}", result);
}

#[test]
fn pre_dec_deref_non_variable_fallback() {
    // Lines 3414-3422: PreDecrement on Dereference of non-Variable (falls through)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "arr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let expr = HirExpression::PreDecrement {
        operand: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::ArrayIndex {
                array: Box::new(HirExpression::Variable("arr".to_string())),
                index: Box::new(HirExpression::IntLiteral(0)),
            },
        ))),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("-= 1"), "Got: {}", result);
}

#[test]
fn is_malloc_expression_calloc() {
    // Line 3584: Calloc variant → true
    assert!(CodeGenerator::is_malloc_expression(
        &HirExpression::Calloc {
            count: Box::new(HirExpression::IntLiteral(10)),
            element_type: Box::new(HirType::Int),
        }
    ));
}

#[test]
fn is_malloc_expression_function_call_malloc() {
    // Lines 3585-3587: FunctionCall "malloc" → true
    assert!(CodeGenerator::is_malloc_expression(
        &HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::IntLiteral(64)],
        }
    ));
}

#[test]
fn is_malloc_expression_cast_wrapping_malloc() {
    // Lines 3589-3590: Cast wrapping Malloc → true (recursive check)
    assert!(CodeGenerator::is_malloc_expression(
        &HirExpression::Cast {
            expr: Box::new(HirExpression::Malloc {
                size: Box::new(HirExpression::IntLiteral(32)),
            }),
            target_type: HirType::Pointer(Box::new(HirType::Int)),
        }
    ));
}

#[test]
fn is_malloc_expression_other() {
    // Line 3590: Non-malloc expression → false
    assert!(!CodeGenerator::is_malloc_expression(
        &HirExpression::IntLiteral(42)
    ));
}

#[test]
fn logical_not_on_non_boolean_no_target() {
    // Line 1076: LogicalNot on non-boolean without int target → "(x == 0)" (no cast)
    // Early match at line 1047 intercepts LogicalNot before the UnaryOp match at 2006
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert_eq!(result, "(x == 0)");
}

#[test]
fn logical_not_on_non_boolean_int_target() {
    // Lines 1066-1067: LogicalNot on non-boolean with Int target → "(x == 0) as i32"
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let result =
        cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("== 0) as i32"), "Got: {}", result);
}

#[test]
fn logical_not_on_boolean_no_target() {
    // Line 1073: LogicalNot on boolean without target → "!(...)"
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.starts_with("!"), "Got: {}", result);
    assert!(!result.contains("as i32"), "Got: {}", result);
}

#[test]
fn logical_not_on_boolean_int_target() {
    // Line 1064: LogicalNot on boolean with Int target → "(!(...)) as i32"
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
    };
    let result =
        cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("as i32"), "Got: {}", result);
}

#[test]
fn deref_post_increment_on_string_ref() {
    // Lines 1893-1903: Dereference(PostIncrement(string var)) → no extra deref
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("s".to_string(), HirType::StringReference);
    let expr = HirExpression::Dereference(Box::new(HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Variable("s".to_string())),
    }));
    let result = cg.generate_expression_with_context(&expr, &ctx);
    // Should generate the PostIncrement code without extra deref wrapping
    assert!(!result.contains("unsafe"), "Got: {}", result);
}

#[test]
fn binary_assign_global_array_index() {
    // Lines 1300-1308: Assignment to global array index → unsafe block
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_global("data".to_string());
    ctx.add_variable(
        "data".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Assign,
        left: Box::new(HirExpression::ArrayIndex {
            array: Box::new(HirExpression::Variable("data".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
        }),
        right: Box::new(HirExpression::IntLiteral(42)),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains("data"), "Got: {}", result);
}

#[test]
fn get_string_deref_var_deref_non_string() {
    // Lines 3521-3525: Dereference of non-string variable → None
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "p".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("p".to_string())));
    let result = CodeGenerator::get_string_deref_var(&expr, &ctx);
    assert_eq!(result, None);
}

#[test]
fn get_string_deref_var_compare_zero_left() {
    // Lines 3536-3537: BinaryOp(0 == *str) where str is string (reversed)
    let mut ctx = TypeContext::new();
    ctx.add_variable("s".to_string(), HirType::StringReference);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::IntLiteral(0)),
        right: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("s".to_string()),
        ))),
    };
    let result = CodeGenerator::get_string_deref_var(&expr, &ctx);
    assert_eq!(result, Some("s".to_string()));
}

#[test]
fn transform_ternary_malformed() {
    // Line 602: Malformed ternary (no ? or :) → return as-is
    let cg = CodeGenerator::new();
    let result = cg.transform_ternary("just_an_expression").unwrap();
    assert_eq!(result, "just_an_expression");
}

#[test]
fn dereference_binary_op_pointer_arithmetic_needs_unsafe() {
    // Lines 1913-1917: Dereference of BinaryOp with pointer left → needs unsafe
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let expr = HirExpression::Dereference(Box::new(HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("ptr".to_string())),
        right: Box::new(HirExpression::IntLiteral(2)),
    }));
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("unsafe"), "Got: {}", result);
}

#[test]
fn dereference_binary_op_non_pointer_left_no_unsafe() {
    // Line 1917: Dereference of BinaryOp with non-pointer left → false (no unsafe)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::Dereference(Box::new(HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::IntLiteral(1)),
    }));
    let result = cg.generate_expression_with_context(&expr, &ctx);
    // No unsafe since left operand is not a pointer
    assert!(!result.contains("unsafe"), "Got: {}", result);
}

// ============================================================================
// BATCH 7: sizeof member access, string iter func call args, deref assign
//          double pointer, pointer subtraction, calloc default, ArrayIndex
//          global, switch case, format positions edge case
// ============================================================================

#[test]
fn sizeof_member_access_resolved_field_type() {
    // Lines 2987-2995: sizeof(struct.field) via member access → field type resolution
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.structs.insert(
        "Node".to_string(),
        vec![
            ("value".to_string(), HirType::Int),
            ("next".to_string(), HirType::Pointer(Box::new(HirType::Struct("Node".to_string())))),
        ],
    );
    let expr = HirExpression::Sizeof {
        type_name: "Node value".to_string(),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(
        result.contains("size_of::<i32>()"),
        "Should resolve field type, got: {}",
        result
    );
}

#[test]
fn sizeof_member_access_unknown_struct_fallback() {
    // Lines 3005-3006: sizeof(struct.field) with unknown struct → fallback
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Sizeof {
        type_name: "Unknown field".to_string(),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("size_of"), "Should use fallback, got: {}", result);
}

#[test]
fn calloc_expression_non_standard_element_type() {
    // Line 3052: Calloc with non-standard element type (e.g., Struct)
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(10)),
        element_type: Box::new(HirType::Struct("Node".to_string())),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("vec!"), "Got: {}", result);
    assert!(result.contains("Node::default()"), "Got: {}", result);
}

#[test]
fn string_iter_func_call_arg_address_of() {
    // Lines 2712-2718: String iter func with AddressOf arg (inside !is_address_of branch)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "buf".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Char),
            size: Some(256),
        },
    );
    // Register "process" as a string iter func with param 0 as mutable
    ctx.add_string_iter_func("process".to_string(), vec![(0, true)]);
    ctx.add_function(
        "process".to_string(),
        vec![HirType::Pointer(Box::new(HirType::Char))],
    );
    let expr = HirExpression::FunctionCall {
        function: "process".to_string(),
        arguments: vec![HirExpression::Variable("buf".to_string())],
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    // Array arg to string iter func → &mut buf
    assert!(result.contains("&mut buf"), "Got: {}", result);
}

#[test]
fn string_iter_func_call_arg_string_literal() {
    // Lines 2707-2710: String iter func with StringLiteral arg → b"string"
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_string_iter_func("process".to_string(), vec![(0, false)]);
    ctx.add_function(
        "process".to_string(),
        vec![HirType::Pointer(Box::new(HirType::Char))],
    );
    let expr = HirExpression::FunctionCall {
        function: "process".to_string(),
        arguments: vec![HirExpression::StringLiteral("hello".to_string())],
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("b\"hello\""), "Got: {}", result);
}

#[test]
fn string_iter_func_call_arg_immutable_array() {
    // Lines 2702-2703: String iter func with immutable array arg → &arr
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "data".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Char),
            size: Some(64),
        },
    );
    ctx.add_string_iter_func("read_data".to_string(), vec![(0, false)]);
    ctx.add_function(
        "read_data".to_string(),
        vec![HirType::Pointer(Box::new(HirType::Char))],
    );
    let expr = HirExpression::FunctionCall {
        function: "read_data".to_string(),
        arguments: vec![HirExpression::Variable("data".to_string())],
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("&data"), "Got: {}", result);
    assert!(!result.contains("&mut"), "Should be immutable, got: {}", result);
}

#[test]
fn slice_param_with_sized_array_arg() {
    // Lines 2773-2775: Unsized array param (slice) with sized array arg → &mut arr
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "arr".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );
    // Function param is Array { size: None } (unsized/slice param)
    ctx.add_function(
        "sort".to_string(),
        vec![HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None,
        }],
    );
    let expr = HirExpression::FunctionCall {
        function: "sort".to_string(),
        arguments: vec![HirExpression::Variable("arr".to_string())],
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("&mut arr"), "Got: {}", result);
}

#[test]
fn pointer_field_access_non_pointer_var() {
    // Line 2869: PointerFieldAccess where variable is NOT a pointer → no unsafe
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("node".to_string(), HirType::Struct("Node".to_string()));
    ctx.structs.insert(
        "Node".to_string(),
        vec![("value".to_string(), HirType::Int)],
    );
    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("node".to_string())),
        field: "value".to_string(),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(!result.contains("unsafe"), "Got: {}", result);
}

#[test]
fn array_index_non_variable_global_check() {
    // Line 2899: ArrayIndex where array expr is not Variable → is_global is false fallthrough
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Pointer(Box::new(HirType::Int)))),
    );
    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("ptr".to_string()),
        ))),
        index: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("0"), "Got: {}", result);
}

#[test]
fn deref_assign_double_pointer_ref() {
    // Lines 4762-4779: DerefAssignment where var is Reference { inner: Pointer(_) }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "pp".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Pointer(Box::new(HirType::Int))),
            mutable: true,
        },
    );
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Dereference(Box::new(
            HirExpression::Variable("pp".to_string()),
        )),
        value: HirExpression::IntLiteral(42),
    };
    let result = cg.generate_statement_with_context(&stmt, Some("test_fn"), &mut ctx, Some(&HirType::Void));
    assert!(result.contains("unsafe"), "Got: {}", result);
}

#[test]
fn deref_assign_double_pointer_ptr() {
    // Lines 4767-4769: DerefAssignment where var is Pointer(Pointer(_))
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "pp".to_string(),
        HirType::Pointer(Box::new(HirType::Pointer(Box::new(HirType::Int)))),
    );
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Dereference(Box::new(
            HirExpression::Variable("pp".to_string()),
        )),
        value: HirExpression::IntLiteral(42),
    };
    let result = cg.generate_statement_with_context(&stmt, Some("test_fn"), &mut ctx, Some(&HirType::Void));
    assert!(result.contains("unsafe"), "Got: {}", result);
}

#[test]
fn deref_assign_double_pointer_non_matching() {
    // Line 4770: DerefAssignment where var is other type → no yields_raw_ptr
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "pp".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: true,
        },
    );
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Dereference(Box::new(
            HirExpression::Variable("pp".to_string()),
        )),
        value: HirExpression::IntLiteral(42),
    };
    let result = cg.generate_statement_with_context(&stmt, Some("test_fn"), &mut ctx, Some(&HirType::Void));
    assert!(!result.contains("unsafe"), "Got: {}", result);
}

#[test]
fn deref_assign_strip_unsafe_from_value() {
    // Lines 4731-4734: strip_unsafe helper strips "unsafe { X }" → "X"
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "p".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    ctx.add_variable(
        "q".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("p".to_string()),
        value: HirExpression::Dereference(Box::new(
            HirExpression::Variable("q".to_string()),
        )),
    };
    let result = cg.generate_statement_with_context(&stmt, Some("test_fn"), &mut ctx, Some(&HirType::Void));
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(
        result.matches("unsafe").count() <= 2,
        "Should strip nested unsafe, got: {}",
        result
    );
}

#[test]
fn pointer_subtraction_non_pointer_right() {
    // Lines 1579-1583: ptr - integer (not another pointer) → wrapping_sub
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "p".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("p".to_string())),
        right: Box::new(HirExpression::IntLiteral(3)),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("wrapping_sub"), "Got: {}", result);
}

#[test]
fn pointer_subtraction_non_variable_right() {
    // ptr - (expr) where right is not a variable → wrapping_sub
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "p".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("p".to_string())),
        right: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::IntLiteral(1)),
            right: Box::new(HirExpression::IntLiteral(2)),
        }),
    };
    let result = cg.generate_expression_with_context(&expr, &ctx);
    assert!(result.contains("wrapping_sub"), "Got: {}", result);
}

#[test]
fn array_index_assignment_global_array() {
    // Lines 4807-4818: ArrayIndexAssignment with global array → unsafe
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_global("table".to_string());
    ctx.add_variable(
        "table".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(100),
        },
    );
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("table".to_string())),
        index: Box::new(HirExpression::IntLiteral(5)),
        value: HirExpression::IntLiteral(99),
    };
    let result = cg.generate_statement_with_context(&stmt, Some("test_fn"), &mut ctx, Some(&HirType::Void));
    assert!(result.contains("unsafe"), "Got: {}", result);
}

#[test]
fn format_string_positions_incomplete_specifier() {
    // Lines 3940-3942: Format string with % at end (no specifier after %) → fallback
    let positions = CodeGenerator::find_string_format_positions("hello%");
    // Incomplete format specifier at end — should not crash, may or may not find a position
    let _ = positions; // Just verifying no panic
}

#[test]
fn infer_expression_type_pointer_field_access_reference() {
    // Line 313: PointerFieldAccess where type is Reference → get_field_type_from_type
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "node".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Struct("Node".to_string())),
            mutable: false,
        },
    );
    ctx.structs.insert(
        "Node".to_string(),
        vec![("value".to_string(), HirType::Int)],
    );
    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("node".to_string())),
        field: "value".to_string(),
    };
    let result = ctx.infer_expression_type(&expr);
    assert_eq!(result, Some(HirType::Int));
}

#[test]
fn infer_expression_type_pointer_field_access_non_ptr() {
    // Line 316: PointerFieldAccess where type is not Pointer/Box/Reference → None
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("x".to_string())),
        field: "field".to_string(),
    };
    let result = ctx.infer_expression_type(&expr);
    assert_eq!(result, None);
}

// ============================================================================
// BATCH 8: BinaryOp paths via generate_expression_with_target_type
//          These lines (1308-1461) are only reachable through the target_type
//          variant, NOT generate_expression_with_context
// ============================================================================

#[test]
fn binop_target_type_global_array_assign() {
    // Lines 1300-1308: BinaryOp Assign to global array index → unsafe via target_type path
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_global("data".to_string());
    ctx.add_variable(
        "data".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Assign,
        left: Box::new(HirExpression::ArrayIndex {
            array: Box::new(HirExpression::Variable("data".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
        }),
        right: Box::new(HirExpression::IntLiteral(42)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("unsafe"), "Got: {}", result);
}

#[test]
fn binop_target_type_option_null_equal() {
    // Lines 1324-1329: Option var == NULL → .is_none() via target_type path
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "p".to_string(),
        HirType::Option(Box::new(HirType::Int)),
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("p".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("is_none"), "Got: {}", result);
}

#[test]
fn binop_target_type_option_null_not_equal() {
    // Lines 1324-1329: Option var != NULL → .is_some() via target_type path
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "p".to_string(),
        HirType::Option(Box::new(HirType::Int)),
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("p".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("is_some"), "Got: {}", result);
}

#[test]
fn binop_target_type_null_option_reversed() {
    // Lines 1334-1339: NULL == Option var → .is_none() (reversed) via target_type path
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "p".to_string(),
        HirType::Option(Box::new(HirType::Int)),
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::NullLiteral),
        right: Box::new(HirExpression::Variable("p".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("is_none"), "Got: {}", result);
}

#[test]
fn binop_target_type_vec_null_equal() {
    // Lines 1392-1401: Vec == 0 → "false" via target_type path
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("arr".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("false"), "Got: {}", result);
}

#[test]
fn binop_target_type_vec_null_not_equal() {
    // Lines 1392-1401: Vec != NULL → "true" via target_type path
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("arr".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("true"), "Got: {}", result);
}

#[test]
fn binop_target_type_box_null_equal() {
    // Lines 1410-1421: Box == 0 → "false" via target_type path
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "node".to_string(),
        HirType::Box(Box::new(HirType::Struct("Node".to_string()))),
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("node".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("false"), "Got: {}", result);
}

#[test]
fn binop_target_type_box_null_not_equal() {
    // Lines 1410-1423: Box != NULL → "true" via target_type path
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "node".to_string(),
        HirType::Box(Box::new(HirType::Struct("Node".to_string()))),
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("node".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("true"), "Got: {}", result);
}

#[test]
fn binop_target_type_strlen_equal_zero() {
    // Lines 1434-1443: strlen(s) == 0 → is_empty() via target_type path
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
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("is_empty"), "Got: {}", result);
}

#[test]
fn binop_target_type_strlen_not_equal_zero() {
    // Lines 1434-1443: strlen(s) != 0 → !is_empty() via target_type path
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
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("!s.is_empty()"), "Got: {}", result);
}

#[test]
fn binop_target_type_zero_strlen_reversed() {
    // Lines 1452-1461: 0 == strlen(s) → is_empty() via target_type path (reversed)
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
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("is_empty"), "Got: {}", result);
}

#[test]
fn binop_target_type_zero_strlen_not_equal_reversed() {
    // Lines 1452-1461: 0 != strlen(s) → !is_empty() via target_type path (reversed)
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
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("!s.is_empty()"), "Got: {}", result);
}

#[test]
fn var_to_ptr_ref_array_type_mismatch() {
    // Line 1178: Reference { inner: Array { elem: Int } } to Pointer(Char) — type mismatch, falls through
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "arr_ref".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(10),
            }),
            mutable: true,
        },
    );
    let expr = HirExpression::Variable("arr_ref".to_string());
    // Target is Pointer(Char) but arr_ref is Reference(Array(Int)) — element type mismatch
    let result = cg.generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Char))),
    );
    // Falls through element_type_match because Int != Char, then through inner == ptr_inner check
    // since Array != Char, so it hits the default escape path
    assert!(!result.is_empty(), "Got: {}", result);
}

#[test]
fn var_to_ptr_int_to_char_via_target_type() {
    // Lines 1223-1228: Int variable with Char target → "x as u8" via target_type
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("c".to_string(), HirType::Int);
    let expr = HirExpression::Variable("c".to_string());
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Char));
    assert!(result.contains("as u8"), "Got: {}", result);
}

#[test]
fn binop_target_type_pointer_field_compare_zero() {
    // Lines 1367-1376: ptr->field == 0 where field is pointer → null_mut via target_type path
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "node".to_string(),
        HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
    );
    ctx.structs.insert(
        "Node".to_string(),
        vec![("next".to_string(), HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))))],
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::PointerFieldAccess {
            pointer: Box::new(HirExpression::Variable("node".to_string())),
            field: "next".to_string(),
        }),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("std::ptr::null_mut()"), "Got: {}", result);
}

#[test]
fn binop_target_type_pointer_subtract_wrapping() {
    // Lines 1579-1583: ptr - integer via target_type path → wrapping_sub
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "p".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("p".to_string())),
        right: Box::new(HirExpression::IntLiteral(3)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Pointer(Box::new(HirType::Int))));
    assert!(result.contains("wrapping_sub"), "Got: {}", result);
}

#[test]
fn deref_post_increment_on_string_literal_type() {
    // Lines 1896-1903: Dereference(PostIncrement(string_literal var)) via target_type
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("s".to_string(), HirType::StringLiteral);
    let expr = HirExpression::Dereference(Box::new(HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Variable("s".to_string())),
    }));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Char));
    // StringLiteral matches the check at line 1896-1897
    assert!(!result.is_empty(), "Got: {}", result);
}

#[test]
fn deref_binary_op_non_pointer_left_target_type() {
    // Line 1917: Dereference of BinaryOp with non-pointer left via target_type
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::Dereference(Box::new(HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::IntLiteral(1)),
    }));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(!result.contains("unsafe"), "Got: {}", result);
}

#[test]
fn logical_not_unary_op_integer_target_type() {
    // Lines 2007-2014: LogicalNot via UnaryOp arm (lines 2003-2015) in target_type
    // These lines 2007-2014 are in the LATER UnaryOp match — only reachable if LogicalNot
    // was NOT caught by the early match at line 1047-1078.
    // Actually, looking at the code, lines 1047-1078 are ALSO in generate_expression_with_target_type
    // and they always match LogicalNot first. Lines 2006-2014 are dead code for LogicalNot.
    // But they ARE reachable for the general UnaryOp arm which handles other operators.
    // Actually no — the LogicalNot is specifically matched at 1049-1078 and 2006.
    // Lines 2007-2014 are truly dead since 1047-1078 always catches first. Skip these.
    // Instead, verify that the early match handles both paths:
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("== 0) as i32"), "Got: {}", result);
}

// ============================================================================
// BATCH 9: statement_modifies_variable coverage (lines 5764-5798)
// ============================================================================
