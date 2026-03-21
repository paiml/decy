    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("[0i32; 4]"),
        "Empty sized array → [0i32; 4]: {}",
        code
    );
}

#[test]
fn expr_context_compound_literal_array_empty_unsized() {
    // (int[]){} → []
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None,
        },
        initializers: vec![],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code == "[]",
        "Empty unsized array → []: {}",
        code
    );
}

#[test]
fn expr_context_compound_literal_array_single_repeat() {
    // (int[4]){0} → [0; 4]
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(4),
        },
        initializers: vec![HirExpression::IntLiteral(0)],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("[0; 4]"),
        "Single init repeat → [0; 4]: {}",
        code
    );
}

#[test]
fn expr_context_compound_literal_array_partial() {
    // (int[4]){1, 2} → [1, 2, 0i32, 0i32] (padded with defaults)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(4),
        },
        initializers: vec![
            HirExpression::IntLiteral(1),
            HirExpression::IntLiteral(2),
        ],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("1") && code.contains("2") && code.contains("0i32"),
        "Partial array init → padded: {}",
        code
    );
}

#[test]
fn expr_context_compound_literal_array_full() {
    // (int[3]){1, 2, 3} → [1, 2, 3]
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
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
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("[1, 2, 3]"),
        "Full array init → [1, 2, 3]: {}",
        code
    );
}

#[test]
fn expr_context_compound_literal_other_type() {
    // Compound literal with non-struct, non-array type → comment
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Int,
        initializers: vec![HirExpression::IntLiteral(42)],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("Compound literal"),
        "Non-struct/array compound → comment: {}",
        code
    );
}

#[test]
fn expr_context_is_not_null() {
    // IsNotNull(p) → if let Some(_) = p
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::IsNotNull(Box::new(HirExpression::Variable("p".to_string())));
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("if let Some(_) = p"),
        "IsNotNull → if let Some: {}",
        code
    );
}

#[test]
fn expr_context_null_literal() {
    // NullLiteral → None
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let code = cg.generate_expression_with_context(&HirExpression::NullLiteral, &mut ctx);
    assert_eq!(code, "None");
}

#[test]
fn expr_context_slice_index() {
    // SliceIndex → safe indexing with as usize
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::SliceIndex {
        slice: Box::new(HirExpression::Variable("data".to_string())),
        index: Box::new(HirExpression::Variable("i".to_string())),
        element_type: HirType::Int,
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("data[(i) as usize]"),
        "SliceIndex → safe indexing: {}",
        code
    );
}

#[test]
fn expr_context_field_access_keyword_escape() {
    // obj.type → obj.r#type (keyword escaping)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FieldAccess {
        object: Box::new(HirExpression::Variable("obj".to_string())),
        field: "type".to_string(),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("r#type"),
        "Field access keyword escape → r#type: {}",
        code
    );
}

// =============================================================================
// Batch 53: PostIncrement/PreIncrement/PostDecrement/PreDecrement variants
//           + Ternary expression
// =============================================================================

#[test]
fn expr_context_post_inc_string_ref() {
    // *key++ where key is &str → string iteration pattern
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("key".to_string(), HirType::StringReference);
    let expr = HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Variable("key".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("as_bytes()[0]") && code.contains("__tmp"),
        "PostInc string ref → byte iteration: {}",
        code
    );
}

#[test]
fn expr_context_post_inc_deref_raw_ptr() {
    // (*p)++ where p is raw pointer → unsafe block
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("p".to_string()),
        ))),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("unsafe") && code.contains("*p += 1"),
        "PostInc deref ptr → unsafe: {}",
        code
    );
}

#[test]
fn expr_context_post_inc_pointer_var() {
    // ptr++ where ptr is pointer → wrapping_add
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Variable("ptr".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("wrapping_add(1)"),
        "PostInc pointer → wrapping_add: {}",
        code
    );
}

