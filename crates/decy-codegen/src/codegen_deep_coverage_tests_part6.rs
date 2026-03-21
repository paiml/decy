
#[test]
fn pointer_arithmetic_other_field_access() {
    let cg = CodeGenerator::new();
    // ptr = other->data is pointer arithmetic (reassignment from field access)
    let stmt = HirStatement::Assignment {
        target: "ptr".to_string(),
        value: HirExpression::PointerFieldAccess {
            pointer: Box::new(HirExpression::Variable("other".to_string())),
            field: "data".to_string(),
        },
    };
    assert!(
        cg.statement_uses_pointer_arithmetic(&stmt, "ptr"),
        "ptr = other->data should be detected as pointer arithmetic"
    );
}

#[test]
fn pointer_arithmetic_post_increment_in_expression() {
    let cg = CodeGenerator::new();
    // str++ as expression statement
    let stmt = HirStatement::Expression(HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Variable("str".to_string())),
    });
    assert!(
        cg.statement_uses_pointer_arithmetic(&stmt, "str"),
        "str++ should be detected as pointer arithmetic"
    );
}

#[test]
fn pointer_arithmetic_pre_decrement_in_expression() {
    let cg = CodeGenerator::new();
    // --ptr as expression statement
    let stmt = HirStatement::Expression(HirExpression::PreDecrement {
        operand: Box::new(HirExpression::Variable("ptr".to_string())),
    });
    assert!(
        cg.statement_uses_pointer_arithmetic(&stmt, "ptr"),
        "--ptr should be detected as pointer arithmetic"
    );
}

// ============================================================================
// statement_modifies_variable through various types (lines 5770-5795)
// ============================================================================

#[test]
fn modifies_variable_array_index() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
        value: HirExpression::IntLiteral(42),
    };
    assert!(
        cg.statement_modifies_variable(&stmt, "arr"),
        "arr[0] = 42 should detect arr as modified"
    );
}

#[test]
fn modifies_variable_deref_assignment() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("ptr".to_string()),
        value: HirExpression::IntLiteral(42),
    };
    assert!(
        cg.statement_modifies_variable(&stmt, "ptr"),
        "*ptr = 42 should detect ptr as modified"
    );
}

#[test]
fn modifies_variable_in_if_then_block() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::Variable("cond".to_string()),
        then_block: vec![HirStatement::ArrayIndexAssignment {
            array: Box::new(HirExpression::Variable("arr".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
            value: HirExpression::IntLiteral(1),
        }],
        else_block: None,
    };
    assert!(
        cg.statement_modifies_variable(&stmt, "arr"),
        "arr modified in if-then should be detected"
    );
}

#[test]
fn modifies_variable_in_else_block() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::Variable("cond".to_string()),
        then_block: vec![],
        else_block: Some(vec![HirStatement::ArrayIndexAssignment {
            array: Box::new(HirExpression::Variable("arr".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
            value: HirExpression::IntLiteral(1),
        }]),
    };
    assert!(
        cg.statement_modifies_variable(&stmt, "arr"),
        "arr modified in else should be detected"
    );
}

#[test]
fn modifies_variable_in_while_loop() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::While {
        condition: HirExpression::Variable("cond".to_string()),
        body: vec![HirStatement::ArrayIndexAssignment {
            array: Box::new(HirExpression::Variable("arr".to_string())),
            index: Box::new(HirExpression::Variable("i".to_string())),
            value: HirExpression::IntLiteral(0),
        }],
    };
    assert!(
        cg.statement_modifies_variable(&stmt, "arr"),
        "arr modified in while should be detected"
    );
}

// ============================================================================
// pointer_to_slice_type (line 5809, 5813)
// ============================================================================

#[test]
fn pointer_to_slice_immutable() {
    let cg = CodeGenerator::new();
    let result = cg.pointer_to_slice_type(&HirType::Pointer(Box::new(HirType::Int)), false);
    assert_eq!(result, "&[i32]", "Immutable pointer should become &[i32]");
}

#[test]
fn pointer_to_slice_mutable() {
    let cg = CodeGenerator::new();
    let result = cg.pointer_to_slice_type(&HirType::Pointer(Box::new(HirType::Char)), true);
    assert_eq!(result, "&mut [u8]", "Mutable pointer to char should become &mut [u8]");
}

// ============================================================================
// expression_uses_pointer_subtraction (lines 5739-5744)
// var - other and other - var patterns
// ============================================================================

#[test]
fn pointer_subtraction_left_operand() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("end".to_string())),
        right: Box::new(HirExpression::Variable("start".to_string())),
    };
    assert!(
        cg.expression_uses_pointer_subtraction(&expr, "end"),
        "end - start should detect end as pointer subtraction"
    );
}

#[test]
fn pointer_subtraction_right_operand() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("end".to_string())),
        right: Box::new(HirExpression::Variable("start".to_string())),
    };
    assert!(
        cg.expression_uses_pointer_subtraction(&expr, "start"),
        "end - start should detect start as pointer subtraction"
    );
}

// ============================================================================
// Batch 3: Function call argument transformations (lines 2667-2811)
// ============================================================================

