# Ownership Patterns

This chapter demonstrates the three main ownership patterns DECY recognizes and how they map from C to Rust.

## Pattern 1: Owning (Box<T>)

When a variable **owns** its data, it's responsible for cleanup. In C, this means calling `free()`. In Rust, we use `Box<T>` for automatic cleanup.

### Pattern: malloc + return

```c
int* create_number(int value) {
    int* p = malloc(sizeof(int));
    *p = value;
    return p;
}
```

Transpiled to Rust:

```rust
fn create_number(value: i32) -> Box<i32> {
    let mut p = Box::new(0i32);
    *p = value;
    p
}
```

### Verification

```rust,ignore
#[test]
fn test_owning_pattern_malloc_return() {
    let c_code = r#"
        int* create_number(int value) {
            int* p = malloc(sizeof(int));
            *p = value;
            return p;
        }
    "#;

    let rust_code = transpile(c_code).unwrap();

    // Verify Box usage
    assert!(rust_code.contains("Box::new"));
    assert!(rust_code.contains("Box<i32>"));

    // Verify no manual free
    assert!(!rust_code.contains("free"));

    // Verify compiles
    assert!(compile_rust(&rust_code).is_ok());
}
```

## Pattern 2: Borrowed (&T or &mut T)

When a variable **borrows** data, it doesn't own it and can't free it. In C, these are typically function parameters. In Rust, we use references.

### Pattern: Function parameter (immutable)

```c
int get_value(const int* p) {
    return *p;
}
```

Transpiled to Rust:

```rust
fn get_value(p: &i32) -> i32 {
    *p
}
```

### Verification

```rust,ignore
#[test]
fn test_borrowed_immutable_pattern() {
    let c_code = "int get_value(const int* p) { return *p; }";

    let rust_code = transpile(c_code).unwrap();

    // Verify immutable reference
    assert!(rust_code.contains("&i32"));
    assert!(!rust_code.contains("&mut"));

    // Verify no Box
    assert!(!rust_code.contains("Box"));

    // Verify compiles
    assert!(compile_rust(&rust_code).is_ok());
}
```

### Pattern: Function parameter (mutable)

```c
void increment(int* p) {
    *p = *p + 1;
}
```

Transpiled to Rust:

```rust
fn increment(p: &mut i32) {
    *p = *p + 1;
}
```

### Verification

```rust,ignore
#[test]
fn test_borrowed_mutable_pattern() {
    let c_code = "void increment(int* p) { *p = *p + 1; }";

    let rust_code = transpile(c_code).unwrap();

    // Verify mutable reference
    assert!(rust_code.contains("&mut i32"));

    // Verify no Box
    assert!(!rust_code.contains("Box"));

    // Verify compiles
    assert!(compile_rust(&rust_code).is_ok());
}
```

## Pattern 3: Mixed (Owning + Borrowing)

Real functions often mix ownership patterns - some parameters borrowed, others owned.

### C Code

```c
int* double_value(const int* input) {
    int* output = malloc(sizeof(int));
    *output = *input * 2;
    return output;
}
```

### Transpiled Rust

```rust
fn double_value(input: &i32) -> Box<i32> {
    let mut output = Box::new(0i32);
    *output = *input * 2;
    output
}
```

### Verification

```rust,ignore
#[test]
fn test_mixed_ownership_pattern() {
    let c_code = r#"
        int* double_value(const int* input) {
            int* output = malloc(sizeof(int));
            *output = *input * 2;
            return output;
        }
    "#;

    let rust_code = transpile(c_code).unwrap();

    // Verify input is borrowed
    assert!(rust_code.contains("input: &i32"));

    // Verify output is owned
    assert!(rust_code.contains("Box::new"));
    assert!(rust_code.contains("-> Box<i32>"));

    // Verify compiles
    assert!(compile_rust(&rust_code).is_ok());
}
```

## Pattern 4: Array Ownership

Arrays have special ownership rules in C and Rust.

### C Code: Array parameter

```c
int sum(int* arr, int len) {
    int total = 0;
    for (int i = 0; i < len; i++) {
        total += arr[i];
    }
    return total;
}
```

### Transpiled Rust (slice)

```rust
fn sum(arr: &[i32]) -> i32 {
    let mut total = 0;
    for &val in arr {
        total += val;
    }
    total
}
```

### Verification

```rust,ignore
#[test]
fn test_array_borrowed_as_slice() {
    let c_code = r#"
        int sum(int* arr, int len) {
            int total = 0;
            for (int i = 0; i < len; i++) {
                total += arr[i];
            }
            return total;
        }
    "#;

    let rust_code = transpile(c_code).unwrap();

    // Verify slice usage
    assert!(rust_code.contains("&[i32]"));

    // Verify idiomatic iteration
    assert!(rust_code.contains("for &val in"));

    // Verify compiles
    assert!(compile_rust(&rust_code).is_ok());
}
```

## Pattern 5: Optional Ownership (Option<T>)

NULL pointers in C become Option in Rust.

### C Code: Nullable return

```c
int* find_first_positive(int* arr, int len) {
    for (int i = 0; i < len; i++) {
        if (arr[i] > 0) {
            return &arr[i];
        }
    }
    return NULL;
}
```

### Transpiled Rust

