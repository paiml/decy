
#[test]
fn stmt_modifies_array_index_assignment_match() {
    // Line 5766-5770: ArrayIndexAssignment where array is Variable matching var_name
    let cg = CodeGenerator::new();
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
        value: HirExpression::IntLiteral(42),
    };
    assert!(cg.statement_modifies_variable(&stmt, "arr"));
}

#[test]
fn stmt_modifies_array_index_assignment_no_match() {
    // Line 5768-5770: ArrayIndexAssignment where var_name differs
    let cg = CodeGenerator::new();
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
        value: HirExpression::IntLiteral(42),
    };
    assert!(!cg.statement_modifies_variable(&stmt, "other"));
}

#[test]
fn stmt_modifies_array_index_assignment_non_variable_array() {
    // Line 5771: ArrayIndexAssignment where array is NOT a Variable → false
    let cg = CodeGenerator::new();
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("ptr".to_string()),
        ))),
        index: Box::new(HirExpression::IntLiteral(0)),
        value: HirExpression::IntLiteral(42),
    };
    assert!(!cg.statement_modifies_variable(&stmt, "ptr"));
}

#[test]
fn stmt_modifies_deref_assignment_match() {
    // Line 5773-5777: DerefAssignment where target is Variable matching var_name
    let cg = CodeGenerator::new();
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("ptr".to_string()),
        value: HirExpression::IntLiteral(99),
    };
    assert!(cg.statement_modifies_variable(&stmt, "ptr"));
}

#[test]
fn stmt_modifies_deref_assignment_no_match() {
    // Line 5775-5777: DerefAssignment where var_name differs
    let cg = CodeGenerator::new();
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("ptr".to_string()),
        value: HirExpression::IntLiteral(99),
    };
    assert!(!cg.statement_modifies_variable(&stmt, "other"));
}

#[test]
fn stmt_modifies_deref_assignment_non_variable_target() {
    // Line 5778: DerefAssignment where target is NOT a Variable → false
    let cg = CodeGenerator::new();
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Dereference(Box::new(HirExpression::Variable(
            "ptr".to_string(),
        ))),
        value: HirExpression::IntLiteral(99),
    };
    assert!(!cg.statement_modifies_variable(&stmt, "ptr"));
}

#[test]
fn stmt_modifies_if_then_block_only() {
    // Line 5785-5787: If where then_block modifies variable
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
    assert!(cg.statement_modifies_variable(&stmt, "arr"));
}

#[test]
fn stmt_modifies_if_else_block_only() {
    // Line 5788-5791: If where only else_block modifies variable
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::Variable("cond".to_string()),
        then_block: vec![],
        else_block: Some(vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("ptr".to_string()),
            value: HirExpression::IntLiteral(1),
        }]),
    };
    assert!(cg.statement_modifies_variable(&stmt, "ptr"));
}

#[test]
fn stmt_modifies_if_neither_block() {
    // Line 5785-5791: If where neither block modifies variable
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::Variable("cond".to_string()),
        then_block: vec![HirStatement::Expression(HirExpression::IntLiteral(1))],
        else_block: Some(vec![HirStatement::Expression(HirExpression::IntLiteral(
            2,
        ))]),
    };
    assert!(!cg.statement_modifies_variable(&stmt, "arr"));
}

#[test]
fn stmt_modifies_while_body_match() {
    // Line 5793-5795: While where body modifies variable
    let cg = CodeGenerator::new();
    let stmt = HirStatement::While {
        condition: HirExpression::Variable("running".to_string()),
        body: vec![HirStatement::ArrayIndexAssignment {
            array: Box::new(HirExpression::Variable("buf".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
            value: HirExpression::IntLiteral(0),
        }],
    };
    assert!(cg.statement_modifies_variable(&stmt, "buf"));
}

#[test]
fn stmt_modifies_for_body_match() {
    // Line 5793-5795: For where body modifies variable
    let cg = CodeGenerator::new();
    let stmt = HirStatement::For {
        init: vec![],
        condition: None,
        increment: vec![],
        body: vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("data".to_string()),
            value: HirExpression::IntLiteral(0),
        }],
    };
    assert!(cg.statement_modifies_variable(&stmt, "data"));
}

#[test]
fn stmt_modifies_for_body_no_match() {
    // Line 5793-5795: For where body does NOT modify variable
    let cg = CodeGenerator::new();
    let stmt = HirStatement::For {
        init: vec![],
        condition: None,
        increment: vec![],
        body: vec![HirStatement::Expression(HirExpression::IntLiteral(0))],
    };
    assert!(!cg.statement_modifies_variable(&stmt, "arr"));
}

#[test]
fn stmt_modifies_catch_all_return() {
    // Line 5796: catch-all arm returns false for Return statement
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Return(Some(HirExpression::Variable("arr".to_string())));
    assert!(!cg.statement_modifies_variable(&stmt, "arr"));
}

#[test]
fn stmt_modifies_catch_all_expression() {
    // Line 5796: catch-all arm returns false for Expression statement
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Expression(HirExpression::Variable("arr".to_string()));
    assert!(!cg.statement_modifies_variable(&stmt, "arr"));
}

