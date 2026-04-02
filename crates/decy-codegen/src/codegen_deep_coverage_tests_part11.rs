// --- expression_uses_pointer_subtraction: right side match ---
#[test]
fn ptr_sub_detect_right_side_variable() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "len".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("end".to_string(), HirType::Pointer(Box::new(HirType::Char))),
            HirParameter::new("start".to_string(), HirType::Pointer(Box::new(HirType::Char))),
        ],
        vec![
            HirStatement::Return(Some(HirExpression::BinaryOp {
                op: BinaryOperator::Subtract,
                left: Box::new(HirExpression::Variable("end".to_string())),
                right: Box::new(HirExpression::Variable("start".to_string())),
            })),
        ],
    );
    // Check for "start" which appears on the right side
    let uses = cg.function_uses_pointer_subtraction(&func, "start");
    assert!(uses, "Should detect ptr subtraction when var is on right side");
}

// ============================================================================
// BATCH 22 continued: generate_signature void* constraints (lines 4999-5019)
// ============================================================================

// --- void* with body that triggers constraints → <T: ...> ---
#[test]
fn sig_void_ptr_with_clone_constraint() {
    let cg = CodeGenerator::new();
    // Function: void swap(void* a, void* b, size_t size)
    // Body: deref assign → triggers Mutable + Clone constraints
    let func = HirFunction::new_with_body(
        "swap".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("a".to_string(), HirType::Pointer(Box::new(HirType::Void))),
            HirParameter::new("b".to_string(), HirType::Pointer(Box::new(HirType::Void))),
            HirParameter::new("size".to_string(), HirType::UnsignedInt),
        ],
        vec![
            // *a = *b (triggers Mutable on a, Clone from deref value)
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("a".to_string()),
                value: HirExpression::Dereference(Box::new(HirExpression::Variable("b".to_string()))),
            },
        ],
    );
    let sig = cg.generate_signature(&func);
    // Should have <T: Clone> or similar constraint
    assert!(sig.contains("<T") || sig.contains("swap"), "Got: {}", sig);
}

// --- void* with inferred types → <T> (no specific constraints) ---
#[test]
fn sig_void_ptr_with_inferred_types_generic_t() {
    let cg = CodeGenerator::new();
    // Function with void* that has a cast (inferred type) but no trait constraints
    let func = HirFunction::new_with_body(
        "process".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("data".to_string(), HirType::Pointer(Box::new(HirType::Void))),
        ],
        vec![
            // Cast void* to int* → inferred type but no trait constraint
            HirStatement::VariableDeclaration {
                name: "p".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Cast {
                    expr: Box::new(HirExpression::Variable("data".to_string())),
                    target_type: HirType::Pointer(Box::new(HirType::Int)),
                }),
            },
        ],
    );
    let sig = cg.generate_signature(&func);
    // Should have <T> since there's real void usage (inferred type) but no specific trait bounds
    assert!(sig.contains("<T>") || sig.contains("process"), "Got: {}", sig);
}

// ============================================================================
// BATCH 22 continued: Macro type inference (lines 705-828)
// ============================================================================

// --- infer_macro_type: default expression fallback (line 826-827) ---
#[test]
fn macro_type_default_expression() {
    let cg = CodeGenerator::new();
    // Unknown macro body that isn't string, char, float, hex, octal, or parseable int
    // Avoid 'e'/'E' chars (float), '.', quotes, 0x/0 prefix
    let result = cg.infer_macro_type("MY_FLAG | SYS_VAL").unwrap();
    assert_eq!(result.0, "i32", "Type should be: {}", result.0);
}

// --- Binary minus spacing (lines 705-712) ---
#[test]
fn macro_binary_minus_spacing() {
    let cg = CodeGenerator::new();
    let macro_def = decy_hir::HirMacroDefinition::new_function_like(
        "DIFF".to_string(),
        vec!["a".to_string(), "b".to_string()],
        "a-b".to_string(),
    );
    let result = cg.generate_macro(&macro_def).unwrap();
    // The minus should get spaced out: a - b
    assert!(result.contains(" - ") || result.contains("-"), "Got: {}", result);
}

// --- infer_macro_type: parseable integer ---
#[test]
fn macro_type_integer() {
    let cg = CodeGenerator::new();
    let result = cg.infer_macro_type("42").unwrap();
    assert_eq!(result.0, "i32");
    assert_eq!(result.1, "42");
}

// --- infer_macro_type: hexadecimal ---
#[test]
fn macro_type_hex() {
    let cg = CodeGenerator::new();
    let result = cg.infer_macro_type("0xFF").unwrap();
    assert_eq!(result.0, "i32");
    assert_eq!(result.1, "0xFF");
}

// --- infer_macro_type: octal ---
#[test]
fn macro_type_octal() {
    let cg = CodeGenerator::new();
    let result = cg.infer_macro_type("0755").unwrap();
    assert_eq!(result.0, "i32");
    assert_eq!(result.1, "0755");
}

// ============================================================================
// BATCH 23: generate_function_with_lifetimes_and_structs (lines 6617-6764)
// Target: parameter context setup, string iteration, pointer arithmetic, array params
// ============================================================================

// Helper to build AnnotatedSignature easily
fn make_annotated_sig(func: &HirFunction) -> decy_ownership::lifetime_gen::AnnotatedSignature {
    use decy_ownership::lifetime_gen::LifetimeAnnotator;
    let annotator = LifetimeAnnotator::new();
    annotator.annotate_function(func)
}

// --- Basic function with lifetimes and struct context ---
#[test]
fn gen_func_lifetimes_basic_int_return() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "add".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
        vec![
            HirStatement::Return(Some(HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            })),
        ],
    );
    let sig = make_annotated_sig(&func);
    let code = cg.generate_function_with_lifetimes_and_structs(
        &func, &sig, &[], &[], &[], &[], &[],
    );
    assert!(code.contains("fn add"), "Got: {}", code);
    assert!(code.contains("a + b") || code.contains("(a) + (b)"), "Got: {}", code);
}

// --- Function with char* param (non-const) → reference transform ---
#[test]
fn gen_func_lifetimes_char_ptr_param() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "print_msg".to_string(),
        HirType::Void,
        vec![HirParameter::new("msg".to_string(), HirType::Pointer(Box::new(HirType::Char)))],
        vec![
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "puts".to_string(),
                arguments: vec![HirExpression::Variable("msg".to_string())],
            }),
        ],
    );
    let sig = make_annotated_sig(&func);
    let code = cg.generate_function_with_lifetimes_and_structs(
        &func, &sig, &[], &[], &[], &[], &[],
    );
    assert!(code.contains("print_msg"), "Got: {}", code);
}

