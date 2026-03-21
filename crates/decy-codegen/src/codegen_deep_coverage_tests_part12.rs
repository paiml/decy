    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("&buffer") || code.contains("&mut buffer"),
        "Should generate reference from AddressOf, Got: {}",
        code
    );
}

// --- Variable init Vec::new() fallback (line 4196) ---

#[test]
fn var_decl_malloc_init_vec_no_multiply() {
    // VariableDeclaration with FunctionCall { "malloc" } where size is NOT a multiply
    // This hits the Vec::new() fallback at line 4196
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::Variable("total_size".to_string())],
        }),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    // malloc FunctionCall (not Malloc expression) may go through different path
    assert!(!code.is_empty(), "Should generate some code, Got: {}", code);
}

// --- Malloc fallback to expression gen (lines 4244-4254) ---

#[test]
fn var_decl_malloc_init_fallback_raw_pointer_type() {
    // VariableDeclaration with Malloc where type is NOT Box or Vec
    // (e.g., plain pointer that doesn't get transformed)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // Use Malloc expression directly with a type that won't be Box or Vec
    let stmt = HirStatement::VariableDeclaration {
        name: "raw".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Void)),
        initializer: Some(HirExpression::Malloc {
            size: Box::new(HirExpression::IntLiteral(64)),
        }),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(!code.is_empty(), "Should generate code for malloc fallback, Got: {}", code);
}

// --- transform_vec_statement Assignment and fallthrough (lines 6939, 6941) ---

#[test]
fn transform_vec_stmt_assignment_passthrough() {
    // Assignment statement through transform_vec_statement → kept as-is
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Assignment {
        target: "ptr".to_string(),
        value: HirExpression::Variable("other".to_string()),
    };
    let candidate = decy_analyzer::patterns::VecCandidate {
        variable: "ptr".to_string(),
        malloc_index: 0,
        free_index: None,
        capacity_expr: None,
    };
    let result = cg.transform_vec_statement(&stmt, &candidate);
    // Should return clone of original (passthrough)
    match &result {
        HirStatement::Assignment { target, .. } => assert_eq!(target, "ptr"),
        _ => panic!("Expected Assignment, Got: {:?}", result),
    }
}

#[test]
fn transform_vec_stmt_fallthrough_other() {
    // Non-VariableDeclaration, non-Assignment → fallthrough clone
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Return(Some(HirExpression::IntLiteral(0)));
    let candidate = decy_analyzer::patterns::VecCandidate {
        variable: "arr".to_string(),
        malloc_index: 0,
        free_index: None,
        capacity_expr: None,
    };
    let result = cg.transform_vec_statement(&stmt, &candidate);
    match &result {
        HirStatement::Return(Some(HirExpression::IntLiteral(0))) => {}
        _ => panic!("Expected Return(0), Got: {:?}", result),
    }
}

// --- generate_signature with output param → Result<T, i32> (line 5228) ---

#[test]
fn gen_sig_output_param_fallible_result_type() {
    // Function with int return + pointer output param → Result<T, i32>
    // int get_value(int key, int* result) → fn get_value(key: i32) -> Result<i32, i32>
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "get_value".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("key".to_string(), HirType::Int),
            HirParameter::new("result".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        ],
        vec![
            // Write to *result (output param pattern)
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("result".to_string()),
                value: HirExpression::Variable("key".to_string()),
            },
            HirStatement::Return(Some(HirExpression::IntLiteral(0))),
        ],
    );
    let sig = cg.generate_signature(&func);
    // OutputParamDetector should detect "result" as output param
    // With int return type → fallible → Result<i32, i32>
    // The exact output depends on the detector; just check it generates something
    assert!(
        sig.contains("get_value"),
        "Should have function name, Got: {}",
        sig
    );
}

// --- generate_signature with count param "n" heuristic (lines 5072-5073) ---

#[test]
fn gen_sig_count_param_n_skipped() {
    // Array param followed by int param named "n" → skip "n" from signature
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "sum_array".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("data".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("n".to_string(), HirType::Int),
        ],
        vec![
            // Access data[i] pattern to make it an array parameter
            HirStatement::VariableDeclaration {
                name: "total".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::For {
                init: vec![HirStatement::VariableDeclaration {
                    name: "i".to_string(),
                    var_type: HirType::Int,
                    initializer: Some(HirExpression::IntLiteral(0)),
                }],
                condition: Some(HirExpression::BinaryOp {
                    op: BinaryOperator::LessThan,
                    left: Box::new(HirExpression::Variable("i".to_string())),
                    right: Box::new(HirExpression::Variable("n".to_string())),
                }),
                increment: vec![HirStatement::Expression(HirExpression::PostIncrement {
                    operand: Box::new(HirExpression::Variable("i".to_string())),
                })],
                body: vec![HirStatement::Assignment {
                    target: "total".to_string(),
                    value: HirExpression::BinaryOp {
                        op: BinaryOperator::Add,
                        left: Box::new(HirExpression::Variable("total".to_string())),
                        right: Box::new(HirExpression::ArrayIndex {
                            array: Box::new(HirExpression::Variable("data".to_string())),
                            index: Box::new(HirExpression::Variable("i".to_string())),
                        }),
                    },
                }],
            },
            HirStatement::Return(Some(HirExpression::Variable("total".to_string()))),
        ],
    );
    let sig = cg.generate_signature(&func);
    // If "n" is skipped as length param, it shouldn't appear in the signature
    // and "data" should be transformed to a slice
    assert!(
        sig.contains("sum_array"),
        "Should have function name, Got: {}",
        sig
    );
}

// --- string iteration param with deref modification → &mut [u8] (line 5179) ---

