//! DECY-172: Tests for size_t typedef preservation in variable declarations
//!
//! When C code uses `size_t` for variable declarations, the generated Rust should:
//! 1. Use the `size_t` type alias (which maps to `usize`)
//! 2. This ensures compatibility with methods like `.len()` that return `usize`
//!
//! Bug:
//! - C: `size_t str_len = strlen(str);`
//! - Generated: `let mut str_len: u32 = str.len();` // Type mismatch!
//!
//! Fix:
//! - Generated: `let mut str_len: size_t = str.len();` // Uses typedef alias

use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType};

/// Create a code generator
fn create_generator() -> CodeGenerator {
    CodeGenerator::new()
}

#[test]
fn test_size_t_variable_uses_typedef_alias() {
    // C code:
    // size_t len = strlen(str);
    //
    // Expected Rust:
    // let mut len: size_t = str.len();
    //
    // NOT:
    // let mut len: u32 = str.len();  // This causes type mismatch!

    let gen = create_generator();

    // Create function with size_t variable
    let func = HirFunction::new_with_body(
        "test_func".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "str".to_string(),
            HirType::Pointer(Box::new(HirType::Char)),
        )],
        vec![
            // size_t len = strlen(str);
            HirStatement::VariableDeclaration {
                name: "len".to_string(),
                var_type: HirType::TypeAlias("size_t".to_string()),
                initializer: Some(HirExpression::FunctionCall {
                    function: "strlen".to_string(),
                    arguments: vec![HirExpression::Variable("str".to_string())],
                }),
            },
        ],
    );

    let code = gen.generate_function(&func);

    println!("Generated: {}", code);

    // Should use size_t type alias, not u32
    assert!(
        code.contains(": size_t") || code.contains(": usize"),
        "Variable should use size_t or usize type, not u32. Got: {}",
        code
    );

    // Should NOT use u32
    assert!(
        !code.contains(": u32"),
        "Variable should NOT use u32 type for size_t. Got: {}",
        code
    );
}

#[test]
fn test_ssize_t_variable_uses_typedef_alias() {
    // C code:
    // ssize_t count = some_function();
    //
    // Expected Rust:
    // let mut count: ssize_t = some_function();
    //
    // NOT:
    // let mut count: i32 = some_function();

    let gen = create_generator();

    let func = HirFunction::new_with_body(
        "test_func".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "count".to_string(),
            var_type: HirType::TypeAlias("ssize_t".to_string()),
            initializer: Some(HirExpression::IntLiteral(0)),
        }],
    );

    let code = gen.generate_function(&func);

    println!("Generated: {}", code);

    // Should use ssize_t type alias, not i32
    assert!(
        code.contains(": ssize_t") || code.contains(": isize"),
        "Variable should use ssize_t or isize type. Got: {}",
        code
    );
}

#[test]
fn test_ptrdiff_t_variable_uses_typedef_alias() {
    // C code:
    // ptrdiff_t diff = ptr2 - ptr1;

    let gen = create_generator();

    let func = HirFunction::new_with_body(
        "test_func".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "diff".to_string(),
            var_type: HirType::TypeAlias("ptrdiff_t".to_string()),
            initializer: Some(HirExpression::IntLiteral(0)),
        }],
    );

    let code = gen.generate_function(&func);

    println!("Generated: {}", code);

    // Should use ptrdiff_t type alias, not i32
    assert!(
        code.contains(": ptrdiff_t") || code.contains(": isize"),
        "Variable should use ptrdiff_t or isize type. Got: {}",
        code
    );
}