#[test]
fn expr_context_post_inc_regular() {
    // x++ → { let __tmp = x; x += 1; __tmp }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("__tmp") && code.contains("+= 1"),
        "PostInc regular → tmp + +=1: {}",
        code
    );
}

#[test]
fn expr_context_pre_inc_deref_raw_ptr() {
    // ++(*p) where p is raw pointer → unsafe block
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PreIncrement {
        operand: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("p".to_string()),
        ))),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("unsafe") && code.contains("*p += 1"),
        "PreInc deref ptr → unsafe: {}",
        code
    );
}

#[test]
fn expr_context_pre_inc_pointer_var() {
    // ++ptr where ptr is pointer → wrapping_add
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PreIncrement {
        operand: Box::new(HirExpression::Variable("ptr".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("wrapping_add(1)"),
        "PreInc pointer → wrapping_add: {}",
        code
    );
}

#[test]
fn expr_context_pre_inc_regular() {
    // ++x → { x += 1; x }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::PreIncrement {
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("+= 1") && !code.contains("__tmp"),
        "PreInc regular → += 1 no tmp: {}",
        code
    );
}

#[test]
fn expr_context_post_dec_deref_raw_ptr() {
    // (*p)-- where p is raw pointer → unsafe block
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PostDecrement {
        operand: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("p".to_string()),
        ))),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("unsafe") && code.contains("*p -= 1"),
        "PostDec deref ptr → unsafe: {}",
        code
    );
}

#[test]
fn expr_context_post_dec_pointer_var() {
    // ptr-- where ptr is pointer → wrapping_sub
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PostDecrement {
        operand: Box::new(HirExpression::Variable("ptr".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("wrapping_sub(1)"),
        "PostDec pointer → wrapping_sub: {}",
        code
    );
}

#[test]
fn expr_context_post_dec_regular() {
    // x-- → { let __tmp = x; x -= 1; __tmp }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::PostDecrement {
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("__tmp") && code.contains("-= 1"),
        "PostDec regular → tmp + -=1: {}",
        code
    );
}

#[test]
fn expr_context_pre_dec_deref_raw_ptr() {
    // --(*p) where p is raw pointer → unsafe block
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PreDecrement {
        operand: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("p".to_string()),
        ))),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("unsafe") && code.contains("*p -= 1"),
        "PreDec deref ptr → unsafe: {}",
        code
    );
}

#[test]
fn expr_context_pre_dec_pointer_var() {
    // --ptr where ptr is pointer → wrapping_sub
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PreDecrement {
        operand: Box::new(HirExpression::Variable("ptr".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("wrapping_sub(1)"),
        "PreDec pointer → wrapping_sub: {}",
        code
    );
}

#[test]
fn expr_context_pre_dec_regular() {
    // --x → { x -= 1; x }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::PreDecrement {
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("-= 1") && !code.contains("__tmp"),
        "PreDec regular → -= 1 no tmp: {}",
        code
    );
}

#[test]
fn expr_context_ternary_bool_condition() {
    // (a > b) ? a : b → if a > b { a } else { b }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
        then_expr: Box::new(HirExpression::Variable("a".to_string())),
        else_expr: Box::new(HirExpression::Variable("b".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("if a > b { a } else { b }"),
        "Ternary bool cond → if/else: {}",
        code
    );
}

#[test]
fn expr_context_ternary_non_bool_condition() {
    // x ? 1 : 0 → if x != 0 { 1 } else { 0 }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::Variable("x".to_string())),
        then_expr: Box::new(HirExpression::IntLiteral(1)),
        else_expr: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("!= 0") && code.contains("if") && code.contains("else"),
        "Ternary non-bool cond → != 0: {}",
        code
    );
}

#[test]
fn expr_context_post_inc_string_literal() {
    // StringLiteral type var ++ → string iteration
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("s".to_string(), HirType::StringLiteral);
    let expr = HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Variable("s".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("as_bytes()[0]") && code.contains("u32"),
        "PostInc StringLiteral → byte iteration with u32: {}",
        code
    );
}

#[test]
fn expr_context_compound_literal_array_unsized_with_elems() {
    // (int[]){1, 2, 3} unsized → [1, 2, 3]
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None,
        },
        initializers: vec![
            HirExpression::IntLiteral(10),
            HirExpression::IntLiteral(20),
        ],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("[10, 20]"),
        "Unsized array with elems → [10, 20]: {}",
        code
    );
}

