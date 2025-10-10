# Code Generation Verification

Code generation is the final stage of the DECY pipeline, converting the High-level IR (HIR) with ownership and lifetime annotations into idiomatic Rust code.

## Architecture

```
HIR + Ownership + Lifetimes → CodeGenerator → Rust Source Code
```

## Basic Function Generation

### Input HIR

```rust,ignore
HirFunction {
    name: "add",
    return_type: HirType::Int,
    parameters: vec![
        HirParameter { name: "a", ty: HirType::Int },
        HirParameter { name: "b", ty: HirType::Int },
    ],
    body: vec![
        HirStatement::Return(
            HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Var("a")),
                right: Box::new(HirExpression::Var("b")),
            }
        )
    ],
    lifetimes: vec![],
}
```

### Expected Rust Output

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Verification Test

```rust,ignore
use decy_codegen::CodeGenerator;

#[test]
fn test_generate_simple_function() {
    let hir_func = create_add_function();  // Helper to create HIR

    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_function(&hir_func);

    assert!(rust_code.contains("fn add"));
    assert!(rust_code.contains("i32"));
    assert!(rust_code.contains("a + b"));
    assert!(!rust_code.contains("return"));  // Should use implicit return
}
```

## Type Mapping

### C Types → Rust Types

| C Type | Rust Type |
|--------|-----------|
| `int` | `i32` |
| `char` | `i8` |
| `float` | `f32` |
| `double` | `f64` |
| `void` | `()` |
| `int*` | `*mut i32` or `&mut i32` |
| `const int*` | `*const i32` or `&i32` |

### Verification Test

```rust,ignore
#[test]
fn test_type_mapping() {
    let codegen = CodeGenerator::new();

    assert_eq!(codegen.map_type(&HirType::Int), "i32");
    assert_eq!(codegen.map_type(&HirType::Char), "i8");
    assert_eq!(codegen.map_type(&HirType::Float), "f32");
    assert_eq!(codegen.map_type(&HirType::Double), "f64");
    assert_eq!(codegen.map_type(&HirType::Void), "()");
}

#[test]
fn test_pointer_mapping() {
    let codegen = CodeGenerator::new();

    let int_ptr = HirType::Pointer(Box::new(HirType::Int));
    assert_eq!(codegen.map_type(&int_ptr), "*mut i32");

    let const_int_ptr = HirType::ConstPointer(Box::new(HirType::Int));
    assert_eq!(codegen.map_type(&const_int_ptr), "*const i32");
}
```

## Ownership Pattern Generation

### malloc → Box

```rust,ignore
// C code
int* p = malloc(sizeof(int));
*p = 42;

// Generated Rust
let mut p = Box::new(0i32);
*p = 42;
```

### Verification Test

```rust,ignore
#[test]
fn test_malloc_becomes_box() {
    let hir_stmt = HirStatement::VarDecl {
        name: "p".to_string(),
        ty: HirType::Pointer(Box::new(HirType::Int)),
        init: Some(HirExpression::Malloc {
            size: 4,
            ty: HirType::Int,
        }),
        ownership: OwnershipPattern::Owning,
    };

    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_statement(&hir_stmt);

    assert!(rust_code.contains("Box::new"));
    assert!(!rust_code.contains("malloc"));
}
```

### Function Parameters → References

```rust,ignore
// C code
void modify(int* p) {
    *p = 10;
}

// Generated Rust
fn modify(p: &mut i32) {
    *p = 10;
}
```

### Verification Test

```rust,ignore
#[test]
fn test_parameter_pointer_becomes_reference() {
    let hir_func = HirFunction {
        name: "modify".to_string(),
        return_type: HirType::Void,
        parameters: vec![
            HirParameter {
                name: "p".to_string(),
                ty: HirType::Pointer(Box::new(HirType::Int)),
                ownership: OwnershipPattern::Borrowed,
                is_mutable: true,
            }
        ],
        body: vec![
            HirStatement::Assignment {
                target: HirExpression::Dereference(
                    Box::new(HirExpression::Var("p"))
                ),
                value: HirExpression::Literal(10),
            }
        ],
        lifetimes: vec![],
    };

    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_function(&hir_func);

    assert!(rust_code.contains("&mut i32"));
    assert!(!rust_code.contains("*mut"));
}
```

## Lifetime Annotation Generation

### Single Lifetime

```rust,ignore
// C code
const char* get_message() {
    static const char* msg = "Hello";
    return msg;
}

// Generated Rust
fn get_message() -> &'static str {
    "Hello"
}
```

### Multiple Lifetimes

```rust,ignore
// C code
int* choose(int* a, int* b, int condition) {
    return condition ? a : b;
}

// Generated Rust
fn choose<'a>(a: &'a i32, b: &'a i32, condition: i32) -> &'a i32 {
    if condition != 0 { a } else { b }
}
```