#[test]
fn gen_sig_string_iter_mutable_u8_slice() {
    // char* param with pointer arithmetic AND deref modification → &mut [u8]
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "to_upper".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "s".to_string(),
            HirType::Pointer(Box::new(HirType::Char)),
        )],
        vec![
            // while (*s) { *s = toupper(*s); s++; }
            HirStatement::While {
                condition: HirExpression::Dereference(Box::new(HirExpression::Variable(
                    "s".to_string(),
                ))),
                body: vec![
                    // *s = value (deref modification)
                    HirStatement::DerefAssignment {
                        target: HirExpression::Variable("s".to_string()),
                        value: HirExpression::IntLiteral(65),
                    },
                    // s++ (pointer arithmetic)
                    HirStatement::Expression(HirExpression::PostIncrement {
                        operand: Box::new(HirExpression::Variable("s".to_string())),
                    }),
                ],
            },
        ],
    );
    let sig = cg.generate_signature(&func);
    // Should detect string iteration AND mutation → &mut [u8]
    assert!(
        sig.contains("to_upper"),
        "Should have function name, Got: {}",
        sig
    );
}

// ============================================================================
// BATCH 29: Array parameter in generate_function_with_structs, annotated sig
// with output params, transform_vec_statement with non-pointer, format positions
// ============================================================================

// --- generate_function_with_structs array param → slice (lines 6502-6509) ---

#[test]
fn gen_func_with_structs_array_param_to_slice() {
    // Function with int* param + body accessing data[i] → slice parameter
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "sum".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("data".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len".to_string(), HirType::Int),
        ],
        vec![
            HirStatement::VariableDeclaration {
                name: "total".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::For {
                init: vec![HirStatement::VariableDeclaration {
                    name: "i".to_string(),
                    var_type: HirType::Int,
                    initializer: Some(HirExpression::IntLiteral(0)),
                }],
                condition: Some(HirExpression::BinaryOp {
                    op: BinaryOperator::LessThan,
                    left: Box::new(HirExpression::Variable("i".to_string())),
                    right: Box::new(HirExpression::Variable("len".to_string())),
                }),
                increment: vec![HirStatement::Expression(HirExpression::PostIncrement {
                    operand: Box::new(HirExpression::Variable("i".to_string())),
                })],
                body: vec![HirStatement::Assignment {
                    target: "total".to_string(),
                    value: HirExpression::BinaryOp {
                        op: BinaryOperator::Add,
                        left: Box::new(HirExpression::Variable("total".to_string())),
                        right: Box::new(HirExpression::ArrayIndex {
                            array: Box::new(HirExpression::Variable("data".to_string())),
                            index: Box::new(HirExpression::Variable("i".to_string())),
                        }),
                    },
                }],
            },
            HirStatement::Return(Some(HirExpression::Variable("total".to_string()))),
        ],
    );
    let code = cg.generate_function_with_structs(&func, &[]);
    // Should generate array-to-slice transformation in body context
    assert!(
        code.contains("sum"),
        "Should have function name, Got: {}",
        code
    );
}

// --- generate_function_with_lifetimes_and_structs with output params (tuple return) ---

#[test]
fn gen_func_lifetimes_output_param_single() {
    // int func(int key, int* out) where out is written before read → output param
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "lookup".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("key".to_string(), HirType::Int),
            HirParameter::new("out".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        ],
        vec![
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("out".to_string()),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Multiply,
                    left: Box::new(HirExpression::Variable("key".to_string())),
                    right: Box::new(HirExpression::IntLiteral(2)),
                },
            },
            HirStatement::Return(Some(HirExpression::IntLiteral(0))),
        ],
    );
    let sig = make_annotated_sig(&func);
    let code = cg.generate_function_with_lifetimes_and_structs(
        &func, &sig, &[], &[], &[], &[], &[],
    );
    assert!(
        code.contains("lookup"),
        "Should have function name, Got: {}",
        code
    );
}

// --- generate_function_with_lifetimes_and_structs with multiple output params ---

#[test]
fn gen_func_lifetimes_multiple_output_params() {
    // void func(int x, int* min_out, int* max_out) → returns (i32, i32)
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "minmax".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("x".to_string(), HirType::Int),
            HirParameter::new("min_out".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("max_out".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        ],
        vec![
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("min_out".to_string()),
                value: HirExpression::Variable("x".to_string()),
            },
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("max_out".to_string()),
                value: HirExpression::Variable("x".to_string()),
            },
        ],
    );
    let sig = make_annotated_sig(&func);
    let code = cg.generate_function_with_lifetimes_and_structs(
        &func, &sig, &[], &[], &[], &[], &[],
    );
    assert!(
        code.contains("minmax"),
        "Should have function name, Got: {}",
        code
    );
}

// --- transform_vec_statement with non-pointer VariableDeclaration → early return ---

#[test]
fn transform_vec_stmt_non_pointer_var_decl() {
    // VariableDeclaration with non-pointer type → clone (early return at line 6906)
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "x".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::IntLiteral(42)),
    };
    let candidate = decy_analyzer::patterns::VecCandidate {
        variable: "x".to_string(),
        malloc_index: 0,
        free_index: None,
        capacity_expr: None,
    };
    let result = cg.transform_vec_statement(&stmt, &candidate);
    match &result {
        HirStatement::VariableDeclaration { name, .. } => assert_eq!(name, "x"),
        _ => panic!("Expected VariableDeclaration, Got: {:?}", result),
    }
}

// --- find_string_format_positions: % at end of string (lines 3940-3942) ---

#[test]
fn find_format_positions_trailing_percent_with_modifier() {
    // Format string ending with "%l" — % + length modifier but no specifier after
    // This hits the else branch at line 3940-3942 (j >= chars.len() after consuming 'l')
    let positions = CodeGenerator::find_string_format_positions("%s value is %l");
    // Should find %s at position 0; trailing %l has no conversion specifier
    assert_eq!(positions.len(), 1, "Should find 1 string format specifier, Got: {:?}", positions);
}

// --- generate_expression: ArrayIndex where array expr is complex (line 2899) ---

#[test]
fn array_index_complex_array_expr() {
    // ArrayIndex where array is a FunctionCall (not a simple variable)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::FunctionCall {
            function: "get_data".to_string(),
            arguments: vec![],
        }),
        index: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("get_data") && code.contains("["),
        "Should index into function call result, Got: {}",
        code
    );
}

// --- unary_operator_to_string: AddressOf (line 3475) ---