#[test]
fn call_arg_string_iter_param_array_mutable() {
    // Lines 2697-2704: String iter func with mutable array arg
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "buf".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Char),
            size: Some(256),
        },
    );
    ctx.add_string_iter_func("process".to_string(), vec![(0, true)]);
    let expr = HirExpression::FunctionCall {
        function: "process".to_string(),
        arguments: vec![HirExpression::Variable("buf".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("&mut buf"),
        "Mutable string iter array arg should be &mut buf, got: {}",
        code
    );
}

#[test]
fn call_arg_string_iter_param_array_immutable() {
    // Lines 2697-2704: String iter func with immutable array arg
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "buf".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Char),
            size: Some(256),
        },
    );
    ctx.add_string_iter_func("scan".to_string(), vec![(0, false)]);
    let expr = HirExpression::FunctionCall {
        function: "scan".to_string(),
        arguments: vec![HirExpression::Variable("buf".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("&buf"),
        "Immutable string iter array arg should be &buf, got: {}",
        code
    );
}

#[test]
fn call_arg_string_iter_param_string_literal() {
    // Lines 2707-2710: String iter func with string literal arg
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_string_iter_func("process".to_string(), vec![(0, true)]);
    let expr = HirExpression::FunctionCall {
        function: "process".to_string(),
        arguments: vec![HirExpression::StringLiteral("hello".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("b\"hello\""),
        "String literal for string iter should become byte string, got: {}",
        code
    );
}

#[test]
fn call_arg_string_iter_param_address_of_mutable() {
    // Lines 2712-2718: String iter func with AddressOf arg, mutable
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_string_iter_func("process".to_string(), vec![(0, true)]);
    let expr = HirExpression::FunctionCall {
        function: "process".to_string(),
        arguments: vec![HirExpression::AddressOf(Box::new(
            HirExpression::Variable("buffer".to_string()),
        ))],
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("&mut buffer"),
        "Mutable string iter AddressOf should be &mut, got: {}",
        code
    );
}

#[test]
fn call_arg_ref_param_pointer_var() {
    // Lines 2749-2760: Reference param with pointer variable arg → unsafe { &mut *ptr }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    ctx.add_function(
        "process".to_string(),
        vec![HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: true,
        }],
    );
    let expr = HirExpression::FunctionCall {
        function: "process".to_string(),
        arguments: vec![HirExpression::Variable("ptr".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("unsafe") && code.contains("&mut *ptr"),
        "Pointer var for ref param should use unsafe deref, got: {}",
        code
    );
}

#[test]
fn call_arg_raw_pointer_param_string_literal() {
    // Lines 2740-2741: Raw pointer param with string literal arg
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_function(
        "write_bytes".to_string(),
        vec![HirType::Pointer(Box::new(HirType::Char))],
    );
    let expr = HirExpression::FunctionCall {
        function: "write_bytes".to_string(),
        arguments: vec![HirExpression::StringLiteral("data".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains(".as_ptr() as *mut u8"),
        "String literal for raw pointer param should use .as_ptr(), got: {}",
        code
    );
}

#[test]
fn call_arg_slice_param_sized_array() {
    // Lines 2769-2776: Slice param with sized array variable
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "data".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Char),
            size: Some(10),
        },
    );
    ctx.add_function(
        "process".to_string(),
        vec![HirType::Array {
            element_type: Box::new(HirType::Char),
            size: None,
        }],
    );
    let expr = HirExpression::FunctionCall {
        function: "process".to_string(),
        arguments: vec![HirExpression::Variable("data".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("&mut data"),
        "Sized array for unsized slice param should get &mut, got: {}",
        code
    );
}

#[test]
fn call_arg_int_param_char_literal() {
    // Lines 2784-2787: Int param with CharLiteral arg
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_function("putchar".to_string(), vec![HirType::Int]);
    let expr = HirExpression::FunctionCall {
        function: "putchar".to_string(),
        arguments: vec![HirExpression::CharLiteral(b' ' as i8)],
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("32i32"),
        "CharLiteral ' ' for Int param should be 32i32, got: {}",
        code
    );
}

#[test]
fn call_arg_string_func_pointer_field_access() {
    // Lines 2804-2811: String func with PointerFieldAccess arg
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "strcmp".to_string(),
        arguments: vec![
            HirExpression::PointerFieldAccess {
                pointer: Box::new(HirExpression::Variable("entry".to_string())),
                field: "key".to_string(),
            },
            HirExpression::StringLiteral("target".to_string()),
        ],
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("CStr::from_ptr"),
        "PointerFieldAccess for strcmp should use CStr::from_ptr, got: {}",
        code
    );
}

// ============================================================================
// Batch 3: Variable declaration with malloc edge cases (lines 4142-4254)
// ============================================================================

#[test]
fn stmt_char_pointer_array_with_size() {
    // Lines 4137-4154: char *arr[2] = {"hello", "world"} → let arr: [&str; 2] = ["hello", "world"]
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "msgs".to_string(),
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
                HirExpression::StringLiteral("hello".to_string()),
                HirExpression::StringLiteral("world".to_string()),
            ],
        }),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("[&str; 2]"),
        "Char pointer array with size should be [&str; 2], got: {}",
        code
    );
}

#[test]
fn stmt_malloc_other_type_fallback() {
    // Lines 4199-4202: Malloc with non-Box/non-Vec type falls back to Box::new(0i32)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "x".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::Malloc {
            size: Box::new(HirExpression::IntLiteral(4)),
        }),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("Box::new(0i32)"),
        "Malloc with Int type should fallback to Box::new(0i32), got: {}",
        code
    );
}

#[test]
fn stmt_is_malloc_init_other_type_fallback() {
    // Lines 4244-4254: is_malloc_init with non-Box/non-Vec _actual_type
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // Create a variable with Pointer type and a FunctionCall to "malloc"
    // This triggers the is_malloc_init path (not the Malloc expression path)
    let stmt = HirStatement::VariableDeclaration {
        name: "data".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::IntLiteral(100)],
        }),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    // The Pointer type becomes _actual_type which falls to the _ arm
    assert!(
        !code.is_empty(),
        "malloc init with Pointer type should produce code"
    );
}

// ============================================================================
// Batch 3: Null comparison in while/for (lines 5495-5509)
// ============================================================================