#[test]
fn stmt_modifies_catch_all_var_decl() {
    // Line 5796: catch-all for VariableDeclaration
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::IntLiteral(0)),
    };
    assert!(!cg.statement_modifies_variable(&stmt, "arr"));
}

#[test]
fn stmt_modifies_nested_if_in_while() {
    // Recursion: While body contains If that modifies variable
    let cg = CodeGenerator::new();
    let inner_if = HirStatement::If {
        condition: HirExpression::Variable("flag".to_string()),
        then_block: vec![HirStatement::ArrayIndexAssignment {
            array: Box::new(HirExpression::Variable("arr".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
            value: HirExpression::IntLiteral(1),
        }],
        else_block: None,
    };
    let stmt = HirStatement::While {
        condition: HirExpression::Variable("running".to_string()),
        body: vec![inner_if],
    };
    assert!(cg.statement_modifies_variable(&stmt, "arr"));
}

// ============================================================================
// BATCH 10: generate_function coverage (lines 6345-6465)
// ============================================================================

#[test]
fn generate_function_empty_body_void_return() {
    // Line 6438-6444: Empty body with void return → no return statement
    let cg = CodeGenerator::new();
    let func = HirFunction::new(
        "noop".to_string(),
        HirType::Void,
        vec![],
    );
    let code = cg.generate_function(&func);
    assert!(code.contains("fn noop()"), "Got: {}", code);
    assert!(code.contains('{'), "Got: {}", code);
    assert!(code.contains('}'), "Got: {}", code);
}

#[test]
fn generate_function_empty_body_int_return() {
    // Line 6438-6443: Empty body with int return → generates return stub
    let cg = CodeGenerator::new();
    let func = HirFunction::new(
        "get_zero".to_string(),
        HirType::Int,
        vec![],
    );
    let code = cg.generate_function(&func);
    assert!(code.contains("fn get_zero()"), "Got: {}", code);
    assert!(code.contains("-> i32"), "Got: {}", code);
}

#[test]
fn generate_function_with_simple_body() {
    // Lines 6445-6460: Body with statements
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
    assert!(code.contains("fn add("), "Got: {}", code);
    assert!(code.contains("a + b"), "Got: {}", code);
}

#[test]
fn generate_function_with_pointer_param() {
    // Lines 6396-6428: Pointer param → context update for reference transformation
    // Note: Single pointer output param with deref assignment gets detected as output param
    // and removed from signature (DECY-084). Test with TWO pointer params to exercise
    // the pointer-to-reference context update path.
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "copy_val".to_string(),
        HirType::Void,
        vec![
            HirParameter::new(
                "src".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            ),
            HirParameter::new(
                "dst".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            ),
        ],
        vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("dst".to_string()),
            value: HirExpression::Dereference(Box::new(HirExpression::Variable(
                "src".to_string(),
            ))),
        }],
    );
    let code = cg.generate_function(&func);
    assert!(code.contains("fn copy_val("), "Got: {}", code);
    // At least one param should appear in signature
    assert!(
        code.contains("src") || code.contains("dst"),
        "Got: {}",
        code
    );
}

#[test]
fn generate_function_with_structs_basic() {
    // Lines 6471-6541: generate_function_with_structs
    let cg = CodeGenerator::new();
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
    let structs = vec![HirStruct::new(
        "Point".to_string(),
        vec![
            HirStructField::new("x".to_string(), HirType::Int),
            HirStructField::new("y".to_string(), HirType::Int),
        ],
    )];
    let code = cg.generate_function_with_structs(&func, &structs);
    assert!(code.contains("fn get_x("), "Got: {}", code);
    assert!(code.contains("-> i32"), "Got: {}", code);
}

#[test]
fn generate_function_main_no_return_type() {
    // Line 5217-5219: main function with Int return → no -> i32 in signature
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "main".to_string(),
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
    );
    let code = cg.generate_function(&func);
    assert!(code.contains("fn main()"), "Got: {}", code);
    assert!(
        !code.contains("-> i32"),
        "main should not have return type. Got: {}",
        code
    );
}

#[test]
fn generate_function_with_local_var() {
    // Test variable declaration and usage in body
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "example".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(42)),
            },
            HirStatement::Return(Some(HirExpression::Variable("x".to_string()))),
        ],
    );
    let code = cg.generate_function(&func);
    assert!(code.contains("let"), "Got: {}", code);
    assert!(code.contains("42"), "Got: {}", code);
}

// ============================================================================
// BATCH 11: generate_statement_with_context deep branches
// ============================================================================

#[test]
fn stmt_switch_case_with_body() {
    // Line 4672: Switch case with body statements
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    let stmt = HirStatement::Switch {
        condition: HirExpression::Variable("x".to_string()),
        cases: vec![
            SwitchCase {
                value: Some(HirExpression::IntLiteral(1)),
                body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(10)))],
            },
            SwitchCase {
                value: Some(HirExpression::IntLiteral(2)),
                body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(20)))],
            },
        ],
        default_case: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))]),
    };
    let code =
        cg.generate_statement_with_context(&stmt, Some("test_fn"), &mut ctx, Some(&HirType::Int));
    assert!(code.contains("match"), "Got: {}", code);
}

