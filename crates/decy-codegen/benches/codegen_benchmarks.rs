//! Benchmarks for code generation
//!
//! Measures performance of Rust code generation from HIR.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use decy_codegen::CodeGenerator;
use decy_hir::{BinaryOperator, HirExpression, HirFunction, HirParameter, HirStatement, HirType};

// ============================================================================
// Benchmark: Type Mapping
// ============================================================================

fn bench_type_mapping(c: &mut Criterion) {
    let mut group = c.benchmark_group("type_mapping");

    // Simple types
    group.bench_function("simple_int", |b| {
        b.iter(|| CodeGenerator::map_type(black_box(&HirType::Int)))
    });

    group.bench_function("simple_float", |b| {
        b.iter(|| CodeGenerator::map_type(black_box(&HirType::Float)))
    });

    // Pointer types
    let ptr_type = HirType::Pointer(Box::new(HirType::Int));
    group.bench_function("pointer", |b| {
        b.iter(|| CodeGenerator::map_type(black_box(&ptr_type)))
    });

    // Box types
    let box_type = HirType::Box(Box::new(HirType::Int));
    group.bench_function("box_type", |b| {
        b.iter(|| CodeGenerator::map_type(black_box(&box_type)))
    });

    // Vec types
    let vec_type = HirType::Vec(Box::new(HirType::Int));
    group.bench_function("vec_type", |b| {
        b.iter(|| CodeGenerator::map_type(black_box(&vec_type)))
    });

    // Nested types (pointer to pointer)
    let nested_ptr = HirType::Pointer(Box::new(HirType::Pointer(Box::new(HirType::Int))));
    group.bench_function("nested_pointer", |b| {
        b.iter(|| CodeGenerator::map_type(black_box(&nested_ptr)))
    });

    group.finish();
}

// ============================================================================
// Benchmark: Expression Generation
// ============================================================================

fn bench_expression_generation(c: &mut Criterion) {
    let codegen = CodeGenerator::new();
    let mut group = c.benchmark_group("expression_generation");

    // Simple literal
    let int_literal = HirExpression::IntLiteral(42);
    group.bench_function("int_literal", |b| {
        b.iter(|| codegen.generate_expression(black_box(&int_literal)))
    });

    // String literal
    let str_literal = HirExpression::StringLiteral("hello world".to_string());
    group.bench_function("string_literal", |b| {
        b.iter(|| codegen.generate_expression(black_box(&str_literal)))
    });

    // Simple binary operation
    let binary_op = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::IntLiteral(10)),
        right: Box::new(HirExpression::IntLiteral(20)),
    };
    group.bench_function("simple_binary_op", |b| {
        b.iter(|| codegen.generate_expression(black_box(&binary_op)))
    });

    // Nested binary operations: (a + b) * (c - d)
    let nested_binary = HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
        right: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Subtract,
            left: Box::new(HirExpression::Variable("c".to_string())),
            right: Box::new(HirExpression::Variable("d".to_string())),
        }),
    };
    group.bench_function("nested_binary_op", |b| {
        b.iter(|| codegen.generate_expression(black_box(&nested_binary)))
    });

    // Function call with arguments
    let func_call = HirExpression::FunctionCall {
        function: "calculate".to_string(),
        arguments: vec![
            HirExpression::IntLiteral(1),
            HirExpression::IntLiteral(2),
            HirExpression::IntLiteral(3),
        ],
    };
    group.bench_function("function_call", |b| {
        b.iter(|| codegen.generate_expression(black_box(&func_call)))
    });

    group.finish();
}

// ============================================================================
// Benchmark: Statement Generation
// ============================================================================

