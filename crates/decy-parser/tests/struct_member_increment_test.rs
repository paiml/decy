//! DECY-166: Tests for struct member increment/decrement parsing
//!
//! When incrementing a struct member like `sb->length++`, the parser should:
//! 1. Recognize this is a member access increment (not a simple variable increment)
//! 2. Generate a FieldAssignment statement with value = field + 1
//!
//! NOT incorrectly extract just the base variable `sb`.

use decy_parser::{Ast, CParser};

/// Create a C parser for testing
fn parse_c_code(code: &str) -> anyhow::Result<Ast> {
    let parser = CParser::new()?;
    parser.parse(code)
}

#[test]
fn test_struct_member_post_increment_is_field_assignment() {
    // C code:
    // typedef struct { int length; } StringBuilder;
    // void test(StringBuilder* sb) { sb->length++; }
    //
    // Expected: The sb->length++ should be parsed as FieldAssignment,
    // NOT as PostIncrement with target = "sb"

    let code = r#"
typedef struct {
    int length;
} StringBuilder;

void test(StringBuilder* sb) {
    sb->length++;
}
"#;

    let ast = parse_c_code(code).expect("Failed to parse C code");

    // Find the test function
    let test_fn = ast
        .functions()
        .iter()
        .find(|f| f.name == "test")
        .expect("test function not found");

    // Check the function body has statements
    assert!(
        !test_fn.body.is_empty(),
        "test function should have statements"
    );

    // The first statement should be a field assignment (sb->length = sb->length + 1)
    // or an expression statement with PostIncrement on PointerFieldAccess
    let first_stmt = &test_fn.body[0];

    // Debug print the statement type
    println!("First statement: {:?}", first_stmt);

    // The statement should involve a field operation, not just a simple variable
    let stmt_str = format!("{:?}", first_stmt);

    // Should NOT be PostIncrement { target: "sb" } - that's wrong!
    assert!(
        !stmt_str.contains("PostIncrement") || !stmt_str.contains(r#"target: "sb""#),
        "sb->length++ should NOT be parsed as PostIncrement {{ target: \"sb\" }}, got: {}",
        stmt_str
    );

    // Should contain reference to "length" field
    assert!(
        stmt_str.contains("length") || stmt_str.contains("FieldAssignment"),
        "Statement should reference the 'length' field or be a FieldAssignment, got: {}",
        stmt_str
    );
}

#[test]
fn test_struct_member_pre_increment_is_field_assignment() {
    // C code: ++sb->length should also be handled correctly
    let code = r#"
typedef struct {
    int count;
} Counter;

void increment(Counter* c) {
    ++c->count;
}
"#;

    let ast = parse_c_code(code).expect("Failed to parse C code");

    let incr_fn = ast
        .functions()
        .iter()
        .find(|f| f.name == "increment")
        .expect("increment function not found");

    assert!(!incr_fn.body.is_empty(), "function should have statements");

    let first_stmt = &incr_fn.body[0];
    println!("First statement: {:?}", first_stmt);

    let stmt_str = format!("{:?}", first_stmt);

    // Should NOT be PreIncrement { target: "c" }
    assert!(
        !stmt_str.contains("PreIncrement") || !stmt_str.contains(r#"target: "c""#),
        "++c->count should NOT be parsed as PreIncrement {{ target: \"c\" }}, got: {}",
        stmt_str
    );

    // Should reference the field
    assert!(
        stmt_str.contains("count") || stmt_str.contains("FieldAssignment"),
        "Statement should reference 'count' field or be a FieldAssignment, got: {}",
        stmt_str
    );
}

#[test]
fn test_struct_member_decrement() {
    // C code: sb->length-- should also work
    let code = r#"
typedef struct {
    int value;
} Data;

void decrement(Data* d) {
    d->value--;
}
"#;

    let ast = parse_c_code(code).expect("Failed to parse C code");

    let decr_fn = ast
        .functions()
        .iter()
        .find(|f| f.name == "decrement")
        .expect("decrement function not found");

    assert!(!decr_fn.body.is_empty(), "function should have statements");

    let first_stmt = &decr_fn.body[0];
    println!("First statement: {:?}", first_stmt);

    let stmt_str = format!("{:?}", first_stmt);

    // Should NOT be PostDecrement { target: "d" }
    assert!(
        !stmt_str.contains("PostDecrement") || !stmt_str.contains(r#"target: "d""#),
        "d->value-- should NOT be parsed as PostDecrement {{ target: \"d\" }}, got: {}",
        stmt_str
    );

    // Should reference the field
    assert!(
        stmt_str.contains("value") || stmt_str.contains("FieldAssignment"),
        "Statement should reference 'value' field or be a FieldAssignment, got: {}",
        stmt_str
    );
}
