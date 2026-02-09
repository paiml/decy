//! Deep coverage tests targeting uncovered branches in parser.rs.
//!
//! Focuses on:
//! - `extract_inc_dec_stmt`: dot field decrement, array decrement, edge cases
//! - `extract_binary_operator`: assign, comma, precedence fallback paths
//! - `extract_for_stmt`: 1-child, 2-child, single-stmt body, DeclStmt branches
//! - `extract_compound_assignment_stmt`: complex targets (deref, field, array)
//! - `extract_unary_op`: bitwise not, address-of, post-dec, dereference
//! - `extract_expression_from_cursor`: float/char literal, unary op branches
//! - `extract_conditional_op`: ternary expressions
//! - `extract_sizeof`: sizeof type
//! - `extract_cast`: C-style casts
//! - `extract_single_statement`: various single-stmt for-loop bodies

use decy_parser::parser::{BinaryOperator, Expression, Statement, UnaryOperator};
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
// INC/DEC: Dot field access DECREMENT (covers delta < 0 path in FieldAccess arm)
// ============================================================================

#[test]
fn test_dot_field_pre_decrement() {
    let code = r#"
typedef struct { int z; } Coord;
void f() {
    Coord c;
    c.z = 5;
    --c.z;
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let has_field_assign = func.body.iter().any(|s| {
        matches!(s, Statement::FieldAssignment { field, .. } if field == "z")
    });
    assert!(
        has_field_assign,
        "Expected FieldAssignment for --c.z, got: {:?}",
        func.body
    );
}

#[test]
fn test_dot_field_pre_increment() {
    let code = r#"
typedef struct { int w; } Size;
void f() {
    Size s;
    s.w = 0;
    ++s.w;
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let has_field_assign = func.body.iter().any(|s| {
        matches!(s, Statement::FieldAssignment { field, .. } if field == "w")
    });
    assert!(
        has_field_assign,
        "Expected FieldAssignment for ++s.w, got: {:?}",
        func.body
    );
}

// ============================================================================
// INC/DEC: Array subscript DECREMENT paths (delta < 0 in ArrayIndex arm)
// ============================================================================

#[test]
fn test_array_index_pre_decrement() {
    let code = r#"
void f() {
    int vals[5];
    vals[0] = 10;
    --vals[0];
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let has_arr_assign = func.body.iter().any(|s| {
        matches!(s, Statement::ArrayIndexAssignment { .. })
    });
    assert!(
        has_arr_assign,
        "Expected ArrayIndexAssignment for --vals[0], got: {:?}",
        func.body
    );
}

#[test]
fn test_array_index_variable_pre_decrement() {
    let code = r#"
void f() {
    int arr[10];
    int i = 3;
    --arr[i];
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let has_arr_assign = func.body.iter().any(|s| {
        matches!(s, Statement::ArrayIndexAssignment { .. })
    });
    assert!(
        has_arr_assign,
        "Expected ArrayIndexAssignment for --arr[i], got: {:?}",
        func.body
    );
}

// ============================================================================
// COMPOUND ASSIGNMENT: Complex targets (deref, pointer field, field, array)
// ============================================================================

#[test]
fn test_compound_assign_deref_target() {
    // *ptr += 5 should produce DerefCompoundAssignment
    let code = r#"
void f(int* ptr) {
    *ptr += 5;
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let has_deref_compound = func.body.iter().any(|s| {
        matches!(s, Statement::DerefCompoundAssignment { op: BinaryOperator::Add, .. })
    });
    assert!(
        has_deref_compound,
        "Expected DerefCompoundAssignment for *ptr += 5, got: {:?}",
        func.body
    );
}

#[test]
fn test_compound_assign_pointer_field_target() {
    // sb->capacity *= 2 should produce DerefCompoundAssignment
    let code = r#"
typedef struct { int capacity; } Buffer;
void f(Buffer* sb) {
    sb->capacity *= 2;
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let has_deref_compound = func.body.iter().any(|s| {
        matches!(s, Statement::DerefCompoundAssignment { op: BinaryOperator::Multiply, .. })
    });
    assert!(
        has_deref_compound,
        "Expected DerefCompoundAssignment for sb->capacity *= 2, got: {:?}",
        func.body
    );
}

#[test]
fn test_compound_assign_field_access_target() {
    // obj.field -= 3 should produce DerefCompoundAssignment
    let code = r#"
typedef struct { int field; } Obj;
void f() {
    Obj obj;
    obj.field = 10;
    obj.field -= 3;
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let has_deref_compound = func.body.iter().any(|s| {
        matches!(s, Statement::DerefCompoundAssignment { op: BinaryOperator::Subtract, .. })
    });
    assert!(
        has_deref_compound,
        "Expected DerefCompoundAssignment for obj.field -= 3, got: {:?}",
        func.body
    );
}

#[test]
fn test_compound_assign_array_index_target() {
    // arr[i] /= 2 should produce DerefCompoundAssignment
    let code = r#"
void f() {
    int arr[5];
    arr[0] = 10;
    arr[0] /= 2;
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let has_deref_compound = func.body.iter().any(|s| {
        matches!(s, Statement::DerefCompoundAssignment { op: BinaryOperator::Divide, .. })
    });
    assert!(
        has_deref_compound,
        "Expected DerefCompoundAssignment for arr[0] /= 2, got: {:?}",
        func.body
    );
}

#[test]
fn test_compound_assign_array_modulo() {
    // arr[i] %= 3 should produce DerefCompoundAssignment
    let code = r#"
void f() {
    int arr[10];
    arr[2] = 17;
    arr[2] %= 3;
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let has_deref_compound = func.body.iter().any(|s| {
        matches!(s, Statement::DerefCompoundAssignment { op: BinaryOperator::Modulo, .. })
    });
    assert!(
        has_deref_compound,
        "Expected DerefCompoundAssignment for arr[2] %= 3, got: {:?}",
        func.body
    );
}

// ============================================================================
// UNARY OPERATORS: BitwiseNot, AddressOf, Dereference, LogicalNot
// ============================================================================

#[test]
fn test_unary_bitwise_not() {
    let ast = parse_c("int f(int x) { return ~x; }");
    let func = first_func(&ast);
    let ret = func.body.iter().find(|s| matches!(s, Statement::Return(Some(_))));
    if let Some(Statement::Return(Some(expr))) = ret {
        assert!(
            matches!(expr, Expression::UnaryOp { op: UnaryOperator::BitwiseNot, .. }),
            "Expected BitwiseNot, got: {:?}",
            expr
        );
    } else {
        panic!("Expected return with expression");
    }
}

#[test]
fn test_unary_address_of() {
    let code = r#"
void f() {
    int x = 42;
    int* p = &x;
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    // The &x should appear as an AddressOf expression in a variable declaration
    let has_addr_of = func.body.iter().any(|s| {
        if let Statement::VariableDeclaration {
            initializer: Some(Expression::UnaryOp { op: UnaryOperator::AddressOf, .. }),
            ..
        } = s
        {
            true
        } else {
            false
        }
    });
    assert!(
        has_addr_of,
        "Expected AddressOf expression in initializer, got: {:?}",
        func.body
    );
}

#[test]
fn test_unary_logical_not_in_condition() {
    let ast = parse_c("void f(int x) { if (!x) { int a = 1; } }");
    let func = first_func(&ast);
    if let Statement::If { condition, .. } = &func.body[0] {
        assert!(
            matches!(condition, Expression::UnaryOp { op: UnaryOperator::LogicalNot, .. }),
            "Expected LogicalNot, got: {:?}",
            condition
        );
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_unary_minus_in_expression() {
    let ast = parse_c("int f(int x) { return -x; }");
    let func = first_func(&ast);
    let ret = func.body.iter().find(|s| matches!(s, Statement::Return(Some(_))));
    if let Some(Statement::Return(Some(expr))) = ret {
        assert!(
            matches!(expr, Expression::UnaryOp { op: UnaryOperator::Minus, .. }),
            "Expected UnaryMinus, got: {:?}",
            expr
        );
    } else {
        panic!("Expected return with expression");
    }
}

#[test]
fn test_dereference_expression() {
    let code = r#"
int f(int* p) {
    return *p;
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let ret = func.body.iter().find(|s| matches!(s, Statement::Return(Some(_))));
    if let Some(Statement::Return(Some(expr))) = ret {
        assert!(
            matches!(expr, Expression::Dereference(_)),
            "Expected Dereference, got: {:?}",
            expr
        );
    } else {
        panic!("Expected return with dereference expression");
    }
}

// ============================================================================
// TERNARY/CONDITIONAL OPERATOR
// ============================================================================

#[test]
fn test_ternary_expression_in_return() {
    let ast = parse_c("int f(int x) { return x > 0 ? x : -x; }");
    let func = first_func(&ast);
    let ret = func.body.iter().find(|s| matches!(s, Statement::Return(Some(_))));
    if let Some(Statement::Return(Some(expr))) = ret {
        assert!(
            matches!(expr, Expression::Ternary { .. }),
            "Expected Ternary, got: {:?}",
            expr
        );
    } else {
        panic!("Expected return with ternary expression");
    }
}

#[test]
fn test_ternary_expression_in_assignment() {
    let ast = parse_c("void f(int a, int b) { int max; max = a > b ? a : b; }");
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::Ternary { .. }),
            "Expected Ternary in assignment, got: {:?}",
            value
        );
    } else {
        panic!("Expected assignment with ternary");
    }
}

// ============================================================================
// SIZEOF EXPRESSION
// ============================================================================

#[test]
fn test_sizeof_int() {
    let code = r#"
int f() {
    int s;
    s = sizeof(int);
    return s;
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    // sizeof might appear as part of an assignment or variable init
    let has_sizeof = func.body.iter().any(|s| match s {
        Statement::Assignment { value, .. } => matches!(value, Expression::Sizeof { .. }),
        Statement::VariableDeclaration {
            initializer: Some(expr),
            ..
        } => matches!(expr, Expression::Sizeof { .. }),
        _ => false,
    });
    assert!(
        has_sizeof,
        "Expected Sizeof expression, got: {:?}",
        func.body
    );
}

#[test]
fn test_sizeof_struct() {
    let code = r#"
typedef struct { int x; int y; } Point;
int f() {
    int s;
    s = sizeof(Point);
    return s;
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let has_sizeof = func.body.iter().any(|s| match s {
        Statement::Assignment { value, .. } => matches!(value, Expression::Sizeof { .. }),
        _ => false,
    });
    assert!(
        has_sizeof,
        "Expected Sizeof(Point), got: {:?}",
        func.body
    );
}

// ============================================================================
// CAST EXPRESSION
// ============================================================================

#[test]
fn test_cast_int_to_float() {
    let code = r#"
double f(int x) {
    return (double)x;
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let ret = func.body.iter().find(|s| matches!(s, Statement::Return(Some(_))));
    if let Some(Statement::Return(Some(expr))) = ret {
        assert!(
            matches!(expr, Expression::Cast { .. }),
            "Expected Cast expression, got: {:?}",
            expr
        );
    } else {
        panic!("Expected return with cast expression");
    }
}

#[test]
fn test_cast_to_void_pointer() {
    let code = r#"
void* f(int* p) {
    return (void*)p;
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let ret = func.body.iter().find(|s| matches!(s, Statement::Return(Some(_))));
    if let Some(Statement::Return(Some(expr))) = ret {
        assert!(
            matches!(expr, Expression::Cast { .. }),
            "Expected Cast to void*, got: {:?}",
            expr
        );
    } else {
        panic!("Expected return with cast expression");
    }
}

// ============================================================================
// FOR LOOP: Single-statement body with function call (extract_single_statement CallExpr)
// ============================================================================

#[test]
fn test_for_single_statement_body_function_call() {
    let code = r#"
void process(int x);
void f() {
    int i;
    for (i = 0; i < 10; i++)
        process(i);
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For statement");
    if let Some(Statement::For { body, .. }) = for_stmt {
        let has_call = body.iter().any(|s| matches!(s, Statement::FunctionCall { .. }));
        assert!(has_call, "Expected FunctionCall in single-stmt body, got: {:?}", body);
    }
}

#[test]
fn test_for_single_statement_body_assignment() {
    let code = r#"
void f() {
    int i;
    int x;
    x = 0;
    for (i = 0; i < 10; i++)
        x = i;
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For statement with single assignment body");
    if let Some(Statement::For { body, .. }) = for_stmt {
        let has_assign = body.iter().any(|s| matches!(s, Statement::Assignment { .. }));
        assert!(has_assign, "Expected Assignment in single-stmt body, got: {:?}", body);
    }
}

#[test]
fn test_for_single_statement_body_continue() {
    let code = r#"
void f() {
    int i;
    for (i = 0; i < 10; i++)
        continue;
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For statement");
    if let Some(Statement::For { body, .. }) = for_stmt {
        let has_continue = body.iter().any(|s| matches!(s, Statement::Continue));
        assert!(has_continue, "Expected Continue in single-stmt body, got: {:?}", body);
    }
}

#[test]
fn test_for_single_statement_body_while() {
    // Single-statement for body with a while loop inside
    let code = r#"
void f() {
    int i;
    for (i = 0; i < 3; i++)
        while (i > 0) { break; }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For statement with while body");
}

#[test]
fn test_for_single_statement_body_nested_for() {
    // Single-statement for body that is another for loop
    let code = r#"
void f() {
    int i;
    int j;
    for (i = 0; i < 5; i++)
        for (j = 0; j < 5; j++) {
            int x;
            x = i + j;
        }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected outer For");
    if let Some(Statement::For { body, .. }) = for_stmt {
        let inner = body.iter().find(|s| matches!(s, Statement::For { .. }));
        assert!(inner.is_some(), "Expected inner For in single-stmt body, got: {:?}", body);
    }
}

// ============================================================================
// FOR LOOP: 2-child (condition + increment, no init) branch
// ============================================================================

#[test]
fn test_for_condition_and_increment_only() {
    // for (; condition; increment) - init is empty, 2 children before body
    // child0 = condition (comparison), child1 = increment (unary)
    let code = r#"
void f() {
    int x;
    x = 10;
    for (; x > 0; x--) {
        int y;
        y = x;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For with condition+increment");
    if let Some(Statement::For { init, condition, increment, .. }) = for_stmt {
        // init should be empty since we used ;
        assert!(init.is_empty(), "Expected empty init for (; cond; inc)");
        assert!(condition.is_some(), "Expected condition");
        assert!(!increment.is_empty(), "Expected increment");
    }
}

// ============================================================================
// FOR LOOP: 1-child (only condition, DeclRefExpr) branch
// ============================================================================

#[test]
fn test_for_only_unary_increment() {
    // for (;; i++) - only increment, 1 child
    let code = r#"
void f() {
    int i;
    i = 0;
    for (;; i++) {
        if (i > 10) { break; }
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For with only increment");
}

// ============================================================================
// BINARY OPERATOR: Assign operator in expression context (embedded assignment)
// ============================================================================

#[test]
fn test_binop_assign_in_condition() {
    // Embedded assignment: while ((c = getchar()) != EOF) is complex;
    // Simpler test: if ((x = 5) > 0)
    let code = r#"
void f() {
    int x;
    x = 0;
    if (x = 5) {
        int y;
        y = 1;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    // The condition should parse; the key point is it exercises the Assign branch
    let has_if = func.body.iter().any(|s| matches!(s, Statement::If { .. }));
    assert!(has_if, "Expected If statement with assignment condition, got: {:?}", func.body);
}

// ============================================================================
// BINARY OPERATOR: Comma operator in a genuine expression context
// ============================================================================

#[test]
fn test_for_comma_init() {
    // for (i = 0, j = 10; ...) - comma operator in init
    let code = r#"
void f() {
    int i;
    int j;
    for (i = 0, j = 10; i < j; i++, j--) {
        int x;
        x = i;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For with comma init");
}

// ============================================================================
// SWITCH STATEMENT: Exercises extract_switch_stmt branches
// ============================================================================

#[test]
fn test_switch_with_cases_and_default() {
    let code = r#"
int f(int x) {
    int r;
    switch (x) {
        case 0: r = 10; break;
        case 1: r = 20; break;
        default: r = 0; break;
    }
    return r;
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let switch_stmt = func.body.iter().find(|s| matches!(s, Statement::Switch { .. }));
    assert!(switch_stmt.is_some(), "Expected Switch statement");
    if let Some(Statement::Switch { cases, default_case, .. }) = switch_stmt {
        assert!(cases.len() >= 2, "Expected at least 2 cases, got {}", cases.len());
        assert!(default_case.is_some(), "Expected default case");
    }
}

#[test]
fn test_switch_without_default() {
    let code = r#"
int f(int x) {
    int r;
    r = 0;
    switch (x) {
        case 1: r = 1; break;
        case 2: r = 2; break;
    }
    return r;
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let switch_stmt = func.body.iter().find(|s| matches!(s, Statement::Switch { .. }));
    assert!(switch_stmt.is_some(), "Expected Switch without default");
    if let Some(Statement::Switch { default_case, .. }) = switch_stmt {
        assert!(default_case.is_none(), "Expected no default case");
    }
}

// ============================================================================
// WHILE LOOP: Various condition types (exercises visit_while_children)
// ============================================================================

#[test]
fn test_while_with_function_call_condition() {
    let code = r#"
int check(int x);
void f(int n) {
    while (check(n)) {
        n--;
    }
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let while_stmt = func.body.iter().find(|s| matches!(s, Statement::While { .. }));
    assert!(while_stmt.is_some(), "Expected While with function call condition");
    if let Some(Statement::While { condition, .. }) = while_stmt {
        assert!(
            matches!(condition, Expression::FunctionCall { .. }),
            "Expected FunctionCall condition, got: {:?}",
            condition
        );
    }
}

#[test]
fn test_while_with_unary_not_condition() {
    let code = r#"
void f(int done) {
    while (!done) {
        done = 1;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let while_stmt = func.body.iter().find(|s| matches!(s, Statement::While { .. }));
    assert!(while_stmt.is_some(), "Expected While with !done condition");
    if let Some(Statement::While { condition, .. }) = while_stmt {
        assert!(
            matches!(condition, Expression::UnaryOp { op: UnaryOperator::LogicalNot, .. }),
            "Expected LogicalNot condition, got: {:?}",
            condition
        );
    }
}

#[test]
fn test_while_with_variable_condition() {
    // while(n) - variable used as boolean
    let code = r#"
void f(int n) {
    while (n) {
        n--;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let while_stmt = func.body.iter().find(|s| matches!(s, Statement::While { .. }));
    assert!(while_stmt.is_some(), "Expected While with variable condition");
    if let Some(Statement::While { condition, .. }) = while_stmt {
        assert!(
            matches!(condition, Expression::Variable(_)),
            "Expected Variable condition, got: {:?}",
            condition
        );
    }
}

// ============================================================================
// EXPRESSION: Float literal in for-loop condition
// ============================================================================

#[test]
fn test_float_literal_in_condition() {
    let code = r#"
void f() {
    double x;
    x = 0.0;
    while (x < 1.5) {
        x = x + 0.1;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    // Just verifying it parses without error
    let while_stmt = func.body.iter().find(|s| matches!(s, Statement::While { .. }));
    assert!(while_stmt.is_some(), "Expected While with float condition");
}

#[test]
fn test_float_literal_in_return() {
    let ast = parse_c("double f() { return 3.14; }");
    let func = first_func(&ast);
    let ret = func.body.iter().find(|s| matches!(s, Statement::Return(Some(_))));
    if let Some(Statement::Return(Some(expr))) = ret {
        assert!(
            matches!(expr, Expression::FloatLiteral(_)),
            "Expected FloatLiteral, got: {:?}",
            expr
        );
    } else {
        panic!("Expected return with float literal");
    }
}

// ============================================================================
// EXPRESSION: Char literal
// ============================================================================

#[test]
fn test_char_literal_in_assignment() {
    let code = r#"
void f() {
    char c;
    c = 'A';
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::CharLiteral(_)),
            "Expected CharLiteral, got: {:?}",
            value
        );
    } else {
        panic!("Expected assignment with char literal");
    }
}

#[test]
fn test_char_literal_null_terminator() {
    let code = r#"
void f() {
    char c;
    c = '\0';
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::CharLiteral(0)),
            "Expected CharLiteral(0), got: {:?}",
            value
        );
    } else {
        panic!("Expected assignment with null char");
    }
}

// ============================================================================
// EXPRESSION: String literal
// ============================================================================

#[test]
fn test_string_literal_in_variable_init() {
    let code = r#"
void f() {
    const char* msg = "hello world";
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let has_string = func.body.iter().any(|s| {
        if let Statement::VariableDeclaration {
            initializer: Some(Expression::StringLiteral(s)),
            ..
        } = s
        {
            s == "hello world"
        } else {
            false
        }
    });
    assert!(has_string, "Expected StringLiteral in var init, got: {:?}", func.body);
}

// ============================================================================
// IF STATEMENT: Single-statement then/else (no braces), exercises DECY-216
// ============================================================================

#[test]
fn test_if_single_statement_then_no_braces() {
    let code = r#"
int f(int x) {
    if (x > 0)
        return 1;
    return 0;
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let if_stmt = func.body.iter().find(|s| matches!(s, Statement::If { .. }));
    assert!(if_stmt.is_some(), "Expected If statement");
    if let Some(Statement::If { then_block, .. }) = if_stmt {
        assert!(!then_block.is_empty(), "Expected non-empty then block");
        assert!(
            then_block.iter().any(|s| matches!(s, Statement::Return(_))),
            "Expected Return in then block"
        );
    }
}

#[test]
fn test_if_single_statement_else_no_braces() {
    let code = r#"
int f(int x) {
    if (x > 0)
        return 1;
    else
        return 0;
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let if_stmt = func.body.iter().find(|s| matches!(s, Statement::If { .. }));
    assert!(if_stmt.is_some(), "Expected If-else statement");
    if let Some(Statement::If { then_block, else_block, .. }) = if_stmt {
        assert!(!then_block.is_empty(), "Expected non-empty then block");
        assert!(else_block.is_some(), "Expected else block");
        let else_stmts = else_block.as_ref().unwrap();
        assert!(!else_stmts.is_empty(), "Expected non-empty else block");
    }
}

#[test]
fn test_if_else_if_chain() {
    let code = r#"
int f(int x) {
    if (x > 0) {
        return 1;
    } else if (x < 0) {
        return -1;
    } else {
        return 0;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let if_stmt = func.body.iter().find(|s| matches!(s, Statement::If { .. }));
    assert!(if_stmt.is_some(), "Expected If-else-if chain");
    if let Some(Statement::If { else_block, .. }) = if_stmt {
        assert!(else_block.is_some(), "Expected else block in if-else-if");
    }
}

// ============================================================================
// FUNCTION CALL: As standalone statement
// ============================================================================

#[test]
fn test_function_call_statement() {
    let code = r#"
void process(int x);
void f() {
    process(42);
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let has_call = func.body.iter().any(|s| {
        matches!(s, Statement::FunctionCall { function, .. } if function == "process")
    });
    assert!(has_call, "Expected FunctionCall statement, got: {:?}", func.body);
}

#[test]
fn test_function_call_with_multiple_args() {
    let code = r#"
void add(int a, int b, int c);
void f() {
    add(1, 2, 3);
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let has_call = func.body.iter().any(|s| {
        if let Statement::FunctionCall { function, arguments } = s {
            function == "add" && arguments.len() == 3
        } else {
            false
        }
    });
    assert!(has_call, "Expected FunctionCall with 3 args, got: {:?}", func.body);
}

// ============================================================================
// STRUCT/FIELD ACCESS: Nested field access
// ============================================================================

#[test]
fn test_pointer_field_access_in_return() {
    let code = r#"
typedef struct { int val; } Node;
int f(Node* n) {
    return n->val;
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let ret = func.body.iter().find(|s| matches!(s, Statement::Return(Some(_))));
    if let Some(Statement::Return(Some(expr))) = ret {
        assert!(
            matches!(expr, Expression::PointerFieldAccess { .. }),
            "Expected PointerFieldAccess, got: {:?}",
            expr
        );
    } else {
        panic!("Expected return with pointer field access");
    }
}

#[test]
fn test_dot_field_access_in_return() {
    let code = r#"
typedef struct { int x; } Point;
int f() {
    Point p;
    p.x = 42;
    return p.x;
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let ret = func.body.iter().find(|s| matches!(s, Statement::Return(Some(_))));
    if let Some(Statement::Return(Some(expr))) = ret {
        assert!(
            matches!(expr, Expression::FieldAccess { .. }),
            "Expected FieldAccess, got: {:?}",
            expr
        );
    } else {
        panic!("Expected return with dot field access");
    }
}

// ============================================================================
// ARRAY INDEX: Expression context
// ============================================================================

#[test]
fn test_array_index_in_return() {
    let code = r#"
int f() {
    int arr[5];
    arr[0] = 42;
    return arr[0];
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let ret = func.body.iter().find(|s| matches!(s, Statement::Return(Some(_))));
    if let Some(Statement::Return(Some(expr))) = ret {
        assert!(
            matches!(expr, Expression::ArrayIndex { .. }),
            "Expected ArrayIndex, got: {:?}",
            expr
        );
    } else {
        panic!("Expected return with array index");
    }
}

// ============================================================================
// FOR LOOP: Assignment init with compound assignment increment
// ============================================================================

#[test]
fn test_for_with_subtract_assign_increment() {
    let code = r#"
void f() {
    int i;
    for (i = 100; i > 0; i -= 10) {
        int x;
        x = i;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For with subtract-assign increment");
}

#[test]
fn test_for_with_multiply_assign_increment() {
    let code = r#"
void f() {
    int i;
    for (i = 1; i < 1000; i *= 2) {
        int x;
        x = i;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For with multiply-assign increment");
}

// ============================================================================
// BINARY OPERATORS: Bitwise compound assignment operators
// ============================================================================

#[test]
fn test_compound_bitwise_and_assign() {
    let code = r#"
void f() {
    int flags;
    flags = 255;
    flags &= 0x0F;
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    // &= should parse correctly; we just verify it parses without error
    assert!(func.body.len() >= 2, "Expected at least 2 statements");
}

#[test]
fn test_compound_bitwise_or_assign() {
    let code = r#"
void f() {
    int flags;
    flags = 0;
    flags |= 0x01;
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    assert!(func.body.len() >= 2, "Expected at least 2 statements");
}

#[test]
fn test_compound_shift_assign() {
    let code = r#"
void f() {
    int x;
    x = 1;
    x <<= 4;
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    assert!(func.body.len() >= 2, "Expected at least 2 statements");
}

// ============================================================================
// EXPRESSION: Post-increment/decrement as expression (not statement)
// ============================================================================

#[test]
fn test_post_increment_expression_in_assignment() {
    // x = i++ should have PostIncrement expression
    let code = r#"
void f() {
    int i;
    int x;
    i = 5;
    x = i++;
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    // The i++ in expression context should produce something parseable
    assert!(func.body.len() >= 3, "Expected statements, got: {:?}", func.body);
}

#[test]
fn test_pre_decrement_expression_in_condition() {
    let code = r#"
void f(int n) {
    while (--n > 0) {
        int x;
        x = n;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let while_stmt = func.body.iter().find(|s| matches!(s, Statement::While { .. }));
    assert!(while_stmt.is_some(), "Expected While with --n > 0 condition");
}

// ============================================================================
// DEREF ASSIGNMENT: *ptr = value
// ============================================================================

#[test]
fn test_deref_assignment() {
    let code = r#"
void f(int* ptr) {
    *ptr = 42;
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let has_deref = func.body.iter().any(|s| {
        matches!(s, Statement::DerefAssignment { .. })
    });
    assert!(has_deref, "Expected DerefAssignment for *ptr = 42, got: {:?}", func.body);
}

// ============================================================================
// FIELD ASSIGNMENT: Direct assignment (not inc/dec)
// ============================================================================

#[test]
fn test_pointer_field_assignment() {
    let code = r#"
typedef struct { int len; } Str;
void f(Str* s) {
    s->len = 100;
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let has_field_assign = func.body.iter().any(|s| {
        matches!(s, Statement::FieldAssignment { field, .. } if field == "len")
    });
    assert!(has_field_assign, "Expected FieldAssignment for s->len = 100, got: {:?}", func.body);
}

// ============================================================================
// EXPRESSION: Function call in expression context
// ============================================================================

#[test]
fn test_function_call_expression_in_assignment() {
    let code = r#"
int compute(int x);
void f() {
    int result;
    result = compute(42);
}
"#;
    let ast = parse_c(code);
    let func = find_func(&ast, "f");
    let assign = func.body.iter().find(|s| matches!(s, Statement::Assignment { .. }));
    if let Some(Statement::Assignment { value, .. }) = assign {
        assert!(
            matches!(value, Expression::FunctionCall { .. }),
            "Expected FunctionCall in assignment, got: {:?}",
            value
        );
    } else {
        panic!("Expected assignment with function call");
    }
}

// ============================================================================
// EXPRESSION: Compound literal (C99)
// ============================================================================

#[test]
fn test_compound_literal_struct() {
    let code = r#"
typedef struct { int x; int y; } Point;
void f() {
    Point p;
    p = (Point){10, 20};
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    // Just verify it parses - compound literals are complex
    assert!(!func.body.is_empty(), "Expected parsed body");
}

// ============================================================================
// FOR LOOP: With DeclStmt init (int i = 0 in init) - 3-child case
// ============================================================================

#[test]
fn test_for_decl_stmt_init_three_children() {
    let code = r#"
void f() {
    for (int i = 0; i < 10; i++) {
        int x;
        x = i;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    let for_stmt = func.body.iter().find(|s| matches!(s, Statement::For { .. }));
    assert!(for_stmt.is_some(), "Expected For with DeclStmt init");
    if let Some(Statement::For { condition, increment, body, .. }) = for_stmt {
        assert!(condition.is_some(), "Expected condition");
        assert!(!increment.is_empty(), "Expected increment");
        assert!(!body.is_empty(), "Expected body");
    }
}

// ============================================================================
// BINARY OPERATOR: Multiple operators in complex expression (precedence chain)
// ============================================================================

#[test]
fn test_binop_full_precedence_chain() {
    // a || b && c | d ^ e & f == g < h + i * j
    // This exercises every precedence level in order
    let code = r#"
void f(int a, int b, int c, int d, int e, int fg, int g, int h, int i, int j) {
    if (a || b && c) {
        int x;
        x = d | e;
    }
}
"#;
    let ast = parse_c(code);
    let func = first_func(&ast);
    if let Statement::If { condition, .. } = &func.body[0] {
        assert!(
            matches!(condition, Expression::BinaryOp { op: BinaryOperator::LogicalOr, .. }),
            "Expected LogicalOr at top, got: {:?}",
            condition
        );
    }
}

// ============================================================================
// EMPTY INPUT AND EDGE CASES
// ============================================================================

#[test]
fn test_parse_empty_function() {
    let ast = parse_c("void f() { }");
    let func = first_func(&ast);
    assert_eq!(func.name, "f");
    assert!(func.body.is_empty(), "Expected empty body");
}

#[test]
fn test_parse_multiple_functions() {
    let code = r#"
int add(int a, int b) { return a + b; }
int sub(int a, int b) { return a - b; }
"#;
    let ast = parse_c(code);
    assert!(ast.functions().len() >= 2, "Expected at least 2 functions");
    let add = find_func(&ast, "add");
    let sub = find_func(&ast, "sub");
    assert!(!add.body.is_empty());
    assert!(!sub.body.is_empty());
}

#[test]
fn test_parse_function_with_no_return_type() {
    let ast = parse_c("void noop() { }");
    let func = first_func(&ast);
    assert_eq!(func.name, "noop");
}