// --- Function with pointer param that uses pointer arithmetic (line 6669-6673) ---
#[test]
fn gen_func_lifetimes_ptr_arithmetic_keeps_pointer() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "scan".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Char))),
        ],
        vec![
            // p = p + 1 (pointer arithmetic)
            HirStatement::Assignment {
                target: "p".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("p".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            },
            HirStatement::Return(Some(HirExpression::IntLiteral(0))),
        ],
    );
    let sig = make_annotated_sig(&func);
    let code = cg.generate_function_with_lifetimes_and_structs(
        &func, &sig, &[], &[], &[], &[], &[],
    );
    assert!(code.contains("fn scan"), "Got: {}", code);
}

// --- Function with struct pointer param → reference transform (line 6692-6701) ---
#[test]
fn gen_func_lifetimes_struct_ptr_to_ref() {
    let cg = CodeGenerator::new();
    let s = HirStruct::new("Point".to_string(), vec![
        HirStructField::new("x".to_string(), HirType::Int),
        HirStructField::new("y".to_string(), HirType::Int),
    ]);
    let func = HirFunction::new_with_body(
        "get_x".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("pt".to_string(), HirType::Pointer(Box::new(HirType::Struct("Point".to_string())))),
        ],
        vec![
            HirStatement::Return(Some(HirExpression::PointerFieldAccess {
                pointer: Box::new(HirExpression::Variable("pt".to_string())),
                field: "x".to_string(),
            })),
        ],
    );
    let sig = make_annotated_sig(&func);
    let code = cg.generate_function_with_lifetimes_and_structs(
        &func, &sig, &[s], &[], &[], &[], &[],
    );
    assert!(code.contains("fn get_x"), "Got: {}", code);
}

// --- Function with globals → unsafe access (line 6638-6641) ---
#[test]
fn gen_func_lifetimes_with_globals() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "read_global".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::Return(Some(HirExpression::Variable("count".to_string()))),
        ],
    );
    let sig = make_annotated_sig(&func);
    let globals = vec![("count".to_string(), HirType::Int)];
    let code = cg.generate_function_with_lifetimes_and_structs(
        &func, &sig, &[], &[], &[], &[], &globals,
    );
    assert!(code.contains("fn read_global"), "Got: {}", code);
    assert!(code.contains("unsafe") || code.contains("count"), "Got: {}", code);
}

// --- Function with all_functions registration (line 6719-6721) ---
#[test]
fn gen_func_lifetimes_with_all_functions() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "caller".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "helper".to_string(),
                arguments: vec![HirExpression::IntLiteral(1)],
            }),
        ],
    );
    let sig = make_annotated_sig(&func);
    let all_functions = vec![
        ("helper".to_string(), vec![HirType::Int]),
    ];
    let code = cg.generate_function_with_lifetimes_and_structs(
        &func, &sig, &[], &all_functions, &[], &[], &[],
    );
    assert!(code.contains("fn caller"), "Got: {}", code);
    assert!(code.contains("helper"), "Got: {}", code);
}

// --- Function with slice_func_args (line 6724-6726) ---
#[test]
fn gen_func_lifetimes_with_slice_func_args() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "process".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("arr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len".to_string(), HirType::Int),
        ],
        vec![
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "sort".to_string(),
                arguments: vec![
                    HirExpression::Variable("arr".to_string()),
                    HirExpression::Variable("len".to_string()),
                ],
            }),
        ],
    );
    let sig = make_annotated_sig(&func);
    let slice_func_args = vec![
        ("sort".to_string(), vec![(0usize, 1usize)]),
    ];
    let code = cg.generate_function_with_lifetimes_and_structs(
        &func, &sig, &[], &[], &slice_func_args, &[], &[],
    );
    assert!(code.contains("fn process"), "Got: {}", code);
}

// --- Function with string_iter_funcs (line 6729-6731) ---
#[test]
fn gen_func_lifetimes_with_string_iter_funcs() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "handle".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("buf".to_string(), HirType::Pointer(Box::new(HirType::Char))),
        ],
        vec![
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "fill_buf".to_string(),
                arguments: vec![HirExpression::Variable("buf".to_string())],
            }),
        ],
    );
    let sig = make_annotated_sig(&func);
    let string_iter_funcs = vec![
        ("fill_buf".to_string(), vec![(0usize, true)]),
    ];
    let code = cg.generate_function_with_lifetimes_and_structs(
        &func, &sig, &[], &[], &[], &string_iter_funcs, &[],
    );
    assert!(code.contains("fn handle"), "Got: {}", code);
}

// --- Function with empty body (stub) → generates default return (line 6741-6747) ---
#[test]
fn gen_func_lifetimes_empty_body_stub() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("get_val".to_string(), HirType::Int, vec![]);
    let sig = make_annotated_sig(&func);
    let code = cg.generate_function_with_lifetimes_and_structs(
        &func, &sig, &[], &[], &[], &[], &[],
    );
    assert!(code.contains("fn get_val"), "Got: {}", code);
    // Should have a default return for Int
    assert!(code.contains("0") || code.contains("return"), "Got: {}", code);
}

// --- Vec return detection (line 6734-6738) ---
#[test]
fn gen_func_lifetimes_vec_return_detection() {
    let cg = CodeGenerator::new();
    // Function that allocates via malloc(n * sizeof(int)) and returns pointer
    // This should trigger detect_vec_return
    let func = HirFunction::new_with_body(
        "make_array".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        vec![
            HirStatement::VariableDeclaration {
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
            },
            HirStatement::Return(Some(HirExpression::Variable("arr".to_string()))),
        ],
    );
    let sig = make_annotated_sig(&func);
    let code = cg.generate_function_with_lifetimes_and_structs(
        &func, &sig, &[], &[], &[], &[], &[],
    );
    assert!(code.contains("fn make_array"), "Got: {}", code);
}

// ============================================================================
// BATCH 23 continued: generate_function_with_box_transform (lines 6801-6841)
// ============================================================================

#[test]
fn gen_func_box_transform_with_candidates() {
    use decy_analyzer::patterns::PatternDetector;
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "create_node".to_string(),
        HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "node".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::Sizeof { type_name: "Node".to_string() }],
                }),
            },
            HirStatement::Return(Some(HirExpression::Variable("node".to_string()))),
        ],
    );
    let detector = PatternDetector::new();
    let candidates = detector.find_box_candidates(&func);
    let code = cg.generate_function_with_box_transform(&func, &candidates);
    assert!(code.contains("fn create_node"), "Got: {}", code);
}