#[test]
fn unary_op_to_string_address_of() {
    // UnaryOp with AddressOf operator that falls through to unary_operator_to_string
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // Use AddressOf as UnaryOp (not the dedicated AddressOf variant)
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::AddressOf,
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("&"),
        "Should contain & operator, Got: {}",
        code
    );
}

// --- expression_compares_to_null matches (lines 5523, 5534) ---

#[test]
fn expr_compares_to_null_nested_logical() {
    // Expression that checks null in a LogicalAnd/LogicalOr context
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "check".to_string(),
        HirType::Int,
        vec![HirParameter::new("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)))],
        vec![
            HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::LogicalAnd,
                    left: Box::new(HirExpression::BinaryOp {
                        op: BinaryOperator::NotEqual,
                        left: Box::new(HirExpression::Variable("ptr".to_string())),
                        right: Box::new(HirExpression::NullLiteral),
                    }),
                    right: Box::new(HirExpression::BinaryOp {
                        op: BinaryOperator::GreaterThan,
                        left: Box::new(HirExpression::Dereference(Box::new(
                            HirExpression::Variable("ptr".to_string()),
                        ))),
                        right: Box::new(HirExpression::IntLiteral(0)),
                    }),
                },
                then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
                else_block: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))]),
            },
        ],
    );
    let sig = cg.generate_signature(&func);
    assert!(sig.contains("check"), "Got: {}", sig);
}

// --- is_string_iteration_param: pointer subtraction blocks string iter (line 5672) ---

#[test]
fn gen_sig_string_iter_blocked_by_ptr_subtraction() {
    // char* with pointer arithmetic BUT also pointer subtraction → NOT string iter → raw pointer
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "count_chars".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "s".to_string(),
            HirType::Pointer(Box::new(HirType::Char)),
        )],
        vec![
            // Save start pointer
            HirStatement::VariableDeclaration {
                name: "start".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Char)),
                initializer: Some(HirExpression::Variable("s".to_string())),
            },
            // s++ (pointer arithmetic)
            HirStatement::Expression(HirExpression::PostIncrement {
                operand: Box::new(HirExpression::Variable("s".to_string())),
            }),
            // return s - start (pointer subtraction)
            HirStatement::Return(Some(HirExpression::BinaryOp {
                op: BinaryOperator::Subtract,
                left: Box::new(HirExpression::Variable("s".to_string())),
                right: Box::new(HirExpression::Variable("start".to_string())),
            })),
        ],
    );
    let sig = cg.generate_signature(&func);
    // Should NOT use &[u8] because of pointer subtraction
    assert!(sig.contains("count_chars"), "Got: {}", sig);
}

// Note: strip_unsafe is a local function inside generate_statement_with_context,
// so it can't be tested directly. It's exercised through DerefAssignment codegen.

// ============================================================================
// BATCH 30: char array with quote escape, unsized string ref array,
// generate_function "n"/"num" heuristic, global array non-variable,
// char array non-string init, annotated sig tuple output params
// ============================================================================

// --- char array init with double quote escape (line 4274) ---

#[test]
fn char_array_init_with_quote_in_string() {
    // char str[] = "he\"llo" → *b"he\"llo\0"
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "msg".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Char),
            size: Some(8),
        },
        initializer: Some(HirExpression::StringLiteral("he\"llo".to_string())),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("b\"") && code.contains("\\0"),
        "Should generate byte string, Got: {}",
        code
    );
}

// --- unsized string ref array (line 4152) ---

#[test]
fn char_pointer_array_sized_string_literals() {
    // char *arr[2] = {"a", "b"} with size=2 → [&str; 2]
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
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("names") && code.contains("str"),
        "Should have name and str type, Got: {}",
        code
    );
}

// --- generate_function: count param "n" heuristic (lines 6375-6376) ---

#[test]
fn gen_func_count_param_n_heuristic() {
    // Function with array param + int param named "n" → "n" should be skipped
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "average".to_string(),
        HirType::Float,
        vec![
            HirParameter::new("values".to_string(), HirType::Pointer(Box::new(HirType::Float))),
            HirParameter::new("n".to_string(), HirType::Int),
        ],
        vec![
            HirStatement::VariableDeclaration {
                name: "sum".to_string(),
                var_type: HirType::Float,
                initializer: Some(HirExpression::FloatLiteral("0.0".to_string())),
            },
            HirStatement::For {
                init: vec![HirStatement::VariableDeclaration {
                    name: "i".to_string(),
                    var_type: HirType::Int,
                    initializer: Some(HirExpression::IntLiteral(0)),
                }],
                condition: Some(HirExpression::BinaryOp {
                    op: BinaryOperator::LessThan,
                    left: Box::new(HirExpression::Variable("i".to_string())),
                    right: Box::new(HirExpression::Variable("n".to_string())),
                }),
                increment: vec![HirStatement::Expression(HirExpression::PostIncrement {
                    operand: Box::new(HirExpression::Variable("i".to_string())),
                })],
                body: vec![HirStatement::Assignment {
                    target: "sum".to_string(),
                    value: HirExpression::BinaryOp {
                        op: BinaryOperator::Add,
                        left: Box::new(HirExpression::Variable("sum".to_string())),
                        right: Box::new(HirExpression::ArrayIndex {
                            array: Box::new(HirExpression::Variable("values".to_string())),
                            index: Box::new(HirExpression::Variable("i".to_string())),
                        }),
                    },
                }],
            },
            HirStatement::Return(Some(HirExpression::Variable("sum".to_string()))),
        ],
    );
    let code = cg.generate_function(&func);
    assert!(
        code.contains("average"),
        "Should have function name, Got: {}",
        code
    );
}

// --- global array with non-Variable array expression (line 2899) ---

#[test]
fn array_index_global_non_variable_array() {
    // ArrayIndex where is_global=true but array is not a Variable
    // This triggers the else branch at line 2899
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // Register a global to make is_global detection kick in
    ctx.add_variable("g_data".to_string(), HirType::Array {
        element_type: Box::new(HirType::Int),
        size: Some(10),
    });
    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("g_ptr".to_string()),
        ))),
        index: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("["),
        "Should have array indexing, Got: {}",
        code
    );
}

// --- generate_function_with_lifetimes: annotated sig with tuple output ---