### Verification Test

```rust,ignore
#[test]
fn test_lifetime_annotation_generation() {
    let hir_func = HirFunction {
        name: "choose".to_string(),
        return_type: HirType::Pointer(Box::new(HirType::Int)),
        parameters: vec![
            HirParameter {
                name: "a".to_string(),
                ty: HirType::Pointer(Box::new(HirType::Int)),
                ownership: OwnershipPattern::Borrowed,
                lifetime: Some(Lifetime::Named("a".to_string())),
            },
            HirParameter {
                name: "b".to_string(),
                ty: HirType::Pointer(Box::new(HirType::Int)),
                ownership: OwnershipPattern::Borrowed,
                lifetime: Some(Lifetime::Named("a".to_string())),
            },
        ],
        body: vec![/* ... */],
        lifetimes: vec![Lifetime::Named("a".to_string())],
    };

    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_function(&hir_func);

    assert!(rust_code.contains("<'a>"));
    assert!(rust_code.contains("&'a i32"));
}
```

## Property Tests for Codegen

### Property: Generated Code Always Compiles

```rust,ignore
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_generated_code_compiles(hir_func in valid_hir_function()) {
        let codegen = CodeGenerator::new();
        let rust_code = codegen.generate_function(&hir_func);

        // Property: All generated code must compile
        prop_assert!(
            compile_rust(&rust_code).is_ok(),
            "Generated code must compile:\n{}",
            rust_code
        );
    }
}
```

### Property: Generated Code Passes Clippy

```rust,ignore
proptest! {
    #[test]
    fn prop_generated_code_passes_clippy(hir_func in valid_hir_function()) {
        let codegen = CodeGenerator::new();
        let rust_code = codegen.generate_function(&hir_func);

        // Property: Generated code has zero clippy warnings
        prop_assert!(
            clippy_check(&rust_code).is_ok(),
            "Generated code must pass clippy:\n{}",
            rust_code
        );
    }
}
```

### Property: Generated Code is Formatted

```rust,ignore
proptest! {
    #[test]
    fn prop_generated_code_is_formatted(hir_func in valid_hir_function()) {
        let codegen = CodeGenerator::new();
        let rust_code = codegen.generate_function(&hir_func);

        // Property: Generated code should be properly formatted
        let formatted = format_rust(&rust_code).unwrap();
        prop_assert_eq!(&rust_code, &formatted, "Generated code should be formatted");
    }
}
```

### Property: No Unsafe Blocks for Safe Code

```rust,ignore
proptest! {
    #[test]
    fn prop_no_unsafe_for_safe_code(hir_func in memory_safe_hir()) {
        let codegen = CodeGenerator::new();
        let rust_code = codegen.generate_function(&hir_func);

        // Property: Memory-safe HIR → no unsafe blocks
        prop_assert!(
            !rust_code.contains("unsafe"),
            "Safe HIR should not generate unsafe blocks"
        );
    }
}
```

## Compilation Verification

Test that generated code actually compiles:

```rust,ignore
use std::process::Command;
use std::fs;

fn compile_rust(code: &str) -> Result<()> {
    let temp_file = "/tmp/decy_codegen_test.rs";
    fs::write(temp_file, code)?;

    let output = Command::new("rustc")
        .args(&["--crate-type", "lib", temp_file])
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Compilation failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

#[test]
fn test_compilation_simple_function() {
    let hir_func = create_add_function();
    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_function(&hir_func);

    compile_rust(&rust_code).expect("Generated code should compile");
}
```

## Clippy Verification

Test that generated code passes clippy:

```rust,ignore
fn clippy_check(code: &str) -> Result<()> {
    let temp_dir = tempdir()?;
    let lib_file = temp_dir.path().join("lib.rs");
    fs::write(&lib_file, code)?;

    let output = Command::new("cargo")
        .args(&["clippy", "--", "-D", "warnings"])
        .current_dir(&temp_dir)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Clippy failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

#[test]
fn test_clippy_simple_function() {
    let hir_func = create_add_function();
    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_function(&hir_func);

    clippy_check(&rust_code).expect("Generated code should pass clippy");
}
```

## Formatting

Generate properly formatted Rust code:

```rust,ignore
impl CodeGenerator {
    pub fn generate_function(&self, func: &HirFunction) -> String {
        let mut code = String::new();

        // Generate function signature
        self.generate_signature(func, &mut code);

        // Generate body
        code.push_str(" {\n");
        for stmt in &func.body {
            code.push_str("    ");  // 4-space indent
            self.generate_statement(stmt, &mut code);
            code.push('\n');
        }
        code.push_str("}\n");

        code
    }
}

#[test]
fn test_formatting_indentation() {
    let hir_func = create_function_with_multiple_statements();
    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_function(&hir_func);

    // Verify proper indentation
    let lines: Vec<&str> = rust_code.lines().collect();
    for line in &lines[1..lines.len()-1] {  // Skip first and last line
        if !line.is_empty() {
            assert!(
                line.starts_with("    "),
                "Body statements should be indented with 4 spaces"
            );
        }
    }
}
```