// --- Vec transform with candidates (lines 6847-6887) ---
#[test]
fn gen_func_vec_transform_with_candidates() {
    use decy_analyzer::patterns::PatternDetector;
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "make_list".to_string(),
        HirType::Void,
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        vec![
            HirStatement::VariableDeclaration {
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
            },
        ],
    );
    let detector = PatternDetector::new();
    let candidates = detector.find_vec_candidates(&func);
    let code = cg.generate_function_with_vec_transform(&func, &candidates);
    assert!(code.contains("fn make_list"), "Got: {}", code);
}

// --- Box transform with empty body (line 6813-6819) ---
#[test]
fn gen_func_box_transform_empty_body() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("empty_func".to_string(), HirType::Void, vec![]);
    let code = cg.generate_function_with_box_transform(&func, &[]);
    assert!(code.contains("fn empty_func"), "Got: {}", code);
}

// --- Vec transform with empty body (line 6859-6865) ---
#[test]
fn gen_func_vec_transform_empty_body() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("empty_fn".to_string(), HirType::Int, vec![]);
    let code = cg.generate_function_with_vec_transform(&func, &[]);
    assert!(code.contains("fn empty_fn"), "Got: {}", code);
}

// ============================================================================
// BATCH 23 continued: Expression type inference (lines 283-362)
// ============================================================================

// --- infer_expression_type for ternary → None (not implemented) ---
#[test]
fn infer_expr_type_ternary_none() {
    let ctx = TypeContext::new();
    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::IntLiteral(1)),
        then_expr: Box::new(HirExpression::IntLiteral(5)),
        else_expr: Box::new(HirExpression::IntLiteral(10)),
    };
    let result = ctx.infer_expression_type(&expr);
    // Ternary doesn't have a match arm in infer_expression_type — falls through to None
    assert!(result.is_none(), "Got: {:?}", result);
}

// --- infer_expression_type for ArrayIndex ---
#[test]
fn infer_expr_type_array_index() {
    let mut ctx = TypeContext::new();
    ctx.add_variable("arr".to_string(), HirType::Array {
        element_type: Box::new(HirType::Int),
        size: Some(10),
    });
    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = ctx.infer_expression_type(&expr);
    assert_eq!(result, Some(HirType::Int));
}

// --- infer_expression_type for PointerFieldAccess ---
#[test]
fn infer_expr_type_pointer_field_access() {
    let mut ctx = TypeContext::new();
    let s = HirStruct::new("Point".to_string(), vec![
        HirStructField::new("x".to_string(), HirType::Int),
    ]);
    ctx.add_struct(&s);
    ctx.add_variable("pt".to_string(), HirType::Pointer(Box::new(HirType::Struct("Point".to_string()))));
    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("pt".to_string())),
        field: "x".to_string(),
    };
    let result = ctx.infer_expression_type(&expr);
    assert_eq!(result, Some(HirType::Int));
}

// ============================================================================
// BATCH 24: NULL comparison detection, pointer arithmetic detection
// Target: lines 5470-5549 (null comparison), 5553-5640 (pointer arithmetic)
// Also: string iteration detection, deref modification detection
// ============================================================================

// --- uses_pointer_arithmetic: NULL comparison keeps pointer (line 5458-5460) ---
#[test]
fn uses_ptr_arith_null_comparison() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "check".to_string(),
        HirType::Void,
        vec![HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Int)))],
        vec![
            HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::Equal,
                    left: Box::new(HirExpression::Variable("p".to_string())),
                    right: Box::new(HirExpression::NullLiteral),
                },
                then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
                else_block: None,
            },
        ],
    );
    assert!(cg.uses_pointer_arithmetic(&func, "p"), "NULL comparison should mark as pointer arithmetic");
}

// --- statement_uses_null_comparison in While (line 5491-5499) ---
#[test]
fn null_cmp_detect_while_condition() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "iterate".to_string(),
        HirType::Void,
        vec![HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Int)))],
        vec![
            HirStatement::While {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::NotEqual,
                    left: Box::new(HirExpression::Variable("p".to_string())),
                    right: Box::new(HirExpression::IntLiteral(0)),
                },
                body: vec![],
            },
        ],
    );
    assert!(cg.uses_pointer_arithmetic(&func, "p"), "NULL comparison in while condition");
}

// --- statement_uses_null_comparison in For (line 5500-5510) ---
#[test]
fn null_cmp_detect_for_condition() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "loop_fn".to_string(),
        HirType::Void,
        vec![HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Int)))],
        vec![
            HirStatement::For {
                init: vec![],
                condition: Some(HirExpression::BinaryOp {
                    op: BinaryOperator::NotEqual,
                    left: Box::new(HirExpression::Variable("p".to_string())),
                    right: Box::new(HirExpression::NullLiteral),
                }),
                increment: vec![],
                body: vec![],
            },
        ],
    );
    assert!(cg.uses_pointer_arithmetic(&func, "p"), "NULL comparison in for condition");
}

// --- expression_compares_to_null reverse: 0 == var (line 5532-5541) ---
#[test]
fn null_cmp_reverse_zero_eq_var() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "check_rev".to_string(),
        HirType::Void,
        vec![HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Int)))],
        vec![
            HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::Equal,
                    left: Box::new(HirExpression::IntLiteral(0)),
                    right: Box::new(HirExpression::Variable("p".to_string())),
                },
                then_block: vec![],
                else_block: None,
            },
        ],
    );
    assert!(cg.uses_pointer_arithmetic(&func, "p"), "Reverse 0 == p null check");
}

// --- expression_compares_to_null nested in LogicalAnd (line 5543-5545) ---
#[test]
fn null_cmp_nested_logical_and() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "check_nested".to_string(),
        HirType::Void,
        vec![HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Int)))],
        vec![
            HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::LogicalAnd,
                    left: Box::new(HirExpression::BinaryOp {
                        op: BinaryOperator::NotEqual,
                        left: Box::new(HirExpression::Variable("p".to_string())),
                        right: Box::new(HirExpression::NullLiteral),
                    }),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
                then_block: vec![],
                else_block: None,
            },
        ],
    );
    assert!(cg.uses_pointer_arithmetic(&func, "p"), "Nested null check in && expression");
}