// =============================================================================
// Batch 54: generate_struct derive variants, generate_enum, generate_typedef,
//           generate_constant, generate_global_variable
// =============================================================================

#[test]
fn struct_derive_default_no_float_no_large_copyable() {
    // Small int-only struct → Debug, Clone, Copy, Default, PartialEq, Eq
    let cg = CodeGenerator::new();
    let s = decy_hir::HirStruct::new(
        "Point".to_string(),
        vec![
            decy_hir::HirStructField::new("x".to_string(), HirType::Int),
            decy_hir::HirStructField::new("y".to_string(), HirType::Int),
        ],
    );
    let code = cg.generate_struct(&s);
    assert!(
        code.contains("Copy") && code.contains("Default") && code.contains("Eq"),
        "Int-only struct → Copy+Default+Eq: {}",
        code
    );
}

#[test]
fn struct_derive_float_no_large_copyable() {
    // Float field struct → no Eq, has PartialEq, Copy, Default
    let cg = CodeGenerator::new();
    let s = decy_hir::HirStruct::new(
        "Coord".to_string(),
        vec![decy_hir::HirStructField::new(
            "x".to_string(),
            HirType::Float,
        )],
    );
    let code = cg.generate_struct(&s);
    assert!(
        code.contains("Copy") && code.contains("Default") && code.contains("PartialEq")
            && !code.contains(", Eq"),
        "Float struct → Copy+Default+PartialEq, no Eq: {}",
        code
    );
}

#[test]
fn struct_derive_large_array_no_default() {
    // Large array field → no Default, has Copy, Eq
    let cg = CodeGenerator::new();
    let s = decy_hir::HirStruct::new(
        "Buffer".to_string(),
        vec![decy_hir::HirStructField::new(
            "data".to_string(),
            HirType::Array {
                element_type: Box::new(HirType::Char),
                size: Some(256),
            },
        )],
    );
    let code = cg.generate_struct(&s);
    assert!(
        !code.contains("Default") && code.contains("Copy") && code.contains("Eq"),
        "Large array struct → no Default, has Copy+Eq: {}",
        code
    );
}

#[test]
fn struct_derive_large_array_float() {
    // Large array + float → no Default, no Eq
    let cg = CodeGenerator::new();
    let s = decy_hir::HirStruct::new(
        "FloatBuf".to_string(),
        vec![
            decy_hir::HirStructField::new(
                "data".to_string(),
                HirType::Array {
                    element_type: Box::new(HirType::Double),
                    size: Some(100),
                },
            ),
            decy_hir::HirStructField::new("scale".to_string(), HirType::Double),
        ],
    );
    let code = cg.generate_struct(&s);
    assert!(
        !code.contains("Default") && !code.contains(", Eq"),
        "Large array + float → no Default, no Eq: {}",
        code
    );
}

#[test]
fn struct_derive_non_copy_vec_field() {
    // Vec field → no Copy
    let cg = CodeGenerator::new();
    let s = decy_hir::HirStruct::new(
        "List".to_string(),
        vec![decy_hir::HirStructField::new(
            "items".to_string(),
            HirType::Vec(Box::new(HirType::Int)),
        )],
    );
    let code = cg.generate_struct(&s);
    assert!(
        !code.contains("Copy") && code.contains("Default"),
        "Vec field → no Copy, has Default: {}",
        code
    );
}

#[test]
fn struct_derive_reference_field_lifetime() {
    // Reference field → lifetime annotation <'a>
    let cg = CodeGenerator::new();
    let s = decy_hir::HirStruct::new(
        "Ref".to_string(),
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
        "Reference field → struct<'a>: {}",
        code
    );
}

