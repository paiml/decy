/// Sprint 19 Codegen Tests
/// Tests for DECY-064: Codegen integration of Sprint 19 features
///
/// Tests code generation for:
/// - Cast expressions (DECY-059)
/// - Compound literals (DECY-060)
use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirType};

#[test]
fn test_codegen_cast_int_to_int() {
    // C: (int)x
    // Rust: x as i32

    let cast_expr = HirExpression::Cast {
        target_type: HirType::Int,
        expr: Box::new(HirExpression::Variable("x".to_string())),
    };

    let codegen = CodeGenerator::new();
    let result = codegen.generate_expression(&cast_expr);

    assert_eq!(result, "x as i32", "Should generate 'x as i32'");
}

#[test]
fn test_codegen_cast_float_to_int() {
    // C: (int)3.14
    // Rust: 3.14 as i32 (but we use IntLiteral(3) for simplicity)

    let cast_expr = HirExpression::Cast {
        target_type: HirType::Int,
        expr: Box::new(HirExpression::IntLiteral(3)),
    };

    let codegen = CodeGenerator::new();
    let result = codegen.generate_expression(&cast_expr);

    assert_eq!(result, "3 as i32", "Should generate cast with as operator");
}

#[test]
fn test_codegen_cast_pointer() {
    // C: (void*)ptr
    // Rust: ptr as *mut std::ffi::c_void

    let cast_expr = HirExpression::Cast {
        target_type: HirType::Pointer(Box::new(HirType::Void)),
        expr: Box::new(HirExpression::Variable("ptr".to_string())),
    };

    let codegen = CodeGenerator::new();
    let result = codegen.generate_expression(&cast_expr);

    assert!(result.contains(" as "), "Should use 'as' operator");
    assert!(result.contains("ptr"), "Should include variable name");
}

#[test]
fn test_codegen_nested_cast() {
    // C: (int)((long)x)
    // Rust: (x as i64) as i32

    let inner_cast = HirExpression::Cast {
        target_type: HirType::Int, // Simplified: using Int for long
        expr: Box::new(HirExpression::Variable("x".to_string())),
    };

    let outer_cast = HirExpression::Cast {
        target_type: HirType::Int,
        expr: Box::new(inner_cast),
    };

    let codegen = CodeGenerator::new();
    let result = codegen.generate_expression(&outer_cast);

    // Should contain nested casts
    assert!(result.contains(" as "), "Should have cast operators");
}

#[test]
fn test_codegen_compound_literal_struct() {
    // C: (struct Point){10, 20}
    // Rust: Point { field0: 10, field1: 20 } (simplified - ideally with actual field names)

    let compound_expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Point".to_string()),
        initializers: vec![HirExpression::IntLiteral(10), HirExpression::IntLiteral(20)],
    };

    let codegen = CodeGenerator::new();
    let result = codegen.generate_expression(&compound_expr);

    assert!(result.contains("Point"), "Should include struct name");
    assert!(result.contains("10"), "Should include first initializer");
    assert!(result.contains("20"), "Should include second initializer");
    assert!(
        result.contains("{") && result.contains("}"),
        "Should have braces"
    );
}

#[test]
fn test_codegen_compound_literal_array() {
    // C: (int[]){1, 2, 3, 4, 5}
    // Rust: vec![1, 2, 3, 4, 5] or [1, 2, 3, 4, 5]

    let compound_expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(5),
        },
        initializers: vec![
            HirExpression::IntLiteral(1),
            HirExpression::IntLiteral(2),
            HirExpression::IntLiteral(3),
            HirExpression::IntLiteral(4),
            HirExpression::IntLiteral(5),
        ],
    };

    let codegen = CodeGenerator::new();
    let result = codegen.generate_expression(&compound_expr);

    // Should generate either vec![...] or [...]
    assert!(
        result.contains("vec![") || result.contains("["),
        "Should have array/vec syntax"
    );
    assert!(result.contains("1"), "Should include first element");
    assert!(result.contains("5"), "Should include last element");
}

#[test]
fn test_codegen_cast_in_binary_op() {
    // C: (int)x + 10
    // Rust: (x as i32) + 10

    use decy_hir::BinaryOperator;

    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Cast {
            target_type: HirType::Int,
            expr: Box::new(HirExpression::Variable("x".to_string())),
        }),
        right: Box::new(HirExpression::IntLiteral(10)),
    };

    let codegen = CodeGenerator::new();
    let result = codegen.generate_expression(&expr);

    assert!(result.contains(" as "), "Should have cast");
    assert!(result.contains("+"), "Should have addition");
    assert!(result.contains("10"), "Should have literal");
}

#[test]
fn test_codegen_compound_literal_empty() {
    // C: (struct Empty){}
    // Rust: Empty {} or Empty::default()

    let compound_expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Empty".to_string()),
        initializers: vec![],
    };

    let codegen = CodeGenerator::new();
    let result = codegen.generate_expression(&compound_expr);

    assert!(result.contains("Empty"), "Should include struct name");
}