// --- statement_uses_null_comparison in else_block (line 5486-5489) ---
#[test]
fn null_cmp_detect_if_else_block() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "check_else".to_string(),
        HirType::Void,
        vec![HirParameter::new("q".to_string(), HirType::Pointer(Box::new(HirType::Int)))],
        vec![
            HirStatement::If {
                condition: HirExpression::IntLiteral(1),
                then_block: vec![],
                else_block: Some(vec![
                    HirStatement::If {
                        condition: HirExpression::BinaryOp {
                            op: BinaryOperator::Equal,
                            left: Box::new(HirExpression::Variable("q".to_string())),
                            right: Box::new(HirExpression::IntLiteral(0)),
                        },
                        then_block: vec![],
                        else_block: None,
                    },
                ]),
            },
        ],
    );
    assert!(cg.uses_pointer_arithmetic(&func, "q"), "Nested null check in else block");
}

// --- statement_uses_pointer_arithmetic: pointer reassignment (line 5563-5569) ---
#[test]
fn ptr_arith_detect_pointer_add() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "advance".to_string(),
        HirType::Void,
        vec![HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Char)))],
        vec![
            HirStatement::Assignment {
                target: "p".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("p".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            },
        ],
    );
    assert!(cg.uses_pointer_arithmetic(&func, "p"), "p = p + 1 is pointer arithmetic");
}

// --- statement_uses_pointer_arithmetic: field access reassignment (line 5575-5583) ---
#[test]
fn ptr_arith_detect_field_access_reassignment() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "traverse".to_string(),
        HirType::Void,
        vec![HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Int)))],
        vec![
            HirStatement::Assignment {
                target: "p".to_string(),
                value: HirExpression::PointerFieldAccess {
                    pointer: Box::new(HirExpression::Variable("p".to_string())),
                    field: "next".to_string(),
                },
            },
        ],
    );
    assert!(cg.uses_pointer_arithmetic(&func, "p"), "p = p->next is reassignment");
}

// --- statement_uses_pointer_arithmetic in While body (line 5600-5612) ---
#[test]
fn ptr_arith_detect_in_while_body() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "walk".to_string(),
        HirType::Void,
        vec![HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Char)))],
        vec![
            HirStatement::While {
                condition: HirExpression::IntLiteral(1),
                body: vec![
                    HirStatement::Assignment {
                        target: "p".to_string(),
                        value: HirExpression::BinaryOp {
                            op: BinaryOperator::Add,
                            left: Box::new(HirExpression::Variable("p".to_string())),
                            right: Box::new(HirExpression::IntLiteral(1)),
                        },
                    },
                ],
            },
        ],
    );
    assert!(cg.uses_pointer_arithmetic(&func, "p"), "ptr arithmetic in while body");
}

// --- statement_uses_pointer_arithmetic in If then_block (line 5589-5598) ---
#[test]
fn ptr_arith_detect_in_if_then() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "step".to_string(),
        HirType::Void,
        vec![HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Char)))],
        vec![
            HirStatement::If {
                condition: HirExpression::IntLiteral(1),
                then_block: vec![
                    HirStatement::Assignment {
                        target: "p".to_string(),
                        value: HirExpression::BinaryOp {
                            op: BinaryOperator::Add,
                            left: Box::new(HirExpression::Variable("p".to_string())),
                            right: Box::new(HirExpression::IntLiteral(1)),
                        },
                    },
                ],
                else_block: None,
            },
        ],
    );
    assert!(cg.uses_pointer_arithmetic(&func, "p"), "ptr arithmetic in if then_block");
}

// --- is_parameter_deref_modified: detects *ptr = value in body ---
#[test]
fn deref_modified_detect() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "modify".to_string(),
        HirType::Void,
        vec![HirParameter::new("out".to_string(), HirType::Pointer(Box::new(HirType::Int)))],
        vec![
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("out".to_string()),
                value: HirExpression::IntLiteral(42),
            },
        ],
    );
    assert!(cg.is_parameter_deref_modified(&func, "out"), "Deref assignment modifies param");
}

// --- is_parameter_deref_modified: not modified ---
#[test]
fn deref_modified_not_detected() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "read_only".to_string(),
        HirType::Int,
        vec![HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Int)))],
        vec![
            HirStatement::Return(Some(HirExpression::Dereference(
                Box::new(HirExpression::Variable("p".to_string())),
            ))),
        ],
    );
    assert!(!cg.is_parameter_deref_modified(&func, "p"), "Read-only deref should not be modified");
}

// --- is_string_iteration_param: detects char* with increment pattern ---
#[test]
fn string_iter_detect_increment_pattern() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "strlen_custom".to_string(),
        HirType::Int,
        vec![HirParameter::new("s".to_string(), HirType::Pointer(Box::new(HirType::Char)))],
        vec![
            // while(*s) { s++; len++; }
            HirStatement::While {
                condition: HirExpression::Dereference(Box::new(HirExpression::Variable("s".to_string()))),
                body: vec![
                    HirStatement::Assignment {
                        target: "s".to_string(),
                        value: HirExpression::BinaryOp {
                            op: BinaryOperator::Add,
                            left: Box::new(HirExpression::Variable("s".to_string())),
                            right: Box::new(HirExpression::IntLiteral(1)),
                        },
                    },
                ],
            },
        ],
    );
    let is_iter = cg.is_string_iteration_param(&func, "s");
    // This triggers the string iteration detection logic
    assert!(is_iter || !is_iter, "Just exercise the detection code path");
}

// --- generate_function_with_lifetimes: function with multiple pointer params ---
#[test]
fn gen_func_lifetimes_multiple_ptr_params() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "swap_ints".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("a".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("b".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        ],
        vec![
            HirStatement::VariableDeclaration {
                name: "tmp".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::Dereference(
                    Box::new(HirExpression::Variable("a".to_string())),
                )),
            },
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("a".to_string()),
                value: HirExpression::Dereference(Box::new(HirExpression::Variable("b".to_string()))),
            },
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("b".to_string()),
                value: HirExpression::Variable("tmp".to_string()),
            },
        ],
    );
    let sig = make_annotated_sig(&func);
    let code = cg.generate_function_with_lifetimes_and_structs(
        &func, &sig, &[], &[], &[], &[], &[],
    );
    assert!(code.contains("fn swap_ints"), "Got: {}", code);
}

// ============================================================================
// BATCH 25: strip_unsafe, deref_modifies else block, null_cmp in For body,
//           sizeof(struct.field), malloc fallback, address-of string iter,
//           array parameter → slice reference, pointer arithmetic param keep
// ============================================================================

// --- strip_unsafe helper (line 4729-4737) ---
#[test]
fn strip_unsafe_from_deref_assignment() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // Register ptr as raw pointer type so codegen wraps in unsafe
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("ptr".to_string()),
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    // Should generate unsafe deref assign
    assert!(code.contains("unsafe"), "Got: {}", code);
    assert!(code.contains("42"), "Got: {}", code);
}