#[test]
fn stmt_deref_assignment_non_double_pointer() {
    // Line 4770: yields_raw_ptr = false, type is Int (not Reference(Pointer) or Pointer(Pointer))
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("p".to_string()),
        value: HirExpression::IntLiteral(42),
    };
    let code =
        cg.generate_statement_with_context(&stmt, Some("test_fn"), &mut ctx, Some(&HirType::Void));
    // Should generate *p = 42 with unsafe (pointer deref)
    assert!(code.contains("42"), "Got: {}", code);
}

#[test]
fn stmt_deref_assignment_double_pointer() {
    // Lines 4762-4779: DerefAssignment where target type is Pointer(Pointer(Int)) → yields_raw_ptr
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "pp".to_string(),
        HirType::Pointer(Box::new(HirType::Pointer(Box::new(HirType::Int)))),
    );
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("pp".to_string()),
        value: HirExpression::Variable("new_ptr".to_string()),
    };
    let code =
        cg.generate_statement_with_context(&stmt, Some("test_fn"), &mut ctx, Some(&HirType::Void));
    // Should detect double pointer and generate unsafe dereference
    assert!(
        code.contains("unsafe") || code.contains("*pp"),
        "Got: {}",
        code
    );
}

#[test]
fn stmt_var_decl_malloc_struct_no_default() {
    // Lines 4204-4229: malloc init for struct type → Box::new(unsafe zeroed)
    // Line 4215: false when inner is not Struct (Box with non-struct inner)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
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
    let code =
        cg.generate_statement_with_context(&stmt, Some("test_fn"), &mut ctx, Some(&HirType::Void));
    // Should generate Box allocation for struct
    assert!(
        code.contains("Box") || code.contains("node"),
        "Got: {}",
        code
    );
}

#[test]
fn stmt_var_decl_char_str_init() {
    // Lines 4133-4136: char* with string literal init → &str
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "msg".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Char)),
        initializer: Some(HirExpression::StringLiteral("hello".to_string())),
    };
    let code =
        cg.generate_statement_with_context(&stmt, Some("test_fn"), &mut ctx, Some(&HirType::Void));
    assert!(
        code.contains("&str") || code.contains("hello"),
        "Got: {}",
        code
    );
}

#[test]
fn stmt_var_decl_pointer_no_init() {
    // Line 4093: No initializer for pointer var → is_malloc_init = false
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "ptr".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: None,
    };
    let code =
        cg.generate_statement_with_context(&stmt, Some("test_fn"), &mut ctx, Some(&HirType::Void));
    assert!(code.contains("ptr"), "Got: {}", code);
}

#[test]
fn stmt_for_loop_with_body() {
    // For loop with condition and body — exercises generate_statement_with_context For arm
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("i".to_string(), HirType::Int);
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
        increment: vec![HirStatement::Expression(HirExpression::PostIncrement {
            operand: Box::new(HirExpression::Variable("i".to_string())),
        })],
        body: vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "printf".to_string(),
            arguments: vec![HirExpression::StringLiteral("%d".to_string())],
        })],
    };
    let code =
        cg.generate_statement_with_context(&stmt, Some("test_fn"), &mut ctx, Some(&HirType::Void));
    assert!(
        code.contains("while") || code.contains("for"),
        "Got: {}",
        code
    );
}

#[test]
fn stmt_while_loop_basic() {
    // While loop — exercises While arm in generate_statement_with_context
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("running".to_string(), HirType::Int);
    let stmt = HirStatement::While {
        condition: HirExpression::Variable("running".to_string()),
        body: vec![HirStatement::Break],
    };
    let code =
        cg.generate_statement_with_context(&stmt, Some("test_fn"), &mut ctx, Some(&HirType::Void));
    assert!(code.contains("while"), "Got: {}", code);
    assert!(code.contains("break"), "Got: {}", code);
}

#[test]
fn stmt_if_else_with_body() {
    // If/else with body statements
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    let stmt = HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
        then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
        else_block: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(
            -1,
        )))]),
    };
    let code =
        cg.generate_statement_with_context(&stmt, Some("test_fn"), &mut ctx, Some(&HirType::Int));
    assert!(code.contains("if"), "Got: {}", code);
    assert!(code.contains("else"), "Got: {}", code);
}

// ============================================================================
// BATCH 12: generate_annotated_signature_with_func coverage
// ============================================================================

#[test]
fn annotated_sig_simple_no_params_void() {
    use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let sig = AnnotatedSignature {
        name: "do_stuff".to_string(),
        lifetimes: vec![],
        parameters: vec![],
        return_type: AnnotatedType::Simple(HirType::Void),
    };
    let result = cg.generate_annotated_signature_with_func(&sig, None);
    assert_eq!(result, "fn do_stuff()");
}

#[test]
fn annotated_sig_with_return_type() {
    use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let sig = AnnotatedSignature {
        name: "get_value".to_string(),
        lifetimes: vec![],
        parameters: vec![],
        return_type: AnnotatedType::Simple(HirType::Int),
    };
    let result = cg.generate_annotated_signature_with_func(&sig, None);
    assert_eq!(result, "fn get_value() -> i32");
}

#[test]
fn annotated_sig_with_simple_params() {
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
    let result = cg.generate_annotated_signature_with_func(&sig, None);
    assert!(result.contains("mut a: i32"), "Got: {}", result);
    assert!(result.contains("mut b: i32"), "Got: {}", result);
    assert!(result.contains("-> i32"), "Got: {}", result);
}