#[test]
fn null_comparison_in_while_condition() {
    // Lines 5494-5498: null comparison detected in while condition
    let cg = CodeGenerator::new();
    let stmt = HirStatement::While {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::NotEqual,
            left: Box::new(HirExpression::Variable("ptr".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
        body: vec![],
    };
    assert!(
        cg.statement_uses_null_comparison(&stmt, "ptr"),
        "While with ptr != 0 should detect null comparison"
    );
}

#[test]
fn null_comparison_in_for_condition() {
    // Lines 5503-5506: null comparison detected in for condition
    let cg = CodeGenerator::new();
    let stmt = HirStatement::For {
        init: vec![],
        condition: Some(HirExpression::BinaryOp {
            op: BinaryOperator::NotEqual,
            left: Box::new(HirExpression::Variable("node".to_string())),
            right: Box::new(HirExpression::NullLiteral),
        }),
        increment: vec![],
        body: vec![],
    };
    assert!(
        cg.statement_uses_null_comparison(&stmt, "node"),
        "For with node != NULL should detect null comparison"
    );
}

#[test]
fn null_comparison_reversed_in_expression() {
    // Lines 5532-5539: 0 == var pattern
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::NullLiteral),
            right: Box::new(HirExpression::Variable("ptr".to_string())),
        },
        then_block: vec![],
        else_block: None,
    };
    assert!(
        cg.statement_uses_null_comparison(&stmt, "ptr"),
        "NULL == ptr should detect null comparison"
    );
}

// ============================================================================
// Batch 3: Pointer arithmetic detection (lines 5563-5628)
// ============================================================================

#[test]
fn ptr_arithmetic_field_access_reassignment() {
    // Lines 5577-5582: ptr = ptr->next pattern
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Assignment {
        target: "head".to_string(),
        value: HirExpression::PointerFieldAccess {
            pointer: Box::new(HirExpression::Variable("head".to_string())),
            field: "next".to_string(),
        },
    };
    assert!(
        cg.statement_uses_pointer_arithmetic(&stmt, "head"),
        "head = head->next should detect pointer arithmetic"
    );
}

#[test]
fn ptr_arithmetic_any_pointer_field_assign() {
    // Lines 5590-5591: ptr = other->field (any PointerFieldAccess)
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Assignment {
        target: "ptr".to_string(),
        value: HirExpression::PointerFieldAccess {
            pointer: Box::new(HirExpression::Variable("other".to_string())),
            field: "data".to_string(),
        },
    };
    assert!(
        cg.statement_uses_pointer_arithmetic(&stmt, "ptr"),
        "ptr = other->data should detect pointer arithmetic"
    );
}

#[test]
fn ptr_arithmetic_in_expression_stmt_post_increment() {
    // Lines 5610-5611, 5624-5628: ptr++ as expression statement
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Expression(HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Variable("str".to_string())),
    });
    assert!(
        cg.statement_uses_pointer_arithmetic(&stmt, "str"),
        "str++ as expression should detect pointer arithmetic"
    );
}

#[test]
fn ptr_arithmetic_in_expression_stmt_pre_decrement() {
    // Lines 5624-5628: --ptr as expression statement
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Expression(HirExpression::PreDecrement {
        operand: Box::new(HirExpression::Variable("ptr".to_string())),
    });
    assert!(
        cg.statement_uses_pointer_arithmetic(&stmt, "ptr"),
        "--ptr as expression should detect pointer arithmetic"
    );
}

#[test]
fn ptr_arithmetic_in_while_body() {
    // Lines 5613-5615: Pointer arithmetic nested in while body
    let cg = CodeGenerator::new();
    let stmt = HirStatement::While {
        condition: HirExpression::IntLiteral(1),
        body: vec![HirStatement::Assignment {
            target: "ptr".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("ptr".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
    };
    assert!(
        cg.statement_uses_pointer_arithmetic(&stmt, "ptr"),
        "ptr = ptr + 1 in while body should detect pointer arithmetic"
    );
}

#[test]
fn ptr_arithmetic_in_for_body() {
    // Lines 5613-5615: Pointer arithmetic nested in for body
    let cg = CodeGenerator::new();
    let stmt = HirStatement::For {
        init: vec![],
        condition: None,
        increment: vec![],
        body: vec![HirStatement::Expression(HirExpression::PostIncrement {
            operand: Box::new(HirExpression::Variable("p".to_string())),
        })],
    };
    assert!(
        cg.statement_uses_pointer_arithmetic(&stmt, "p"),
        "p++ in for body should detect pointer arithmetic"
    );
}

// ============================================================================
// Batch 3: statement_modifies_variable through control flow (lines 5780-5795)
// ============================================================================

#[test]
fn modifies_var_array_index_in_if() {
    // Lines 5780-5791: Array index modification inside if block
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::IntLiteral(1),
        then_block: vec![HirStatement::ArrayIndexAssignment {
            array: Box::new(HirExpression::Variable("arr".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
            value: HirExpression::IntLiteral(42),
        }],
        else_block: None,
    };
    assert!(
        cg.statement_modifies_variable(&stmt, "arr"),
        "arr[0] = 42 in if block should detect modification"
    );
}

#[test]
fn modifies_var_deref_in_else() {
    // Lines 5788-5791: Deref assignment in else block
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::IntLiteral(1),
        then_block: vec![],
        else_block: Some(vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("ptr".to_string()),
            value: HirExpression::IntLiteral(99),
        }]),
    };
    assert!(
        cg.statement_modifies_variable(&stmt, "ptr"),
        "*ptr = 99 in else block should detect modification"
    );
}

#[test]
fn modifies_var_in_while_body() {
    // Lines 5793-5795: Modification inside while body
    let cg = CodeGenerator::new();
    let stmt = HirStatement::While {
        condition: HirExpression::IntLiteral(1),
        body: vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("ptr".to_string()),
            value: HirExpression::IntLiteral(0),
        }],
    };
    assert!(
        cg.statement_modifies_variable(&stmt, "ptr"),
        "*ptr = 0 in while body should detect modification"
    );
}