fn bench_statement_generation(c: &mut Criterion) {
    let codegen = CodeGenerator::new();
    let mut group = c.benchmark_group("statement_generation");

    // Variable declaration with initializer
    let var_decl = HirStatement::VariableDeclaration {
        name: "x".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::IntLiteral(42)),
    };
    group.bench_function("var_declaration", |b| {
        b.iter(|| codegen.generate_statement(black_box(&var_decl)))
    });

    // Return statement
    let return_stmt = HirStatement::Return(Some(HirExpression::Variable("x".to_string())));
    group.bench_function("return_statement", |b| {
        b.iter(|| codegen.generate_statement(black_box(&return_stmt)))
    });

    // If statement with then and else blocks
    let if_stmt = HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(10)),
        },
        then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
        else_block: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(
            0,
        )))]),
    };
    group.bench_function("if_statement", |b| {
        b.iter(|| codegen.generate_statement(black_box(&if_stmt)))
    });

    // While loop
    let while_stmt = HirStatement::While {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(10)),
        },
        body: vec![
            HirStatement::Assignment {
                target: "sum".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("sum".to_string())),
                    right: Box::new(HirExpression::Variable("i".to_string())),
                },
            },
            HirStatement::Assignment {
                target: "i".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("i".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            },
        ],
    };
    group.bench_function("while_loop", |b| {
        b.iter(|| codegen.generate_statement(black_box(&while_stmt)))
    });

    group.finish();
}

// ============================================================================
// Benchmark: Signature Generation
// ============================================================================

fn bench_signature_generation(c: &mut Criterion) {
    let codegen = CodeGenerator::new();

    // Simple function with no parameters
    let simple_func = HirFunction::new("test".to_string(), HirType::Void, vec![]);
    c.bench_function("signature_no_params", |b| {
        b.iter(|| codegen.generate_signature(black_box(&simple_func)))
    });

    // Function with parameters
    let func_with_params = HirFunction::new(
        "add".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
    );
    c.bench_function("signature_with_params", |b| {
        b.iter(|| codegen.generate_signature(black_box(&func_with_params)))
    });

    // Function with many parameters (10 params)
    let many_params: Vec<HirParameter> = (0..10)
        .map(|i| HirParameter::new(format!("param{}", i), HirType::Int))
        .collect();
    let func_many_params = HirFunction::new("complex".to_string(), HirType::Int, many_params);
    c.bench_function("signature_many_params", |b| {
        b.iter(|| codegen.generate_signature(black_box(&func_many_params)))
    });
}

// ============================================================================
// Benchmark: Complete Function Generation
// ============================================================================

fn bench_function_generation(c: &mut Criterion) {
    let codegen = CodeGenerator::new();
    let mut group = c.benchmark_group("function_generation");

    // Empty function
    let empty_func = HirFunction::new("empty".to_string(), HirType::Void, vec![]);
    group.bench_function("empty_function", |b| {
        b.iter(|| codegen.generate_function(black_box(&empty_func)))
    });

    // Simple function with return
    let simple_func = HirFunction::new_with_body(
        "get_answer".to_string(),
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::IntLiteral(42)))],
    );
    group.bench_function("simple_function", |b| {
        b.iter(|| codegen.generate_function(black_box(&simple_func)))
    });

    // Function with parameters and body
    let func_with_body = HirFunction::new_with_body(
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
    group.bench_function("function_with_params", |b| {
        b.iter(|| codegen.generate_function(black_box(&func_with_body)))
    });

    // Complex function with control flow
    let complex_func = HirFunction::new_with_body(
        "calculate".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("x".to_string(), HirType::Int),
            HirParameter::new("y".to_string(), HirType::Int),
        ],
        vec![
            HirStatement::VariableDeclaration {
                name: "result".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::GreaterThan,
                    left: Box::new(HirExpression::Variable("x".to_string())),
                    right: Box::new(HirExpression::Variable("y".to_string())),
                },
                then_block: vec![HirStatement::Assignment {
                    target: "result".to_string(),
                    value: HirExpression::Variable("x".to_string()),
                }],
                else_block: Some(vec![HirStatement::Assignment {
                    target: "result".to_string(),
                    value: HirExpression::Variable("y".to_string()),
                }]),
            },
            HirStatement::Return(Some(HirExpression::Variable("result".to_string()))),
        ],
    );
    group.bench_function("complex_function", |b| {
        b.iter(|| codegen.generate_function(black_box(&complex_func)))
    });

    group.finish();
}