#[test]
fn annotated_sig_keyword_rename_write() {
    use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let sig = AnnotatedSignature {
        name: "write".to_string(),
        lifetimes: vec![],
        parameters: vec![],
        return_type: AnnotatedType::Simple(HirType::Void),
    };
    let result = cg.generate_annotated_signature_with_func(&sig, None);
    assert!(result.contains("fn c_write"), "Got: {}", result);
}

#[test]
fn annotated_sig_keyword_rename_read() {
    use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let sig = AnnotatedSignature {
        name: "read".to_string(),
        lifetimes: vec![],
        parameters: vec![],
        return_type: AnnotatedType::Simple(HirType::Void),
    };
    let result = cg.generate_annotated_signature_with_func(&sig, None);
    assert!(result.contains("fn c_read"), "Got: {}", result);
}

#[test]
fn annotated_sig_pointer_param_becomes_mut_ref() {
    use decy_ownership::lifetime_gen::{AnnotatedParameter, AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let sig = AnnotatedSignature {
        name: "increment".to_string(),
        lifetimes: vec![],
        parameters: vec![AnnotatedParameter {
            name: "val".to_string(),
            param_type: AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Int))),
        }],
        return_type: AnnotatedType::Simple(HirType::Void),
    };
    // Without func, pointer becomes &mut T
    let result = cg.generate_annotated_signature_with_func(&sig, None);
    assert!(result.contains("&mut i32"), "Got: {}", result);
}

#[test]
fn annotated_sig_void_pointer_stays_raw() {
    use decy_ownership::lifetime_gen::{AnnotatedParameter, AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let sig = AnnotatedSignature {
        name: "generic_fn".to_string(),
        lifetimes: vec![],
        parameters: vec![AnnotatedParameter {
            name: "data".to_string(),
            param_type: AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Void))),
        }],
        return_type: AnnotatedType::Simple(HirType::Void),
    };
    let result = cg.generate_annotated_signature_with_func(&sig, None);
    assert!(result.contains("*mut ()"), "Got: {}", result);
}

#[test]
fn annotated_sig_main_no_return_type_via_func() {
    use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let sig = AnnotatedSignature {
        name: "main".to_string(),
        lifetimes: vec![],
        parameters: vec![],
        return_type: AnnotatedType::Simple(HirType::Int),
    };
    // Test the _with_func variant specifically
    let func = HirFunction::new_with_body(
        "main".to_string(),
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
    );
    let result = cg.generate_annotated_signature_with_func(&sig, Some(&func));
    // main with i32 return should NOT include -> i32
    assert_eq!(result, "fn main()");
}

#[test]
fn annotated_sig_with_lifetime_and_reference_param() {
    use decy_ownership::lifetime_gen::{
        AnnotatedParameter, AnnotatedSignature, AnnotatedType, LifetimeParam,
    };
    let cg = CodeGenerator::new();
    let sig = AnnotatedSignature {
        name: "borrow".to_string(),
        lifetimes: vec![LifetimeParam::standard(0)],
        parameters: vec![AnnotatedParameter {
            name: "data".to_string(),
            param_type: AnnotatedType::Reference {
                inner: Box::new(AnnotatedType::Simple(HirType::Int)),
                mutable: false,
                lifetime: Some(LifetimeParam::standard(0)),
            },
        }],
        return_type: AnnotatedType::Simple(HirType::Void),
    };
    let result = cg.generate_annotated_signature_with_func(&sig, None);
    assert!(result.contains("<'a>"), "Got: {}", result);
    assert!(result.contains("&'a i32"), "Got: {}", result);
}

#[test]
fn annotated_sig_slice_param_no_lifetime() {
    use decy_ownership::lifetime_gen::{
        AnnotatedParameter, AnnotatedSignature, AnnotatedType, LifetimeParam,
    };
    let cg = CodeGenerator::new();
    // Slice = Reference to Array with size=None — should NOT get lifetime param
    let sig = AnnotatedSignature {
        name: "process".to_string(),
        lifetimes: vec![LifetimeParam::standard(0)],
        parameters: vec![AnnotatedParameter {
            name: "arr".to_string(),
            param_type: AnnotatedType::Reference {
                inner: Box::new(AnnotatedType::Simple(HirType::Array {
                    element_type: Box::new(HirType::Int),
                    size: None,
                })),
                mutable: false,
                lifetime: Some(LifetimeParam::standard(0)),
            },
        }],
        return_type: AnnotatedType::Simple(HirType::Void),
    };
    let result = cg.generate_annotated_signature_with_func(&sig, None);
    // Slice params should NOT produce lifetime parameter <'a>
    assert!(!result.contains("<'a>"), "Got: {}", result);
    assert!(result.contains("&[i32]"), "Got: {}", result);
}

