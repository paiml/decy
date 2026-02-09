//! Deep coverage tests for parser.rs targeting uncovered branches.
//!
//! Targets:
//! - `extract_inc_dec_stmt` (line 1370): prefix/postfix ++/--, member access, array index
//! - `extract_binary_operator` (line 3130): all operator variants and precedence selection
//! - `extract_for_stmt` (line 1850): standard for, empty parts, nested, comma operator

use decy_parser::parser::{BinaryOperator, Expression, Statement};
use decy_parser::CParser;

/// Helper: parse C code and return the AST
fn parse_c(code: &str) -> decy_parser::Ast {
    let parser = CParser::new().expect("Parser creation failed");
    parser.parse(code).expect("Parse failed")
}

/// Helper: get first function from AST
fn first_func(ast: &decy_parser::Ast) -> &decy_parser::Function {
    ast.functions()
        .first()
        .expect("Expected at least one function")
}

/// Helper: find a function by name
fn find_func<'a>(ast: &'a decy_parser::Ast, name: &str) -> &'a decy_parser::Function {
    ast.functions()
        .iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("Function '{}' not found", name))
}

// ============================================================================
// INCREMENT / DECREMENT: Prefix and Postfix on Simple Variables
// ============================================================================

#[test]
fn test_prefix_increment_variable() {
    let ast = parse_c("void f() { int x = 0; ++x; }");
    let func = first_func(&ast);
    let has_pre_inc = func.body.iter().any(|s| {
        matches!(s, Statement::PreIncrement { target } if target == "x")
    });
    assert!(has_pre_inc, "Expected PreIncrement for x, got: {:?}", func.body);
}

#[test]
fn test_postfix_increment_variable() {
    let ast = parse_c("void f() { int x = 0; x++; }");
    let func = first_func(&ast);
    let has_post_inc = func.body.iter().any(|s| {
        matches!(s, Statement::PostIncrement { target } if target == "x")
    });
    assert!(has_post_inc, "Expected PostIncrement for x, got: {:?}", func.body);
}

#[test]
fn test_prefix_decrement_variable() {
    let ast = parse_c("void f() { int x = 5; --x; }");
    let func = first_func(&ast);
    let has_pre_dec = func.body.iter().any(|s| {
        matches!(s, Statement::PreDecrement { target } if target == "x")
    });
    assert!(has_pre_dec, "Expected PreDecrement for x, got: {:?}", func.body);
}

#[test]
fn test_postfix_decrement_variable() {
    let ast = parse_c("void f() { int x = 5; x--; }");
    let func = first_func(&ast);
    let has_post_dec = func.body.iter().any(|s| {
        matches!(s, Statement::PostDecrement { target } if target == "x")
    });
    assert!(has_post_dec, "Expected PostDecrement for x, got: {:?}", func.body);
}

#[test]
fn test_prefix_increment_different_var_name() {
    let ast = parse_c("void f() { int counter = 0; ++counter; }");
    let func = first_func(&ast);
    let has_pre_inc = func.body.iter().any(|s| {
        matches!(s, Statement::PreIncrement { target } if target == "counter")
    });
    assert!(has_pre_inc, "Expected PreIncrement for counter, got: {:?}", func.body);
}

#[test]
fn test_postfix_increment_different_var_name() {
    let ast = parse_c("void f() { int idx = 0; idx++; }");
    let func = first_func(&ast);
    let has_post_inc = func.body.iter().any(|s| {
        matches!(s, Statement::PostIncrement { target } if target == "idx")
    });
    assert!(has_post_inc, "Expected PostIncrement for idx, got: {:?}", func.body);
}

// ============================================================================
// INCREMENT / DECREMENT: Multiple in same function
// ============================================================================

#[test]
fn test_multiple_inc_dec_in_function() {
    let ast = parse_c(
        "void f() { int a = 0; int b = 0; a++; ++b; a--; --b; }",
    );
    let func = first_func(&ast);
    let post_inc = func.body.iter().any(|s| matches!(s, Statement::PostIncrement { target } if target == "a"));
    let pre_inc = func.body.iter().any(|s| matches!(s, Statement::PreIncrement { target } if target == "b"));
    let post_dec = func.body.iter().any(|s| matches!(s, Statement::PostDecrement { target } if target == "a"));
    let pre_dec = func.body.iter().any(|s| matches!(s, Statement::PreDecrement { target } if target == "b"));
    assert!(post_inc, "Expected PostIncrement for a");
    assert!(pre_inc, "Expected PreIncrement for b");
    assert!(post_dec, "Expected PostDecrement for a");
    assert!(pre_dec, "Expected PreDecrement for b");
}

// ============================================================================
// INCREMENT / DECREMENT: Struct member via pointer (->)
// ============================================================================