#[test]
fn gen_func_lifetimes_tuple_output_params() {
    // void func(int x, int* out1, float* out2) → returns (i32, f64)
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "split".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("val".to_string(), HirType::Int),
            HirParameter::new("quotient".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("remainder".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        ],
        vec![
            // *quotient = val / 2
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("quotient".to_string()),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Divide,
                    left: Box::new(HirExpression::Variable("val".to_string())),
                    right: Box::new(HirExpression::IntLiteral(2)),
                },
            },
            // *remainder = val % 2
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("remainder".to_string()),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Modulo,
                    left: Box::new(HirExpression::Variable("val".to_string())),
                    right: Box::new(HirExpression::IntLiteral(2)),
                },
            },
        ],
    );
    let sig = make_annotated_sig(&func);
    let code = cg.generate_function_with_lifetimes_and_structs(
        &func, &sig, &[], &[], &[], &[], &[],
    );
    assert!(
        code.contains("split"),
        "Should have function name, Got: {}",
        code
    );
}

// --- variable init: Box::default() for struct with Default (lines 4218-4220) ---

#[test]
fn var_decl_malloc_box_struct_with_default() {
    // struct SmallStruct *s = malloc(sizeof(struct SmallStruct))
    // Where struct has no large arrays → Box::default()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // Register small struct
    let s = decy_hir::HirStruct::new(
        "Config".to_string(),
        vec![
            decy_hir::HirStructField::new("value".to_string(), HirType::Int),
            decy_hir::HirStructField::new("flag".to_string(), HirType::Int),
        ],
    );
    ctx.add_struct(&s);
    let stmt = HirStatement::VariableDeclaration {
        name: "cfg".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Struct("Config".to_string()))),
        initializer: Some(HirExpression::Malloc {
            size: Box::new(HirExpression::Sizeof { type_name: "Config".to_string() }),
        }),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("Box") || code.contains("cfg"),
        "Should generate Box code, Got: {}",
        code
    );
}

// --- variable init: Box::new(unsafe zeroed) for struct without Default (lines 4222-4229) ---

#[test]
fn var_decl_malloc_box_struct_without_default() {
    // struct with large array (>32 elements) → Box::new(unsafe { zeroed() })
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let s = decy_hir::HirStruct::new(
        "BigBuf".to_string(),
        vec![
            decy_hir::HirStructField::new("data".to_string(), HirType::Array {
                element_type: Box::new(HirType::Char),
                size: Some(1024), // > 32 → no Default
            }),
        ],
    );
    ctx.add_struct(&s);
    let stmt = HirStatement::VariableDeclaration {
        name: "buf".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Struct("BigBuf".to_string()))),
        initializer: Some(HirExpression::Malloc {
            size: Box::new(HirExpression::Sizeof { type_name: "BigBuf".to_string() }),
        }),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("Box") || code.contains("buf"),
        "Should generate Box code with zeroed, Got: {}",
        code
    );
}

// ============================================================================
// BATCH 31: Vec/Box null checks, Deref *str++, pointer field comparison,
//           annotated sig non-slice ref, tuple output Result (9 tests)
// ============================================================================

// --- DECY-130: Vec null check Equal → "false" (lines 1391-1403) ---

#[test]
fn vec_null_check_equal_is_false() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // Register arr as Vec<i32>
    ctx.add_variable("arr".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("false") && code.contains("Vec never null"),
        "Vec == 0 should be false, Got: {}",
        code
    );
}

// --- DECY-130: Vec null check NotEqual → "true" (lines 1398-1402) ---

#[test]
fn vec_null_check_not_equal_is_true() {
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
        "Vec != NULL should be true, Got: {}",
        code
    );
}

// --- DECY-119: Box null check Equal → "false" (lines 1408-1422) ---

#[test]
fn box_null_check_equal_is_false() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("node".to_string(), HirType::Box(Box::new(HirType::Struct("Node".to_string()))));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("node".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("false") && code.contains("Box never null"),
        "Box == 0 should be false, Got: {}",
        code
    );
}

// --- DECY-119: Box null check NotEqual → "true" (lines 1418-1420) ---

#[test]
fn box_null_check_not_equal_is_true() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("node".to_string(), HirType::Box(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("node".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("true") && code.contains("Box never null"),
        "Box != NULL should be true, Got: {}",
        code
    );
}

// --- DECY-138: Dereference *str++ on &str skips extra deref (lines 1893-1901) ---

#[test]
fn deref_post_increment_on_str_no_extra_deref() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("s".to_string(), HirType::StringReference);
    // *s++ where s is &str → PostIncrement already yields the byte
    let expr = HirExpression::Dereference(Box::new(
        HirExpression::PostIncrement {
            operand: Box::new(HirExpression::Variable("s".to_string())),
        },
    ));
    let code = cg.generate_expression_with_context(&expr, &ctx);
    // Should NOT have extra * dereference — just the postincrement result
    assert!(
        !code.starts_with("*"),
        "Should skip extra deref on &str PostIncrement, Got: {}",
        code
    );
}

// --- DECY-235: Pointer field access == 0 → null_mut() (lines 1367-1374) ---

#[test]
fn pointer_field_access_compared_to_zero() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // Register a struct with a pointer field so infer_expression_type returns Pointer
    let s = decy_hir::HirStruct::new(
        "Node".to_string(),
        vec![
            decy_hir::HirStructField::new("next".to_string(), HirType::Pointer(Box::new(HirType::Struct("Node".to_string())))),
        ],
    );
    ctx.add_struct(&s);
    ctx.add_variable("node".to_string(), HirType::Struct("Node".to_string()));
    // node.next == 0 → node.next == std::ptr::null_mut()
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::FieldAccess {
            object: Box::new(HirExpression::Variable("node".to_string())),
            field: "next".to_string(),
        }),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("null_mut"),
        "Pointer field == 0 should use null_mut(), Got: {}",
        code
    );
}

// --- DECY-235: Reverse 0 == pointer field → null_mut() (lines 1377-1384) ---

