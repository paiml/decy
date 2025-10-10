# Lifetime Annotations

This chapter demonstrates how DECY automatically infers and generates Rust lifetime annotations from C code.

## Why Lifetimes?

In C, pointer relationships are implicit. The compiler doesn't track how long pointers remain valid.

In Rust, lifetimes make these relationships **explicit** and **checked at compile time**, preventing:
- Use-after-free bugs
- Dangling pointers
- Double-free errors

## Pattern 1: Single Lifetime

When all references in a function have the same lifetime.

### C Code

```c
int* get_first(int* a, int* b) {
    return a;
}
```

### Transpiled Rust

```rust
fn get_first<'a>(a: &'a i32, b: &'a i32) -> &'a i32 {
    a
}
```

The lifetime `'a` says: "The returned reference lives as long as the input references."

### Verification

```rust,ignore
#[test]
fn test_single_lifetime_annotation() {
    let c_code = r#"
        int* get_first(int* a, int* b) {
            return a;
        }
    "#;

    let rust_code = transpile(c_code).unwrap();

    // Verify lifetime annotation
    assert!(rust_code.contains("<'a>"));
    assert!(rust_code.contains("&'a i32"));

    // Verify all parameters use same lifetime
    assert_eq!(rust_code.matches("&'a").count(), 3);  // a, b, return

    // Verify compiles
    assert!(compile_rust(&rust_code).is_ok());
}
```

## Pattern 2: Return Lifetime Tied to Input

Most common pattern: return value lifetime matches an input parameter.

### C Code: Return one of two inputs

```c
int* choose(int* a, int* b, int condition) {
    if (condition) {
        return a;
    } else {
        return b;
    }
}
```

### Transpiled Rust

```rust
fn choose<'a>(a: &'a i32, b: &'a i32, condition: i32) -> &'a i32 {
    if condition != 0 {
        a
    } else {
        b
    }
}
```

### Verification

```rust,ignore
#[test]
fn test_return_lifetime_from_inputs() {
    let c_code = r#"
        int* choose(int* a, int* b, int condition) {
            if (condition) {
                return a;
            } else {
                return b;
            }
        }
    "#;

    let rust_code = transpile(c_code).unwrap();

    // Verify lifetime parameter
    assert!(rust_code.contains("fn choose<'a>"));

    // Verify pointer parameters have lifetime
    assert!(rust_code.contains("a: &'a i32"));
    assert!(rust_code.contains("b: &'a i32"));

    // Verify return has lifetime
    assert!(rust_code.contains("-> &'a i32"));

    // Verify non-pointer parameter has no lifetime
    assert!(rust_code.contains("condition: i32"));

    // Verify compiles
    assert!(compile_rust(&rust_code).is_ok());
}
```

## Pattern 3: Static Lifetime

String literals and global data have `'static` lifetime.

### C Code

```c
const char* get_message() {
    return "Hello, World!";
}
```

### Transpiled Rust

```rust
fn get_message() -> &'static str {
    "Hello, World!"
}
```

### Verification

```rust,ignore
#[test]
fn test_static_lifetime_for_literals() {
    let c_code = r#"
        const char* get_message() {
            return "Hello, World!";
        }
    "#;

    let rust_code = transpile(c_code).unwrap();

    // Verify static lifetime
    assert!(rust_code.contains("&'static str"));

    // Verify no lifetime parameter needed
    assert!(!rust_code.contains("<'a>"));

    // Verify compiles
    assert!(compile_rust(&rust_code).is_ok());
}
```

## Pattern 4: Struct with Lifetimes

Structs that hold references need lifetime annotations.

### C Code

```c
struct Holder {
    int* value;
};

struct Holder* create_holder(int* v) {
    struct Holder* h = malloc(sizeof(struct Holder));
    h->value = v;
    return h;
}
```

### Transpiled Rust

```rust
struct Holder<'a> {
    value: &'a i32,
}

fn create_holder<'a>(v: &'a i32) -> Box<Holder<'a>> {
    Box::new(Holder { value: v })
}
```