#[test]
fn test_pointer_field_post_increment() {
    let code = r#"
typedef struct { int count; } Data;
void f(Data* d) {
    d->count++;
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    // Should be a FieldAssignment, not a simple PostIncrement
    let has_field_assign = func.body.iter().any(|s| {
        matches!(s, Statement::FieldAssignment { field, .. } if field == "count")
    });
    assert!(
        has_field_assign,
        "Expected FieldAssignment for d->count++, got: {:?}",
        func.body
    );
}

#[test]
fn test_pointer_field_pre_increment() {
    let code = r#"
typedef struct { int val; } Node;
void f(Node* n) {
    ++n->val;
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let has_field_assign = func.body.iter().any(|s| {
        matches!(s, Statement::FieldAssignment { field, .. } if field == "val")
    });
    assert!(
        has_field_assign,
        "Expected FieldAssignment for ++n->val, got: {:?}",
        func.body
    );
}

#[test]
fn test_pointer_field_post_decrement() {
    let code = r#"
typedef struct { int refs; } Obj;
void f(Obj* o) {
    o->refs--;
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let has_field_assign = func.body.iter().any(|s| {
        matches!(s, Statement::FieldAssignment { field, .. } if field == "refs")
    });
    assert!(
        has_field_assign,
        "Expected FieldAssignment for o->refs--, got: {:?}",
        func.body
    );
}

#[test]
fn test_pointer_field_pre_decrement() {
    let code = r#"
typedef struct { int depth; } Stack;
void f(Stack* s) {
    --s->depth;
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let has_field_assign = func.body.iter().any(|s| {
        matches!(s, Statement::FieldAssignment { field, .. } if field == "depth")
    });
    assert!(
        has_field_assign,
        "Expected FieldAssignment for --s->depth, got: {:?}",
        func.body
    );
}

// ============================================================================
// INCREMENT / DECREMENT: Struct member via dot access
// ============================================================================

#[test]
fn test_dot_field_post_increment() {
    let code = r#"
typedef struct { int x; } Point;
void f() {
    Point p;
    p.x = 0;
    p.x++;
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let has_field_assign = func.body.iter().any(|s| {
        matches!(s, Statement::FieldAssignment { field, .. } if field == "x")
    });
    assert!(
        has_field_assign,
        "Expected FieldAssignment for p.x++, got: {:?}",
        func.body
    );
}

#[test]
fn test_dot_field_post_decrement() {
    let code = r#"
typedef struct { int y; } Vec2;
void f() {
    Vec2 v;
    v.y = 10;
    v.y--;
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let has_field_assign = func.body.iter().any(|s| {
        matches!(s, Statement::FieldAssignment { field, .. } if field == "y")
    });
    assert!(
        has_field_assign,
        "Expected FieldAssignment for v.y--, got: {:?}",
        func.body
    );
}

// ============================================================================
// INCREMENT / DECREMENT: Array subscript
// ============================================================================

#[test]
fn test_array_index_post_increment() {
    let code = r#"
void f() {
    int arr[10];
    int i = 0;
    arr[i]++;
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    // Should produce ArrayIndexAssignment
    let has_arr_assign = func.body.iter().any(|s| {
        matches!(s, Statement::ArrayIndexAssignment { .. })
    });
    assert!(
        has_arr_assign,
        "Expected ArrayIndexAssignment for arr[i]++, got: {:?}",
        func.body
    );
}

#[test]
fn test_array_index_post_decrement() {
    let code = r#"
void f() {
    int counts[5];
    counts[2]--;
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let has_arr_assign = func.body.iter().any(|s| {
        matches!(s, Statement::ArrayIndexAssignment { .. })
    });
    assert!(
        has_arr_assign,
        "Expected ArrayIndexAssignment for counts[2]--, got: {:?}",
        func.body
    );
}

#[test]
fn test_array_index_pre_increment() {
    let code = r#"
void f() {
    int data[8];
    ++data[0];
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let has_arr_assign = func.body.iter().any(|s| {
        matches!(s, Statement::ArrayIndexAssignment { .. })
    });
    assert!(
        has_arr_assign,
        "Expected ArrayIndexAssignment for ++data[0], got: {:?}",
        func.body
    );
}

#[test]
fn test_array_complex_index_increment() {
    // Array subscript with expression index: ndigit[c - '0']++
    let code = r#"
void f(int c) {
    int ndigit[10];
    ndigit[c - 48]++;
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let has_arr_assign = func.body.iter().any(|s| {
        matches!(s, Statement::ArrayIndexAssignment { .. })
    });
    assert!(
        has_arr_assign,
        "Expected ArrayIndexAssignment for ndigit[c-48]++, got: {:?}",
        func.body
    );
}

// ============================================================================
// BINARY OPERATORS: Arithmetic
// ============================================================================

#[test]
fn test_binop_add() {
    let ast = parse_c("int f(int a, int b) { int r; r = a + b; return r; }");
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::BinaryOp { op: BinaryOperator::Add, .. }),
            "Expected Add, got: {:?}", value
        );
    } else {
        panic!("No assignment found");
    }
}

#[test]
fn test_binop_subtract() {
    let ast = parse_c("int f(int a, int b) { int r; r = a - b; return r; }");
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::BinaryOp { op: BinaryOperator::Subtract, .. }),
            "Expected Subtract, got: {:?}", value
        );
    } else {
        panic!("No assignment found");
    }
}

#[test]
fn test_binop_multiply() {
    let ast = parse_c("int f(int a, int b) { int r; r = a * b; return r; }");
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::BinaryOp { op: BinaryOperator::Multiply, .. }),
            "Expected Multiply, got: {:?}", value
        );
    } else {
        panic!("No assignment found");
    }
}

#[test]
fn test_binop_divide() {
    let ast = parse_c("int f(int a, int b) { int r; r = a / b; return r; }");
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::BinaryOp { op: BinaryOperator::Divide, .. }),
            "Expected Divide, got: {:?}", value
        );
    } else {
        panic!("No assignment found");
    }
}