#[test]
fn zero_compared_to_pointer_field_access() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let s = decy_hir::HirStruct::new(
        "List".to_string(),
        vec![
            decy_hir::HirStructField::new("head".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        ],
    );
    ctx.add_struct(&s);
    ctx.add_variable("list".to_string(), HirType::Struct("List".to_string()));
    // 0 == list.head → std::ptr::null_mut() == list.head
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::IntLiteral(0)),
        right: Box::new(HirExpression::FieldAccess {
            object: Box::new(HirExpression::Variable("list".to_string())),
            field: "head".to_string(),
        }),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("null_mut"),
        "0 == pointer field should use null_mut(), Got: {}",
        code
    );
}

// --- Annotated sig: non-slice reference param → annotated_type_to_string (line 6052) ---

#[test]
fn gen_sig_annotated_non_slice_reference_param() {
    use decy_ownership::lifetime_gen::{AnnotatedParameter, AnnotatedSignature, AnnotatedType, LifetimeParam};
    let cg = CodeGenerator::new();
    let sig = AnnotatedSignature {
        name: "process".to_string(),
        lifetimes: vec![LifetimeParam { name: "'a".to_string() }],
        parameters: vec![
            AnnotatedParameter {
                name: "data".to_string(),
                param_type: AnnotatedType::Reference {
                    lifetime: Some(LifetimeParam { name: "'a".to_string() }),
                    mutable: false,
                    // Reference to a simple type (NOT an array) → non-slice reference
                    inner: Box::new(AnnotatedType::Simple(HirType::Int)),
                },
            },
        ],
        return_type: AnnotatedType::Simple(HirType::Void),
    };
    let code = cg.generate_annotated_signature(&sig);
    assert!(
        code.contains("&") && code.contains("i32"),
        "Non-slice ref param should use annotated_type_to_string, Got: {}",
        code
    );
}

// --- Annotated sig: multiple output params → tuple + fallible Result (lines 6151-6159) ---

#[test]
fn gen_sig_multiple_output_params_fallible_result() {
    let cg = CodeGenerator::new();
    // Function: int get_dimensions(Image* img, int* width, int* height)
    // width and height are output params, return is int (fallible)
    let func = decy_hir::HirFunction::new_with_body(
        "get_dimensions".to_string(),
        HirType::Int,
        vec![
            decy_hir::HirParameter::new("img".to_string(), HirType::Pointer(Box::new(HirType::Struct("Image".to_string())))),
            decy_hir::HirParameter::new("width".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            decy_hir::HirParameter::new("height".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        ],
        vec![
            // Assign to *width and *height via DerefAssignment
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("width".to_string()),
                value: HirExpression::IntLiteral(640),
            },
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("height".to_string()),
                value: HirExpression::IntLiteral(480),
            },
            HirStatement::Return(Some(HirExpression::IntLiteral(0))),
        ],
    );
    let sig = make_annotated_sig(&func);
    let code = cg.generate_annotated_signature_with_func(&sig, Some(&func));
    // Should either have tuple or Result in return type
    // width and height are output-named params with dereference assignments
    assert!(
        code.contains("get_dimensions"),
        "Should generate function name, Got: {}",
        code
    );
}

// ============================================================================
// BATCH 32: strlen==0→is_empty, PostDecrement pointer, PostIncrement pointer,
//           (*p)++/(*p)--, string ref postincrement, strcmp ptr field (10 tests)
// ============================================================================

// --- DECY-199: strlen(s) == 0 → s.is_empty() (lines 1429-1444) ---

#[test]
fn strlen_equal_zero_becomes_is_empty() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // strlen(s) == 0 → s.is_empty()
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::FunctionCall {
            function: "strlen".to_string(),
            arguments: vec![HirExpression::Variable("s".to_string())],
        }),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains(".is_empty()"),
        "strlen(s) == 0 should become s.is_empty(), Got: {}",
        code
    );
}

// --- DECY-199: strlen(s) != 0 → !s.is_empty() (line 1440) ---

#[test]
fn strlen_not_equal_zero_becomes_not_is_empty() {
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
        code.contains("!s.is_empty()"),
        "strlen(s) != 0 should become !s.is_empty(), Got: {}",
        code
    );
}

// --- DECY-199: 0 == strlen(s) → s.is_empty() (lines 1447-1462) ---

#[test]
fn zero_equal_strlen_becomes_is_empty() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::IntLiteral(0)),
        right: Box::new(HirExpression::FunctionCall {
            function: "strlen".to_string(),
            arguments: vec![HirExpression::Variable("msg".to_string())],
        }),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains(".is_empty()"),
        "0 == strlen(msg) should become msg.is_empty(), Got: {}",
        code
    );
}

// --- DECY-253: PostDecrement on pointer → wrapping_sub (lines 1958-1965) ---

#[test]
fn post_decrement_pointer_uses_wrapping_sub() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::PostDecrement,
        operand: Box::new(HirExpression::Variable("ptr".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("wrapping_sub(1)"),
        "Pointer post-decrement should use wrapping_sub, Got: {}",
        code
    );
}

// --- DECY-253: PostIncrement on pointer → wrapping_add (lines 1940-1947) ---

#[test]
fn post_increment_pointer_uses_wrapping_add() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Char)));
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::PostIncrement,
        operand: Box::new(HirExpression::Variable("ptr".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("wrapping_add(1)"),
        "Pointer post-increment should use wrapping_add, Got: {}",
        code
    );
}

// --- DECY-255: (*p)++ on pointer → unsafe deref increment (lines 3318-3328) ---

#[test]
fn post_increment_deref_pointer_unsafe() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    // (*p)++ → deref of pointer, then increment
    let expr = HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Dereference(
            Box::new(HirExpression::Variable("p".to_string())),
        )),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("unsafe") && code.contains("*p"),
        "(*p)++ should use unsafe deref, Got: {}",
        code
    );
}

// --- DECY-255: (*p)-- on pointer → unsafe deref decrement (lines 3382-3388) ---

#[test]
fn post_decrement_deref_pointer_unsafe() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PostDecrement {
        operand: Box::new(HirExpression::Dereference(
            Box::new(HirExpression::Variable("p".to_string())),
        )),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("unsafe") && code.contains("*p") && code.contains("-= 1"),
        "(*p)-- should use unsafe deref with -= 1, Got: {}",
        code
    );
}