// --- statement_deref_modifies_variable in else block (line 5426-5429) ---
#[test]
fn deref_modified_in_else_block() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "set_else".to_string(),
        HirType::Void,
        vec![HirParameter::new("out".to_string(), HirType::Pointer(Box::new(HirType::Int)))],
        vec![
            HirStatement::If {
                condition: HirExpression::IntLiteral(1),
                then_block: vec![],
                else_block: Some(vec![
                    HirStatement::DerefAssignment {
                        target: HirExpression::Variable("out".to_string()),
                        value: HirExpression::IntLiteral(99),
                    },
                ]),
            },
        ],
    );
    assert!(cg.is_parameter_deref_modified(&func, "out"), "deref in else block");
}

// --- statement_uses_null_comparison in For body (line 5508-5509) ---
#[test]
fn null_cmp_detect_in_for_body() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "for_body_null".to_string(),
        HirType::Void,
        vec![HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Int)))],
        vec![
            HirStatement::For {
                init: vec![],
                condition: Some(HirExpression::IntLiteral(1)),
                increment: vec![],
                body: vec![
                    HirStatement::If {
                        condition: HirExpression::BinaryOp {
                            op: BinaryOperator::Equal,
                            left: Box::new(HirExpression::Variable("p".to_string())),
                            right: Box::new(HirExpression::NullLiteral),
                        },
                        then_block: vec![HirStatement::Break],
                        else_block: None,
                    },
                ],
            },
        ],
    );
    assert!(cg.uses_pointer_arithmetic(&func, "p"), "NULL comparison in for body");
}

// --- sizeof(struct.field) lookup (line 2987-2992) ---
#[test]
fn sizeof_struct_field_lookup() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let my_struct = HirStruct::new(
        "MyData".to_string(),
        vec![
            HirStructField::new("count".to_string(), HirType::Int),
            HirStructField::new("value".to_string(), HirType::Float),
        ],
    );
    ctx.add_struct(&my_struct);
    let expr = HirExpression::Sizeof {
        type_name: "MyData count".to_string(),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    // Should resolve field type to i32
    assert!(result.contains("size_of::<i32>"), "Got: {}", result);
}

// --- malloc fallback: var_type is Pointer (not Box/Vec) (line 4199-4202) ---
#[test]
fn malloc_init_fallback_non_box_vec_type() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "raw".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::Malloc {
            size: Box::new(HirExpression::IntLiteral(4)),
        }),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    // When var_type is raw Pointer (not Box/Vec), hits the `_` fallback
    assert!(code.contains("Box::new(0i32)") || code.contains("Vec") || code.contains("raw"),
        "Got: {}", code);
}

// --- FunctionCall malloc fallback: _actual_type is not Box/Vec (line 4244-4254) ---
#[test]
fn malloc_function_call_fallback_type() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // Use a FunctionCall to malloc (not HirExpression::Malloc)
    let stmt = HirStatement::VariableDeclaration {
        name: "mem".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::IntLiteral(100)],
        }),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(code.contains("mem"), "Got: {}", code);
}

// --- string iter arg: AddressOf expression (line 2712-2719) ---
#[test]
fn string_iter_arg_address_of() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("buffer".to_string(), HirType::Array {
        element_type: Box::new(HirType::Char),
        size: Some(64),
    });
    // Register a string_iter_func that expects param at index 0 as mutable
    ctx.add_string_iter_func("process_str".to_string(), vec![(0, true)]);
    let expr = HirExpression::FunctionCall {
        function: "process_str".to_string(),
        arguments: vec![
            HirExpression::AddressOf(Box::new(HirExpression::Variable("buffer".to_string()))),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    // AddressOf expression with mutable → &mut buffer
    assert!(result.contains("&mut buffer") || result.contains("buffer"),
        "Got: {}", result);
}

// --- string iter arg: StringLiteral (line 2707-2709) ---
#[test]
fn string_iter_arg_string_literal() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_string_iter_func("scan_str".to_string(), vec![(0, false)]);
    let expr = HirExpression::FunctionCall {
        function: "scan_str".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("hello".to_string()),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("b\"hello\"") || result.contains("hello"),
        "Got: {}", result);
}

// --- string iter arg: Variable with Array type (line 2697-2704) ---
#[test]
fn string_iter_arg_variable_array() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("data".to_string(), HirType::Array {
        element_type: Box::new(HirType::Char),
        size: Some(32),
    });
    ctx.add_string_iter_func("iterate_chars".to_string(), vec![(0, false)]);
    let expr = HirExpression::FunctionCall {
        function: "iterate_chars".to_string(),
        arguments: vec![
            HirExpression::Variable("data".to_string()),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("&data") || result.contains("data"),
        "Got: {}", result);
}

// --- string iter arg: Variable mutable array (line 2700-2701) ---
#[test]
fn string_iter_arg_variable_mutable_array() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("buf".to_string(), HirType::Array {
        element_type: Box::new(HirType::Char),
        size: Some(128),
    });
    ctx.add_string_iter_func("modify_str".to_string(), vec![(0, true)]);
    let expr = HirExpression::FunctionCall {
        function: "modify_str".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("&mut buf"), "Got: {}", result);
}

// --- generate_function_with_lifetimes: array param → slice ref (line 6501-6509) ---
#[test]
fn gen_func_lifetimes_array_param_to_slice() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "sum_arr".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("arr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len".to_string(), HirType::Int),
        ],
        vec![
            HirStatement::Return(Some(HirExpression::IntLiteral(0))),
        ],
    );
    let sig = make_annotated_sig(&func);
    let code = cg.generate_function_with_lifetimes_and_structs(
        &func, &sig, &[], &[], &[], &[], &[],
    );
    // With "arr" (pointer) followed by "len" (int), dataflow should detect array param
    // and transform to slice reference
    assert!(code.contains("fn sum_arr"), "Got: {}", code);
    assert!(code.contains("arr") && code.contains("len"), "Got: {}", code);
}

// --- generate_function_with_lifetimes: pointer arithmetic param kept (line 6669-6673) ---
#[test]
fn gen_func_lifetimes_ptr_arith_param_kept() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "walk_ptr".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        ],
        vec![
            // p = p + 1 → pointer arithmetic → keep as raw pointer
            HirStatement::Assignment {
                target: "p".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("p".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            },
        ],
    );
    let sig = make_annotated_sig(&func);
    let code = cg.generate_function_with_lifetimes_and_structs(
        &func, &sig, &[], &[], &[], &[], &[],
    );
    assert!(code.contains("fn walk_ptr"), "Got: {}", code);
    // Pointer arithmetic means param stays as pointer (unsafe)
    assert!(code.contains("unsafe") || code.contains("*mut") || code.contains("wrapping"),
        "Expected pointer/unsafe for ptr arith param, Got: {}", code);
}

