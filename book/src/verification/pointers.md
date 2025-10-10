# Pointer Handling

This chapter demonstrates how DECY transpiles C pointers into safe Rust references, raw pointers, and Box types.

## Strategy

DECY uses **dataflow analysis** and **ownership inference** to determine the safest Rust representation:

| C Pattern | Rust Pattern | Example |
|-----------|--------------|---------|
| `malloc` + `free` | `Box<T>` | Owned heap allocation |
| Function parameter | `&T` or `&mut T` | Borrowed reference |
| Pointer arithmetic | `*mut T` | Raw pointer (unsafe) |
| NULL checks | `Option<&T>` | Nullable reference |

## malloc → Box<T>

### C Code

```c
int* create_int() {
    int* p = malloc(sizeof(int));
    *p = 42;
    return p;
}
```

### Transpiled Rust

```rust
fn create_int() -> Box<i32> {
    let mut p = Box::new(0i32);
    *p = 42;
    p
}
```

### Verification

```rust,ignore
use decy_core::transpile;

#[test]
fn test_malloc_becomes_box() {
    let c_code = r#"
        int* create_int() {
            int* p = malloc(sizeof(int));
            *p = 42;
            return p;
        }
    "#;

    let rust_code = transpile(c_code).unwrap();

    // Verify Box usage
    assert!(rust_code.contains("Box::new"));
    assert!(rust_code.contains("Box<i32>"));

    // Verify no malloc
    assert!(!rust_code.contains("malloc"));

    // Verify compiles
    assert!(compile_rust(&rust_code).is_ok());
}
```

## Function Parameters → References

### C Code

```c
void increment(int* p) {
    *p = *p + 1;
}
```

### Transpiled Rust

```rust
fn increment(p: &mut i32) {
    *p = *p + 1;
}
```

### Verification

```rust,ignore
#[test]
fn test_parameter_pointer_becomes_reference() {
    let c_code = "void increment(int* p) { *p = *p + 1; }";

    let rust_code = transpile(c_code).unwrap();

    // Verify reference usage
    assert!(rust_code.contains("&mut i32"));

    // Verify no raw pointers
    assert!(!rust_code.contains("*mut"));
    assert!(!rust_code.contains("*const"));

    // Verify compiles
    assert!(compile_rust(&rust_code).is_ok());
}
```

## Const Pointers → Immutable References

### C Code

```c
int sum(const int* arr, int len) {
    int total = 0;
    for (int i = 0; i < len; i++) {
        total += arr[i];
    }
    return total;
}
```