#[test]
fn modifies_var_in_for_body() {
    // Lines 5793-5795: Modification inside for body
    let cg = CodeGenerator::new();
    let stmt = HirStatement::For {
        init: vec![],
        condition: None,
        increment: vec![],
        body: vec![HirStatement::ArrayIndexAssignment {
            array: Box::new(HirExpression::Variable("data".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
            value: HirExpression::IntLiteral(1),
        }],
    };
    assert!(
        cg.statement_modifies_variable(&stmt, "data"),
        "data[0] = 1 in for body should detect modification"
    );
}

// ============================================================================
// Batch 3: pointer_to_slice_type non-pointer fallback (lines 5811-5813)
// ============================================================================

#[test]
fn pointer_to_slice_type_non_pointer_fallback() {
    // Lines 5811-5813: Non-pointer type falls back to map_type
    let cg = CodeGenerator::new();
    let result = cg.pointer_to_slice_type(&HirType::Int, false);
    assert_eq!(result, "i32", "Non-pointer should fallback to map_type");
}

// ============================================================================
// Batch 3: generate_function with length param mapping (lines 6374-6382)
// ============================================================================

#[test]
fn generate_function_with_length_param_mapping() {
    // Lines 6370-6382: Array param with length param named "count"
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "process".to_string(),
        HirType::Void,
        vec![
            HirParameter::new(
                "arr".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            ),
            HirParameter::new("count".to_string(), HirType::Int),
        ],
        vec![HirStatement::Return(None)],
    );
    let code = cg.generate_function(&func);
    // The "count" param should be mapped as length param for "arr" if detected as array
    assert!(
        code.contains("fn process"),
        "Should generate function, got: {}",
        code
    );
}

// ============================================================================
// Batch 3: generate_function_with_structs context (lines 6502-6537)
// ============================================================================

#[test]
fn generate_function_with_structs_pointer_param_context() {
    // Lines 6496-6520: Pointer param in generate_function_with_structs
    let cg = CodeGenerator::new();
    let struct_def = HirStruct::new(
        "Node".to_string(),
        vec![HirStructField::new("value".to_string(), HirType::Int)],
    );
    let func = HirFunction::new_with_body(
        "process_node".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "node".to_string(),
            HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
        )],
        vec![HirStatement::Return(None)],
    );
    let code = cg.generate_function_with_structs(
        &func,
        &[struct_def],
    );
    assert!(
        code.contains("fn process_node"),
        "Should generate function with structs, got: {}",
        code
    );
}

#[test]
fn generate_function_with_structs_empty_body() {
    // Lines 6524-6537: Empty body with struct context
    let cg = CodeGenerator::new();
    let func = HirFunction::new(
        "stub".to_string(),
        HirType::Int,
        vec![],
    );
    let code = cg.generate_function_with_structs(
        &func,
        &[],
    );
    // generate_function_with_structs doesn't generate return stub for empty body
    assert!(
        code.contains("fn stub"),
        "Should generate function header, got: {}",
        code
    );
}

// ============================================================================
// Batch 3: generate_function_with_box_transform empty body (lines 6817-6818)
// ============================================================================

#[test]
fn generate_function_with_box_transform_empty_body() {
    // Lines 6813-6819: Empty body with box transform
    let cg = CodeGenerator::new();
    let func = HirFunction::new(
        "alloc".to_string(),
        HirType::Int,
        vec![],
    );
    let code = cg.generate_function_with_box_transform(&func, &[]);
    assert!(
        code.contains("fn alloc") && code.contains("return 0"),
        "Empty body should generate return stub, got: {}",
        code
    );
}

// ============================================================================
// Batch 3: generate_function_with_vec_transform empty body (lines 6861-6865)
// ============================================================================

#[test]
fn generate_function_with_vec_transform_empty_body() {
    // Lines 6859-6865: Empty body with vec transform
    let cg = CodeGenerator::new();
    let func = HirFunction::new(
        "create_vec".to_string(),
        HirType::Int,
        vec![],
    );
    let code = cg.generate_function_with_vec_transform(&func, &[]);
    assert!(
        code.contains("fn create_vec") && code.contains("return 0"),
        "Empty body should generate return stub, got: {}",
        code
    );
}

// ============================================================================
// Batch 3: transform_vec_statement edge cases (lines 6906-6923)
// ============================================================================

#[test]
fn transform_vec_statement_no_capacity() {
    // Lines 6919-6923: VecCandidate with no capacity_expr
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "items".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::Malloc {
            size: Box::new(HirExpression::IntLiteral(40)),
        }),
    };
    let candidate = decy_analyzer::patterns::VecCandidate {
        variable: "items".to_string(),
        malloc_index: 0,
        free_index: None,
        capacity_expr: None,
    };
    let result = cg.transform_vec_statement(&stmt, &candidate);
    if let HirStatement::VariableDeclaration { initializer, .. } = &result {
        assert!(
            initializer.is_some(),
            "Should have Vec::new() initializer"
        );
    } else {
        panic!("Expected VariableDeclaration");
    }
}

#[test]
fn transform_vec_statement_non_pointer_type() {
    // Lines 6905-6906: Non-pointer var_type → return original
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "x".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::IntLiteral(0)),
    };
    let candidate = decy_analyzer::patterns::VecCandidate {
        variable: "x".to_string(),
        malloc_index: 0,
        free_index: None,
        capacity_expr: None,
    };
    let result = cg.transform_vec_statement(&stmt, &candidate);
    // Non-pointer: returns original stmt
    if let HirStatement::VariableDeclaration { var_type, .. } = &result {
        assert!(
            matches!(var_type, HirType::Int),
            "Non-pointer type should return original, got: {:?}",
            var_type
        );
    }
}

// ============================================================================
// Batch 3: generate_function_with_box_and_vec_transform empty body (lines 6964-6967)
// ============================================================================