#[test]
fn test_binop_modulo() {
    let ast = parse_c("int f(int a, int b) { int r; r = a % b; return r; }");
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::BinaryOp { op: BinaryOperator::Modulo, .. }),
            "Expected Modulo, got: {:?}", value
        );
    } else {
        panic!("No assignment found");
    }
}

// ============================================================================
// BINARY OPERATORS: Bitwise
// ============================================================================

#[test]
fn test_binop_bitwise_and() {
    let ast = parse_c("int f(int a, int b) { int r; r = a & b; return r; }");
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::BinaryOp { op: BinaryOperator::BitwiseAnd, .. }),
            "Expected BitwiseAnd, got: {:?}", value
        );
    } else {
        panic!("No assignment found");
    }
}

#[test]
fn test_binop_bitwise_or() {
    let ast = parse_c("int f(int a, int b) { int r; r = a | b; return r; }");
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::BinaryOp { op: BinaryOperator::BitwiseOr, .. }),
            "Expected BitwiseOr, got: {:?}", value
        );
    } else {
        panic!("No assignment found");
    }
}

#[test]
fn test_binop_bitwise_xor() {
    let ast = parse_c("int f(int a, int b) { int r; r = a ^ b; return r; }");
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::BinaryOp { op: BinaryOperator::BitwiseXor, .. }),
            "Expected BitwiseXor, got: {:?}", value
        );
    } else {
        panic!("No assignment found");
    }
}

#[test]
fn test_binop_left_shift() {
    let ast = parse_c("int f(int a) { int r; r = a << 2; return r; }");
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::BinaryOp { op: BinaryOperator::LeftShift, .. }),
            "Expected LeftShift, got: {:?}", value
        );
    } else {
        panic!("No assignment found");
    }
}

#[test]
fn test_binop_right_shift() {
    let ast = parse_c("int f(int a) { int r; r = a >> 3; return r; }");
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::BinaryOp { op: BinaryOperator::RightShift, .. }),
            "Expected RightShift, got: {:?}", value
        );
    } else {
        panic!("No assignment found");
    }
}

// ============================================================================
// BINARY OPERATORS: Comparison
// ============================================================================

