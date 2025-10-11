//! Benchmarks for the C parser (DECY-001)
//!
//! Measures parsing performance for various C code patterns.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use decy_parser::CParser;

fn bench_simple_function(c: &mut Criterion) {
    let parser = CParser::new().expect("Failed to create parser");
    let source = "int main() { return 0; }";

    c.bench_function("parse_simple_main", |b| {
        b.iter(|| parser.parse(black_box(source)).expect("Parse failed"))
    });
}

fn bench_function_with_parameters(c: &mut Criterion) {
    let parser = CParser::new().expect("Failed to create parser");
    let source = "int add(int a, int b, int c) { return a + b + c; }";

    c.bench_function("parse_function_with_params", |b| {
        b.iter(|| parser.parse(black_box(source)).expect("Parse failed"))
    });
}

fn bench_complex_function(c: &mut Criterion) {
    let parser = CParser::new().expect("Failed to create parser");
    let source = r#"
        int factorial(int n) {
            if (n <= 1) {
                return 1;
            }
            int result = 1;
            while (n > 1) {
                result = result * n;
                n = n - 1;
            }
            return result;
        }
    "#;

    c.bench_function("parse_complex_function", |b| {
        b.iter(|| parser.parse(black_box(source)).expect("Parse failed"))
    });
}

fn bench_multiple_functions(c: &mut Criterion) {
    let parser = CParser::new().expect("Failed to create parser");

    let mut group = c.benchmark_group("parse_multiple_functions");

    for num_functions in [1, 5, 10, 20].iter() {
        let mut source = String::new();
        for i in 0..*num_functions {
            source.push_str(&format!("int func_{}(int x) {{ return x + {}; }}\n", i, i));
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(num_functions),
            num_functions,
            |b, _| b.iter(|| parser.parse(black_box(&source)).expect("Parse failed")),
        );
    }
    group.finish();
}

fn bench_pointer_operations(c: &mut Criterion) {
    let parser = CParser::new().expect("Failed to create parser");
    let source = r#"
        void process(int* ptr) {
            int* p = ptr;
            int x = *p;
            *p = x + 1;
            int** pp = &p;
        }
    "#;

    c.bench_function("parse_pointer_operations", |b| {
        b.iter(|| parser.parse(black_box(source)).expect("Parse failed"))
    });
}

fn bench_struct_definition(c: &mut Criterion) {
    let parser = CParser::new().expect("Failed to create parser");
    let source = r#"
        struct Point {
            int x;
            int y;
            int z;
        };

        struct Point create_point(int x, int y, int z) {
            struct Point p;
            p.x = x;
            p.y = y;
            p.z = z;
            return p;
        }
    "#;

    c.bench_function("parse_struct_definition", |b| {
        b.iter(|| parser.parse(black_box(source)).expect("Parse failed"))
    });
}

fn bench_control_flow(c: &mut Criterion) {
    let parser = CParser::new().expect("Failed to create parser");
    let source = r#"
        int classify(int x) {
            if (x < 0) {
                return -1;
            } else if (x > 0) {
                return 1;
            } else {
                return 0;
            }
        }

        int sum_range(int n) {
            int sum = 0;
            for (int i = 0; i < n; i++) {
                sum = sum + i;
            }
            return sum;
        }

        int find_first(int* arr, int size, int target) {
            int i = 0;
            while (i < size) {
                if (arr[i] == target) {
                    return i;
                }
                i = i + 1;
            }
            return -1;
        }
    "#;

    c.bench_function("parse_control_flow", |b| {
        b.iter(|| parser.parse(black_box(source)).expect("Parse failed"))
    });
}

fn bench_type_variations(c: &mut Criterion) {
    let parser = CParser::new().expect("Failed to create parser");
    let source = r#"
        int int_func(int x) { return x; }
        float float_func(float x) { return x; }
        double double_func(double x) { return x; }
        char char_func(char c) { return c; }
        void void_func() { }
        int* ptr_func(int* p) { return p; }
    "#;

    c.bench_function("parse_type_variations", |b| {
        b.iter(|| parser.parse(black_box(source)).expect("Parse failed"))
    });
}

criterion_group!(
    benches,
    bench_simple_function,
    bench_function_with_parameters,
    bench_complex_function,
    bench_multiple_functions,
    bench_pointer_operations,
    bench_struct_definition,
    bench_control_flow,
    bench_type_variations,
);
criterion_main!(benches);