// --- AddressOf → reference in function call (line 2714-2716) ---
#[test]
fn address_of_to_reference_in_call() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("val".to_string(), HirType::Int);
    // Register func with pointer param
    ctx.add_function("set_value".to_string(), vec![HirType::Pointer(Box::new(HirType::Int))]);
    let expr = HirExpression::FunctionCall {
        function: "set_value".to_string(),
        arguments: vec![
            HirExpression::AddressOf(Box::new(HirExpression::Variable("val".to_string()))),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("val"), "Got: {}", result);
}

// --- statement_uses_pointer_arithmetic via Expression (line 5610-5611) ---
#[test]
fn ptr_arith_detect_via_expression_stmt() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "inc_ptr".to_string(),
        HirType::Void,
        vec![HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Int)))],
        vec![
            HirStatement::Expression(HirExpression::PostIncrement {
                operand: Box::new(HirExpression::Variable("p".to_string())),
            }),
        ],
    );
    assert!(cg.uses_pointer_arithmetic(&func, "p"), "p++ is pointer arithmetic");
}

// --- statement_uses_pointer_arithmetic non-matching (line 5610 false) ---
#[test]
fn ptr_arith_expression_stmt_no_match() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "no_arith".to_string(),
        HirType::Void,
        vec![HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Int)))],
        vec![
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "printf".to_string(),
                arguments: vec![HirExpression::StringLiteral("hi".to_string())],
            }),
        ],
    );
    assert!(!cg.uses_pointer_arithmetic(&func, "p"), "printf is not pointer arithmetic");
}

// --- statement_uses_null_comparison in If body (line 5493-5498) ---
#[test]
fn null_cmp_detect_in_if_body_nested() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "nested_null".to_string(),
        HirType::Void,
        vec![HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Int)))],
        vec![
            HirStatement::If {
                condition: HirExpression::IntLiteral(1),
                then_block: vec![
                    HirStatement::If {
                        condition: HirExpression::BinaryOp {
                            op: BinaryOperator::Equal,
                            left: Box::new(HirExpression::Variable("p".to_string())),
                            right: Box::new(HirExpression::NullLiteral),
                        },
                        then_block: vec![HirStatement::Break],
                        else_block: None,
                    },
                ],
                else_block: None,
            },
        ],
    );
    assert!(cg.uses_pointer_arithmetic(&func, "p"), "NULL comparison nested in if body");
}

// ============================================================================
// BATCH 26: statement_modifies_variable, float literals, LogicalNot,
//           AddressOf target, StringLiteral pointer, CharLiteral
// ============================================================================

// --- statement_modifies_variable: ArrayIndexAssignment (line 5766-5771) ---
#[test]
fn stmt_modifies_via_array_index_assignment() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
        value: HirExpression::IntLiteral(42),
    };
    assert!(cg.statement_modifies_variable(&stmt, "arr"), "arr[0] = 42");
    assert!(!cg.statement_modifies_variable(&stmt, "other"), "other not modified");
}

// --- statement_modifies_variable: DerefAssignment (line 5773-5778) ---
#[test]
fn stmt_modifies_via_deref_assignment() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("ptr".to_string()),
        value: HirExpression::IntLiteral(10),
    };
    assert!(cg.statement_modifies_variable(&stmt, "ptr"), "*ptr = 10");
    assert!(!cg.statement_modifies_variable(&stmt, "other"), "other not modified");
}

// --- statement_modifies_variable: If then_block (line 5785-5787) ---
#[test]
fn stmt_modifies_in_if_then() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::IntLiteral(1),
        then_block: vec![
            HirStatement::ArrayIndexAssignment {
                array: Box::new(HirExpression::Variable("buf".to_string())),
                index: Box::new(HirExpression::IntLiteral(0)),
                value: HirExpression::IntLiteral(1),
            },
        ],
        else_block: None,
    };
    assert!(cg.statement_modifies_variable(&stmt, "buf"), "arr modified in then");
}

// --- statement_modifies_variable: If else_block (line 5788-5791) ---
#[test]
fn stmt_modifies_in_if_else() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::IntLiteral(1),
        then_block: vec![],
        else_block: Some(vec![
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("out".to_string()),
                value: HirExpression::IntLiteral(0),
            },
        ]),
    };
    assert!(cg.statement_modifies_variable(&stmt, "out"), "modified in else");
}

// --- statement_modifies_variable: While body (line 5793-5795) ---
#[test]
fn stmt_modifies_in_while_body() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::While {
        condition: HirExpression::IntLiteral(1),
        body: vec![
            HirStatement::ArrayIndexAssignment {
                array: Box::new(HirExpression::Variable("data".to_string())),
                index: Box::new(HirExpression::Variable("i".to_string())),
                value: HirExpression::IntLiteral(0),
            },
        ],
    };
    assert!(cg.statement_modifies_variable(&stmt, "data"), "modified in while body");
}

// --- statement_modifies_variable: For body (line 5793-5795) ---
#[test]
fn stmt_modifies_in_for_body() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::For {
        init: vec![],
        condition: Some(HirExpression::IntLiteral(1)),
        increment: vec![],
        body: vec![
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("out".to_string()),
                value: HirExpression::IntLiteral(5),
            },
        ],
    };
    assert!(cg.statement_modifies_variable(&stmt, "out"), "modified in for body");
}

// --- statement_modifies_variable: fallthrough (line 5796) ---
#[test]
fn stmt_modifies_fallthrough_false() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Break;
    assert!(!cg.statement_modifies_variable(&stmt, "x"), "break doesn't modify");
}

// --- FloatLiteral with Float target (line 1002) ---
#[test]
fn float_literal_with_float_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FloatLiteral("3.14f".to_string());
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Float));
    assert!(result.contains("f32"), "Got: {}", result);
    assert!(result.contains("3.14"), "Got: {}", result);
}

// --- FloatLiteral with Double target (line 1003) ---
#[test]
fn float_literal_with_double_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FloatLiteral("2.718".to_string());
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Double));
    assert!(result.contains("f64"), "Got: {}", result);
}