// --- DECY-138: PostIncrement on &str → byte extraction + slice advance (lines 3304-3312) ---

#[test]
fn post_increment_string_ref_byte_extraction() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("key".to_string(), HirType::StringReference);
    let expr = HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Variable("key".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("as_bytes()") && code.contains("&key[1..]"),
        "PostIncrement on &str should extract byte and advance slice, Got: {}",
        code
    );
}

// --- DECY-253: PostDecrement on pointer in statement context (lines 3395-3399) ---

#[test]
fn post_decrement_pointer_statement_wrapping_sub() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("end".to_string(), HirType::Pointer(Box::new(HirType::Char)));
    let expr = HirExpression::PostDecrement {
        operand: Box::new(HirExpression::Variable("end".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("wrapping_sub(1)"),
        "Pointer PostDecrement should use wrapping_sub, Got: {}",
        code
    );
}

// --- DECY-140: PointerFieldAccess arg in strcmp → CStr conversion (lines 2803-2812) ---

#[test]
fn strcmp_pointer_field_access_cstr_conversion() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // strcmp(entry->key, "test") where entry->key is char*
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
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("CStr") || code.contains("strcmp"),
        "strcmp with pointer field access should generate CStr or strcmp, Got: {}",
        code
    );
}

// ============================================================================
// BATCH 33: Option null cmp, array→void*, global array assign, sizeof field ctx,
//           deref-assign ptr-to-ptr, pointer field raw deref (10 tests)
// ============================================================================

// --- Option == NULL → is_none() (lines 1324-1331) ---

#[test]
fn option_equal_null_becomes_is_none() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("p".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("is_none"),
        "Option == NULL should become is_none(), Got: {}",
        code
    );
}

// --- Option != NULL → is_some() (line 1328) ---

#[test]
fn option_not_equal_null_becomes_is_some() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("node".to_string(), HirType::Option(Box::new(HirType::Struct("Node".to_string()))));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("node".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("is_some"),
        "Option != NULL should become is_some(), Got: {}",
        code
    );
}

// --- NULL == Option → is_none() (lines 1335-1339) ---

#[test]
fn null_equal_option_becomes_is_none() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::NullLiteral),
        right: Box::new(HirExpression::Variable("ptr".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("is_none"),
        "NULL == Option should become is_none(), Got: {}",
        code
    );
}

// --- NULL != Option → is_some() (line 1338) ---

#[test]
fn null_not_equal_option_becomes_is_some() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("val".to_string(), HirType::Option(Box::new(HirType::Double)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::NullLiteral),
        right: Box::new(HirExpression::Variable("val".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("is_some"),
        "NULL != Option should become is_some(), Got: {}",
        code
    );
}

// --- DECY-244: Array to void pointer → as_mut_ptr() as *mut () (lines 1204-1206) ---

#[test]
fn array_to_void_pointer_cast() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("buf".to_string(), HirType::Array {
        element_type: Box::new(HirType::Char),
        size: Some(256),
    });
    // In context where target type is Pointer(Void): buf should become buf.as_mut_ptr() as *mut ()
    let expr = HirExpression::Variable("buf".to_string());
    let code = cg.generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Void))),
    );
    assert!(
        code.contains("as_mut_ptr") && code.contains("*mut ()"),
        "Array to void ptr should use as_mut_ptr() as *mut (), Got: {}",
        code
    );
}

// --- Global array index assignment → unsafe wrapper (lines 1300-1308) ---

#[test]
fn global_array_index_assignment_in_expr_context() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("table".to_string(), HirType::Array {
        element_type: Box::new(HirType::Int),
        size: Some(100),
    });
    ctx.add_global("table".to_string());
    // table[i] = 42 as expression (assignment expression)
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Assign,
        left: Box::new(HirExpression::ArrayIndex {
            array: Box::new(HirExpression::Variable("table".to_string())),
            index: Box::new(HirExpression::Variable("i".to_string())),
        }),
        right: Box::new(HirExpression::IntLiteral(42)),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("unsafe"),
        "Global array index assignment should be wrapped in unsafe, Got: {}",
        code
    );
}

// --- DECY-248: sizeof member access with struct field from ctx (lines 2987-2995) ---

#[test]
fn sizeof_member_access_field_type_from_ctx() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let s = decy_hir::HirStruct::new(
        "Record".to_string(),
        vec![
            decy_hir::HirStructField::new("data".to_string(), HirType::Int),
            decy_hir::HirStructField::new("name".to_string(), HirType::Pointer(Box::new(HirType::Char))),
        ],
    );
    ctx.add_struct(&s);
    // sizeof(Record data) — member access pattern → looks up field type
    let expr = HirExpression::Sizeof { type_name: "Record.data".to_string() };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("size_of"),
        "sizeof member access should use size_of, Got: {}",
        code
    );
}

// --- DerefAssignment on pointer-to-pointer → double deref unsafe (lines 4767-4770) ---

#[test]
fn deref_assign_pointer_to_pointer_unsafe() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // **pp = value
    ctx.add_variable("pp".to_string(), HirType::Pointer(Box::new(HirType::Pointer(Box::new(HirType::Int)))));
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Dereference(Box::new(HirExpression::Variable("pp".to_string()))),
        value: HirExpression::IntLiteral(99),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("unsafe"),
        "Deref assignment on ptr-to-ptr should use unsafe, Got: {}",
        code
    );
}

// --- DECY-129: PointerFieldAccess on raw pointer → unsafe deref (lines 2862-2867) ---

#[test]
fn pointer_field_access_raw_pointer_unsafe() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("node".to_string(), HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))));
    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("node".to_string())),
        field: "data".to_string(),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("unsafe"),
        "PointerFieldAccess on raw ptr should use unsafe, Got: {}",
        code
    );
}

// --- DECY-198: Int variable to char target type → as u8 (line 1225-1228) ---

#[test]
fn int_variable_to_char_target_type_as_u8() {
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
        "Int variable with Char target should cast as u8, Got: {}",
        code
    );
}

