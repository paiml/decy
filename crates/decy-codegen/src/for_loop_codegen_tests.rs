//! Tests for for loop code generation (DECY-022 RED phase).

use super::*;
use decy_hir::{BinaryOperator, HirExpression, HirStatement, HirType};

#[test]
fn test_generate_simple_for_loop() {
    // for(int i = 0; i < 10; i++) { sum += i; }
    let codegen = CodeGenerator::new();

    let for_loop = HirStatement::For {
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
        increment: vec![HirStatement::Assignment {
            target: "i".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
        body: vec![HirStatement::Assignment {
            target: "sum".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("sum".to_string())),
                right: Box::new(HirExpression::Variable("i".to_string())),
            },
        }],
    };

    let code = codegen.generate_statement(&for_loop);

    // Should generate Rust-style code
    assert!(code.contains("let mut i: i32 = 0"));
    assert!(code.contains("while i < 10"));
    assert!(code.contains("sum = sum + i"));
    assert!(code.contains("i = i + 1"));
}

#[test]
fn test_generate_for_loop_without_init() {
    // for(; i < 10; i++) { }
    let codegen = CodeGenerator::new();

    let for_loop = HirStatement::For {
        init: vec![],
        condition: Some(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(10)),
        }),
        increment: vec![HirStatement::Assignment {
            target: "i".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
        body: vec![],
    };

    let code = codegen.generate_statement(&for_loop);

    // Should not generate init
    assert!(!code.contains("let mut"));
    assert!(code.contains("while i < 10"));
    assert!(code.contains("i = i + 1"));
}

#[test]
fn test_generate_for_loop_without_increment() {
    // for(int i = 0; i < 10; ) { i++; }
    let codegen = CodeGenerator::new();

    let for_loop = HirStatement::For {
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
        body: vec![HirStatement::Assignment {
            target: "i".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
    };

    let code = codegen.generate_statement(&for_loop);

    assert!(code.contains("let mut i: i32 = 0"));
    assert!(code.contains("while i < 10"));
    // Increment should be in body only, not at end of loop
}

#[test]
fn test_generate_for_loop_infinite() {
    // for(;;) { break; }
    let codegen = CodeGenerator::new();

    let for_loop = HirStatement::For {
        init: vec![],
        condition: Some(HirExpression::IntLiteral(1)), // true
        increment: vec![],
        body: vec![HirStatement::Break],
    };

    let code = codegen.generate_statement(&for_loop);

    // Should generate loop (infinite loop in Rust)
    assert!(code.contains("loop") || code.contains("while"));
    assert!(code.contains("break"));
}

#[test]
fn test_generate_nested_for_loops() {
    let codegen = CodeGenerator::new();

    let inner_loop = HirStatement::For {
        init: vec![HirStatement::VariableDeclaration {
            name: "j".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(0)),
        }],
        condition: Some(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("j".to_string())),
            right: Box::new(HirExpression::IntLiteral(10)),
        }),
        increment: vec![HirStatement::Assignment {
            target: "j".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("j".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
        body: vec![],
    };

    let outer_loop = HirStatement::For {
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
        increment: vec![HirStatement::Assignment {
            target: "i".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
        body: vec![inner_loop],
    };

    let code = codegen.generate_statement(&outer_loop);

    // Should have nested while loops
    assert!(code.contains("let mut i: i32 = 0"));
    assert!(code.contains("let mut j: i32 = 0"));
}

#[test]
fn test_generate_for_loop_with_break_continue() {
    let codegen = CodeGenerator::new();

    let for_loop = HirStatement::For {
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
        increment: vec![HirStatement::Assignment {
            target: "i".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
        body: vec![HirStatement::Break],
    };

    let code = codegen.generate_statement(&for_loop);

    assert!(code.contains("break"));
}

#[test]
fn test_for_loop_code_structure() {
    let codegen = CodeGenerator::new();

    let for_loop = HirStatement::For {
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
        increment: vec![HirStatement::Assignment {
            target: "i".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
        body: vec![],
    };

    let code = codegen.generate_statement(&for_loop);

    // Verify structure: init before loop, condition in while, increment at end of body
    let init_pos = code.find("let mut i").expect("Should have init");
    let while_pos = code.find("while").expect("Should have while");
    let increment_pos = code.find("i = i + 1").expect("Should have increment");

    assert!(init_pos < while_pos, "Init should come before while");
    assert!(
        while_pos < increment_pos,
        "While should come before increment"
    );
}

#[test]
fn test_for_infinite_loop_no_condition() {
    // for(;;) { break; } → loop { break; }
    let codegen = CodeGenerator::new();

    let for_loop = HirStatement::For {
        init: vec![],
        condition: None, // for(;;) has no condition
        increment: vec![],
        body: vec![HirStatement::Break],
    };

    let code = codegen.generate_statement(&for_loop);
    assert!(
        code.contains("loop"),
        "for(;;) should generate `loop`, got: {}",
        code
    );
    assert!(
        !code.contains("while"),
        "for(;;) should NOT generate `while`, got: {}",
        code
    );
    assert!(code.contains("break"), "body should contain break");
}