#[test]
fn struct_flexible_array_member() {
    // Array { size: None } → Vec<T>
    let cg = CodeGenerator::new();
    let s = decy_hir::HirStruct::new(
        "FlexBuf".to_string(),
        vec![
            decy_hir::HirStructField::new("len".to_string(), HirType::Int),
            decy_hir::HirStructField::new(
                "data".to_string(),
                HirType::Array {
                    element_type: Box::new(HirType::Char),
                    size: None,
                },
            ),
        ],
    );
    let code = cg.generate_struct(&s);
    assert!(
        code.contains("Vec<u8>"),
        "Flexible array member → Vec<u8>: {}",
        code
    );
}

#[test]
fn struct_keyword_field_escape() {
    // Field named "type" → r#type
    let cg = CodeGenerator::new();
    let s = decy_hir::HirStruct::new(
        "Token".to_string(),
        vec![decy_hir::HirStructField::new(
            "type".to_string(),
            HirType::Int,
        )],
    );
    let code = cg.generate_struct(&s);
    assert!(
        code.contains("r#type"),
        "Field 'type' → r#type: {}",
        code
    );
}

#[test]
fn enum_with_explicit_values() {
    let cg = CodeGenerator::new();
    let e = decy_hir::HirEnum::new(
        "Color".to_string(),
        vec![
            decy_hir::HirEnumVariant::new("RED".to_string(), Some(1)),
            decy_hir::HirEnumVariant::new("GREEN".to_string(), None),
            decy_hir::HirEnumVariant::new("BLUE".to_string(), Some(10)),
        ],
    );
    let code = cg.generate_enum(&e);
    assert!(
        code.contains("type Color = i32")
            && code.contains("RED: i32 = 1")
            && code.contains("GREEN: i32 = 2")
            && code.contains("BLUE: i32 = 10"),
        "Enum with explicit values: {}",
        code
    );
}

#[test]
fn enum_empty_name() {
    // Anonymous enum → no type alias
    let cg = CodeGenerator::new();
    let e = decy_hir::HirEnum::new(
        "".to_string(),
        vec![decy_hir::HirEnumVariant::new("VALUE".to_string(), Some(42))],
    );
    let code = cg.generate_enum(&e);
    assert!(
        !code.contains("type  =") && code.contains("VALUE: i32 = 42"),
        "Anonymous enum → no type alias: {}",
        code
    );
}

#[test]
fn typedef_simple() {
    let cg = CodeGenerator::new();
    let td = decy_hir::HirTypedef::new("Integer".to_string(), HirType::Int);
    let code = cg.generate_typedef(&td).unwrap();
    assert!(
        code.contains("pub type Integer = i32"),
        "Simple typedef: {}",
        code
    );
}

#[test]
fn typedef_array_assertion() {
    // typedef char check[sizeof(int) == 4 ? 1 : -1] → const assertion
    let cg = CodeGenerator::new();
    let td = decy_hir::HirTypedef::new(
        "check".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Char),
            size: Some(1),
        },
    );
    let code = cg.generate_typedef(&td).unwrap();
    assert!(
        code.contains("assert!"),
        "Typedef assertion → const assert: {}",
        code
    );
}

#[test]
fn typedef_array_fixed() {
    // typedef int IntArray[10] → pub type IntArray = [i32; 10]
    let cg = CodeGenerator::new();
    let td = decy_hir::HirTypedef::new(
        "IntArray".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );
    let code = cg.generate_typedef(&td).unwrap();
    assert!(
        code.contains("pub type IntArray = [i32; 10]"),
        "Fixed array typedef: {}",
        code
    );
}

#[test]
fn typedef_size_t() {
    let cg = CodeGenerator::new();
    let td = decy_hir::HirTypedef::new("size_t".to_string(), HirType::UnsignedInt);
    let code = cg.generate_typedef(&td).unwrap();
    assert!(
        code.contains("pub type size_t = usize"),
        "size_t → usize: {}",
        code
    );
}

