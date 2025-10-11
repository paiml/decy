//! End-to-end pipeline benchmarks
//!
//! Measures performance of the complete C-to-Rust transpilation pipeline.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use decy_core::{transpile, transpile_with_box_transform};

// ============================================================================
// Simple Function Benchmarks
// ============================================================================

fn bench_simple_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline_simple");

    // Minimal function
    let minimal = "int main() { return 0; }";
    group.bench_function("minimal_function", |b| {
        b.iter(|| transpile(black_box(minimal)))
    });

    // Function with parameters
    let with_params = "int add(int a, int b) { return a + b; }";
    group.bench_function("function_with_params", |b| {
        b.iter(|| transpile(black_box(with_params)))
    });

    // Function with variables
    let with_vars = r#"
        int calculate(int x) {
            int result;
            result = x * 2;
            return result;
        }
    "#;
    group.bench_function("function_with_variables", |b| {
        b.iter(|| transpile(black_box(with_vars)))
    });

    group.finish();
}

// ============================================================================
// Control Flow Benchmarks
// ============================================================================

fn bench_control_flow(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline_control_flow");

    // If statement
    let if_stmt = r#"
        int max(int a, int b) {
            if (a > b) {
                return a;
            } else {
                return b;
            }
        }
    "#;
    group.bench_function("if_statement", |b| b.iter(|| transpile(black_box(if_stmt))));

    // While loop
    let while_loop = r#"
        int sum_to_n(int n) {
            int sum;
            int i;
            sum = 0;
            i = 1;
            while (i <= n) {
                sum = sum + i;
                i = i + 1;
            }
            return sum;
        }
    "#;
    group.bench_function("while_loop", |b| {
        b.iter(|| transpile(black_box(while_loop)))
    });

    // For loop
    let for_loop = r#"
        int factorial(int n) {
            int result;
            int i;
            result = 1;
            for (i = 1; i <= n; i = i + 1) {
                result = result * i;
            }
            return result;
        }
    "#;
    group.bench_function("for_loop", |b| b.iter(|| transpile(black_box(for_loop))));

    group.finish();
}

// ============================================================================
// Multiple Functions Scaling
// ============================================================================

fn bench_multiple_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline_scaling");

    for num_functions in [1, 3, 5, 10].iter() {
        let mut c_code = String::new();
        for i in 0..*num_functions {
            c_code.push_str(&format!("int func_{}(int x) {{ return x + {}; }}\n", i, i));
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(num_functions),
            num_functions,
            |b, _| b.iter(|| transpile(black_box(&c_code))),
        );
    }

    group.finish();
}

// ============================================================================
// Box Transformation Pipeline
// ============================================================================

fn bench_box_transformation_pipeline(c: &mut Criterion) {
    let malloc_code = r#"
        int* create_value() {
            int* p;
            p = malloc(sizeof(int));
            return p;
        }
    "#;

    c.bench_function("pipeline_box_transform", |b| {
        b.iter(|| transpile_with_box_transform(black_box(malloc_code)))
    });
}

// ============================================================================
// Complex Realistic Code
// ============================================================================

fn bench_realistic_code(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline_realistic");

    // Calculator function with multiple operations
    let calculator = r#"
        int calculate(int a, int b, int op) {
            int result;
            if (op == 1) {
                result = a + b;
            } else if (op == 2) {
                result = a - b;
            } else if (op == 3) {
                result = a * b;
            } else {
                result = 0;
            }
            return result;
        }
    "#;
    group.bench_function("calculator", |b| {
        b.iter(|| transpile(black_box(calculator)))
    });

    // Nested control flow
    let nested = r#"
        int complex_logic(int x, int y) {
            int result;
            result = 0;
            if (x > 0) {
                if (y > 0) {
                    result = x + y;
                } else {
                    result = x - y;
                }
            } else {
                if (y > 0) {
                    result = y - x;
                } else {
                    result = 0;
                }
            }
            return result;
        }
    "#;
    group.bench_function("nested_control", |b| {
        b.iter(|| transpile(black_box(nested)))
    });

    // Multiple variables and operations
    let multi_var = r#"
        int process(int input) {
            int a;
            int b;
            int c;
            int result;
            a = input * 2;
            b = a + 10;
            c = b / 2;
            result = c - 5;
            return result;
        }
    "#;
    group.bench_function("multiple_variables", |b| {
        b.iter(|| transpile(black_box(multi_var)))
    });

    group.finish();
}

// ============================================================================
// Comparison: With vs Without Analysis
// ============================================================================

fn bench_analysis_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline_analysis_overhead");

    let code = r#"
        int calculate(int a, int b) {
            int result;
            result = a + b;
            return result;
        }
    "#;

    // Full pipeline with ownership & lifetime analysis
    group.bench_function("with_analysis", |b| b.iter(|| transpile(black_box(code))));

    // Box transformation only (simpler pipeline)
    group.bench_function("box_transform_only", |b| {
        b.iter(|| transpile_with_box_transform(black_box(code)))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_simple_functions,
    bench_control_flow,
    bench_multiple_functions,
    bench_box_transformation_pipeline,
    bench_realistic_code,
    bench_analysis_overhead,
);
criterion_main!(benches);