### Verification

```rust,ignore
#[test]
fn test_struct_with_lifetime() {
    let c_code = r#"
        struct Holder {
            int* value;
        };

        struct Holder* create_holder(int* v) {
            struct Holder* h = malloc(sizeof(struct Holder));
            h->value = v;
            return h;
        }
    "#;

    let rust_code = transpile(c_code).unwrap();

    // Verify struct has lifetime parameter
    assert!(rust_code.contains("struct Holder<'a>"));

    // Verify field has lifetime
    assert!(rust_code.contains("value: &'a i32"));

    // Verify function propagates lifetime
    assert!(rust_code.contains("fn create_holder<'a>"));
    assert!(rust_code.contains("-> Box<Holder<'a>>"));

    // Verify compiles
    assert!(compile_rust(&rust_code).is_ok());
}
```

## Pattern 5: No Lifetime Needed

Functions that don't return references don't need lifetime annotations.

### C Code

```c
void process(int* p) {
    *p = *p + 1;
}
```

### Transpiled Rust

```rust
fn process(p: &mut i32) {
    *p = *p + 1;
}
```

No lifetime annotation needed - the function doesn't return a reference!

### Verification

```rust,ignore
#[test]
fn test_no_lifetime_when_not_needed() {
    let c_code = "void process(int* p) { *p = *p + 1; }";

    let rust_code = transpile(c_code).unwrap();

    // Verify no lifetime parameter
    assert!(!rust_code.contains("<'a>"));

    // Verify has reference parameter
    assert!(rust_code.contains("&mut i32"));

    // Verify compiles
    assert!(compile_rust(&rust_code).is_ok());
}
```

## Property Tests for Lifetimes

### Property: Functions Returning References Have Lifetimes

```rust,ignore
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_returning_ref_has_lifetime(func_name in "[a-z]+") {
        let c_code = format!(
            "int* {}(int* p) {{ return p; }}",
            func_name
        );

        let rust_code = transpile(&c_code).unwrap();

        // Property: Returning reference → has lifetime
        prop_assert!(rust_code.contains("<'a>"));
        prop_assert!(rust_code.contains("&'a"));
    }
}
```

### Property: Non-Returning Functions Don't Need Lifetimes

```rust,ignore
proptest! {
    #[test]
    fn prop_void_function_no_lifetime(func_name in "[a-z]+") {
        let c_code = format!(
            "void {}(int* p) {{ *p = 0; }}",
            func_name
        );

        let rust_code = transpile(&c_code).unwrap();

        // Property: Not returning reference → no lifetime
        prop_assert!(!rust_code.contains("<'a>"));
    }
}
```

### Property: Lifetime Annotations Are Valid

```rust,ignore
proptest! {
    #[test]
    fn prop_lifetime_annotations_valid(c_code in valid_c_function_with_pointers()) {
        let rust_code = transpile(&c_code).unwrap();

        // Property: Generated lifetime annotations must be valid Rust
        prop_assert!(compile_rust(&rust_code).is_ok());
    }
}
```

## Complex Example: Multiple References

### C Code

```c
int* find_max(int* arr, int len) {
    if (len == 0) return NULL;

    int* max = &arr[0];
    for (int i = 1; i < len; i++) {
        if (arr[i] > *max) {
            max = &arr[i];
        }
    }
    return max;
}
```

### Transpiled Rust

```rust
fn find_max(arr: &[i32]) -> Option<&i32> {
    if arr.is_empty() {
        return None;
    }

    let mut max = &arr[0];
    for val in &arr[1..] {
        if val > max {
            max = val;
        }
    }
    Some(max)
}
```

Note: Lifetime is elided (implicit) here - Rust's lifetime elision rules apply!

### Verification

