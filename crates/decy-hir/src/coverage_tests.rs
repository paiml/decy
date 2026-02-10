//! Coverage tests for decy-hir (targeting 95%+ coverage)
//!
//! These tests target uncovered code paths in the HIR types.

use super::*;

// ============================================================================
// HirStructField Coverage Tests
// ============================================================================

#[test]
fn test_hir_struct_field_accessors() {
    let field = HirStructField::new("x".to_string(), HirType::Int);
    assert_eq!(field.name(), "x");
    assert_eq!(field.field_type(), &HirType::Int);
}

#[test]
fn test_hir_struct_field_clone_and_debug() {
    let field = HirStructField::new("y".to_string(), HirType::Float);
    let cloned = field.clone();
    assert_eq!(cloned.name(), field.name());

    let debug_str = format!("{:?}", field);
    assert!(debug_str.contains("HirStructField"));
}

// ============================================================================
// HirStruct Coverage Tests
// ============================================================================

#[test]
fn test_hir_struct_accessors() {
    let fields = vec![
        HirStructField::new("x".to_string(), HirType::Int),
        HirStructField::new("y".to_string(), HirType::Int),
    ];
    let s = HirStruct::new("Point".to_string(), fields);
    assert_eq!(s.name(), "Point");
    assert_eq!(s.fields().len(), 2);
}

// ============================================================================
// HirEnum Coverage Tests
// ============================================================================

#[test]
fn test_hir_enum_accessors() {
    let variants = vec![
        HirEnumVariant::new("A".to_string(), Some(0)),
        HirEnumVariant::new("B".to_string(), Some(1)),
    ];
    let e = HirEnum::new("Letters".to_string(), variants);
    assert_eq!(e.name(), "Letters");
    assert_eq!(e.variants().len(), 2);
}

#[test]
fn test_hir_enum_variant_accessors() {
    let v = HirEnumVariant::new("RED".to_string(), Some(1));
    assert_eq!(v.name(), "RED");
    assert_eq!(v.value(), Some(1));

    let v_none = HirEnumVariant::new("AUTO".to_string(), None);
    assert_eq!(v_none.value(), None);
}

// ============================================================================
// HirTypedef Coverage Tests
// ============================================================================

#[test]
fn test_hir_typedef_accessors() {
    let td = HirTypedef::new("uint".to_string(), HirType::UnsignedInt);
    assert_eq!(td.name(), "uint");
    assert_eq!(td.underlying_type(), &HirType::UnsignedInt);
}

#[test]
fn test_hir_typedef_clone_and_debug() {
    let td = HirTypedef::new(
        "int_ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let cloned = td.clone();
    assert_eq!(cloned.name(), td.name());

    let debug = format!("{:?}", td);
    assert!(debug.contains("HirTypedef"));
}

// ============================================================================
// HirConstant Coverage Tests
// ============================================================================

#[test]
fn test_hir_constant_accessors() {
    let c = HirConstant::new(
        "MAX".to_string(),
        HirType::Int,
        HirExpression::IntLiteral(100),
    );
    assert_eq!(c.name(), "MAX");
    assert_eq!(c.const_type(), &HirType::Int);
}

// ============================================================================
// HirMacroDefinition Coverage Tests
// ============================================================================

#[test]
fn test_hir_macro_definition_object_like() {
    let macro_def = HirMacroDefinition::new_object_like("MAX".to_string(), "100".to_string());
    assert_eq!(macro_def.name(), "MAX");
    assert!(macro_def.parameters().is_empty());
    assert_eq!(macro_def.body(), "100");
    assert!(!macro_def.is_function_like());
}

#[test]
fn test_hir_macro_definition_function_like() {
    let macro_def = HirMacroDefinition::new_function_like(
        "ADD".to_string(),
        vec!["a".to_string(), "b".to_string()],
        "((a) + (b))".to_string(),
    );
    assert_eq!(macro_def.parameters().len(), 2);
    assert!(macro_def.is_function_like());
}

// ============================================================================
// HirParameter Coverage Tests
// ============================================================================

#[test]
fn test_hir_parameter_accessors() {
    let param = HirParameter::new("x".to_string(), HirType::Int);
    assert_eq!(param.name(), "x");
    assert_eq!(param.param_type(), &HirType::Int);
}

#[test]
fn test_hir_parameter_with_pointer() {
    let param = HirParameter::new("ptr".to_string(), HirType::Pointer(Box::new(HirType::Char)));
    assert!(matches!(param.param_type(), HirType::Pointer(_)));
}

// ============================================================================
// HirFunction Coverage Tests
// ============================================================================

#[test]
fn test_hir_function_accessors() {
    let func = HirFunction::new("test".to_string(), HirType::Void, vec![]);
    assert_eq!(func.name(), "test");
    assert_eq!(func.return_type(), &HirType::Void);
    assert!(func.parameters().is_empty());
}

#[test]
fn test_hir_function_with_params() {
    let params = vec![
        HirParameter::new("a".to_string(), HirType::Int),
        HirParameter::new("b".to_string(), HirType::Int),
    ];
    let func = HirFunction::new("add".to_string(), HirType::Int, params);
    assert_eq!(func.parameters().len(), 2);
}

// ============================================================================
// HirStatement Coverage Tests
// ============================================================================

#[test]
fn test_hir_statement_variable_declaration() {
    let stmt = HirStatement::VariableDeclaration {
        name: "x".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::IntLiteral(0)),
    };

    if let HirStatement::VariableDeclaration { name, .. } = stmt {
        assert_eq!(name, "x");
    }
}