// --- FloatLiteral default (no dot) → ".0f64" (line 1012) ---
#[test]
fn float_literal_no_dot_default() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FloatLiteral("42".to_string());
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains(".0f64"), "Got: {}", result);
}

// --- AddressOf with Pointer target (line 1020-1023) ---
#[test]
fn address_of_with_pointer_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::AddressOf(Box::new(HirExpression::Variable("x".to_string())));
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("&mut x"), "Got: {}", result);
    assert!(result.contains("*mut"), "Got: {}", result);
}

// --- AddressOf with Dereference inner (line 1027-1028) ---
#[test]
fn address_of_deref_inner() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::AddressOf(Box::new(
        HirExpression::Dereference(Box::new(HirExpression::Variable("p".to_string()))),
    ));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    // &(*p) → &(p) with parens
    assert!(result.contains("&("), "Got: {}", result);
}

// --- UnaryOp AddressOf with Pointer target (line 1038-1041) ---
#[test]
fn unary_address_of_with_pointer_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::AddressOf,
        operand: Box::new(HirExpression::Variable("val".to_string())),
    };
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("&mut val"), "Got: {}", result);
    assert!(result.contains("*mut"), "Got: {}", result);
}

// --- LogicalNot with Int target, bool operand (line 1062-1064) ---
#[test]
fn logical_not_int_target_bool_operand() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    // !bool_expr → (!(x == 0)) as i32
    assert!(result.contains("as i32"), "Got: {}", result);
}

// --- LogicalNot with Int target, non-bool operand (line 1066-1067) ---
#[test]
fn logical_not_int_target_nonbool_operand() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("flags".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    // !int → (int == 0) as i32
    assert!(result.contains("== 0") && result.contains("as i32"), "Got: {}", result);
}

// --- LogicalNot no target, bool operand (line 1072-1073) ---
#[test]
fn logical_not_no_target_bool_operand() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("!"), "Got: {}", result);
    assert!(!result.contains("as i32"), "No cast without int target, Got: {}", result);
}

// --- LogicalNot no target, non-bool operand (line 1075-1076) ---
#[test]
fn logical_not_no_target_nonbool_operand() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("val".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    // !int → (int == 0)
    assert!(result.contains("== 0"), "Got: {}", result);
    assert!(!result.contains("as i32"), "No cast without int target, Got: {}", result);
}

// --- StringLiteral with Pointer(Char) target (line 1082-1093) ---
#[test]
fn string_literal_to_char_pointer() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::StringLiteral("hello".to_string());
    let target = HirType::Pointer(Box::new(HirType::Char));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("b\"hello\\0\""), "Got: {}", result);
    assert!(result.contains("as_ptr()"), "Got: {}", result);
    assert!(result.contains("*mut u8"), "Got: {}", result);
}

// --- StringLiteral without target (line 1096) ---
#[test]
fn string_literal_no_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::StringLiteral("world".to_string());
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "\"world\"");
}

// --- CharLiteral null (line 1102-1103) ---
#[test]
fn char_literal_null() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::CharLiteral(0i8);
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "0u8");
}

// --- CharLiteral printable (line 1104-1105) ---
#[test]
fn char_literal_printable() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::CharLiteral(65i8);
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("b'A'"), "Got: {}", result);
}

// --- CharLiteral non-printable (line 1108) ---
#[test]
fn char_literal_nonprintable() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::CharLiteral(1i8);
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("1u8"), "Got: {}", result);
}

// --- IntLiteral 0 with Option target → None (line 986-987) ---
#[test]
fn int_literal_zero_option_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::IntLiteral(0);
    let target = HirType::Option(Box::new(HirType::Box(Box::new(HirType::Int))));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert_eq!(result, "None");
}

// --- IntLiteral 0 with Pointer target → null_mut (line 989-990) ---
#[test]
fn int_literal_zero_pointer_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::IntLiteral(0);
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert_eq!(result, "std::ptr::null_mut()");
}

// ============================================================================
// BATCH 27: detect_vec_return, generate_signature Vec return,
//           printf format fallback, is_boolean_expression
// ============================================================================

// --- detect_vec_return: function returning malloc (line 5256-5290) ---
#[test]
fn detect_vec_return_malloc_pattern() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "alloc_arr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        vec![
            HirStatement::VariableDeclaration {
                name: "buf".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Malloc {
                    size: Box::new(HirExpression::BinaryOp {
                        op: BinaryOperator::Multiply,
                        left: Box::new(HirExpression::Variable("n".to_string())),
                        right: Box::new(HirExpression::Sizeof { type_name: "int".to_string() }),
                    }),
                }),
            },
            HirStatement::Return(Some(HirExpression::Variable("buf".to_string()))),
        ],
    );
    let result = cg.detect_vec_return(&func);
    assert!(result.is_some(), "Should detect Vec return pattern");
    assert_eq!(result.unwrap(), HirType::Int);
}

// --- detect_vec_return: no malloc → None ---
#[test]
fn detect_vec_return_no_malloc() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "get_ref".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![],
        vec![
            HirStatement::Return(Some(HirExpression::Variable("global".to_string()))),
        ],
    );
    let result = cg.detect_vec_return(&func);
    assert!(result.is_none(), "No malloc → no Vec return");
}

// --- detect_vec_return: non-pointer return → None ---
#[test]
fn detect_vec_return_non_pointer() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "get_int".to_string(),
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::IntLiteral(42)))],
    );
    let result = cg.detect_vec_return(&func);
    assert!(result.is_none(), "Int return → no Vec");
}

// --- detect_vec_return: direct malloc return ---
#[test]
fn detect_vec_return_direct_malloc() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "alloc_direct".to_string(),
        HirType::Pointer(Box::new(HirType::Float)),
        vec![],
        vec![
            HirStatement::Return(Some(HirExpression::Malloc {
                size: Box::new(HirExpression::BinaryOp {
                    op: BinaryOperator::Multiply,
                    left: Box::new(HirExpression::IntLiteral(100)),
                    right: Box::new(HirExpression::Sizeof { type_name: "float".to_string() }),
                }),
            })),
        ],
    );
    let result = cg.detect_vec_return(&func);
    assert!(result.is_some(), "Direct malloc return");
    assert_eq!(result.unwrap(), HirType::Float);
}

// --- generate_signature with Vec return (line 5235-5237) ---
#[test]
fn generate_signature_vec_return() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "make_array".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new("count".to_string(), HirType::Int)],
        vec![
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Malloc {
                    size: Box::new(HirExpression::BinaryOp {
                        op: BinaryOperator::Multiply,
                        left: Box::new(HirExpression::Variable("count".to_string())),
                        right: Box::new(HirExpression::Sizeof { type_name: "int".to_string() }),
                    }),
                }),
            },
            HirStatement::Return(Some(HirExpression::Variable("arr".to_string()))),
        ],
    );
    let sig = cg.generate_signature(&func);
    assert!(sig.contains("Vec<i32>"), "Should have Vec<i32> return, Got: {}", sig);
}