```rust,ignore
#[test]
fn test_lifetime_elision() {
    let c_code = r#"
        int* find_max(int* arr, int len) {
            if (len == 0) return NULL;
            int* max = &arr[0];
            for (int i = 1; i < len; i++) {
                if (arr[i] > *max) {
                    max = &arr[i];
                }
            }
            return max;
        }
    "#;

    let rust_code = transpile(c_code).unwrap();

    // Lifetime is elided - not explicitly written
    // But it's still there implicitly!
    assert!(rust_code.contains("-> Option<&i32>"));

    // Verify compiles (borrow checker validates lifetimes)
    assert!(compile_rust(&rust_code).is_ok());
}
```

## Lifetime Elision Rules

Rust can often infer lifetimes automatically. DECY follows these rules:

### Rule 1: Each input reference gets its own lifetime

```rust
fn foo(x: &i32, y: &i32)
// Becomes: fn foo<'a, 'b>(x: &'a i32, y: &'b i32)
```

### Rule 2: If there's exactly one input lifetime, output gets that lifetime

```rust
fn foo(x: &i32) -> &i32
// Becomes: fn foo<'a>(x: &'a i32) -> &'a i32
```

### Rule 3: If there's a `&self` parameter, output gets its lifetime

```rust
fn get_value(&self) -> &i32
// Becomes: fn get_value<'a>(&'a self) -> &'a i32
```

DECY only adds explicit lifetimes when elision rules don't apply!

### Verification

```rust,ignore
#[test]
fn test_lifetime_elision_rules() {
    let test_cases = vec![
        // Rule 2: Single input → elide lifetime
        (
            "int* identity(int* p) { return p; }",
            false,  // Should NOT contain <'a>
        ),
        // Multiple inputs → explicit lifetime needed
        (
            "int* choose(int* a, int* b) { return a; }",
            true,   // Should contain <'a>
        ),
    ];

    for (c_code, should_have_explicit_lifetime) in test_cases {
        let rust_code = transpile(c_code).unwrap();
        let has_explicit = rust_code.contains("<'a>");

        assert_eq!(
            has_explicit, should_have_explicit_lifetime,
            "Lifetime elision rule failed for: {}", c_code
        );

        // All should compile regardless
        assert!(compile_rust(&rust_code).is_ok());
    }
}
```

## Common Lifetime Errors (Prevented)

DECY's lifetime analysis prevents these common errors:

### Error 1: Dangling Pointer

```c
// ❌ C allows this (dangling pointer!)
int* dangling() {
    int x = 5;
    return &x;  // BAD: x destroyed when function returns
}
```

DECY detects this and either:
- Refuses to transpile (safety error)
- Converts to owned type (Box<i32>)

### Error 2: Use After Free

```c
// ❌ C allows this (use after free!)
int* bad() {
    int* p = malloc(sizeof(int));
    free(p);
    return p;  // BAD: p is freed
}
```

DECY's dataflow analysis catches this!

### Verification

```rust,ignore
#[test]
fn test_reject_dangling_pointer() {
    let c_code = r#"
        int* dangling() {
            int x = 5;
            return &x;
        }
    "#;

    let result = transpile(c_code);

    // Should either error or convert to owned
    if let Ok(rust_code) = result {
        // If transpiled, must use Box (owned)
        assert!(rust_code.contains("Box<i32>"));
    } else {
        // Or error is acceptable
        assert!(result.is_err());
    }
}
```

## Summary

Lifetime annotations ensure:

✅ **No dangling pointers**: Compile-time guarantee
✅ **No use-after-free**: Lifetime tracking prevents this
✅ **Explicit relationships**: Clear pointer dependencies
✅ **Automatic inference**: DECY adds lifetimes automatically
✅ **Elision when possible**: Follows Rust idioms
✅ **Compile-time checked**: Borrow checker validates

All generated lifetime annotations:
- Compile without errors
- Pass borrow checker
- Follow Rust lifetime elision rules
- Are minimal (no unnecessary annotations)

## Next Steps

- [Box Transformations](./box-transform.md) - Deep dive into malloc → Box
- [Parser Verification](../components/parser.md) - How C code is parsed