#[test]
fn test_hir_statement_assignment() {
    let stmt = HirStatement::Assignment {
        target: "x".to_string(),
        value: HirExpression::IntLiteral(42),
    };

    if let HirStatement::Assignment { target, value } = stmt {
        assert_eq!(target, "x");
        assert_eq!(value, HirExpression::IntLiteral(42));
    }
}

#[test]
fn test_hir_statement_return() {
    let stmt_with_val = HirStatement::Return(Some(HirExpression::IntLiteral(0)));
    let stmt_void = HirStatement::Return(None);

    assert!(matches!(stmt_with_val, HirStatement::Return(Some(_))));
    assert!(matches!(stmt_void, HirStatement::Return(None)));
}

#[test]
fn test_hir_statement_if() {
    let stmt = HirStatement::If {
        condition: HirExpression::IntLiteral(1),
        then_block: vec![],
        else_block: None,
    };

    assert!(matches!(stmt, HirStatement::If { .. }));
}

#[test]
fn test_hir_statement_if_else() {
    let stmt = HirStatement::If {
        condition: HirExpression::IntLiteral(0),
        then_block: vec![HirStatement::Break],
        else_block: Some(vec![HirStatement::Continue]),
    };

    if let HirStatement::If { else_block, .. } = stmt {
        assert!(else_block.is_some());
    }
}

#[test]
fn test_hir_statement_while() {
    let stmt = HirStatement::While {
        condition: HirExpression::IntLiteral(1),
        body: vec![],
    };

    assert!(matches!(stmt, HirStatement::While { .. }));
}

#[test]
fn test_hir_statement_for() {
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
        increment: vec![],
        body: vec![],
    };

    assert!(matches!(stmt, HirStatement::For { .. }));
}

#[test]
fn test_hir_statement_switch() {
    let stmt = HirStatement::Switch {
        condition: HirExpression::Variable("x".to_string()),
        cases: vec![],
        default_case: None,
    };

    assert!(matches!(stmt, HirStatement::Switch { .. }));
}

#[test]
fn test_hir_statement_break_continue() {
    let break_stmt = HirStatement::Break;
    let continue_stmt = HirStatement::Continue;

    assert!(matches!(break_stmt, HirStatement::Break));
    assert!(matches!(continue_stmt, HirStatement::Continue));
}

// ============================================================================
// HirExpression Coverage Tests
// ============================================================================

#[test]
fn test_hir_expression_literals() {
    let int_lit = HirExpression::IntLiteral(42);
    let float_lit = HirExpression::FloatLiteral("3.14".to_string());
    let char_lit = HirExpression::CharLiteral(65); // 'A'
    let string_lit = HirExpression::StringLiteral("hello".to_string());

    assert_eq!(int_lit, HirExpression::IntLiteral(42));
    assert!(matches!(float_lit, HirExpression::FloatLiteral(_)));
    assert!(matches!(char_lit, HirExpression::CharLiteral(_)));
    assert!(matches!(string_lit, HirExpression::StringLiteral(_)));
}

#[test]
fn test_hir_expression_variable() {
    let var = HirExpression::Variable("foo".to_string());
    assert_eq!(var, HirExpression::Variable("foo".to_string()));
}

#[test]
fn test_hir_expression_binary_op() {
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::IntLiteral(1)),
        right: Box::new(HirExpression::IntLiteral(2)),
    };

    if let HirExpression::BinaryOp { op, .. } = expr {
        assert_eq!(op, BinaryOperator::Add);
    }
}

#[test]
fn test_hir_expression_unary_op() {
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::Minus,
        operand: Box::new(HirExpression::IntLiteral(42)),
    };

    if let HirExpression::UnaryOp { op, .. } = expr {
        assert_eq!(op, UnaryOperator::Minus);
    }
}