#[test]
fn generate_function_with_box_and_vec_transform_empty_body() {
    // Lines 6962-6968: Empty body with combined box+vec transform
    let cg = CodeGenerator::new();
    let func = HirFunction::new(
        "combined".to_string(),
        HirType::Float,
        vec![],
    );
    let code = cg.generate_function_with_box_and_vec_transform(&func, &[], &[]);
    assert!(
        code.contains("fn combined") && code.contains("return 0.0"),
        "Empty body should generate return stub, got: {}",
        code
    );
}

// ============================================================================
// Batch 3: is_copy_type Option (line 7054)
// ============================================================================

#[test]
fn struct_with_option_field_no_copy() {
    // Line 7054: Option type is not Copy → struct can't derive Copy
    let cg = CodeGenerator::new();
    let s = HirStruct::new(
        "MaybeVal".to_string(),
        vec![
            HirStructField::new(
                "val".to_string(),
                HirType::Option(Box::new(HirType::Int)),
            ),
        ],
    );
    let code = cg.generate_struct(&s);
    assert!(
        !code.contains("Copy"),
        "Struct with Option field should NOT derive Copy, got: {}",
        code
    );
}

// ============================================================================
// Batch 3: Global variable edge cases (lines 7410-7421)
// ============================================================================

#[test]
fn global_variable_array_non_int_init() {
    // Line 7410: Array init with non-IntLiteral value → generate_expression
    let cg = CodeGenerator::new();
    let var = HirConstant::new(
        "table".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(3),
        },
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
    let code = cg.generate_global_variable(&var, false, false, false);
    assert!(
        code.contains("static mut table"),
        "Should generate static mut for array global, got: {}",
        code
    );
}

#[test]
fn global_variable_unsized_array_fallback() {
    // Lines 7413-7414: Unsized array (size: None) → fallback to value_expr
    let cg = CodeGenerator::new();
    let var = HirConstant::new(
        "data".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None,
        },
        HirExpression::IntLiteral(0),
    );
    let code = cg.generate_global_variable(&var, false, false, false);
    assert!(
        code.contains("static mut data"),
        "Should generate static mut for unsized array, got: {}",
        code
    );
}

#[test]
fn global_variable_pointer_nonzero_init() {
    // Lines 7420-7421: Pointer with non-zero init → fallback to value_expr
    let cg = CodeGenerator::new();
    let var = HirConstant::new(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        HirExpression::IntLiteral(42),
    );
    let code = cg.generate_global_variable(&var, false, false, false);
    assert!(
        code.contains("42"),
        "Pointer with non-zero init should use value_expr, got: {}",
        code
    );
}

// ============================================================================
// Batch 3: generate_return for various types (lines 6287-6317)
// ============================================================================

#[test]
fn generate_return_array_type() {
    // Lines 6287-6297: Return for array type
    let cg = CodeGenerator::new();
    let ret = cg.generate_return(&HirType::Array {
        element_type: Box::new(HirType::Int),
        size: Some(5),
    });
    assert!(
        ret.contains("[0i32; 5]"),
        "Array return should have [0i32; 5], got: {}",
        ret
    );
}

#[test]
fn generate_return_unsized_array() {
    // Lines 6294-6296: Unsized array → empty string
    let cg = CodeGenerator::new();
    let ret = cg.generate_return(&HirType::Array {
        element_type: Box::new(HirType::Int),
        size: None,
    });
    assert!(
        ret.is_empty(),
        "Unsized array return should be empty, got: {}",
        ret
    );
}

#[test]
fn generate_return_function_pointer() {
    // Lines 6299-6302: FunctionPointer → empty string
    let cg = CodeGenerator::new();
    let ret = cg.generate_return(&HirType::FunctionPointer {
        return_type: Box::new(HirType::Void),
        param_types: vec![],
    });
    assert!(
        ret.is_empty(),
        "FunctionPointer return should be empty, got: {}",
        ret
    );
}

#[test]
fn generate_return_string_literal_type() {
    // Line 6304: StringLiteral → return ""
    let cg = CodeGenerator::new();
    let ret = cg.generate_return(&HirType::StringLiteral);
    assert!(
        ret.contains(r#""""#),
        "StringLiteral return should contain empty string, got: {}",
        ret
    );
}

#[test]
fn generate_return_owned_string_type() {
    // Line 6305: OwnedString → String::new()
    let cg = CodeGenerator::new();
    let ret = cg.generate_return(&HirType::OwnedString);
    assert!(
        ret.contains("String::new()"),
        "OwnedString return should have String::new(), got: {}",
        ret
    );
}

#[test]
fn generate_return_union_type() {
    // Lines 6307-6310: Union → empty string
    let cg = CodeGenerator::new();
    let ret = cg.generate_return(&HirType::Union(vec![
        ("field1".to_string(), HirType::Int),
    ]));
    assert!(
        ret.is_empty(),
        "Union return should be empty, got: {}",
        ret
    );
}

#[test]
fn generate_return_type_alias() {
    // Lines 6313-6317: TypeAlias returns
    let cg = CodeGenerator::new();
    let ret = cg.generate_return(&HirType::TypeAlias("size_t".to_string()));
    assert!(
        ret.contains("0usize"),
        "size_t return should be 0usize, got: {}",
        ret
    );
}

// ============================================================================
// Batch 3: Realloc from NULL with multiply (line 4475)
// ============================================================================

#[test]
fn realloc_from_null_with_multiply() {
    // Lines 4461-4475: Realloc from NULL pointer with multiply size
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("items".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let stmt = HirStatement::Assignment {
        target: "items".to_string(),
        value: HirExpression::Realloc {
            pointer: Box::new(HirExpression::NullLiteral),
            new_size: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(HirExpression::IntLiteral(10)),
                right: Box::new(HirExpression::FunctionCall {
                    function: "sizeof".to_string(),
                    arguments: vec![],
                }),
            }),
        },
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("resize"),
        "Realloc from NULL with multiply should generate resize, got: {}",
        code
    );
}