#[test]
fn typedef_ssize_t() {
    let cg = CodeGenerator::new();
    let td = decy_hir::HirTypedef::new("ssize_t".to_string(), HirType::Int);
    let code = cg.generate_typedef(&td).unwrap();
    assert!(
        code.contains("pub type ssize_t = isize"),
        "ssize_t → isize: {}",
        code
    );
}

#[test]
fn typedef_redundant_struct() {
    // typedef struct Foo Foo → comment (redundant)
    let cg = CodeGenerator::new();
    let td = decy_hir::HirTypedef::new(
        "Foo".to_string(),
        HirType::Struct("Foo".to_string()),
    );
    let code = cg.generate_typedef(&td).unwrap();
    assert!(
        code.contains("redundant"),
        "Redundant struct typedef → comment: {}",
        code
    );
}

#[test]
fn constant_int() {
    let cg = CodeGenerator::new();
    let c = decy_hir::HirConstant::new(
        "MAX".to_string(),
        HirType::Int,
        HirExpression::IntLiteral(100),
    );
    let code = cg.generate_constant(&c);
    assert!(
        code.contains("const MAX: i32 = 100"),
        "Int constant: {}",
        code
    );
}

#[test]
fn constant_string() {
    // String constant: char* → &str
    let cg = CodeGenerator::new();
    let c = decy_hir::HirConstant::new(
        "MSG".to_string(),
        HirType::Pointer(Box::new(HirType::Char)),
        HirExpression::StringLiteral("Hello".to_string()),
    );
    let code = cg.generate_constant(&c);
    assert!(
        code.contains("const MSG: &str") && code.contains("Hello"),
        "String constant → &str: {}",
        code
    );
}

#[test]
fn global_var_extern() {
    let cg = CodeGenerator::new();
    let g = decy_hir::HirConstant::new(
        "counter".to_string(),
        HirType::Int,
        HirExpression::IntLiteral(0),
    );
    let code = cg.generate_global_variable(&g, false, true, false);
    assert!(
        code.contains("extern \"C\"") && code.contains("static counter: i32"),
        "Extern global → extern C: {}",
        code
    );
}

#[test]
fn global_var_const() {
    let cg = CodeGenerator::new();
    let g = decy_hir::HirConstant::new(
        "PI".to_string(),
        HirType::Double,
        HirExpression::FloatLiteral("3.14".to_string()),
    );
    let code = cg.generate_global_variable(&g, true, false, true);
    assert!(
        code.contains("const PI: f64 = 3.14"),
        "Const global → const: {}",
        code
    );
}

#[test]
fn global_var_static_mut() {
    let cg = CodeGenerator::new();
    let g = decy_hir::HirConstant::new(
        "count".to_string(),
        HirType::Int,
        HirExpression::IntLiteral(0),
    );
    let code = cg.generate_global_variable(&g, true, false, false);
    assert!(
        code.contains("static mut count: i32 = 0"),
        "Static global → static mut: {}",
        code
    );
}

#[test]
fn global_var_array_init() {
    // Global array → [default; size]
    let cg = CodeGenerator::new();
    let g = decy_hir::HirConstant::new(
        "table".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(256),
        },
        HirExpression::IntLiteral(0),
    );
    let code = cg.generate_global_variable(&g, true, false, false);
    assert!(
        code.contains("static mut table") && code.contains("[0i32; 256]"),
        "Global array → [default; size]: {}",
        code
    );
}

#[test]
fn global_var_null_pointer() {
    // Global pointer = 0 → null_mut()
    let cg = CodeGenerator::new();
    let g = decy_hir::HirConstant::new(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        HirExpression::IntLiteral(0),
    );
    let code = cg.generate_global_variable(&g, true, false, false);
    assert!(
        code.contains("null_mut()"),
        "Global null pointer → null_mut(): {}",
        code
    );
}