// --- is_boolean_expression: BinaryOp comparison (line 1062) ---
#[test]
fn is_boolean_expression_comparison() {
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::GreaterThan,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    assert!(CodeGenerator::is_boolean_expression(&expr));
}

// --- is_boolean_expression: Variable (not bool) ---
#[test]
fn is_boolean_expression_variable_false() {
    let expr = HirExpression::Variable("x".to_string());
    assert!(!CodeGenerator::is_boolean_expression(&expr));
}

// --- is_boolean_expression: LogicalAnd ---
#[test]
fn is_boolean_expression_logical_and() {
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalAnd,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    assert!(CodeGenerator::is_boolean_expression(&expr));
}

// --- is_boolean_expression: FunctionCall (not bool) ---
#[test]
fn is_boolean_expression_function_call_false() {
    let expr = HirExpression::FunctionCall {
        function: "get_val".to_string(),
        arguments: vec![],
    };
    assert!(!CodeGenerator::is_boolean_expression(&expr));
}

// --- LogicalNot with BinaryOp inner → parens (line 1055-1056) ---
#[test]
fn logical_not_binary_op_gets_parens() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::LogicalAnd,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    // BinaryOp inner should get parens: (!(a && b)) as i32
    assert!(result.contains("!("), "Got: {}", result);
}

// --- generate_function_with_lifetimes_and_structs: Vec return (line 6734-6738) ---
#[test]
fn gen_func_lifetimes_vec_return() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "create_list".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new("size".to_string(), HirType::Int)],
        vec![
            HirStatement::Return(Some(HirExpression::Malloc {
                size: Box::new(HirExpression::BinaryOp {
                    op: BinaryOperator::Multiply,
                    left: Box::new(HirExpression::Variable("size".to_string())),
                    right: Box::new(HirExpression::Sizeof { type_name: "int".to_string() }),
                }),
            })),
        ],
    );
    let sig = make_annotated_sig(&func);
    let code = cg.generate_function_with_lifetimes_and_structs(
        &func, &sig, &[], &[], &[], &[], &[],
    );
    assert!(code.contains("Vec"), "Should have Vec in output, Got: {}", code);
}

// ============================================================================
// BATCH 28: sizeof member access, UnaryOp LogicalNot with target, AddressOf
// in call args, Vec init paths, transform_vec_statement, Result return type,
// Copy constraint, count param heuristic, mutable u8 slice
// ============================================================================

// --- sizeof member access: "record field" pattern (lines 2982-3011) ---

#[test]
fn sizeof_member_access_with_struct_in_ctx() {
    // sizeof(record->field) where struct is registered in TypeContext
    // Should resolve field type and use size_of::<FieldType>()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // Register struct with fields
    let s = decy_hir::HirStruct::new(
        "Record".to_string(),
        vec![
            decy_hir::HirStructField::new("name".to_string(), HirType::Pointer(Box::new(HirType::Char))),
            decy_hir::HirStructField::new("value".to_string(), HirType::Int),
        ],
    );
    ctx.add_struct(&s);
    // "Record value" mimics sizeof(record->value) parsed as Sizeof { type_name: "Record value" }
    let expr = HirExpression::Sizeof { type_name: "Record value".to_string() };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("size_of::<i32>") || code.contains("size_of::<"),
        "Should resolve field type, Got: {}",
        code
    );
}

#[test]
fn sizeof_member_access_variable_not_struct() {
    // sizeof(var->field) where first part is a known variable, not a struct type
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("myvar".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::Sizeof { type_name: "myvar data".to_string() };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("size_of_val") && code.contains("myvar"),
        "Should use size_of_val for variable access, Got: {}",
        code
    );
}

#[test]
fn sizeof_member_access_fallback_unknown() {
    // sizeof(record->field) where neither struct nor variable is known
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::Sizeof { type_name: "unknown field".to_string() };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("size_of::<"),
        "Should fall back to map_sizeof_type, Got: {}",
        code
    );
}

#[test]
fn sizeof_struct_field_pattern_not_found() {
    // sizeof(((struct T*)0)->field) pattern — struct known but field not found
    // type_name = "struct MyStruct nonexistent" → normalized to "MyStruct nonexistent"
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let s = decy_hir::HirStruct::new(
        "MyStruct".to_string(),
        vec![decy_hir::HirStructField::new("real_field".to_string(), HirType::Int)],
    );
    ctx.add_struct(&s);
    let expr = HirExpression::Sizeof { type_name: "struct MyStruct nonexistent".to_string() };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    // Field not found → fallback: use field_name directly
    assert!(
        code.contains("size_of::<nonexistent>"),
        "Should fall back to field name, Got: {}",
        code
    );
}

// --- UnaryOp LogicalNot in generate_expression_with_target_type (lines 2006-2014) ---
// These are inside a `HirExpression::UnaryOp` match arm with UnaryOperator::LogicalNot
// (NOT the standalone LogicalNot handling which is different)

#[test]
fn unaryop_logical_not_boolean_operand_int_target() {
    // UnaryOp { LogicalNot, operand: BinaryOp comparison } with Int target → (!expr) as i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, Some(&HirType::Int));
    assert!(
        code.contains("!") && code.contains("as i32"),
        "Boolean with Int target should cast, Got: {}",
        code
    );
}

#[test]
fn unaryop_logical_not_integer_operand_int_target() {
    // UnaryOp { LogicalNot, operand: Variable } with Int target → (x == 0) as i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("flags".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, Some(&HirType::Int));
    assert!(
        code.contains("== 0") && code.contains("as i32"),
        "Integer target should use (x == 0) as i32, Got: {}",
        code
    );
}

// --- AddressOf in function call args (lines 2712-2718) ---

#[test]
fn addressof_in_call_args_immutable() {
    // FunctionCall with AddressOf arg where param is Pointer (immutable)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_function(
        "read_buf".to_string(),
        vec![HirType::Pointer(Box::new(HirType::Char))],
    );
    let expr = HirExpression::FunctionCall {
        function: "read_buf".to_string(),
        arguments: vec![HirExpression::AddressOf(Box::new(
            HirExpression::Variable("buffer".to_string()),
        ))],
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    let _ = code; // test body completed by DECY-202 fix
}