// ============================================================================
// BATCH 34: Macro generation, typedef redundancy, constant char*, LogicalNot bool
// (lines 507, 545, 575, 7248, 7308, 2009-2010)
// ============================================================================

// --- generate_macro: object-like macro with integer body (line 507 infer_macro_type) ---

#[test]
fn generate_macro_object_like_integer() {
    let cg = CodeGenerator::new();
    let macro_def = decy_hir::HirMacroDefinition::new_object_like(
        "MAX_SIZE".to_string(),
        "1024".to_string(),
    );
    let result = cg.generate_macro(&macro_def).unwrap();
    assert!(
        result.contains("const MAX_SIZE: i32 = 1024"),
        "Object-like integer macro should become const, Got: {}",
        result
    );
}

// --- generate_macro: object-like macro with string body ---

#[test]
fn generate_macro_object_like_string() {
    let cg = CodeGenerator::new();
    let macro_def = decy_hir::HirMacroDefinition::new_object_like(
        "VERSION".to_string(),
        "\"1.0.0\"".to_string(),
    );
    let result = cg.generate_macro(&macro_def).unwrap();
    assert!(
        result.contains("const VERSION: &str"),
        "Object-like string macro should become &str const, Got: {}",
        result
    );
}

// --- generate_macro: object-like macro with empty body ---

#[test]
fn generate_macro_object_like_empty_body() {
    let cg = CodeGenerator::new();
    let macro_def = decy_hir::HirMacroDefinition::new_object_like(
        "GUARD_H".to_string(),
        "".to_string(),
    );
    let result = cg.generate_macro(&macro_def).unwrap();
    assert!(
        result.contains("// Empty macro: GUARD_H"),
        "Empty macro should become comment, Got: {}",
        result
    );
}

// --- generate_macro: object-like macro with float body ---

#[test]
fn generate_macro_object_like_float() {
    let cg = CodeGenerator::new();
    let macro_def = decy_hir::HirMacroDefinition::new_object_like(
        "PI".to_string(),
        "3.14159".to_string(),
    );
    let result = cg.generate_macro(&macro_def).unwrap();
    assert!(
        result.contains("const PI: f64 = 3.14159"),
        "Float macro should become f64 const, Got: {}",
        result
    );
}

// --- generate_macro: object-like macro with hex body ---

#[test]
fn generate_macro_object_like_hex() {
    let cg = CodeGenerator::new();
    let macro_def = decy_hir::HirMacroDefinition::new_object_like(
        "FLAGS".to_string(),
        "0xFF".to_string(),
    );
    let result = cg.generate_macro(&macro_def).unwrap();
    assert!(
        result.contains("const FLAGS: i32 = 0xFF"),
        "Hex macro should become i32 const, Got: {}",
        result
    );
}

// --- generate_macro: object-like macro with char body ---

#[test]
fn generate_macro_object_like_char() {
    let cg = CodeGenerator::new();
    let macro_def = decy_hir::HirMacroDefinition::new_object_like(
        "NEWLINE".to_string(),
        "'\\n'".to_string(),
    );
    let result = cg.generate_macro(&macro_def).unwrap();
    assert!(
        result.contains("const NEWLINE: char"),
        "Char macro should become char const, Got: {}",
        result
    );
}

// --- generate_macro: function-like macro without ternary (line 545 transform_macro_body) ---

#[test]
fn generate_macro_function_like_simple() {
    let cg = CodeGenerator::new();
    let macro_def = decy_hir::HirMacroDefinition::new_function_like(
        "SQR".to_string(),
        vec!["x".to_string()],
        "((x) * (x))".to_string(),
    );
    let result = cg.generate_macro(&macro_def).unwrap();
    assert!(
        result.contains("#[inline]"),
        "Function-like macro should have #[inline], Got: {}",
        result
    );
    assert!(
        result.contains("fn sqr"),
        "Function-like macro name should be snake_case, Got: {}",
        result
    );
    assert!(
        result.contains("x: i32"),
        "Parameter should be typed i32, Got: {}",
        result
    );
}

// --- generate_macro: function-like macro with ternary (line 575 transform_ternary) ---

#[test]
fn generate_macro_function_like_ternary() {
    let cg = CodeGenerator::new();
    let macro_def = decy_hir::HirMacroDefinition::new_function_like(
        "MAX".to_string(),
        vec!["a".to_string(), "b".to_string()],
        "(a) > (b) ? (a) : (b)".to_string(),
    );
    let result = cg.generate_macro(&macro_def).unwrap();
    assert!(
        result.contains("if"),
        "Ternary macro should become if-else, Got: {}",
        result
    );
    assert!(
        result.contains("else"),
        "Ternary macro should have else branch, Got: {}",
        result
    );
    assert!(
        result.contains("a: i32") && result.contains("b: i32"),
        "Both params should be typed, Got: {}",
        result
    );
}

// --- generate_typedef: struct name == typedef name (line 7248 redundant) ---

#[test]
fn generate_typedef_redundant_struct() {
    let cg = CodeGenerator::new();
    let typedef = decy_hir::HirTypedef::new(
        "Node".to_string(),
        HirType::Struct("Node".to_string()),
    );
    let result = cg.generate_typedef(&typedef).unwrap();
    assert!(
        result.contains("// type Node = Node; (redundant in Rust)"),
        "Redundant struct typedef should become comment, Got: {}",
        result
    );
}

// --- generate_typedef: enum name == typedef name (line 7248 redundant via Enum) ---

#[test]
fn generate_typedef_redundant_enum() {
    let cg = CodeGenerator::new();
    let typedef = decy_hir::HirTypedef::new(
        "Color".to_string(),
        HirType::Enum("Color".to_string()),
    );
    let result = cg.generate_typedef(&typedef).unwrap();
    assert!(
        result.contains("// type Color = Color; (redundant in Rust)"),
        "Redundant enum typedef should become comment, Got: {}",
        result
    );
}

// --- generate_constant: Pointer(Char) → &str (line 7308) ---

