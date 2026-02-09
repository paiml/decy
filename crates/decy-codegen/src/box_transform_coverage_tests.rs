//! Coverage tests for box_transform.rs - targeting all uncovered branches.

use crate::box_transform::BoxTransformer;
use decy_analyzer::patterns::BoxCandidate;
use decy_hir::{HirExpression, HirStatement, HirType};

// ============================================================================
// default_value_for_type - all uncovered type variants
// ============================================================================

#[test]
fn test_default_value_unsigned_int() {
    let transformer = BoxTransformer::new();
    let malloc_expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(4)],
    };
    let result = transformer.transform_malloc_to_box(&malloc_expr, &HirType::UnsignedInt);
    match result {
        HirExpression::FunctionCall {
            function,
            arguments,
        } => {
            assert_eq!(function, "Box::new");
            assert_eq!(arguments[0], HirExpression::IntLiteral(0));
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_default_value_float() {
    let transformer = BoxTransformer::new();
    let malloc_expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(4)],
    };
    let result = transformer.transform_malloc_to_box(&malloc_expr, &HirType::Float);
    match result {
        HirExpression::FunctionCall { arguments, .. } => {
            assert_eq!(arguments[0], HirExpression::IntLiteral(0));
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_default_value_double() {
    let transformer = BoxTransformer::new();
    let malloc_expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(8)],
    };
    let result = transformer.transform_malloc_to_box(&malloc_expr, &HirType::Double);
    match result {
        HirExpression::FunctionCall { arguments, .. } => {
            assert_eq!(arguments[0], HirExpression::IntLiteral(0));
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_default_value_signed_char() {
    let transformer = BoxTransformer::new();
    let malloc_expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(1)],
    };
    let result = transformer.transform_malloc_to_box(&malloc_expr, &HirType::SignedChar);
    match result {
        HirExpression::FunctionCall { arguments, .. } => {
            assert_eq!(arguments[0], HirExpression::IntLiteral(0));
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_default_value_option_type() {
    let transformer = BoxTransformer::new();
    let malloc_expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(8)],
    };
    let option_type = HirType::Option(Box::new(HirType::Int));
    let result = transformer.transform_malloc_to_box(&malloc_expr, &option_type);
    match result {
        HirExpression::FunctionCall { arguments, .. } => {
            assert_eq!(arguments[0], HirExpression::NullLiteral);
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_default_value_void_fallback() {
    let transformer = BoxTransformer::new();
    let malloc_expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(0)],
    };
    let result = transformer.transform_malloc_to_box(&malloc_expr, &HirType::Void);
    match result {
        HirExpression::FunctionCall { arguments, .. } => {
            assert_eq!(arguments[0], HirExpression::IntLiteral(0));
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_default_value_pointer_fallback() {
    let transformer = BoxTransformer::new();
    let malloc_expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(8)],
    };
    let pointer_type = HirType::Pointer(Box::new(HirType::Int));
    let result = transformer.transform_malloc_to_box(&malloc_expr, &pointer_type);
    match result {
        HirExpression::FunctionCall { arguments, .. } => {
            assert_eq!(arguments[0], HirExpression::IntLiteral(0));
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_default_value_box_fallback() {
    let transformer = BoxTransformer::new();
    let malloc_expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(8)],
    };
    let box_type = HirType::Box(Box::new(HirType::Int));
    let result = transformer.transform_malloc_to_box(&malloc_expr, &box_type);
    match result {
        HirExpression::FunctionCall { arguments, .. } => {
            assert_eq!(arguments[0], HirExpression::IntLiteral(0));
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_default_value_vec_fallback() {
    let transformer = BoxTransformer::new();
    let malloc_expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(24)],
    };
    let vec_type = HirType::Vec(Box::new(HirType::Int));
    let result = transformer.transform_malloc_to_box(&malloc_expr, &vec_type);
    match result {
        HirExpression::FunctionCall { arguments, .. } => {
            assert_eq!(arguments[0], HirExpression::IntLiteral(0));
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_default_value_reference_fallback() {
    let transformer = BoxTransformer::new();
    let malloc_expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(8)],
    };
    let ref_type = HirType::Reference {
        inner: Box::new(HirType::Int),
        mutable: false,
    };
    let result = transformer.transform_malloc_to_box(&malloc_expr, &ref_type);
    match result {
        HirExpression::FunctionCall { arguments, .. } => {
            assert_eq!(arguments[0], HirExpression::IntLiteral(0));
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_default_value_struct_fallback() {
    let transformer = BoxTransformer::new();
    let malloc_expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(16)],
    };
    let result = transformer.transform_malloc_to_box(&malloc_expr, &HirType::Struct("MyStruct".to_string()));
    match result {
        HirExpression::FunctionCall { arguments, .. } => {
            assert_eq!(arguments[0], HirExpression::IntLiteral(0));
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_default_value_enum_fallback() {
    let transformer = BoxTransformer::new();
    let malloc_expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(4)],
    };
    let result = transformer.transform_malloc_to_box(&malloc_expr, &HirType::Enum("Color".to_string()));
    match result {
        HirExpression::FunctionCall { arguments, .. } => {
            assert_eq!(arguments[0], HirExpression::IntLiteral(0));
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_default_value_union_fallback() {
    let transformer = BoxTransformer::new();
    let malloc_expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(8)],
    };
    let union_type = HirType::Union(vec![
        ("x".to_string(), HirType::Int),
        ("y".to_string(), HirType::Float),
    ]);
    let result = transformer.transform_malloc_to_box(&malloc_expr, &union_type);
    match result {
        HirExpression::FunctionCall { arguments, .. } => {
            assert_eq!(arguments[0], HirExpression::IntLiteral(0));
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_default_value_array_fallback() {
    let transformer = BoxTransformer::new();
    let malloc_expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(40)],
    };
    let array_type = HirType::Array {
        element_type: Box::new(HirType::Int),
        size: Some(10),
    };
    let result = transformer.transform_malloc_to_box(&malloc_expr, &array_type);
    match result {
        HirExpression::FunctionCall { arguments, .. } => {
            assert_eq!(arguments[0], HirExpression::IntLiteral(0));
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_default_value_function_pointer_fallback() {
    let transformer = BoxTransformer::new();
    let malloc_expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(8)],
    };
    let fn_ptr_type = HirType::FunctionPointer {
        param_types: vec![HirType::Int],
        return_type: Box::new(HirType::Int),
    };
    let result = transformer.transform_malloc_to_box(&malloc_expr, &fn_ptr_type);
    match result {
        HirExpression::FunctionCall { arguments, .. } => {
            assert_eq!(arguments[0], HirExpression::IntLiteral(0));
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_default_value_string_literal_fallback() {
    let transformer = BoxTransformer::new();
    let malloc_expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(8)],
    };
    let result = transformer.transform_malloc_to_box(&malloc_expr, &HirType::StringLiteral);
    match result {
        HirExpression::FunctionCall { arguments, .. } => {
            assert_eq!(arguments[0], HirExpression::IntLiteral(0));
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_default_value_owned_string_fallback() {
    let transformer = BoxTransformer::new();
    let malloc_expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(24)],
    };
    let result = transformer.transform_malloc_to_box(&malloc_expr, &HirType::OwnedString);
    match result {
        HirExpression::FunctionCall { arguments, .. } => {
            assert_eq!(arguments[0], HirExpression::IntLiteral(0));
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_default_value_string_reference_fallback() {
    let transformer = BoxTransformer::new();
    let malloc_expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(16)],
    };
    let result = transformer.transform_malloc_to_box(&malloc_expr, &HirType::StringReference);
    match result {
        HirExpression::FunctionCall { arguments, .. } => {
            assert_eq!(arguments[0], HirExpression::IntLiteral(0));
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_default_value_type_alias_fallback() {
    let transformer = BoxTransformer::new();
    let malloc_expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(4)],
    };
    let result = transformer.transform_malloc_to_box(&malloc_expr, &HirType::TypeAlias("size_t".to_string()));
    match result {
        HirExpression::FunctionCall { arguments, .. } => {
            assert_eq!(arguments[0], HirExpression::IntLiteral(0));
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_default_value_mutable_reference_fallback() {
    let transformer = BoxTransformer::new();
    let malloc_expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(8)],
    };
    let ref_type = HirType::Reference {
        inner: Box::new(HirType::Int),
        mutable: true,
    };
    let result = transformer.transform_malloc_to_box(&malloc_expr, &ref_type);
    match result {
        HirExpression::FunctionCall { arguments, .. } => {
            assert_eq!(arguments[0], HirExpression::IntLiteral(0));
        }
        _ => panic!("Expected FunctionCall"),
    }
}

// ============================================================================
// transform_statement - uncovered branches
// ============================================================================

#[test]
fn test_transform_statement_var_decl_non_malloc_function_call() {
    let transformer = BoxTransformer::new();
    let candidate = BoxCandidate {
        variable: "ptr".to_string(),
        malloc_index: 0,
        free_index: None,
    };

    // VariableDeclaration with a function call that is NOT malloc
    let stmt = HirStatement::VariableDeclaration {
        name: "ptr".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::FunctionCall {
            function: "calloc".to_string(),
            arguments: vec![HirExpression::IntLiteral(10), HirExpression::IntLiteral(4)],
        }),
    };

    let transformed = transformer.transform_statement(&stmt, &candidate);
    // Should be unchanged since function is not "malloc"
    assert_eq!(transformed, stmt);
}

#[test]
fn test_transform_statement_var_decl_malloc_non_pointer_type() {
    let transformer = BoxTransformer::new();
    let candidate = BoxCandidate {
        variable: "x".to_string(),
        malloc_index: 0,
        free_index: None,
    };

    // VariableDeclaration with malloc but type is NOT Pointer (edge case)
    let stmt = HirStatement::VariableDeclaration {
        name: "x".to_string(),
        var_type: HirType::Int, // Not a Pointer type
        initializer: Some(HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::IntLiteral(4)],
        }),
    };

    let transformed = transformer.transform_statement(&stmt, &candidate);
    // Should return clone since var_type is not Pointer
    assert_eq!(transformed, stmt);
}

#[test]
fn test_transform_statement_var_decl_no_initializer() {
    let transformer = BoxTransformer::new();
    let candidate = BoxCandidate {
        variable: "ptr".to_string(),
        malloc_index: 0,
        free_index: None,
    };

    // VariableDeclaration with no initializer
    let stmt = HirStatement::VariableDeclaration {
        name: "ptr".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: None,
    };

    let transformed = transformer.transform_statement(&stmt, &candidate);
    assert_eq!(transformed, stmt);
}

#[test]
fn test_transform_statement_var_decl_non_function_call_initializer() {
    let transformer = BoxTransformer::new();
    let candidate = BoxCandidate {
        variable: "ptr".to_string(),
        malloc_index: 0,
        free_index: None,
    };

    // VariableDeclaration with an initializer that is not a FunctionCall
    let stmt = HirStatement::VariableDeclaration {
        name: "ptr".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::Variable("other_ptr".to_string())),
    };

    let transformed = transformer.transform_statement(&stmt, &candidate);
    assert_eq!(transformed, stmt);
}

#[test]
fn test_transform_statement_assignment_non_malloc_function() {
    let transformer = BoxTransformer::new();
    let candidate = BoxCandidate {
        variable: "ptr".to_string(),
        malloc_index: 0,
        free_index: None,
    };

    // Assignment with a function call that is NOT malloc
    let stmt = HirStatement::Assignment {
        target: "ptr".to_string(),
        value: HirExpression::FunctionCall {
            function: "realloc".to_string(),
            arguments: vec![
                HirExpression::Variable("ptr".to_string()),
                HirExpression::IntLiteral(200),
            ],
        },
    };

    let transformed = transformer.transform_statement(&stmt, &candidate);
    assert_eq!(transformed, stmt);
}

#[test]
fn test_transform_statement_assignment_non_function_value() {
    let transformer = BoxTransformer::new();
    let candidate = BoxCandidate {
        variable: "ptr".to_string(),
        malloc_index: 0,
        free_index: None,
    };

    // Assignment with a value that is not a FunctionCall
    let stmt = HirStatement::Assignment {
        target: "ptr".to_string(),
        value: HirExpression::Variable("other".to_string()),
    };

    let transformed = transformer.transform_statement(&stmt, &candidate);
    assert_eq!(transformed, stmt);
}

#[test]
fn test_transform_statement_wildcard_return() {
    let transformer = BoxTransformer::new();
    let candidate = BoxCandidate {
        variable: "x".to_string(),
        malloc_index: 0,
        free_index: None,
    };

    // A Return statement hits the wildcard arm
    let stmt = HirStatement::Return(Some(HirExpression::IntLiteral(0)));
    let transformed = transformer.transform_statement(&stmt, &candidate);
    assert_eq!(transformed, stmt);
}

#[test]
fn test_transform_statement_wildcard_break() {
    let transformer = BoxTransformer::new();
    let candidate = BoxCandidate {
        variable: "x".to_string(),
        malloc_index: 0,
        free_index: None,
    };

    let stmt = HirStatement::Break;
    let transformed = transformer.transform_statement(&stmt, &candidate);
    assert_eq!(transformed, stmt);
}

#[test]
fn test_transform_statement_wildcard_continue() {
    let transformer = BoxTransformer::new();
    let candidate = BoxCandidate {
        variable: "x".to_string(),
        malloc_index: 0,
        free_index: None,
    };

    let stmt = HirStatement::Continue;
    let transformed = transformer.transform_statement(&stmt, &candidate);
    assert_eq!(transformed, stmt);
}

#[test]
fn test_transform_statement_wildcard_if() {
    let transformer = BoxTransformer::new();
    let candidate = BoxCandidate {
        variable: "x".to_string(),
        malloc_index: 0,
        free_index: None,
    };

    let stmt = HirStatement::If {
        condition: HirExpression::IntLiteral(1),
        then_block: vec![],
        else_block: None,
    };
    let transformed = transformer.transform_statement(&stmt, &candidate);
    assert_eq!(transformed, stmt);
}

#[test]
fn test_transform_statement_wildcard_while() {
    let transformer = BoxTransformer::new();
    let candidate = BoxCandidate {
        variable: "x".to_string(),
        malloc_index: 0,
        free_index: None,
    };

    let stmt = HirStatement::While {
        condition: HirExpression::IntLiteral(1),
        body: vec![],
    };
    let transformed = transformer.transform_statement(&stmt, &candidate);
    assert_eq!(transformed, stmt);
}

// ============================================================================
// BoxTransformer Default impl
// ============================================================================

#[test]
fn test_box_transformer_default() {
    let transformer = BoxTransformer::default();
    // Verify it works the same as new()
    let malloc_expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(4)],
    };
    let result = transformer.transform_malloc_to_box(&malloc_expr, &HirType::Int);
    match result {
        HirExpression::FunctionCall { function, .. } => {
            assert_eq!(function, "Box::new");
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_box_transformer_debug() {
    let transformer = BoxTransformer::new();
    let debug_str = format!("{:?}", transformer);
    assert_eq!(debug_str, "BoxTransformer");
}

#[test]
fn test_box_transformer_clone() {
    let transformer = BoxTransformer::new();
    let cloned = transformer.clone();
    let malloc_expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(4)],
    };
    // Both should produce the same result
    let result1 = transformer.transform_malloc_to_box(&malloc_expr, &HirType::Int);
    let result2 = cloned.transform_malloc_to_box(&malloc_expr, &HirType::Int);
    assert_eq!(result1, result2);
}

// ============================================================================
// transform_statement - VariableDeclaration with Pointer to various pointee types
// ============================================================================

#[test]
fn test_transform_var_decl_malloc_with_float_pointer() {
    let transformer = BoxTransformer::new();
    let candidate = BoxCandidate {
        variable: "fptr".to_string(),
        malloc_index: 0,
        free_index: Some(5),
    };

    let stmt = HirStatement::VariableDeclaration {
        name: "fptr".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Float)),
        initializer: Some(HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::IntLiteral(4)],
        }),
    };

    let transformed = transformer.transform_statement(&stmt, &candidate);
    match transformed {
        HirStatement::VariableDeclaration {
            var_type: HirType::Box(inner),
            initializer: Some(HirExpression::FunctionCall { function, .. }),
            ..
        } => {
            assert_eq!(*inner, HirType::Float);
            assert_eq!(function, "Box::new");
        }
        _ => panic!("Expected VariableDeclaration with Box<Float>"),
    }
}

#[test]
fn test_transform_var_decl_malloc_with_double_pointer() {
    let transformer = BoxTransformer::new();
    let candidate = BoxCandidate {
        variable: "dptr".to_string(),
        malloc_index: 0,
        free_index: None,
    };

    let stmt = HirStatement::VariableDeclaration {
        name: "dptr".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Double)),
        initializer: Some(HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::IntLiteral(8)],
        }),
    };

    let transformed = transformer.transform_statement(&stmt, &candidate);
    match transformed {
        HirStatement::VariableDeclaration {
            var_type: HirType::Box(inner),
            ..
        } => {
            assert_eq!(*inner, HirType::Double);
        }
        _ => panic!("Expected VariableDeclaration with Box<Double>"),
    }
}

#[test]
fn test_transform_var_decl_malloc_with_struct_pointer() {
    let transformer = BoxTransformer::new();
    let candidate = BoxCandidate {
        variable: "sptr".to_string(),
        malloc_index: 0,
        free_index: Some(10),
    };

    let stmt = HirStatement::VariableDeclaration {
        name: "sptr".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
        initializer: Some(HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::IntLiteral(32)],
        }),
    };

    let transformed = transformer.transform_statement(&stmt, &candidate);
    match transformed {
        HirStatement::VariableDeclaration {
            name,
            var_type: HirType::Box(inner),
            initializer: Some(HirExpression::FunctionCall { function, .. }),
        } => {
            assert_eq!(name, "sptr");
            assert_eq!(*inner, HirType::Struct("Node".to_string()));
            assert_eq!(function, "Box::new");
        }
        _ => panic!("Expected VariableDeclaration with Box<Struct>"),
    }
}

// ============================================================================
// Edge cases for BoxCandidate with free_index
// ============================================================================

#[test]
fn test_transform_with_free_index_present() {
    let transformer = BoxTransformer::new();
    let candidate = BoxCandidate {
        variable: "ptr".to_string(),
        malloc_index: 2,
        free_index: Some(8),
    };

    let stmt = HirStatement::Assignment {
        target: "ptr".to_string(),
        value: HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::IntLiteral(100)],
        },
    };

    let transformed = transformer.transform_statement(&stmt, &candidate);
    match transformed {
        HirStatement::Assignment {
            target,
            value: HirExpression::FunctionCall { function, .. },
        } => {
            assert_eq!(target, "ptr");
            assert_eq!(function, "Box::new");
        }
        _ => panic!("Expected Assignment with Box::new"),
    }
}