#[test]
fn global_var_const_char_ptr() {
    // const char* global → &str
    let cg = CodeGenerator::new();
    let g = decy_hir::HirConstant::new(
        "name".to_string(),
        HirType::Pointer(Box::new(HirType::Char)),
        HirExpression::StringLiteral("test".to_string()),
    );
    let code = cg.generate_global_variable(&g, true, false, true);
    assert!(
        code.contains("const name: &str"),
        "Const char* global → &str: {}",
        code
    );
}

// =============================================================================
// Batch 55: Statement-level branches: VLA, Return main, Realloc assignment,
//           While string deref, If pointer cond, For(;;), global/errno assign
// =============================================================================

#[test]
fn stmt_context_vla_int_declaration() {
    // int arr[n] → let mut arr = vec![0i32; n]
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("n".to_string(), HirType::Int);
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("vec![0i32;") && code.contains("n"),
        "VLA int → vec![0i32; n]: {}",
        code
    );
}

#[test]
fn stmt_context_vla_char_declaration() {
    // char buf[n] → let mut buf = vec![0u8; n]
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "buf".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Char),
            size: None,
        },
        initializer: Some(HirExpression::Variable("size".to_string())),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("vec![0u8;"),
        "VLA char → vec![0u8; size]: {}",
        code
    );
}

#[test]
fn stmt_context_return_main_int() {
    // return 1 in main → std::process::exit(1)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Return(Some(HirExpression::IntLiteral(1)));
    let code = cg.generate_statement_with_context(&stmt, Some("main"), &mut ctx, None);
    assert!(
        code.contains("std::process::exit(1)"),
        "Return in main → exit: {}",
        code
    );
}

#[test]
fn stmt_context_return_main_void() {
    // return in main with no value → exit(0)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Return(None);
    let code = cg.generate_statement_with_context(&stmt, Some("main"), &mut ctx, None);
    assert!(
        code.contains("std::process::exit(0)"),
        "Return void in main → exit(0): {}",
        code
    );
}

#[test]
fn stmt_context_return_main_char_cast() {
    // return char_expr in main → exit(char as i32)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("c".to_string(), HirType::Char);
    let stmt = HirStatement::Return(Some(HirExpression::Variable("c".to_string())));
    let code = cg.generate_statement_with_context(&stmt, Some("main"), &mut ctx, None);
    assert!(
        code.contains("as i32"),
        "Return char in main → as i32 cast: {}",
        code
    );
}

#[test]
fn stmt_context_return_non_main() {
    // return x in non-main → return x
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Return(Some(HirExpression::Variable("result".to_string())));
    let code = cg.generate_statement_with_context(&stmt, Some("foo"), &mut ctx, None);
    assert!(
        code.contains("return result;"),
        "Return in non-main: {}",
        code
    );
}

#[test]
fn stmt_context_return_void_non_main() {
    // return; → return;
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Return(None);
    let code = cg.generate_statement_with_context(&stmt, Some("foo"), &mut ctx, None);
    assert_eq!(code, "return;");
}

#[test]
fn stmt_context_assign_realloc_zero_size() {
    // buf = realloc(buf, 0) → buf.clear()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("buf".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let stmt = HirStatement::Assignment {
        target: "buf".to_string(),
        value: HirExpression::Realloc {
            pointer: Box::new(HirExpression::Variable("buf".to_string())),
            new_size: Box::new(HirExpression::IntLiteral(0)),
        },
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("buf.clear()"),
        "realloc(buf,0) → clear: {}",
        code
    );
}

#[test]
fn stmt_context_assign_realloc_resize() {
    // buf = realloc(buf, n * sizeof(int)) → buf.resize(n, 0i32)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("buf".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let stmt = HirStatement::Assignment {
        target: "buf".to_string(),
        value: HirExpression::Realloc {
            pointer: Box::new(HirExpression::Variable("buf".to_string())),
            new_size: Box::new(HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Multiply,
                left: Box::new(HirExpression::Variable("n".to_string())),
                right: Box::new(HirExpression::Sizeof {
                    type_name: "int".to_string(),
                }),
            }),
        },
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("buf.resize(n,"),
        "realloc resize → buf.resize: {}",
        code
    );
}