## Mutation Testing for Codegen

### Original Code

```rust,ignore
pub fn map_type(&self, ty: &HirType) -> String {
    match ty {
        HirType::Int => "i32".to_string(),
        HirType::Char => "i8".to_string(),
        HirType::Void => "()".to_string(),
        HirType::Pointer(inner) => {
            format!("*mut {}", self.map_type(inner))
        }
    }
}
```

### Expected Mutants

1. Replace `"i32"` with `"i64"`
2. Replace `"i8"` with `"u8"`
3. Replace `"*mut"` with `"*const"`
4. Replace `"()"` with `""`

### Tests to Catch Mutants

```rust,ignore
#[test]
fn test_int_maps_to_i32_exactly() {
    let codegen = CodeGenerator::new();
    assert_eq!(codegen.map_type(&HirType::Int), "i32");
    assert_ne!(codegen.map_type(&HirType::Int), "i64");
}

#[test]
fn test_char_maps_to_i8_exactly() {
    let codegen = CodeGenerator::new();
    assert_eq!(codegen.map_type(&HirType::Char), "i8");
    assert_ne!(codegen.map_type(&HirType::Char), "u8");
}

#[test]
fn test_void_maps_to_unit() {
    let codegen = CodeGenerator::new();
    assert_eq!(codegen.map_type(&HirType::Void), "()");
    assert!(!codegen.map_type(&HirType::Void).is_empty());
}

#[test]
fn test_pointer_is_mutable() {
    let codegen = CodeGenerator::new();
    let ptr = HirType::Pointer(Box::new(HirType::Int));
    let result = codegen.map_type(&ptr);
    assert!(result.contains("*mut"));
    assert!(!result.contains("*const"));
}
```

## End-to-End Tests

Test complete transpilation pipeline:

```rust,ignore
#[test]
fn test_e2e_simple_function() {
    let c_code = "int add(int a, int b) { return a + b; }";

    // Parse
    let parser = CParser::new().unwrap();
    let ast = parser.parse(c_code).unwrap();

    // Lower to HIR
    let hir = lower_to_hir(&ast).unwrap();

    // Infer ownership
    let hir_with_ownership = infer_ownership(&hir).unwrap();

    // Generate Rust
    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate(&hir_with_ownership);

    // Verify
    assert!(rust_code.contains("fn add"));
    assert!(rust_code.contains("i32"));
    assert!(compile_rust(&rust_code).is_ok());
    assert!(clippy_check(&rust_code).is_ok());
}
```

## Performance Benchmarks

```rust,ignore
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_generate_simple_function(c: &mut Criterion) {
    let hir_func = create_add_function();
    let codegen = CodeGenerator::new();

    c.bench_function("generate_simple_function", |b| {
        b.iter(|| {
            codegen.generate_function(black_box(&hir_func))
        });
    });
}

fn benchmark_generate_complex_function(c: &mut Criterion) {
    let hir_func = create_complex_function_with_100_statements();
    let codegen = CodeGenerator::new();

    c.bench_function("generate_complex_function", |b| {
        b.iter(|| {
            codegen.generate_function(black_box(&hir_func))
        });
    });
}

criterion_group!(benches,
    benchmark_generate_simple_function,
    benchmark_generate_complex_function
);
criterion_main!(benches);
```

## Coverage Requirements

Codegen tests must achieve ≥80% coverage:

```bash
cargo llvm-cov --package decy-codegen
```

Expected output:

```
decy-codegen/src/lib.rs          94.1% coverage ✅
decy-codegen/src/types.rs        92.3% coverage ✅
decy-codegen/src/ownership.rs    93.7% coverage ✅
decy-codegen/src/formatting.rs   89.2% coverage ✅
───────────────────────────────────────────────
Overall                          92.3% coverage ✅
```

## Summary

Code generation verification ensures:

✅ **Correct type mapping**: C types → Rust types
✅ **Ownership patterns**: malloc → Box, parameters → &T
✅ **Lifetime annotations**: Automatic lifetime inference
✅ **Compiles**: All generated code is valid Rust
✅ **Clippy clean**: Zero warnings on generated code
✅ **Formatted**: Proper indentation and style
✅ **Property compliance**: Invariants hold for all inputs
✅ **High coverage**: ≥80% test coverage
✅ **Mutation resistance**: ≥90% mutation kill rate

## Next Steps

- [Simple Function Transpilation](../verification/simple-function.md) - End-to-end examples
- [Test Coverage](../metrics/coverage.md) - Coverage reports