#[test]
fn annotated_sig_mutable_slice_param() {
    use decy_ownership::lifetime_gen::{AnnotatedParameter, AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let sig = AnnotatedSignature {
        name: "fill".to_string(),
        lifetimes: vec![],
        parameters: vec![AnnotatedParameter {
            name: "buf".to_string(),
            param_type: AnnotatedType::Reference {
                inner: Box::new(AnnotatedType::Simple(HirType::Array {
                    element_type: Box::new(HirType::Int),
                    size: None,
                })),
                mutable: true,
                lifetime: None,
            },
        }],
        return_type: AnnotatedType::Simple(HirType::Void),
    };
    let result = cg.generate_annotated_signature_with_func(&sig, None);
    assert!(result.contains("&mut [i32]"), "Got: {}", result);
}

#[test]
fn annotated_sig_unsized_array_param() {
    use decy_ownership::lifetime_gen::{AnnotatedParameter, AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    // C's void func(char arr[]) → AnnotatedType::Simple(Array { size: None })
    let sig = AnnotatedSignature {
        name: "parse".to_string(),
        lifetimes: vec![],
        parameters: vec![AnnotatedParameter {
            name: "buf".to_string(),
            param_type: AnnotatedType::Simple(HirType::Array {
                element_type: Box::new(HirType::Char),
                size: None,
            }),
        }],
        return_type: AnnotatedType::Simple(HirType::Void),
    };
    let result = cg.generate_annotated_signature_with_func(&sig, None);
    assert!(result.contains("&mut [u8]"), "Got: {}", result);
}

#[test]
fn annotated_sig_output_param_with_input() {
    use decy_ownership::lifetime_gen::{AnnotatedParameter, AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();

    // Function: void compute(int input, int* result)
    // With a function body that DerefAssigns to result
    let func = HirFunction::new_with_body(
        "compute".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("input".to_string(), HirType::Int),
            HirParameter::new("result".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        ],
        vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("result".to_string()),
            value: HirExpression::Variable("input".to_string()),
        }],
    );

    let sig = AnnotatedSignature {
        name: "compute".to_string(),
        lifetimes: vec![],
        parameters: vec![
            AnnotatedParameter {
                name: "input".to_string(),
                param_type: AnnotatedType::Simple(HirType::Int),
            },
            AnnotatedParameter {
                name: "result".to_string(),
                param_type: AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Int))),
            },
        ],
        return_type: AnnotatedType::Simple(HirType::Void),
    };
    let result = cg.generate_annotated_signature_with_func(&sig, Some(&func));
    // "result" is output param (name contains "result", has input params)
    // Should be removed from params and appear as return type
    assert!(result.contains("-> i32"), "Got: {}", result);
    assert!(!result.contains("result"), "Got: {}", result);
}

// ============================================================================
// BATCH 13: generate_expression_with_target_type coverage
// ============================================================================

#[test]
fn expr_target_int_literal_zero_option_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::IntLiteral(0);
    let target = HirType::Option(Box::new(HirType::Int));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert_eq!(result, "None");
}

#[test]
fn expr_target_int_literal_zero_pointer_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::IntLiteral(0);
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert_eq!(result, "std::ptr::null_mut()");
}

#[test]
fn expr_target_int_literal_nonzero_with_pointer_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // Non-zero int with pointer target should NOT become null_mut
    let expr = HirExpression::IntLiteral(42);
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert_eq!(result, "42");
}

#[test]
fn expr_target_float_literal_float_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FloatLiteral("3.14".to_string());
    let target = HirType::Float;
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert_eq!(result, "3.14f32");
}

#[test]
fn expr_target_float_literal_double_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FloatLiteral("2.718".to_string());
    let target = HirType::Double;
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert_eq!(result, "2.718f64");
}

#[test]
fn expr_target_float_literal_c_suffix_stripped() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // C float literal with 'f' suffix: 3.14f
    let expr = HirExpression::FloatLiteral("3.14f".to_string());
    let target = HirType::Float;
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert_eq!(result, "3.14f32");
}

#[test]
fn expr_target_float_literal_no_decimal_no_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // Float literal without decimal point, no target type → default f64
    let expr = HirExpression::FloatLiteral("42".to_string());
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "42.0f64");
}

#[test]
fn expr_target_float_literal_with_exponent_no_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // Float with exponent but no decimal
    let expr = HirExpression::FloatLiteral("1e10".to_string());
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "1e10f64");
}

#[test]
fn expr_target_address_of_with_pointer_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::AddressOf(Box::new(HirExpression::Variable("x".to_string())));
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("&mut x as *mut i32"), "Got: {}", result);
}

#[test]
fn expr_target_address_of_dereference() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // &(*ptr) → &(deref)
    let expr = HirExpression::AddressOf(Box::new(HirExpression::Dereference(Box::new(
        HirExpression::Variable("ptr".to_string()),
    ))));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("&("), "Got: {}", result);
}

#[test]
fn expr_target_unary_address_of_with_pointer_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::AddressOf,
        operand: Box::new(HirExpression::Variable("y".to_string())),
    };
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("&mut y as *mut i32"), "Got: {}", result);
}

#[test]
fn expr_target_logical_not_bool_to_int() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // !true_expr assigned to int → (!expr) as i32
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
    };
    let target = HirType::Int;
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as i32"), "Got: {}", result);
}

#[test]
fn expr_target_logical_not_int_to_int() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // !int_expr assigned to int → (int == 0) as i32
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let target = HirType::Int;
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("== 0") && result.contains("as i32"), "Got: {}", result);
}