#[test]
fn stmt_context_assign_realloc_simple_size() {
    // buf = realloc(buf, size) where size is not n*sizeof → resize with as usize
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("buf".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let stmt = HirStatement::Assignment {
        target: "buf".to_string(),
        value: HirExpression::Realloc {
            pointer: Box::new(HirExpression::Variable("buf".to_string())),
            new_size: Box::new(HirExpression::Variable("new_size".to_string())),
        },
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("resize") && code.contains("as usize"),
        "realloc simple size → resize as usize: {}",
        code
    );
}

#[test]
fn stmt_context_assign_errno() {
    // errno = value → unsafe { ERRNO = value; }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Assignment {
        target: "errno".to_string(),
        value: HirExpression::IntLiteral(22),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("unsafe") && code.contains("ERRNO"),
        "errno assign → unsafe ERRNO: {}",
        code
    );
}

#[test]
fn stmt_context_assign_global() {
    // global = value → unsafe { global = value; }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("count".to_string(), HirType::Int);
    ctx.add_global("count".to_string());
    let stmt = HirStatement::Assignment {
        target: "count".to_string(),
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("unsafe") && code.contains("count = 42"),
        "Global assign → unsafe: {}",
        code
    );
}

#[test]
fn stmt_context_if_raw_pointer_null_check() {
    // if (ptr) → if !ptr.is_null()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let stmt = HirStatement::If {
        condition: HirExpression::Variable("ptr".to_string()),
        then_block: vec![HirStatement::Break],
        else_block: None,
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("!ptr.is_null()"),
        "If pointer → !ptr.is_null(): {}",
        code
    );
}

#[test]
fn stmt_context_if_non_bool_condition() {
    // if (x) → if (x) != 0
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    let stmt = HirStatement::If {
        condition: HirExpression::Variable("x".to_string()),
        then_block: vec![HirStatement::Break],
        else_block: None,
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("!= 0"),
        "If non-bool → != 0: {}",
        code
    );
}

#[test]
fn stmt_context_for_infinite_loop() {
    // for(;;) → loop {}
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::For {
        init: vec![],
        condition: None,
        increment: vec![],
        body: vec![HirStatement::Break],
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("loop {"),
        "for(;;) → loop: {}",
        code
    );
}

#[test]
fn stmt_context_for_with_condition() {
    // for(;cond;) → while cond {}
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::For {
        init: vec![],
        condition: Some(HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::Variable("n".to_string())),
        }),
        increment: vec![],
        body: vec![HirStatement::Break],
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("while i < n"),
        "for with cond → while: {}",
        code
    );
}

#[test]
fn stmt_context_char_array_string_init() {
    // char str[N] = "hello" → let mut str: [u8; N] = *b"hello\0"
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "buf".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Char),
            size: Some(10),
        },
        initializer: Some(HirExpression::StringLiteral("hello".to_string())),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("*b\"hello\\0\""),
        "char array string init → *b\"hello\\0\": {}",
        code
    );
}

#[test]
fn stmt_context_char_ptr_string_literal() {
    // char* s = "hello" → let mut s: &str = "hello"
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "s".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Char)),
        initializer: Some(HirExpression::StringLiteral("hello".to_string())),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("&str") && code.contains("\"hello\""),
        "char* string literal → &str: {}",
        code
    );
}

#[test]
fn stmt_context_var_decl_rename_global_shadow() {
    // Local var that shadows global → rename to _local
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("count".to_string(), HirType::Int);
    ctx.add_global("count".to_string());
    let stmt = HirStatement::VariableDeclaration {
        name: "count".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(
        code.contains("count_local"),
        "Local shadowing global → renamed _local: {}",
        code
    );
}
