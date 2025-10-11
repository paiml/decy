//! Benchmarks for pattern detection (Box/Vec candidates)
//!
//! Measures performance of malloc/free pattern analysis.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use decy_analyzer::patterns::PatternDetector;
use decy_hir::{BinaryOperator, HirExpression, HirFunction, HirStatement, HirType};

fn create_simple_box_function() -> HirFunction {
    HirFunction::new_with_body(
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
    )
}

fn create_simple_vec_function() -> HirFunction {
    let size_expr = HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(HirExpression::IntLiteral(10)),
        right: Box::new(HirExpression::IntLiteral(4)),
    };

    HirFunction::new_with_body(
        "allocate_array".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![size_expr],
            }),
        }],
    )
}

fn create_complex_function(num_allocations: usize) -> HirFunction {
    let mut body = vec![];

    for i in 0..num_allocations {
        // Alternate between Box and Vec patterns
        if i % 2 == 0 {
            // Box pattern
            body.push(HirStatement::VariableDeclaration {
                name: format!("ptr_{}", i),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(4)],
                }),
            });
        } else {
            // Vec pattern
            let size_expr = HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(HirExpression::IntLiteral(10)),
                right: Box::new(HirExpression::IntLiteral(4)),
            };
            body.push(HirStatement::VariableDeclaration {
                name: format!("arr_{}", i),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![size_expr],
                }),
            });
        }
    }

    HirFunction::new_with_body("complex".to_string(), HirType::Void, vec![], body)
}

fn bench_box_detection_simple(c: &mut Criterion) {
    let detector = PatternDetector::new();
    let func = create_simple_box_function();

    c.bench_function("detect_box_simple", |b| {
        b.iter(|| detector.find_box_candidates(black_box(&func)))
    });
}

fn bench_vec_detection_simple(c: &mut Criterion) {
    let detector = PatternDetector::new();
    let func = create_simple_vec_function();

    c.bench_function("detect_vec_simple", |b| {
        b.iter(|| detector.find_vec_candidates(black_box(&func)))
    });
}

fn bench_box_detection_scaling(c: &mut Criterion) {
    let detector = PatternDetector::new();
    let mut group = c.benchmark_group("detect_box_scaling");

    for num_allocs in [1, 5, 10, 20, 50].iter() {
        let func = create_complex_function(*num_allocs);

        group.bench_with_input(
            BenchmarkId::from_parameter(num_allocs),
            num_allocs,
            |b, _| b.iter(|| detector.find_box_candidates(black_box(&func))),
        );
    }
    group.finish();
}

fn bench_vec_detection_scaling(c: &mut Criterion) {
    let detector = PatternDetector::new();
    let mut group = c.benchmark_group("detect_vec_scaling");

    for num_allocs in [1, 5, 10, 20, 50].iter() {
        let func = create_complex_function(*num_allocs);

        group.bench_with_input(
            BenchmarkId::from_parameter(num_allocs),
            num_allocs,
            |b, _| b.iter(|| detector.find_vec_candidates(black_box(&func))),
        );
    }
    group.finish();
}

fn bench_combined_detection(c: &mut Criterion) {
    let detector = PatternDetector::new();
    let func = create_complex_function(20);

    c.bench_function("detect_box_and_vec_combined", |b| {
        b.iter(|| {
            let _box_candidates = detector.find_box_candidates(black_box(&func));
            let _vec_candidates = detector.find_vec_candidates(black_box(&func));
        })
    });
}

criterion_group!(
    benches,
    bench_box_detection_simple,
    bench_vec_detection_simple,
    bench_box_detection_scaling,
    bench_vec_detection_scaling,
    bench_combined_detection,
);
criterion_main!(benches);