// ============================================================================
// Batch 3: String iter param assignment (lines 4502-4524)
// ============================================================================

#[test]
fn string_iter_param_advance_assignment() {
    // Lines 4502-4522: ptr = ptr + 1 when ptr is string iter param
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("s".to_string(), HirType::Pointer(Box::new(HirType::Char)));
    ctx.add_string_iter_param("s".to_string(), "s_idx".to_string());
    let stmt = HirStatement::Assignment {
        target: "s".to_string(),
        value: HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("s".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        },
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("s_idx += 1"),
        "String iter param advance should use index, got: {}",
        code
    );
}

#[test]
fn string_iter_param_subtract_assignment() {
    // Lines 4513-4514: ptr = ptr - 1 when ptr is string iter param
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("s".to_string(), HirType::Pointer(Box::new(HirType::Char)));
    ctx.add_string_iter_param("s".to_string(), "s_idx".to_string());
    let stmt = HirStatement::Assignment {
        target: "s".to_string(),
        value: HirExpression::BinaryOp {
            op: BinaryOperator::Subtract,
            left: Box::new(HirExpression::Variable("s".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        },
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("s_idx -= 1"),
        "String iter param subtract should use index, got: {}",
        code
    );
}

#[test]
fn string_iter_param_other_op_assignment() {
    // Lines 4516-4520: ptr = ptr * 2 (non add/subtract) when ptr is string iter param
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("s".to_string(), HirType::Pointer(Box::new(HirType::Char)));
    ctx.add_string_iter_param("s".to_string(), "s_idx".to_string());
    let stmt = HirStatement::Assignment {
        target: "s".to_string(),
        value: HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("s".to_string())),
            right: Box::new(HirExpression::IntLiteral(2)),
        },
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    // Falls through to default format
    assert!(
        !code.is_empty(),
        "Other op on string iter param should produce code"
    );
}

// ============================================================================
// Batch 3: Pointer subtraction through control flow (lines 5693-5727)
// ============================================================================

#[test]
fn pointer_subtraction_in_assignment() {
    // Lines 5696-5697: Pointer subtraction in assignment value
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Assignment {
        target: "len".to_string(),
        value: HirExpression::BinaryOp {
            op: BinaryOperator::Subtract,
            left: Box::new(HirExpression::Variable("end".to_string())),
            right: Box::new(HirExpression::Variable("start".to_string())),
        },
    };
    assert!(
        cg.statement_uses_pointer_subtraction(&stmt, "end"),
        "end - start in assignment should detect subtraction"
    );
}

#[test]
fn pointer_subtraction_in_var_decl() {
    // Lines 5699-5702: Pointer subtraction in VariableDeclaration initializer
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "len".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::BinaryOp {
            op: BinaryOperator::Subtract,
            left: Box::new(HirExpression::Variable("p".to_string())),
            right: Box::new(HirExpression::Variable("start".to_string())),
        }),
    };
    assert!(
        cg.statement_uses_pointer_subtraction(&stmt, "p"),
        "p - start in var decl should detect subtraction"
    );
}

#[test]
fn pointer_subtraction_in_if_condition() {
    // Lines 5703-5716: Pointer subtraction in if condition
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::Subtract,
            left: Box::new(HirExpression::Variable("end".to_string())),
            right: Box::new(HirExpression::Variable("begin".to_string())),
        },
        then_block: vec![],
        else_block: None,
    };
    assert!(
        cg.statement_uses_pointer_subtraction(&stmt, "end"),
        "end - begin in if condition should detect subtraction"
    );
}

#[test]
fn pointer_subtraction_in_while_condition() {
    // Lines 5718-5722: Pointer subtraction in while condition
    let cg = CodeGenerator::new();
    let stmt = HirStatement::While {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::Subtract,
            left: Box::new(HirExpression::Variable("cur".to_string())),
            right: Box::new(HirExpression::Variable("base".to_string())),
        },
        body: vec![],
    };
    assert!(
        cg.statement_uses_pointer_subtraction(&stmt, "cur"),
        "cur - base in while condition should detect subtraction"
    );
}

#[test]
fn pointer_subtraction_in_for_body() {
    // Lines 5724-5726: Pointer subtraction in for body
    let cg = CodeGenerator::new();
    let stmt = HirStatement::For {
        init: vec![],
        condition: None,
        increment: vec![],
        body: vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Subtract,
            left: Box::new(HirExpression::Variable("p".to_string())),
            right: Box::new(HirExpression::Variable("s".to_string())),
        }))],
    };
    assert!(
        cg.statement_uses_pointer_subtraction(&stmt, "p"),
        "p - s in for body should detect subtraction"
    );
}

#[test]
fn pointer_subtraction_through_dereference() {
    // Lines 5752-5753: Pointer subtraction through dereference
    let cg = CodeGenerator::new();
    let expr = HirExpression::Dereference(Box::new(HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("ptr".to_string())),
        right: Box::new(HirExpression::IntLiteral(1)),
    }));
    assert!(
        cg.expression_uses_pointer_subtraction(&expr, "ptr"),
        "*(ptr - 1) should detect subtraction through deref"
    );
}

#[test]
fn pointer_subtraction_through_cast() {
    // Lines 5755-5757: Pointer subtraction through cast
    let cg = CodeGenerator::new();
    let expr = HirExpression::Cast {
        expr: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Subtract,
            left: Box::new(HirExpression::Variable("end".to_string())),
            right: Box::new(HirExpression::Variable("start".to_string())),
        }),
        target_type: HirType::Int,
    };
    assert!(
        cg.expression_uses_pointer_subtraction(&expr, "end"),
        "(int)(end - start) should detect subtraction through cast"
    );
}

// ============================================================================
// Batch 4: sizeof struct member (lines 2978-3011)
// ============================================================================