#[test]
fn test_binop_equal() {
    let ast = parse_c("void f(int a, int b) { if (a == b) { int r; r = 1; } }");
    let func = first_func(&ast);
    if let Statement::If { condition, .. } = &func.body[0] {
        assert!(
            matches!(condition, Expression::BinaryOp { op: BinaryOperator::Equal, .. }),
            "Expected Equal, got: {:?}", condition
        );
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_binop_not_equal() {
    let ast = parse_c("void f(int a, int b) { if (a != b) { int r; r = 1; } }");
    let func = first_func(&ast);
    if let Statement::If { condition, .. } = &func.body[0] {
        assert!(
            matches!(condition, Expression::BinaryOp { op: BinaryOperator::NotEqual, .. }),
            "Expected NotEqual, got: {:?}", condition
        );
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_binop_less_than() {
    let ast = parse_c("void f(int a, int b) { if (a < b) { int r; r = 1; } }");
    let func = first_func(&ast);
    if let Statement::If { condition, .. } = &func.body[0] {
        assert!(
            matches!(condition, Expression::BinaryOp { op: BinaryOperator::LessThan, .. }),
            "Expected LessThan, got: {:?}", condition
        );
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_binop_greater_than() {
    let ast = parse_c("void f(int a, int b) { if (a > b) { int r; r = 1; } }");
    let func = first_func(&ast);
    if let Statement::If { condition, .. } = &func.body[0] {
        assert!(
            matches!(condition, Expression::BinaryOp { op: BinaryOperator::GreaterThan, .. }),
            "Expected GreaterThan, got: {:?}", condition
        );
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_binop_less_equal() {
    let ast = parse_c("void f(int a, int b) { if (a <= b) { int r; r = 1; } }");
    let func = first_func(&ast);
    if let Statement::If { condition, .. } = &func.body[0] {
        assert!(
            matches!(condition, Expression::BinaryOp { op: BinaryOperator::LessEqual, .. }),
            "Expected LessEqual, got: {:?}", condition
        );
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_binop_greater_equal() {
    let ast = parse_c("void f(int a, int b) { if (a >= b) { int r; r = 1; } }");
    let func = first_func(&ast);
    if let Statement::If { condition, .. } = &func.body[0] {
        assert!(
            matches!(condition, Expression::BinaryOp { op: BinaryOperator::GreaterEqual, .. }),
            "Expected GreaterEqual, got: {:?}", condition
        );
    } else {
        panic!("Expected If statement");
    }
}

// ============================================================================
// BINARY OPERATORS: Logical
// ============================================================================

#[test]
fn test_binop_logical_and() {
    let ast = parse_c("void f(int a, int b) { if (a > 0 && b > 0) { int r; r = 1; } }");
    let func = first_func(&ast);
    if let Statement::If { condition, .. } = &func.body[0] {
        assert!(
            matches!(condition, Expression::BinaryOp { op: BinaryOperator::LogicalAnd, .. }),
            "Expected LogicalAnd, got: {:?}", condition
        );
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_binop_logical_or() {
    let ast = parse_c("void f(int a, int b) { if (a > 0 || b > 0) { int r; r = 1; } }");
    let func = first_func(&ast);
    if let Statement::If { condition, .. } = &func.body[0] {
        assert!(
            matches!(condition, Expression::BinaryOp { op: BinaryOperator::LogicalOr, .. }),
            "Expected LogicalOr, got: {:?}", condition
        );
    } else {
        panic!("Expected If statement");
    }
}

// ============================================================================
// BINARY OPERATORS: Precedence selection coverage
// ============================================================================

#[test]
fn test_binop_precedence_or_over_and() {
    // || has lower precedence than &&, so should be the top-level operator
    let ast = parse_c("void f(int a, int b, int c) { if (a > 0 || b > 0 && c > 0) { int r; r = 1; } }");
    let func = first_func(&ast);
    if let Statement::If { condition, .. } = &func.body[0] {
        assert!(
            matches!(condition, Expression::BinaryOp { op: BinaryOperator::LogicalOr, .. }),
            "Expected LogicalOr at top level, got: {:?}", condition
        );
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_binop_precedence_and_over_bitor() {
    // && has lower precedence than |, so should be top-level
    let ast = parse_c("void f(int a, int b, int c) { if (a | b && c) { int r; r = 1; } }");
    let func = first_func(&ast);
    if let Statement::If { condition, .. } = &func.body[0] {
        assert!(
            matches!(condition, Expression::BinaryOp { op: BinaryOperator::LogicalAnd, .. }),
            "Expected LogicalAnd at top level, got: {:?}", condition
        );
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_binop_precedence_bitor_over_xor() {
    // | has lower precedence than ^
    let ast = parse_c("int f(int a, int b, int c) { int r; r = a ^ b | c; return r; }");
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::BinaryOp { op: BinaryOperator::BitwiseOr, .. }),
            "Expected BitwiseOr at top level, got: {:?}", value
        );
    } else {
        panic!("No assignment found");
    }
}

#[test]
fn test_binop_precedence_xor_over_bitand() {
    // ^ has lower precedence than &
    let ast = parse_c("int f(int a, int b, int c) { int r; r = a & b ^ c; return r; }");
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::BinaryOp { op: BinaryOperator::BitwiseXor, .. }),
            "Expected BitwiseXor at top level, got: {:?}", value
        );
    } else {
        panic!("No assignment found");
    }
}

#[test]
fn test_binop_precedence_comparison_over_shift() {
    // < has lower precedence than <<
    let ast = parse_c("void f(int a, int b) { if (a << 1 < b) { int r; r = 1; } }");
    let func = first_func(&ast);
    if let Statement::If { condition, .. } = &func.body[0] {
        assert!(
            matches!(condition, Expression::BinaryOp { op: BinaryOperator::LessThan, .. }),
            "Expected LessThan at top level, got: {:?}", condition
        );
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_binop_precedence_shift_over_add() {
    // << has lower precedence than +
    let ast = parse_c("int f(int a, int b) { int r; r = a + b << 1; return r; }");
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::BinaryOp { op: BinaryOperator::LeftShift, .. }),
            "Expected LeftShift at top level, got: {:?}", value
        );
    } else {
        panic!("No assignment found");
    }
}

#[test]
fn test_binop_precedence_add_over_multiply() {
    // + has lower precedence than *
    let ast = parse_c("int f(int a, int b, int c) { int r; r = a + b * c; return r; }");
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::BinaryOp { op: BinaryOperator::Add, .. }),
            "Expected Add at top level, got: {:?}", value
        );
    } else {
        panic!("No assignment found");
    }
}

#[test]
fn test_binop_precedence_subtract_vs_multiply() {
    let ast = parse_c("int f(int a, int b, int c) { int r; r = a * b - c; return r; }");
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::BinaryOp { op: BinaryOperator::Subtract, .. }),
            "Expected Subtract at top level, got: {:?}", value
        );
    } else {
        panic!("No assignment found");
    }
}

#[test]
fn test_binop_equality_over_relational() {
    // == has lower precedence than <
    let ast = parse_c("void f(int a, int b, int c) { if (a < b == c) { int r; r = 1; } }");
    let func = first_func(&ast);
    if let Statement::If { condition, .. } = &func.body[0] {
        assert!(
            matches!(condition, Expression::BinaryOp { op: BinaryOperator::Equal, .. }),
            "Expected Equal at top level, got: {:?}", condition
        );
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_binop_bitand_over_equality() {
    // & has higher precedence than ==
    let ast = parse_c("void f(int a, int b, int c) { if (a & b == c) { int r; r = 1; } }");
    let func = first_func(&ast);
    if let Statement::If { condition, .. } = &func.body[0] {
        assert!(
            matches!(condition, Expression::BinaryOp { op: BinaryOperator::BitwiseAnd, .. }),
            "Expected BitwiseAnd at top level (C precedence: & binds tighter than ==), got: {:?}",
            condition
        );
    } else {
        panic!("Expected If statement");
    }
}

// ============================================================================
// BINARY OPERATORS: Compound assignment operators
// ============================================================================

#[test]
fn test_compound_add_assign() {
    let ast = parse_c("void f() { int x = 1; x += 5; }");
    let func = first_func(&ast);
    let has_compound = func.body.iter().any(|s| {
        matches!(s, Statement::CompoundAssignment { target, op: BinaryOperator::Add, .. } if target == "x")
    });
    assert!(has_compound, "Expected CompoundAssignment(Add), got: {:?}", func.body);
}

#[test]
fn test_compound_sub_assign() {
    let ast = parse_c("void f() { int x = 10; x -= 3; }");
    let func = first_func(&ast);
    let has_compound = func.body.iter().any(|s| {
        matches!(s, Statement::CompoundAssignment { target, op: BinaryOperator::Subtract, .. } if target == "x")
    });
    assert!(has_compound, "Expected CompoundAssignment(Subtract), got: {:?}", func.body);
}

#[test]
fn test_compound_mul_assign() {
    let ast = parse_c("void f() { int x = 2; x *= 4; }");
    let func = first_func(&ast);
    let has_compound = func.body.iter().any(|s| {
        matches!(s, Statement::CompoundAssignment { target, op: BinaryOperator::Multiply, .. } if target == "x")
    });
    assert!(has_compound, "Expected CompoundAssignment(Multiply), got: {:?}", func.body);
}

#[test]
fn test_compound_div_assign() {
    let ast = parse_c("void f() { int x = 100; x /= 5; }");
    let func = first_func(&ast);
    let has_compound = func.body.iter().any(|s| {
        matches!(s, Statement::CompoundAssignment { target, op: BinaryOperator::Divide, .. } if target == "x")
    });
    assert!(has_compound, "Expected CompoundAssignment(Divide), got: {:?}", func.body);
}

#[test]
fn test_compound_mod_assign() {
    let ast = parse_c("void f() { int x = 17; x %= 5; }");
    let func = first_func(&ast);
    let has_compound = func.body.iter().any(|s| {
        matches!(s, Statement::CompoundAssignment { target, op: BinaryOperator::Modulo, .. } if target == "x")
    });
    assert!(has_compound, "Expected CompoundAssignment(Modulo), got: {:?}", func.body);
}

// ============================================================================
// BINARY OPERATORS: Operator inside parentheses (paren depth tracking)
// ============================================================================

#[test]
fn test_binop_ignores_operator_inside_parens() {
    // The top-level operator is *, not the - inside func()
    let ast = parse_c("int f(int n) { int r; r = n * (n - 1); return r; }");
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::BinaryOp { op: BinaryOperator::Multiply, .. }),
            "Expected Multiply at top level (not - from inside parens), got: {:?}", value
        );
    } else {
        panic!("No assignment found");
    }
}

#[test]
fn test_binop_nested_parens() {
    let ast = parse_c("int f(int a, int b, int c) { int r; r = (a + b) * (c - 1); return r; }");
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::BinaryOp { op: BinaryOperator::Multiply, .. }),
            "Expected Multiply at top level, got: {:?}", value
        );
    } else {
        panic!("No assignment found");
    }
}