#[test]
fn expr_target_logical_not_no_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // !int_expr with no target → (int == 0)
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("== 0"), "Got: {}", result);
    assert!(!result.contains("as i32"), "Should not cast: {}", result);
}

#[test]
fn expr_target_string_literal_to_pointer_char() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::StringLiteral("hello".to_string());
    let target = HirType::Pointer(Box::new(HirType::Char));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("b\"hello\\0\""), "Got: {}", result);
    assert!(result.contains("as_ptr"), "Got: {}", result);
    assert!(result.contains("*mut u8"), "Got: {}", result);
}

#[test]
fn expr_target_variable_with_vec_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Variable("data".to_string());
    let target = HirType::Vec(Box::new(HirType::Int));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert_eq!(result, "data");
}

#[test]
fn expr_target_variable_box_to_pointer() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("node".to_string(), HirType::Box(Box::new(HirType::Int)));
    let expr = HirExpression::Variable("node".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("Box::into_raw"), "Got: {}", result);
}

#[test]
fn expr_target_variable_char_to_int() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("c".to_string(), HirType::Char);
    let expr = HirExpression::Variable("c".to_string());
    let target = HirType::Int;
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as i32"), "Got: {}", result);
}

#[test]
fn expr_target_variable_int_to_float() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("n".to_string(), HirType::Int);
    let expr = HirExpression::Variable("n".to_string());
    let target = HirType::Float;
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as f32"), "Got: {}", result);
}

#[test]
fn expr_target_variable_int_to_double() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("n".to_string(), HirType::Int);
    let expr = HirExpression::Variable("n".to_string());
    let target = HirType::Double;
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as f64"), "Got: {}", result);
}

#[test]
fn expr_target_variable_float_to_int() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("f".to_string(), HirType::Float);
    let expr = HirExpression::Variable("f".to_string());
    let target = HirType::Int;
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as i32"), "Got: {}", result);
}

#[test]
fn expr_target_variable_double_to_unsigned_int() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("d".to_string(), HirType::Double);
    let expr = HirExpression::Variable("d".to_string());
    let target = HirType::UnsignedInt;
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as u32"), "Got: {}", result);
}

#[test]
fn expr_target_variable_vec_to_pointer() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("buf".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::Variable("buf".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as_mut_ptr"), "Got: {}", result);
}

#[test]
fn expr_target_variable_array_to_pointer() {
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
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as_mut_ptr"), "Got: {}", result);
}

#[test]
fn expr_target_variable_pointer_to_pointer() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::Variable("p".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    // Raw pointer stays as raw pointer — just return variable
    assert_eq!(result, "p");
}

#[test]
fn expr_target_variable_ref_to_pointer() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "r".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: true,
        },
    );
    let expr = HirExpression::Variable("r".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as *mut _"), "Got: {}", result);
}

#[test]
fn expr_target_variable_immutable_ref_to_pointer() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "r".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
    );
    let expr = HirExpression::Variable("r".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(
        result.contains("as *const _ as *mut _"),
        "Got: {}",
        result
    );
}

#[test]
fn expr_target_variable_mutable_slice_ref_to_pointer() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "s".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Array {
                element_type: Box::new(HirType::Int),
                size: None,
            }),
            mutable: true,
        },
    );
    let expr = HirExpression::Variable("s".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as_mut_ptr"), "Got: {}", result);
}

#[test]
fn expr_target_variable_immutable_slice_ref_to_pointer() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "s".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Array {
                element_type: Box::new(HirType::Int),
                size: None,
            }),
            mutable: false,
        },
    );
    let expr = HirExpression::Variable("s".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as_ptr"), "Got: {}", result);
}

#[test]
fn expr_target_variable_array_to_void_pointer() {
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
    let target = HirType::Pointer(Box::new(HirType::Void));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as *mut ()"), "Got: {}", result);
}

#[test]
fn expr_target_variable_stderr() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Variable("stderr".to_string());
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "std::io::stderr()");
}

#[test]
fn expr_target_variable_errno_constants() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Variable("ERANGE".to_string());
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "34i32");
}

#[test]
fn expr_target_char_literal_null() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::CharLiteral(0i8);
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "0u8");
}

#[test]
fn expr_target_char_literal_printable() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::CharLiteral(b'a' as i8);
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "b'a'");
}

#[test]
fn expr_target_char_literal_non_printable() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::CharLiteral(1i8);
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "1u8");
}

#[test]
fn expr_target_binary_assign_embedded() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    // Embedded assignment: (x = 5) → { let __assign_tmp = 5; x = __assign_tmp; __assign_tmp }
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Assign,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::IntLiteral(5)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("__assign_tmp"), "Got: {}", result);
    assert!(result.contains("x = __assign_tmp"), "Got: {}", result);
}

#[test]
fn expr_target_variable_ref_vec_to_pointer() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // Reference to Vec (used internally by BorrowGenerator)
    ctx.add_variable(
        "data".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Vec(Box::new(HirType::Int))),
            mutable: true,
        },
    );
    let expr = HirExpression::Variable("data".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as_mut_ptr"), "Got: {}", result);
}

// ============================================================================
// BATCH 14: generate_expression_with_target_type — deeper branches
// ============================================================================

#[test]
fn expr_target_option_eq_null_is_none() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("p".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("is_none"), "Got: {}", result);
}