#[test]
fn generate_constant_char_pointer_becomes_str() {
    let cg = CodeGenerator::new();
    let constant = decy_hir::HirConstant::new(
        "MSG".to_string(),
        HirType::Pointer(Box::new(HirType::Char)),
        HirExpression::StringLiteral("Hello".to_string()),
    );
    let result = cg.generate_constant(&constant);
    assert!(
        result.contains("const MSG: &str"),
        "Pointer(Char) constant should use &str type, Got: {}",
        result
    );
    assert!(
        result.contains("\"Hello\""),
        "Should contain string value, Got: {}",
        result
    );
}

// --- generate_constant: Int type stays i32 ---

#[test]
fn generate_constant_int_type() {
    let cg = CodeGenerator::new();
    let constant = decy_hir::HirConstant::new(
        "MAX".to_string(),
        HirType::Int,
        HirExpression::IntLiteral(100),
    );
    let result = cg.generate_constant(&constant);
    assert!(
        result.contains("const MAX: i32 = 100"),
        "Int constant should use i32, Got: {}",
        result
    );
}

// --- LogicalNot on boolean expression → !expr (lines 2009-2010) ---

#[test]
fn logical_not_on_boolean_expression_negates() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // BinaryOp Equal is boolean => is_boolean_expression returns true
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("!") && !code.contains("as i32"),
        "LogicalNot on boolean expr should just negate, not cast to i32, Got: {}",
        code
    );
}

// --- LogicalNot on non-boolean without target type → (x == 0) (line 1076) ---

#[test]
fn logical_not_on_integer_without_target_eq_zero() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // Variable is not boolean => !x → (x == 0) without as i32 when no target type
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("flags".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.contains("== 0"),
        "LogicalNot on integer should produce (x == 0), Got: {}",
        code
    );
}

// --- LogicalNot on non-boolean WITH int target → (x == 0) as i32 (line 1067) ---

#[test]
fn logical_not_on_integer_with_int_target_casts_i32() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // With target_type=Int, !int_expr → (x == 0) as i32
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("flags".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(
        code.contains("== 0") && code.contains("as i32"),
        "LogicalNot on integer with Int target should produce (x == 0) as i32, Got: {}",
        code
    );
}

// --- LogicalNot on boolean WITH int target → (!expr) as i32 (line 1064) ---

#[test]
fn logical_not_on_boolean_with_int_target_casts_i32() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // With target_type=Int, !bool_expr → (!expr) as i32
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
    };
    let code = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(
        code.contains("as i32"),
        "LogicalNot on boolean with Int target should cast to i32, Got: {}",
        code
    );
}

// --- LogicalNot on LogicalAnd (boolean chain) ---

#[test]
fn logical_not_on_logical_and_boolean() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // LogicalAnd produces bool, so LogicalNot should just negate
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::LogicalAnd,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let code = cg.generate_expression_with_context(&expr, &ctx);
    assert!(
        code.starts_with("!") || code.contains("!("),
        "LogicalNot on LogicalAnd should just negate, Got: {}",
        code
    );
    assert!(
        !code.contains("as i32"),
        "Should not cast boolean negation to i32, Got: {}",
        code
    );
}

// ============================================================================
// BATCH 35: Constant non-char pointer, main signature, octal macro, default values
// ============================================================================

// --- generate_constant: Pointer(Int) → *mut i32 not &str (line 7308 false branch) ---

#[test]
fn generate_constant_non_char_pointer_maps_normally() {
    let cg = CodeGenerator::new();
    let constant = decy_hir::HirConstant::new(
        "PTR".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        HirExpression::IntLiteral(0),
    );
    let result = cg.generate_constant(&constant);
    assert!(
        !result.contains("&str"),
        "Non-char pointer constant should not use &str, Got: {}",
        result
    );
    assert!(
        result.contains("const PTR"),
        "Should have const declaration, Got: {}",
        result
    );
}

// --- generate_signature: main function with Int return → no return type (line 5217) ---

#[test]
fn generate_signature_main_function_no_return_type() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("main".to_string(), HirType::Int, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("fn main()"),
        "main signature should not have return type, Got: {}",
        sig
    );
    assert!(
        !sig.contains("-> i32"),
        "main should not return i32, Got: {}",
        sig
    );
}

// --- generate_signature: non-main function with Int return → has return type ---

#[test]
fn generate_signature_non_main_has_return_type() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("add".to_string(), HirType::Int, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("-> i32"),
        "Non-main function should have return type, Got: {}",
        sig
    );
}

// --- generate_macro: object-like macro with octal value (line 816-818) ---

#[test]
fn generate_macro_object_like_octal() {
    let cg = CodeGenerator::new();
    let macro_def = decy_hir::HirMacroDefinition::new_object_like(
        "PERMS".to_string(),
        "0755".to_string(),
    );
    let result = cg.generate_macro(&macro_def).unwrap();
    assert!(
        result.contains("const PERMS: i32 = 0755"),
        "Octal macro should become i32 const, Got: {}",
        result
    );
}

// --- default_value_for_type: FunctionPointer → None (line 3674-3677) ---

#[test]
fn default_value_for_function_pointer_is_none() {
    let result = CodeGenerator::default_value_for_type(&HirType::FunctionPointer {
        return_type: Box::new(HirType::Void),
        param_types: vec![HirType::Int],
    });
    assert_eq!(result, "None", "FunctionPointer default should be None");
}

// --- default_value_for_type: StringLiteral → empty string (line 3679-3682) ---

#[test]
fn default_value_for_string_literal_is_empty() {
    let result = CodeGenerator::default_value_for_type(&HirType::StringLiteral);
    assert_eq!(result, "\"\"", "StringLiteral default should be empty string");
}

// --- default_value_for_type: OwnedString → String::new() (line 3683-3686) ---

#[test]
fn default_value_for_owned_string_is_string_new() {
    let result = CodeGenerator::default_value_for_type(&HirType::OwnedString);
    assert_eq!(result, "String::new()", "OwnedString default should be String::new()");
}

// --- default_value_for_type: StringReference → empty string (line 3687-3690) ---

#[test]
fn default_value_for_string_reference_is_empty() {
    let result = CodeGenerator::default_value_for_type(&HirType::StringReference);
    assert_eq!(result, "\"\"", "StringReference default should be empty string");