#[test]
fn sizeof_member_access_with_struct_context() {
    // Lines 2987-2995: sizeof(struct->field) with field type in ctx
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let s = HirStruct::new(
        "Point".to_string(),
        vec![
            HirStructField::new("x".to_string(), HirType::Float),
            HirStructField::new("y".to_string(), HirType::Float),
        ],
    );
    ctx.add_struct(&s);
    let expr = HirExpression::Sizeof { type_name: "Point y".to_string() };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("size_of::<f32>()"),
        "sizeof(Point.y) with struct ctx should resolve field type, got: {}",
        code
    );
}

#[test]
fn sizeof_member_access_variable_in_ctx() {
    // Lines 2996-3002: sizeof(var->field) where var is in ctx
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "pt".to_string(),
        HirType::Pointer(Box::new(HirType::Struct("Point".to_string()))),
    );
    let expr = HirExpression::Sizeof { type_name: "pt x".to_string() };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("size_of_val"),
        "sizeof(var->field) with var in ctx should use size_of_val, got: {}",
        code
    );
}

#[test]
fn sizeof_member_access_no_ctx_fallback() {
    // Lines 3004-3006: sizeof(unknown->field) fallback
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Sizeof { type_name: "Unknown field".to_string() };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("size_of"),
        "sizeof with unknown struct should fallback, got: {}",
        code
    );
}

// ============================================================================
// Batch 4: Pre/PostIncrement/Decrement deref non-pointer (lines 3327, 3361, 3390, 3422)
// ============================================================================

#[test]
fn post_increment_deref_non_pointer_variable() {
    // Line 3327: (*x)++ where x is NOT a raw pointer in ctx → fallback
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("ref_val".to_string()),
        ))),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("__tmp"),
        "(*ref_val)++ should use __tmp pattern, got: {}",
        code
    );
}

#[test]
fn pre_increment_deref_non_pointer_variable() {
    // Line 3361: ++(*x) where x is NOT a raw pointer in ctx
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::PreIncrement {
        operand: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("val".to_string()),
        ))),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("+= 1"),
        "++(*val) should have += 1, got: {}",
        code
    );
}

#[test]
fn post_decrement_deref_non_pointer_variable() {
    // Line 3390: (*x)-- where x is NOT a raw pointer in ctx
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::PostDecrement {
        operand: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("cnt".to_string()),
        ))),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("-= 1") && code.contains("__tmp"),
        "(*cnt)-- should use __tmp and -= 1, got: {}",
        code
    );
}

#[test]
fn pre_decrement_deref_non_pointer_variable() {
    // Line 3422: --(*x) where x is NOT a raw pointer in ctx
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::PreDecrement {
        operand: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("cnt".to_string()),
        ))),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("-= 1"),
        "--(*cnt) should have -= 1, got: {}",
        code
    );
}

// ============================================================================
// Batch 4: DerefAssignment paths (lines 4713-4780)
// ============================================================================

#[test]
fn deref_assign_string_iter_param() {
    // Lines 4713-4717: *ptr = val where ptr is string iter param → slice indexing
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("s".to_string(), HirType::Vec(Box::new(HirType::Char)));
    ctx.add_string_iter_param("s".to_string(), "s_idx".to_string());
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("s".to_string()),
        value: HirExpression::IntLiteral(0),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("s[s_idx]"),
        "Deref assign to string iter param should use slice index, got: {}",
        code
    );
}

#[test]
fn deref_assign_raw_pointer_with_unsafe() {
    // Lines 4742-4750: *ptr = val where ptr is raw pointer → unsafe
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("ptr".to_string()),
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("unsafe"),
        "*ptr = 42 with raw pointer should be unsafe, got: {}",
        code
    );
}

#[test]
fn deref_assign_pointer_to_pointer_variable() {
    // Lines 4760-4780: **ptr = val where ptr type is Pointer(Pointer(Int))
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "pp".to_string(),
        HirType::Pointer(Box::new(HirType::Pointer(Box::new(HirType::Int)))),
    );
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Dereference(Box::new(HirExpression::Variable("pp".to_string()))),
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("unsafe"),
        "Pointer-to-pointer deref assignment should be unsafe, got: {}",
        code
    );
}

// ============================================================================
// Batch 4: PointerFieldAccess with raw pointer (lines 2862-2869)
// ============================================================================

#[test]
fn pointer_field_access_with_raw_pointer_ctx() {
    // Lines 2862-2868: ptr->field where ptr is raw pointer → unsafe
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "node".to_string(),
        HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
    );
    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("node".to_string())),
        field: "value".to_string(),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("unsafe"),
        "ptr->field with raw pointer should be unsafe, got: {}",
        code
    );
}

// ============================================================================
// Batch 4: Switch with multiple cases (line 4672)
// ============================================================================

#[test]
fn switch_multiple_cases_generates_match_arms() {
    // Lines 4650-4672: Multiple switch cases
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Switch {
        condition: HirExpression::Variable("x".to_string()),
        cases: vec![
            SwitchCase {
                value: Some(HirExpression::IntLiteral(1)),
                body: vec![HirStatement::Return(Some(HirExpression::StringLiteral(
                    "one".to_string(),
                )))],
            },
            SwitchCase {
                value: Some(HirExpression::IntLiteral(2)),
                body: vec![HirStatement::Return(Some(HirExpression::StringLiteral(
                    "two".to_string(),
                )))],
            },
        ],
        default_case: Some(vec![HirStatement::Return(Some(
            HirExpression::StringLiteral("other".to_string()),
        ))]),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("1 =>") && code.contains("2 =>") && code.contains("_ =>"),
        "Switch should generate multiple match arms, got: {}",
        code
    );
}

// ============================================================================
// Batch 4: VLA element types (lines 4044-4045)
// ============================================================================