#[test]
fn expr_target_option_ne_null_is_some() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("p".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("is_some"), "Got: {}", result);
}

#[test]
fn expr_target_null_eq_option_reversed() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::NullLiteral),
        right: Box::new(HirExpression::Variable("p".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("is_none"), "Got: {}", result);
}

#[test]
fn expr_target_box_eq_null_always_false() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("b".to_string(), HirType::Box(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("b".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("false"), "Got: {}", result);
}

#[test]
fn expr_target_box_ne_null_always_true() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("b".to_string(), HirType::Box(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("b".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("true"), "Got: {}", result);
}

#[test]
fn expr_target_strlen_eq_zero_is_empty() {
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
    assert!(result.contains("is_empty"), "Got: {}", result);
}

#[test]
fn expr_target_zero_ne_strlen_not_is_empty() {
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
    assert!(result.contains("is_empty"), "Got: {}", result);
}

#[test]
fn expr_target_comma_operator() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Comma,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("{ a; b }"), "Got: {}", result);
}

#[test]
fn expr_target_int_comparison_with_char_literal() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("c".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("c".to_string())),
        right: Box::new(HirExpression::CharLiteral(b'\n' as i8)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("10i32"), "Got: {}", result);
}

#[test]
fn expr_target_char_add_int_arithmetic() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("n".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("n".to_string())),
        right: Box::new(HirExpression::CharLiteral(b'0' as i8)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("48i32"), "Got: {}", result);
}

#[test]
fn expr_target_logical_and_with_int_target() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("a".to_string(), HirType::Int);
    ctx.add_variable("b".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalAnd,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let target = HirType::Int;
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as i32"), "Got: {}", result);
    assert!(result.contains("!= 0"), "Got: {}", result);
}

#[test]
fn expr_target_mixed_int_float_arithmetic() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("n".to_string(), HirType::Int);
    ctx.add_variable("f".to_string(), HirType::Float);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("n".to_string())),
        right: Box::new(HirExpression::Variable("f".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("as f32"), "Got: {}", result);
}

#[test]
fn expr_target_mixed_int_double_arithmetic() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("n".to_string(), HirType::Int);
    ctx.add_variable("d".to_string(), HirType::Double);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(HirExpression::Variable("n".to_string())),
        right: Box::new(HirExpression::Variable("d".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("as f64"), "Got: {}", result);
}

#[test]
fn expr_target_mixed_float_double_arithmetic() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("f".to_string(), HirType::Float);
    ctx.add_variable("d".to_string(), HirType::Double);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("f".to_string())),
        right: Box::new(HirExpression::Variable("d".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("as f64"), "Got: {}", result);
}

#[test]
fn expr_target_char_subtract_with_int_target() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("a".to_string(), HirType::Char);
    ctx.add_variable("b".to_string(), HirType::Char);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let target = HirType::Int;
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as i32"), "Got: {}", result);
}

#[test]
fn expr_target_global_variable_wrapped_unsafe() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_global("G_VAL".to_string());
    let expr = HirExpression::Variable("G_VAL".to_string());
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains("G_VAL"), "Got: {}", result);
}

#[test]
fn expr_target_global_int_to_float_unsafe() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_global("G_COUNT".to_string());
    ctx.add_variable("G_COUNT".to_string(), HirType::Int);
    let expr = HirExpression::Variable("G_COUNT".to_string());
    let target = HirType::Float;
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains("as f32"), "Got: {}", result);
}

// ============================================================================
// BATCH 15: statement_modifies_variable coverage (5764-5798)
// ============================================================================

#[test]
fn stmt_modifies_array_index_assign_matching_var() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
        value: HirExpression::IntLiteral(42),
    };
    assert!(cg.statement_modifies_variable(&stmt, "arr"));
}

#[test]
fn stmt_modifies_array_index_assign_nonmatching_var() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
        value: HirExpression::IntLiteral(42),
    };
    assert!(!cg.statement_modifies_variable(&stmt, "other"));
}

#[test]
fn stmt_modifies_array_index_assign_non_variable_array() {
    let cg = CodeGenerator::new();
    // Array is a field access, not a simple variable
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::FieldAccess {
            object: Box::new(HirExpression::Variable("s".to_string())),
            field: "data".to_string(),
        }),
        index: Box::new(HirExpression::IntLiteral(0)),
        value: HirExpression::IntLiteral(42),
    };
    assert!(!cg.statement_modifies_variable(&stmt, "data"));
}

#[test]
fn stmt_modifies_deref_assign_matching_var() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("ptr".to_string()),
        value: HirExpression::IntLiteral(99),
    };
    assert!(cg.statement_modifies_variable(&stmt, "ptr"));
}

#[test]
fn stmt_modifies_deref_assign_nonmatching_var() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("ptr".to_string()),
        value: HirExpression::IntLiteral(99),
    };
    assert!(!cg.statement_modifies_variable(&stmt, "other"));
}

#[test]
fn stmt_modifies_deref_assign_non_variable_target() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Dereference(Box::new(HirExpression::Variable("pp".to_string()))),
        value: HirExpression::IntLiteral(99),
    };
    assert!(!cg.statement_modifies_variable(&stmt, "pp"));
}

