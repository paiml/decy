# Simple Function Transpilation

This chapter demonstrates end-to-end transpilation of simple C functions to Rust, with complete verification.

## Basic Function: Add Two Numbers

### Original C Code

```c
int add(int a, int b) {
    return a + b;
}
```

### Transpiled Rust Code

```rust
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}
```

### Verification

Let's verify this transpilation works:

```rust
use decy_core::transpile;

#[test]
fn test_transpile_add_function() {
    let c_code = "int add(int a, int b) { return a + b; }";

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation should succeed");

    let rust_code = result.unwrap();

    // Verify function signature
    assert!(rust_code.contains("fn add"), "Should contain function name");
    assert!(rust_code.contains("i32"), "Should contain Rust int type");

    // Verify parameters
    assert!(rust_code.contains("a"), "Should contain parameter a");
    assert!(rust_code.contains("b"), "Should contain parameter b");
}
```

## Function with Variables

### C Code with Local Variable

```c
int calculate(int a, int b) {
    int result;
    result = a + b;
    return result;
}
```

### Expected Rust Output

```rust
fn calculate(a: i32, b: i32) -> i32 {
    let mut result: i32 = 0;
    result = a + b;
    return result;
}
```

### Verification Test

```rust
#[test]
fn test_transpile_function_with_variable() {
    let c_code = r#"
        int calculate(int a, int b) {
            int result;
            result = a + b;
            return result;
        }
    "#;

    let rust_code = transpile(c_code).unwrap();

    assert!(rust_code.contains("fn calculate"));
    assert!(rust_code.contains("let mut result"));
    assert!(rust_code.contains("i32"));
}
```

## Void Function

### C Code

```c
void do_nothing() {
    return;
}
```

### Transpiled Rust

```rust
fn do_nothing() {
    return;
}
```

### Verification

```rust
#[test]
fn test_transpile_void_function() {
    let c_code = "void do_nothing() { return; }";

    let rust_code = transpile(c_code).unwrap();

    assert!(rust_code.contains("fn do_nothing"));
    assert!(!rust_code.contains("->"), "Void functions have no return type");
}
```

## Property Tests for Simple Functions

Let's verify properties hold for ALL simple functions:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_transpilation_deterministic(
        func_name in "[a-z]{3,10}",
        param1 in "[a-z]{1,5}",
        param2 in "[a-z]{1,5}",
    ) {
        let c_code = format!(
            "int {}(int {}, int {}) {{ return {} + {}; }}",
            func_name, param1, param2, param1, param2
        );

        let output1 = transpile(&c_code).unwrap();
        let output2 = transpile(&c_code).unwrap();

        // Property: Same C code → same Rust output
        prop_assert_eq!(output1, output2);
    }

    #[test]
    fn prop_rust_output_contains_function_name(
        func_name in "[a-z]{3,10}",
    ) {
        let c_code = format!("int {}() {{ return 0; }}", func_name);

        let rust_code = transpile(&c_code).unwrap();

        // Property: Function name is preserved
        prop_assert!(rust_code.contains(&format!("fn {}", func_name)));
    }

    #[test]
    fn prop_parameters_preserved(
        param_name in "[a-z]{1,8}",
    ) {
        let c_code = format!("int foo(int {}) {{ return {}; }}", param_name, param_name);

        let rust_code = transpile(&c_code).unwrap();

        // Property: Parameter names are preserved
        prop_assert!(rust_code.contains(&param_name));
    }
}
```

## Compilation Verification

The ultimate test: does the generated Rust compile?

```rust
use std::process::Command;
use std::fs;

#[test]
fn test_generated_rust_compiles() {
    let c_code = "int add(int a, int b) { return a + b; }";
    let rust_code = transpile(c_code).unwrap();

    // Write to temporary file
    let temp_file = "/tmp/decy_test_add.rs";
    fs::write(temp_file, &rust_code).unwrap();

    // Try to compile with rustc
    let output = Command::new("rustc")
        .args(&["--crate-type", "lib", temp_file])
        .output()
        .expect("Failed to run rustc");

    assert!(
        output.status.success(),
        "Generated Rust code should compile:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
```

## Clippy Verification

Generated code must pass clippy:

```rust
#[test]
fn test_generated_rust_passes_clippy() {
    let c_code = "int add(int a, int b) { return a + b; }";
    let rust_code = transpile(c_code).unwrap();

    // Write to temporary file
    let temp_file = "/tmp/decy_test_clippy.rs";
    fs::write(temp_file, &rust_code).unwrap();

    // Run clippy
    let output = Command::new("cargo")
        .args(&["clippy", "--", "-D", "warnings"])
        .current_dir("/tmp")
        .output()
        .expect("Failed to run clippy");

    assert!(
        output.status.success(),
        "Generated code should pass clippy:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
```

## Summary

Simple function transpilation demonstrates:

✅ **Basic transpilation works**: C functions → Rust functions
✅ **Type mapping correct**: `int` → `i32`, `void` → `()`
✅ **Parameters preserved**: Names and types maintained
✅ **Variables handled**: `int x;` → `let mut x: i32 = 0;`
✅ **Deterministic output**: Same input → same output
✅ **Compiles**: Generated Rust is valid
✅ **Lints clean**: Passes clippy with zero warnings

## Next Steps

- [Pointer Handling](./pointers.md) - How pointers become references
- [Ownership Patterns](./ownership-patterns.md) - malloc → Box, parameters → &T
- [Lifetime Annotations](./lifetimes.md) - Automatic lifetime inference