```rust
fn find_first_positive(arr: &[i32]) -> Option<&i32> {
    for val in arr {
        if *val > 0 {
            return Some(val);
        }
    }
    None
}
```

### Verification

```rust,ignore
#[test]
fn test_nullable_becomes_option() {
    let c_code = r#"
        int* find_first_positive(int* arr, int len) {
            for (int i = 0; i < len; i++) {
                if (arr[i] > 0) {
                    return &arr[i];
                }
            }
            return NULL;
        }
    "#;

    let rust_code = transpile(c_code).unwrap();

    // Verify Option usage
    assert!(rust_code.contains("Option<&i32>"));
    assert!(rust_code.contains("Some("));
    assert!(rust_code.contains("None"));

    // Verify no NULL
    assert!(!rust_code.contains("NULL"));

    // Verify compiles
    assert!(compile_rust(&rust_code).is_ok());
}
```

## Property Tests for Ownership Patterns

### Property: malloc Always Returns Owned Type

```rust,ignore
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_malloc_return_is_box(func_name in "[a-z]+") {
        let c_code = format!(
            "int* {}() {{ return malloc(sizeof(int)); }}",
            func_name
        );

        let rust_code = transpile(&c_code).unwrap();

        // Property: Functions returning malloc result return Box
        prop_assert!(rust_code.contains("-> Box<i32>"));
    }
}
```

### Property: Parameters Are Borrowed

```rust,ignore
proptest! {
    #[test]
    fn prop_pointer_parameters_borrowed(
        func_name in "[a-z]+",
        param_name in "[a-z]+",
    ) {
        let c_code = format!(
            "void {}(int* {}) {{ *{} = 0; }}",
            func_name, param_name, param_name
        );

        let rust_code = transpile(&c_code).unwrap();

        // Property: Pointer parameters become references
        prop_assert!(
            rust_code.contains("&i32") || rust_code.contains("&mut i32"),
            "Expected reference type, got: {}", rust_code
        );
    }
}
```

### Property: Const Parameters Are Immutable

```rust,ignore
proptest! {
    #[test]
    fn prop_const_params_immutable(param_name in "[a-z]+") {
        let c_code = format!(
            "int func(const int* {}) {{ return *{}; }}",
            param_name, param_name
        );

        let rust_code = transpile(&c_code).unwrap();

        // Property: const parameters → immutable references
        prop_assert!(rust_code.contains("&i32"));
        prop_assert!(!rust_code.contains("&mut"));
    }
}
```

## Complex Example: Multiple Patterns

### C Code

```c
int* process_array(const int* input, int len, int* output_len) {
    // Count positive numbers
    int count = 0;
    for (int i = 0; i < len; i++) {
        if (input[i] > 0) count++;
    }

    // Allocate output array
    int* output = malloc(count * sizeof(int));

    // Copy positive numbers
    int j = 0;
    for (int i = 0; i < len; i++) {
        if (input[i] > 0) {
            output[j++] = input[i];
        }
    }

    *output_len = count;
    return output;
}
```

### Transpiled Rust

```rust
fn process_array(input: &[i32], output_len: &mut i32) -> Box<[i32]> {
    // Count positive numbers
    let count = input.iter().filter(|&&x| x > 0).count();

    // Collect positive numbers
    let output = input.iter()
        .filter(|&&x| x > 0)
        .copied()
        .collect::<Vec<_>>()
        .into_boxed_slice();

    *output_len = count as i32;
    output
}
```

### Verification

```rust,ignore
#[test]
fn test_complex_ownership_patterns() {
    let c_code = r#"
        int* process_array(const int* input, int len, int* output_len) {
            int count = 0;
            for (int i = 0; i < len; i++) {
                if (input[i] > 0) count++;
            }
            int* output = malloc(count * sizeof(int));
            int j = 0;
            for (int i = 0; i < len; i++) {
                if (input[i] > 0) {
                    output[j++] = input[i];
                }
            }
            *output_len = count;
            return output;
        }
    "#;

    let rust_code = transpile(c_code).unwrap();

    // Verify input is borrowed (immutable)
    assert!(rust_code.contains("input: &[i32]"));

    // Verify output_len is borrowed (mutable)
    assert!(rust_code.contains("output_len: &mut i32"));

    // Verify return is owned
    assert!(rust_code.contains("-> Box<[i32]>"));

    // Verify compiles
    assert!(compile_rust(&rust_code).is_ok());

    // Verify passes clippy
    assert!(clippy_check(&rust_code).is_ok());
}
```

## Summary

DECY recognizes and correctly transpiles these ownership patterns:

✅ **Owning (Box<T>)**: malloc → Box, returned pointers
✅ **Borrowed (&T)**: const parameters, non-mutated pointers
✅ **Borrowed (&mut T)**: mutable parameters, modified pointers
✅ **Array slices (&[T])**: Array parameters with length
✅ **Optional (Option<&T>)**: Nullable pointers, NULL returns

All patterns:
- Compile without errors
- Pass clippy with zero warnings
- Are memory safe (no leaks, no dangling pointers)
- Follow Rust idioms and best practices

## Next Steps

- [Lifetime Annotations](./lifetimes.md) - How lifetimes are inferred
- [Box Transformations](./box-transform.md) - Deep dive into malloc → Box