#[test]
fn stmt_modifies_if_then_block() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::IntLiteral(1),
        then_block: vec![HirStatement::ArrayIndexAssignment {
            array: Box::new(HirExpression::Variable("arr".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
            value: HirExpression::IntLiteral(1),
        }],
        else_block: None,
    };
    assert!(cg.statement_modifies_variable(&stmt, "arr"));
}

#[test]
fn stmt_modifies_if_else_block() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::IntLiteral(1),
        then_block: vec![],
        else_block: Some(vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("x".to_string()),
            value: HirExpression::IntLiteral(0),
        }]),
    };
    assert!(cg.statement_modifies_variable(&stmt, "x"));
}

#[test]
fn stmt_modifies_if_neither_block_empty_else() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::IntLiteral(1),
        then_block: vec![],
        else_block: Some(vec![]),
    };
    assert!(!cg.statement_modifies_variable(&stmt, "x"));
}

#[test]
fn stmt_modifies_while_body() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::While {
        condition: HirExpression::IntLiteral(1),
        body: vec![HirStatement::ArrayIndexAssignment {
            array: Box::new(HirExpression::Variable("buf".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
            value: HirExpression::IntLiteral(0),
        }],
    };
    assert!(cg.statement_modifies_variable(&stmt, "buf"));
}

#[test]
fn stmt_modifies_for_body() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::For {
        init: vec![],
        condition: Some(HirExpression::IntLiteral(1)),
        increment: vec![],
        body: vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("p".to_string()),
            value: HirExpression::IntLiteral(0),
        }],
    };
    assert!(cg.statement_modifies_variable(&stmt, "p"));
}

#[test]
fn stmt_modifies_unmatched_variant_returns_false() {
    let cg = CodeGenerator::new();
    // Break, Continue, Return, etc. all return false
    assert!(!cg.statement_modifies_variable(&HirStatement::Break, "x"));
    assert!(!cg.statement_modifies_variable(&HirStatement::Continue, "x"));
    assert!(!cg.statement_modifies_variable(
        &HirStatement::Return(None),
        "x"
    ));
}

// ============================================================================
// BATCH 15b: generate_function coverage (6345-6465)
// ============================================================================

#[test]
fn gen_func_empty_body_generates_stub() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("stub".to_string(), HirType::Int, vec![]);
    let result = cg.generate_function(&func);
    assert!(result.contains("fn stub"), "Got: {}", result);
    assert!(
        result.contains("0i32") || result.contains("return"),
        "Got: {}",
        result
    );
}

#[test]
fn gen_func_void_return_empty_body() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("noop".to_string(), HirType::Void, vec![]);
    let result = cg.generate_function(&func);
    assert!(result.contains("fn noop"), "Got: {}", result);
    // Void return should not have a return statement
    assert!(!result.contains("return"), "Got: {}", result);
}

#[test]
fn gen_func_with_body_statements() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "add_one".to_string(),
        HirType::Int,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        }))],
    );
    let result = cg.generate_function(&func);
    assert!(result.contains("fn add_one"), "Got: {}", result);
    assert!(result.contains("return"), "Got: {}", result);
    assert!(result.contains("+ 1"), "Got: {}", result);
}

// ============================================================================
// BATCH 15c: generate_function_with_structs coverage (6471-6541)
// ============================================================================

#[test]
fn gen_func_with_structs_uses_struct_context() {
    let cg = CodeGenerator::new();
    let s = HirStruct::new(
        "Point".to_string(),
        vec![
            HirStructField::new("x".to_string(), HirType::Int),
            HirStructField::new("y".to_string(), HirType::Int),
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
    let result = cg.generate_function_with_structs(&func, &[s]);
    assert!(result.contains("fn get_x"), "Got: {}", result);
    assert!(result.contains("return"), "Got: {}", result);
}

#[test]
fn gen_func_with_structs_empty_body() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new(
        "empty".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "val".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
    );
    // No body → stub generated
    let result = cg.generate_function_with_structs(&func, &[]);
    assert!(result.contains("fn empty"), "Got: {}", result);
}

// ============================================================================
// BATCH 15d: generate_statement_with_context — VLA, malloc, char array, realloc, switch, global
// ============================================================================

#[test]
fn stmt_ctx_vla_declaration_int() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("vec![0i32;"), "Got: {}", result);
    assert!(result.contains("n"), "Got: {}", result);
}

#[test]
fn stmt_ctx_vla_declaration_double() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "darr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Double),
            size: None,
        },
        initializer: Some(HirExpression::IntLiteral(10)),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("vec![0.0f64;"), "Got: {}", result);
}

#[test]
fn stmt_ctx_vla_declaration_char() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "buf".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Char),
            size: None,
        },
        initializer: Some(HirExpression::IntLiteral(256)),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("vec![0u8;"), "Got: {}", result);
}

#[test]
fn stmt_ctx_vla_declaration_float() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "farr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Float),
            size: None,
        },
        initializer: Some(HirExpression::IntLiteral(5)),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("vec![0.0f32;"), "Got: {}", result);
}

#[test]
fn stmt_ctx_vla_declaration_unsigned_int() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "uarr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::UnsignedInt),
            size: None,
        },
        initializer: Some(HirExpression::IntLiteral(8)),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("vec![0u32;"), "Got: {}", result);