// ============================================================================
// Benchmark: Function Generation Scaling
// ============================================================================

fn bench_function_scaling(c: &mut Criterion) {
    let codegen = CodeGenerator::new();
    let mut group = c.benchmark_group("function_scaling");

    for num_statements in [1, 5, 10, 20, 50].iter() {
        // Create function with N variable declarations
        let mut body = vec![];
        for i in 0..*num_statements {
            body.push(HirStatement::VariableDeclaration {
                name: format!("var{}", i),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(i)),
            });
        }
        body.push(HirStatement::Return(Some(HirExpression::IntLiteral(0))));

        let func = HirFunction::new_with_body("test".to_string(), HirType::Int, vec![], body);

        group.bench_with_input(
            BenchmarkId::from_parameter(num_statements),
            num_statements,
            |b, _| b.iter(|| codegen.generate_function(black_box(&func))),
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark: Box Transformation
// ============================================================================

fn bench_box_transformation(c: &mut Criterion) {
    let codegen = CodeGenerator::new();
    let mut group = c.benchmark_group("box_transformation");

    // Function with malloc pattern (Box candidate)
    let malloc_func = HirFunction::new_with_body(
        "allocate".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "ptr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::IntLiteral(4)],
            }),
        }],
    );

    let box_candidate = decy_analyzer::patterns::BoxCandidate {
        variable: "ptr".to_string(),
        malloc_index: 0,
        free_index: None,
    };

    group.bench_function("single_box_transform", |b| {
        b.iter(|| {
            codegen.generate_function_with_box_transform(
                black_box(&malloc_func),
                black_box(std::slice::from_ref(&box_candidate)),
            )
        })
    });

    // Function with multiple malloc patterns
    let mut body = vec![];
    let mut candidates = vec![];
    for i in 0..10 {
        body.push(HirStatement::VariableDeclaration {
            name: format!("ptr{}", i),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::IntLiteral(4)],
            }),
        });
        candidates.push(decy_analyzer::patterns::BoxCandidate {
            variable: format!("ptr{}", i),
            malloc_index: i,
            free_index: None,
        });
    }
    let multi_malloc_func =
        HirFunction::new_with_body("allocate_many".to_string(), HirType::Void, vec![], body);

    group.bench_function("multiple_box_transform", |b| {
        b.iter(|| {
            codegen.generate_function_with_box_transform(
                black_box(&multi_malloc_func),
                black_box(&candidates),
            )
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark: Vec Transformation
// ============================================================================

fn bench_vec_transformation(c: &mut Criterion) {
    let codegen = CodeGenerator::new();

    // Function with array malloc pattern (Vec candidate)
    let size_expr = HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(HirExpression::IntLiteral(10)),
        right: Box::new(HirExpression::IntLiteral(4)),
    };

    let vec_func = HirFunction::new_with_body(
        "allocate_array".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![size_expr.clone()],
            }),
        }],
    );

    let vec_candidate = decy_analyzer::patterns::VecCandidate {
        variable: "arr".to_string(),
        malloc_index: 0,
        free_index: None,
        capacity_expr: Some(HirExpression::IntLiteral(10)),
    };

    c.bench_function("vec_transform", |b| {
        b.iter(|| {
            codegen.generate_function_with_vec_transform(
                black_box(&vec_func),
                black_box(std::slice::from_ref(&vec_candidate)),
            )
        })
    });
}

criterion_group!(
    benches,
    bench_type_mapping,
    bench_expression_generation,
    bench_statement_generation,
    bench_signature_generation,
    bench_function_generation,
    bench_function_scaling,
    bench_box_transformation,
    bench_vec_transformation,
);
criterion_main!(benches);