// ============================================================================
// FOR LOOPS: Standard for loop
// ============================================================================

#[test]
fn test_for_standard_loop() {
    let code = r#"
void f() {
    int i;
    for (i = 0; i < 10; i++) {
        int x;
        x = i;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For statement, got: {:?}", func.body);
    if let Some(Statement::For { init, condition, increment, body }) = for_stmt {
        assert!(!init.is_empty(), "Expected init to have statements");
        assert!(condition.is_some(), "Expected condition to be present");
        assert!(!increment.is_empty(), "Expected increment to have statements");
        assert!(!body.is_empty(), "Expected body to have statements");
    }
}

#[test]
fn test_for_with_decl_init() {
    let code = r#"
void f() {
    for (int i = 0; i < 5; i++) {
        int x;
        x = i;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For statement");
    if let Some(Statement::For { condition, body, .. }) = for_stmt {
        assert!(condition.is_some(), "Expected condition");
        assert!(!body.is_empty(), "Expected non-empty body");
    }
}

// ============================================================================
// FOR LOOPS: Empty parts
// ============================================================================

#[test]
fn test_for_infinite_loop() {
    let code = r#"
void f() {
    for (;;) {
        break;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For statement for infinite loop");
    if let Some(Statement::For { init, condition, increment, body }) = for_stmt {
        assert!(init.is_empty(), "Infinite loop should have empty init");
        assert!(condition.is_none(), "Infinite loop should have no condition");
        assert!(increment.is_empty(), "Infinite loop should have empty increment");
        assert!(!body.is_empty(), "Body should have break");
    }
}

#[test]
fn test_for_empty_init() {
    // for (; i < 10; i++) where i is declared outside
    let code = r#"
void f() {
    int i;
    i = 0;
    for (; i < 10; i++) {
        int x;
        x = i;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For statement with empty init");
    if let Some(Statement::For { condition, increment, .. }) = for_stmt {
        assert!(condition.is_some(), "Expected condition to be present");
        assert!(!increment.is_empty(), "Expected increment");
    }
}

#[test]
fn test_for_empty_increment() {
    // for (i = 0; i < 10;) with manual increment in body
    let code = r#"
void f() {
    int i;
    for (i = 0; i < 10;) {
        i++;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For statement with empty increment");
    if let Some(Statement::For { init, condition, .. }) = for_stmt {
        assert!(!init.is_empty(), "Expected init to be present");
        assert!(condition.is_some(), "Expected condition");
    }
}

#[test]
fn test_for_empty_condition() {
    // for (i = 0;; i++) - no condition (always true)
    let code = r#"
void f() {
    int i;
    for (i = 0;; i++) {
        if (i > 10) { break; }
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For statement with empty condition");
}

#[test]
fn test_for_only_condition() {
    // for (; x > 0;) - only condition, no init or increment
    let code = r#"
void f() {
    int x;
    x = 10;
    for (; x > 0;) {
        x--;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For statement with only condition");
}

// ============================================================================
// FOR LOOPS: Nested for loops
// ============================================================================

#[test]
fn test_for_nested_loops() {
    let code = r#"
void f() {
    int i;
    int j;
    for (i = 0; i < 5; i++) {
        for (j = 0; j < 5; j++) {
            int sum;
            sum = i + j;
        }
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected outer For statement");
    if let Some(Statement::For { body, .. }) = for_stmt {
        let inner_for = body.iter().find(|s| matches!(s, Statement::For { .. }));
        assert!(inner_for.is_some(), "Expected inner For statement");
    }
}

// ============================================================================
// FOR LOOPS: Single statement body (non-compound)
// ============================================================================

#[test]
fn test_for_single_statement_body_return() {
    // Single-statement for body with return (extract_single_statement handles ReturnStmt)
    let code = r#"
int f() {
    int i;
    for (i = 0; i < 10; i++)
        if (i == 5) { return i; }
    return 0;
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For with single-statement body");
    if let Some(Statement::For { body, .. }) = for_stmt {
        assert!(!body.is_empty(), "Body should have the single if statement");
    }
}

#[test]
fn test_for_single_statement_body_break() {
    // Single-statement for body with break
    let code = r#"
void f() {
    int i;
    for (i = 0; i < 10; i++)
        break;
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For with single-statement body (break)");
    if let Some(Statement::For { body, .. }) = for_stmt {
        assert!(!body.is_empty(), "Body should have break statement");
        assert!(
            body.iter().any(|s| matches!(s, Statement::Break)),
            "Expected Break in body"
        );
    }
}

#[test]
fn test_for_single_statement_body_inc() {
    // Single-statement for body with increment
    let code = r#"
void f() {
    int i;
    int x;
    x = 0;
    for (i = 0; i < 10; i++)
        x++;
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For with single-statement body (inc)");
    if let Some(Statement::For { body, .. }) = for_stmt {
        assert!(!body.is_empty(), "Body should have increment statement");
    }
}

// ============================================================================
// FOR LOOPS: Pre-decrement as increment expression
// ============================================================================

#[test]
fn test_for_with_decrement() {
    let code = r#"
void f() {
    int i;
    for (i = 10; i > 0; i--) {
        int x;
        x = i;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For with decrement");
    if let Some(Statement::For { increment, .. }) = for_stmt {
        assert!(!increment.is_empty(), "Expected decrement as increment step");
    }
}

#[test]
fn test_for_with_pre_decrement() {
    let code = r#"
void f() {
    int i;
    for (i = 10; i > 0; --i) {
        int x;
        x = i;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For with pre-decrement");
}

// ============================================================================
// FOR LOOPS: Assignment-based init
// ============================================================================

#[test]
fn test_for_assignment_init() {
    let code = r#"
void f() {
    int i;
    for (i = 5; i < 100; i++) {
        int x;
        x = i;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For statement");
    if let Some(Statement::For { init, .. }) = for_stmt {
        assert!(!init.is_empty(), "Expected assignment-based init");
    }
}

// ============================================================================
// FOR LOOPS: Condition with various comparison types
// ============================================================================

#[test]
fn test_for_condition_less_equal() {
    let code = r#"
void f() {
    int i;
    for (i = 0; i <= 9; i++) {
        int x;
        x = i;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For statement");
    if let Some(Statement::For { condition, .. }) = for_stmt {
        assert!(condition.is_some(), "Expected condition with <=");
    }
}

#[test]
fn test_for_condition_not_equal() {
    let code = r#"
void f() {
    int i;
    for (i = 0; i != 10; i++) {
        int x;
        x = i;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For statement");
    if let Some(Statement::For { condition, .. }) = for_stmt {
        assert!(condition.is_some(), "Expected condition with !=");
    }
}

#[test]
fn test_for_condition_greater_equal() {
    let code = r#"
void f() {
    int i;
    for (i = 20; i >= 0; i--) {
        int x;
        x = i;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For statement");
    if let Some(Statement::For { condition, .. }) = for_stmt {
        assert!(condition.is_some(), "Expected condition with >=");
    }
}

// ============================================================================
// FOR LOOPS: Compound assignment as increment
// ============================================================================

#[test]
fn test_for_compound_increment() {
    let code = r#"
void f() {
    int i;
    for (i = 0; i < 100; i += 2) {
        int x;
        x = i;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For statement with compound increment");
}

// ============================================================================
// FOR LOOPS: Multiple init and increment (comma operator)
// ============================================================================

#[test]
fn test_for_comma_operator_increment() {
    // for (i = 0, j = 10; i < j; i++, j--)
    let code = r#"
void f() {
    int i;
    int j;
    for (i = 0; i < 10; i++, j--) {
        int x;
        x = i;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For statement with comma increment");
}

// ============================================================================
// MIXED: for loop with binary operators in condition
// ============================================================================

#[test]
fn test_for_with_logical_and_condition() {
    let code = r#"
void f() {
    int i;
    for (i = 0; i >= 0 && i < 10; i++) {
        int x;
        x = i;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For statement with && condition");
}

#[test]
fn test_for_with_logical_or_condition() {
    let code = r#"
void f() {
    int i;
    int done;
    done = 0;
    for (i = 0; i < 100 || done == 0; i++) {
        if (i > 50) { done = 1; }
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For statement with || condition");
}

// ============================================================================
// MIXED: Increment in expression context
// ============================================================================

#[test]
fn test_inc_dec_in_for_increment_part() {
    let code = r#"
void f() {
    int i;
    for (i = 0; i < 10; ++i) {
        int x;
        x = i;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For with pre-increment");
    if let Some(Statement::For { increment, .. }) = for_stmt {
        assert!(!increment.is_empty(), "Expected pre-increment in increment slot");
    }
}

// ============================================================================
// MIXED: Complex expressions with multiple operator types
// ============================================================================

#[test]
fn test_complex_expression_bitwise_and_arithmetic() {
    let ast = parse_c("int f(int a, int b) { int r; r = (a + 1) & (b - 1); return r; }");
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::BinaryOp { op: BinaryOperator::BitwiseAnd, .. }),
            "Expected BitwiseAnd at top level, got: {:?}", value
        );
    } else {
        panic!("No assignment found");
    }
}

#[test]
fn test_complex_expression_shift_and_add() {
    let ast = parse_c("int f(int a, int b) { int r; r = a << 1 + b; return r; }");
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    assert!(assign.is_some(), "Expected assignment with shift and add");
}

#[test]
fn test_right_shift_expression() {
    let ast = parse_c("int f(int x) { int r; r = x >> 4; return r; }");
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::BinaryOp { op: BinaryOperator::RightShift, .. }),
            "Expected RightShift, got: {:?}", value
        );
    }
}

// ============================================================================
// EDGE CASES: Inc/Dec with various integer types
// ============================================================================

#[test]
fn test_inc_dec_unsigned_int() {
    let ast = parse_c("void f() { unsigned int x = 0; x++; }");
    let func = first_func(&ast);
    let has_post_inc = func.body.iter().any(|s| {
        matches!(s, Statement::PostIncrement { target } if target == "x")
    });
    assert!(has_post_inc, "Expected PostIncrement on unsigned int");
}

#[test]
fn test_inc_dec_long() {
    let ast = parse_c("void f() { long n = 100; n--; }");
    let func = first_func(&ast);
    let has_post_dec = func.body.iter().any(|s| {
        matches!(s, Statement::PostDecrement { target } if target == "n")
    });
    assert!(has_post_dec, "Expected PostDecrement on long");
}

#[test]
fn test_inc_dec_char() {
    let ast = parse_c("void f() { char c = 'a'; c++; }");
    let func = first_func(&ast);
    let has_post_inc = func.body.iter().any(|s| {
        matches!(s, Statement::PostIncrement { target } if target == "c")
    });
    assert!(has_post_inc, "Expected PostIncrement on char");
}

// ============================================================================
// FOR LOOPS: Body with multiple statement types
// ============================================================================

#[test]
fn test_for_body_with_if() {
    let code = r#"
void f() {
    int i;
    for (i = 0; i < 10; i++) {
        if (i > 5) {
            break;
        }
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some());
    if let Some(Statement::For { body, .. }) = for_stmt {
        let has_if = body.iter().any(|s| matches!(s, Statement::If { .. }));
        assert!(has_if, "Expected If inside for body");
    }
}

#[test]
fn test_for_body_with_continue() {
    let code = r#"
void f() {
    int i;
    for (i = 0; i < 10; i++) {
        if (i == 5) {
            continue;
        }
        int x;
        x = i;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For statement");
}

#[test]
fn test_for_body_with_break() {
    let code = r#"
void f() {
    int i;
    for (i = 0; i < 100; i++) {
        if (i > 50) {
            break;
        }
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some());
}

// ============================================================================
// FOR LOOPS: Counting down
// ============================================================================

#[test]
fn test_for_count_down() {
    let code = r#"
void f() {
    int i;
    for (i = 100; i > 0; i--) {
        int x;
        x = i;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected for loop counting down");
    if let Some(Statement::For { init, condition, increment, .. }) = for_stmt {
        assert!(!init.is_empty(), "Expected init (i=100)");
        assert!(condition.is_some(), "Expected condition (i>0)");
        assert!(!increment.is_empty(), "Expected decrement (i--)");
    }
}

// ============================================================================
// BINARY OPERATORS: Expressions used as conditions in while loops
// ============================================================================

#[test]
fn test_binop_in_while_condition() {
    let ast = parse_c("void f(int n) { while (n > 0) { n--; } }");
    let func = first_func(&ast);
    let while_stmt = func.body.iter().find(|s| matches!(s, Statement::While { .. }));
    assert!(while_stmt.is_some(), "Expected While statement");
    if let Some(Statement::While { condition, .. }) = while_stmt {
        assert!(
            matches!(condition, Expression::BinaryOp { op: BinaryOperator::GreaterThan, .. }),
            "Expected GreaterThan in while condition, got: {:?}", condition
        );
    }
}

// ============================================================================
// BINARY OPERATORS: Return expressions with operators
// ============================================================================

#[test]
fn test_binop_in_return() {
    let ast = parse_c("int f(int a, int b) { return a + b; }");
    let func = first_func(&ast);
    let ret = func.body.iter().find(|s| matches!(s, Statement::Return(Some(_))));
    assert!(ret.is_some(), "Expected return with expression");
    if let Some(Statement::Return(Some(expr))) = ret {
        assert!(
            matches!(expr, Expression::BinaryOp { op: BinaryOperator::Add, .. }),
            "Expected Add in return, got: {:?}", expr
        );
    }
}

#[test]
fn test_binop_modulo_in_return() {
    let ast = parse_c("int f(int a, int b) { return a % b; }");
    let func = first_func(&ast);
    let ret = func.body.iter().find(|s| matches!(s, Statement::Return(Some(_))));
    if let Some(Statement::Return(Some(expr))) = ret {
        assert!(
            matches!(expr, Expression::BinaryOp { op: BinaryOperator::Modulo, .. }),
            "Expected Modulo in return, got: {:?}", expr
        );
    }
}

// ============================================================================
// EDGE CASES: Multiple binary ops chained
// ============================================================================

#[test]
fn test_three_way_add() {
    let ast = parse_c("int f(int a, int b, int c) { int r; r = a + b + c; return r; }");
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::BinaryOp { op: BinaryOperator::Add, .. }),
            "Expected Add, got: {:?}", value
        );
    } else {
        panic!("No assignment found");
    }
}

#[test]
fn test_chained_comparisons_in_and() {
    let ast = parse_c(
        "void f(int a, int b, int c) { if (a > 0 && b > 0 && c > 0) { int x; x = 1; } }",
    );
    let func = first_func(&ast);
    if let Statement::If { condition, .. } = &func.body[0] {
        assert!(
            matches!(condition, Expression::BinaryOp { op: BinaryOperator::LogicalAnd, .. }),
            "Expected LogicalAnd at top, got: {:?}", condition
        );
    }
}

// ============================================================================
// FOR LOOPS: Variable used as condition without comparison
// ============================================================================

#[test]
fn test_for_variable_as_condition() {
    // for (i = 10; i; i--) - variable used as boolean condition
    let code = r#"
void f() {
    int i;
    for (i = 10; i; i--) {
        int x;
        x = i;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For with variable as condition");
}

// ============================================================================
// FOR LOOPS: Function call as condition
// ============================================================================

#[test]
fn test_for_function_call_condition() {
    let code = r#"
int check(int x);
void f() {
    int i;
    for (i = 0; check(i); i++) {
        int x;
        x = i;
    }
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For with function call condition");
}

// ============================================================================
// FOR LOOPS: Large step in increment
// ============================================================================

#[test]
fn test_for_large_step_increment() {
    let code = r#"
void f() {
    int i;
    for (i = 0; i < 1000; i += 10) {
        int x;
        x = i;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For with large step");
}

// ============================================================================
// INCREMENT/DECREMENT: Inc/Dec on function parameters
// ============================================================================

#[test]
fn test_inc_dec_on_parameter() {
    let ast = parse_c("void f(int n) { n++; }");
    let func = first_func(&ast);
    let has_post_inc = func.body.iter().any(|s| {
        matches!(s, Statement::PostIncrement { target } if target == "n")
    });
    assert!(has_post_inc, "Expected PostIncrement on parameter n");
}

#[test]
fn test_pre_dec_on_parameter() {
    let ast = parse_c("void f(int count) { --count; }");
    let func = first_func(&ast);
    let has_pre_dec = func.body.iter().any(|s| {
        matches!(s, Statement::PreDecrement { target } if target == "count")
    });
    assert!(has_pre_dec, "Expected PreDecrement on parameter count");
}

// ============================================================================
// FOR LOOPS: Deeply nested
// ============================================================================

#[test]
fn test_triple_nested_for() {
    let code = r#"
void f() {
    int i;
    int j;
    int k;
    for (i = 0; i < 3; i++) {
        for (j = 0; j < 3; j++) {
            for (k = 0; k < 3; k++) {
                int sum;
                sum = i + j + k;
            }
        }
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let outer = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(outer.is_some(), "Expected outer for");
    if let Some(Statement::For { body: outer_body, .. }) = outer {
        let mid = outer_body.iter().find(|s| matches!(s, Statement::For { .. }));
        assert!(mid.is_some(), "Expected middle for");
        if let Some(Statement::For { body: mid_body, .. }) = mid {
            let inner = mid_body.iter().find(|s| matches!(s, Statement::For { .. }));
            assert!(inner.is_some(), "Expected inner for");
        }
    }
}

// ============================================================================
// BINARY OPERATORS: Bitwise XOR in isolation
// ============================================================================

#[test]
fn test_binop_xor_in_condition() {
    let ast = parse_c("void f(int a, int b) { if (a ^ b) { int r; r = 1; } }");
    let func = first_func(&ast);
    if let Statement::If { condition, .. } = &func.body[0] {
        assert!(
            matches!(condition, Expression::BinaryOp { op: BinaryOperator::BitwiseXor, .. }),
            "Expected BitwiseXor in condition, got: {:?}", condition
        );
    }
}

// ============================================================================
// BINARY OPERATORS: Comma operator
// ============================================================================

#[test]
fn test_binop_comma_in_for_increment() {
    // The comma operator in for loops is handled specially
    let code = r#"
void f() {
    int i;
    int j;
    j = 10;
    for (i = 0; i < 10; i++, j--) {
        int x;
        x = i + j;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For statement with comma in increment");
    if let Some(Statement::For { increment, .. }) = for_stmt {
        // Comma operator should produce multiple increment statements
        assert!(
            increment.len() >= 2,
            "Expected at least 2 increment statements from comma operator, got {}",
            increment.len()
        );
    }
}