#[test]
fn vla_signed_char_element_type() {
    // Line 4044: VLA with SignedChar → 0i8 default
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "buf".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::SignedChar),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("0i8"),
        "VLA with SignedChar should use 0i8 default, got: {}",
        code
    );
}

#[test]
fn vla_struct_element_type_default() {
    // Line 4045: VLA with struct element → default_value_for_type
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "pts".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Struct("Point".to_string())),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("Point::default()"),
        "VLA with struct element should use default, got: {}",
        code
    );
}

// ============================================================================
// Batch 4: Cast malloc with Vec target (line 3154)
// ============================================================================

#[test]
fn cast_malloc_with_vec_target_type() {
    // Lines 3146-3154: Cast wrapping malloc with Vec target type
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Cast {
        target_type: HirType::Pointer(Box::new(HirType::Int)),
        expr: Box::new(HirExpression::Malloc {
            size: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(HirExpression::IntLiteral(10)),
                right: Box::new(HirExpression::IntLiteral(4)),
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
        "Cast(malloc) with Vec target should generate Vec code, got: {}",
        code
    );
}

// ============================================================================
// Batch 4: generate_signature and generate_function with pointer params
// ============================================================================

#[test]
fn generate_signature_string_iter_param() {
    // Lines 5173-5182: Char* param with pointer arithmetic → string iteration
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "count_chars".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "s".to_string(),
            HirType::Pointer(Box::new(HirType::Char)),
        )],
        vec![
            HirStatement::Expression(HirExpression::PostIncrement {
                operand: Box::new(HirExpression::Variable("s".to_string())),
            }),
            HirStatement::Return(Some(HirExpression::IntLiteral(0))),
        ],
    );
    let code = cg.generate_signature(&func);
    assert!(
        code.contains("fn count_chars"),
        "Should generate signature, got: {}",
        code
    );
}

#[test]
fn generate_function_pointer_param_context() {
    // Lines 6396-6420: generate_function with pointer param gets correct context
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "set_value".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("ptr".to_string()),
                value: HirExpression::IntLiteral(42),
            },
        ],
    );
    let code = cg.generate_function(&func);
    assert!(
        code.contains("fn set_value"),
        "Should generate function, got: {}",
        code
    );
}

#[test]
fn generate_function_with_structs_single_pointer_param() {
    // Lines 6510-6519: Non-array single pointer → Reference context
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "init_node".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "node".to_string(),
            HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
        )],
        vec![HirStatement::Return(None)],
    );
    let code = cg.generate_function_with_structs(&func, &[]);
    assert!(
        code.contains("fn init_node"),
        "Should generate function with structs, got: {}",
        code
    );
}

// ============================================================================
// Batch 5: generate_function_with_lifetimes (lines 6595-6690)
// ============================================================================

#[test]
fn generate_function_with_lifetimes_simple() {
    // Lines 6595-6600: Simple function with lifetimes
    use decy_ownership::lifetime_gen::{
        AnnotatedParameter, AnnotatedSignature, AnnotatedType, LifetimeParam,
    };
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "get_ref".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new(
            "data".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::Variable(
            "data".to_string(),
        )))],
    );
    let sig = AnnotatedSignature {
        name: "get_ref".to_string(),
        lifetimes: vec![LifetimeParam::new("'a".to_string())],
        parameters: vec![AnnotatedParameter {
            name: "data".to_string(),
            param_type: AnnotatedType::Reference {
                inner: Box::new(AnnotatedType::Simple(HirType::Int)),
                mutable: true,
                lifetime: Some(LifetimeParam::new("'a".to_string())),
            },
        }],
        return_type: AnnotatedType::Reference {
            inner: Box::new(AnnotatedType::Simple(HirType::Int)),
            mutable: true,
            lifetime: Some(LifetimeParam::new("'a".to_string())),
        },
    };
    let code = cg.generate_function_with_lifetimes(&func, &sig);
    assert!(
        code.contains("fn get_ref"),
        "Should generate function with lifetimes, got: {}",
        code
    );
}

#[test]
fn generate_function_with_lifetimes_and_structs_pointer_param() {
    // Lines 6617-6690: Full function with lifetimes, structs, and pointer params
    use decy_ownership::lifetime_gen::{
        AnnotatedParameter, AnnotatedSignature, AnnotatedType,
    };
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "process".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "arr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(None)],
    );
    let sig = AnnotatedSignature {
        name: "process".to_string(),
        lifetimes: vec![],
        parameters: vec![AnnotatedParameter {
            name: "arr".to_string(),
            param_type: AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Int))),
        }],
        return_type: AnnotatedType::Simple(HirType::Void),
    };
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
        code.contains("fn process"),
        "Should generate function with lifetimes and structs, got: {}",
        code
    );
}

#[test]
fn generate_function_with_lifetimes_and_structs_array_param() {
    // Lines 6675-6690: Pointer param detected as array → slice context
    use decy_ownership::lifetime_gen::{
        AnnotatedParameter, AnnotatedSignature, AnnotatedType,
    };
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "sum_arr".to_string(),
        HirType::Int,
        vec![
            HirParameter::new(
                "arr".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            ),
            HirParameter::new("n".to_string(), HirType::Int),
        ],
        vec![
            // Access arr[0] to trigger array detection
            HirStatement::Return(Some(HirExpression::ArrayIndex {
                array: Box::new(HirExpression::Variable("arr".to_string())),
                index: Box::new(HirExpression::IntLiteral(0)),
            })),
        ],
    );
    let sig = AnnotatedSignature {
        name: "sum_arr".to_string(),
        lifetimes: vec![],
        parameters: vec![
            AnnotatedParameter {
                name: "arr".to_string(),
                param_type: AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Int))),
            },
            AnnotatedParameter {
                name: "n".to_string(),
                param_type: AnnotatedType::Simple(HirType::Int),
            },
        ],
        return_type: AnnotatedType::Simple(HirType::Int),
    };