#[test]
fn test_hir_expression_function_call() {
    let expr = HirExpression::FunctionCall {
        function: "test".to_string(),
        arguments: vec![HirExpression::IntLiteral(1)],
    };

    if let HirExpression::FunctionCall {
        function,
        arguments,
    } = expr
    {
        assert_eq!(function, "test");
        assert_eq!(arguments.len(), 1);
    }
}

#[test]
fn test_hir_expression_array_index() {
    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
    };

    assert!(matches!(expr, HirExpression::ArrayIndex { .. }));
}

#[test]
fn test_hir_expression_field_access() {
    let expr = HirExpression::FieldAccess {
        object: Box::new(HirExpression::Variable("point".to_string())),
        field: "x".to_string(),
    };

    if let HirExpression::FieldAccess { field, .. } = expr {
        assert_eq!(field, "x");
    }
}

#[test]
fn test_hir_expression_cast() {
    let expr = HirExpression::Cast {
        expr: Box::new(HirExpression::IntLiteral(42)),
        target_type: HirType::Float,
    };

    if let HirExpression::Cast { target_type, .. } = expr {
        assert_eq!(target_type, HirType::Float);
    }
}

#[test]
fn test_hir_expression_sizeof() {
    let sizeof_type = HirExpression::Sizeof {
        type_name: "int".to_string(),
    };
    assert!(matches!(sizeof_type, HirExpression::Sizeof { .. }));
}

#[test]
fn test_hir_expression_address_of() {
    let expr = HirExpression::AddressOf(Box::new(HirExpression::Variable("x".to_string())));
    assert!(matches!(expr, HirExpression::AddressOf(_)));
}

#[test]
fn test_hir_expression_dereference() {
    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("ptr".to_string())));
    assert!(matches!(expr, HirExpression::Dereference(_)));
}

#[test]
fn test_hir_expression_ternary() {
    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::IntLiteral(1)),
        then_expr: Box::new(HirExpression::IntLiteral(2)),
        else_expr: Box::new(HirExpression::IntLiteral(3)),
    };

    assert!(matches!(expr, HirExpression::Ternary { .. }));
}

// ============================================================================
// BinaryOperator Coverage Tests
// ============================================================================

#[test]
fn test_binary_operators_all() {
    let ops = [
        BinaryOperator::Add,
        BinaryOperator::Subtract,
        BinaryOperator::Multiply,
        BinaryOperator::Divide,
        BinaryOperator::Modulo,
        BinaryOperator::Equal,
        BinaryOperator::NotEqual,
        BinaryOperator::LessThan,
        BinaryOperator::GreaterThan,
        BinaryOperator::LessEqual,
        BinaryOperator::GreaterEqual,
        BinaryOperator::LogicalAnd,
        BinaryOperator::LogicalOr,
        BinaryOperator::BitwiseAnd,
        BinaryOperator::BitwiseOr,
        BinaryOperator::BitwiseXor,
        BinaryOperator::LeftShift,
        BinaryOperator::RightShift,
    ];

    for op in ops {
        let debug = format!("{:?}", op);
        assert!(!debug.is_empty());
    }
}

// ============================================================================
// UnaryOperator Coverage Tests
// ============================================================================

#[test]
fn test_unary_operators_all() {
    let ops = [
        UnaryOperator::Minus,
        UnaryOperator::LogicalNot,
        UnaryOperator::BitwiseNot,
        UnaryOperator::PreIncrement,
        UnaryOperator::PreDecrement,
        UnaryOperator::PostIncrement,
        UnaryOperator::PostDecrement,
        UnaryOperator::AddressOf,
    ];

    for op in ops {
        let debug = format!("{:?}", op);
        assert!(!debug.is_empty());
    }
}

// ============================================================================
// SwitchCase Coverage Tests
// ============================================================================

#[test]
fn test_switch_case_fields() {
    let case = SwitchCase {
        value: Some(HirExpression::IntLiteral(1)),
        body: vec![HirStatement::Break],
    };
    assert!(matches!(case.value, Some(HirExpression::IntLiteral(1))));
    assert_eq!(case.body.len(), 1);
}

#[test]
fn test_switch_case_clone_and_debug() {
    let case = SwitchCase {
        value: Some(HirExpression::IntLiteral(42)),
        body: vec![HirStatement::Return(None)],
    };
    let cloned = case.clone();
    assert_eq!(cloned.body.len(), 1);

    let debug = format!("{:?}", case);
    assert!(debug.contains("SwitchCase"));
}