### Transpiled Rust

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
fn test_const_pointer_becomes_slice() {
    let c_code = r#"
        int sum(const int* arr, int len) {
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

    // Verify no raw pointers
    assert!(!rust_code.contains("*const"));

    // Verify idiomatic iteration
    assert!(rust_code.contains("for &val in"));

    // Verify compiles
    assert!(compile_rust(&rust_code).is_ok());
}
```

## Pointer Arithmetic → Raw Pointers

### C Code

```c
void process_array(int* arr, int len) {
    int* end = arr + len;
    for (int* p = arr; p < end; p++) {
        *p = *p * 2;
    }
}
```

### Transpiled Rust

```rust
fn process_array(arr: &mut [i32]) {
    for val in arr.iter_mut() {
        *val = *val * 2;
    }
}
```

Note: DECY converts pointer arithmetic to safe iterator usage when possible!

### Verification

```rust,ignore
#[test]
fn test_pointer_arithmetic_becomes_iterator() {
    let c_code = r#"
        void process_array(int* arr, int len) {
            int* end = arr + len;
            for (int* p = arr; p < end; p++) {
                *p = *p * 2;
            }
        }
    "#;

    let rust_code = transpile(c_code).unwrap();

    // Verify slice usage
    assert!(rust_code.contains("&mut [i32]"));

    // Verify iterator usage
    assert!(rust_code.contains("iter_mut()"));

    // Verify no unsafe pointer arithmetic
    assert!(!rust_code.contains("unsafe"));
    assert!(!rust_code.contains(".add("));
    assert!(!rust_code.contains(".offset("));

    // Verify compiles
    assert!(compile_rust(&rust_code).is_ok());
}
```

## NULL Checks → Option<T>

### C Code

```c
int* find_value(int* arr, int len, int target) {
    for (int i = 0; i < len; i++) {
        if (arr[i] == target) {
            return &arr[i];
        }
    }
    return NULL;
}
```

### Transpiled Rust

```rust
fn find_value(arr: &[i32], target: i32) -> Option<&i32> {
    for val in arr {
        if *val == target {
            return Some(val);
        }
    }
    None
}
```

### Verification

```rust,ignore
#[test]
fn test_null_becomes_option() {
    let c_code = r#"
        int* find_value(int* arr, int len, int target) {
            for (int i = 0; i < len; i++) {
                if (arr[i] == target) {
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
    assert!(!rust_code.contains("null"));

    // Verify compiles
    assert!(compile_rust(&rust_code).is_ok());
}
```

## Pointer to Pointer → Nested References

### C Code

```c
void allocate_array(int** ptr, int size) {
    *ptr = malloc(size * sizeof(int));
}
```

### Transpiled Rust

```rust
fn allocate_array(ptr: &mut Box<[i32]>, size: usize) {
    *ptr = vec![0; size].into_boxed_slice();
}
```

### Verification

```rust,ignore
#[test]
fn test_pointer_to_pointer_becomes_mut_ref_to_box() {
    let c_code = r#"
        void allocate_array(int** ptr, int size) {
            *ptr = malloc(size * sizeof(int));
        }
    "#;

    let rust_code = transpile(c_code).unwrap();

    // Verify nested reference
    assert!(rust_code.contains("&mut Box<[i32]>"));

    // Verify no raw pointers
    assert!(!rust_code.contains("**"));
    assert!(!rust_code.contains("*mut *mut"));

    // Verify compiles
    assert!(compile_rust(&rust_code).is_ok());
}
```

## Property Tests for Pointers

### Property: malloc Always Becomes Box

```rust,ignore
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_malloc_always_box(size in 1..1024usize) {
        let c_code = format!(
            "int* p = malloc({} * sizeof(int));",
            size
        );

        let rust_code = transpile(&c_code).unwrap();

        // Property: malloc → Box
        prop_assert!(rust_code.contains("Box::new"));
    }
}
```

### Property: Function Parameters Become References

```rust,ignore
proptest! {
    #[test]
    fn prop_parameters_become_references(
        func_name in "[a-z]{3,10}",
        param_name in "[a-z]{1,5}",
    ) {
        let c_code = format!(
            "void {}(int* {}) {{ *{} = 0; }}",
            func_name, param_name, param_name
        );

        let rust_code = transpile(&c_code).unwrap();

        // Property: parameter pointers → references
        prop_assert!(
            rust_code.contains("&mut i32") || rust_code.contains("&i32"),
            "Parameter pointers should become references"
        );
    }
}
```

### Property: No Unsafe for Safe Pointers

```rust,ignore
proptest! {
    #[test]
    fn prop_no_unsafe_for_safe_pointers(c_code in safe_pointer_code()) {
        let rust_code = transpile(&c_code).unwrap();

        // Property: Safe pointer usage → no unsafe blocks
        prop_assert!(
            !rust_code.contains("unsafe"),
            "Safe pointer code should not generate unsafe blocks"
        );
    }
}
```

## Compilation Verification

Every pointer transpilation must compile:

```rust,ignore
#[test]
fn test_pointer_examples_compile() {
    let examples = vec![
        // malloc → Box
        "int* p = malloc(sizeof(int));",

        // Parameter → reference
        "void func(int* p) { *p = 0; }",

        // Const pointer → immutable reference
        "int func(const int* p) { return *p; }",

        // Array → slice
        "void func(int* arr, int len) {}",

        // NULL → Option
        "int* func() { return NULL; }",
    ];

    for c_code in examples {
        let rust_code = transpile(c_code)
            .unwrap_or_else(|e| panic!("Failed to transpile: {}\nError: {}", c_code, e));

        compile_rust(&rust_code)
            .unwrap_or_else(|e| panic!("Failed to compile:\n{}\nError: {}", rust_code, e));
    }
}
```

## Safety Analysis

Verify that pointer transpilation maintains memory safety:

```rust,ignore
#[test]
fn test_pointer_safety() {
    let test_cases = vec![
        // malloc without free → automatic cleanup
        (
            "int* p = malloc(sizeof(int)); return p;",
            "Box::new",  // Box auto-drops
        ),

        // Dangling pointer → prevented
        (
            "int* func() { int x = 5; return &x; }",
            "error",  // Should detect this is unsafe
        ),

        // Double free → prevented
        (
            "free(p); free(p);",
            "error",  // Should detect double free
        ),

        // Use after free → prevented
        (
            "free(p); *p = 5;",
            "error",  // Should detect use after free
        ),
    ];

    for (c_code, expected) in test_cases {
        let result = transpile(c_code);

        if expected == "error" {
            assert!(result.is_err(), "Should detect unsafe pattern: {}", c_code);
        } else {
            let rust_code = result.unwrap();
            assert!(rust_code.contains(expected));
            assert!(compile_rust(&rust_code).is_ok());
        }
    }
}
```

## Performance

Verify that pointer transpilation doesn't introduce overhead:

```rust,ignore
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_pointer_transpilation(c: &mut Criterion) {
    let c_code = r#"
        int* create_array(int size) {
            int* arr = malloc(size * sizeof(int));
            for (int i = 0; i < size; i++) {
                arr[i] = i;
            }
            return arr;
        }
    "#;

    c.bench_function("transpile_pointer_code", |b| {
        b.iter(|| {
            transpile(black_box(c_code)).unwrap()
        });
    });
}

criterion_group!(benches, benchmark_pointer_transpilation);
criterion_main!(benches);
```

## Summary

Pointer handling demonstrates:

✅ **malloc → Box**: Automatic memory management
✅ **Parameters → &T**: Safe borrowing
✅ **Const → &T**: Immutable references
✅ **Arrays → slices**: Safe array access
✅ **NULL → Option**: Null safety
✅ **Pointer arithmetic → iterators**: Safe iteration
✅ **No unsafe**: When possible, avoid unsafe code
✅ **Compiles**: All generated code is valid Rust
✅ **Memory safe**: No leaks, dangling pointers, or double frees

## Next Steps

- [Ownership Patterns](./ownership-patterns.md) - Deep dive into ownership inference
- [Lifetime Annotations](./lifetimes.md) - How lifetimes are determined
