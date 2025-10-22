//! End-to-end transpilation tests for function pointers (DECY-054 RED phase)
//!
//! Tests verify complete pipeline: parse → HIR → codegen for function pointers.
//! These tests should FAIL initially - function pointer variables not transpiled.
//!
//! References:
//! - K&R §5.11: Pointers to Functions
//! - ISO C99 §6.7.5.3: Function declarators

use decy_core::transpile;

#[test]
fn test_transpile_simple_function_pointer_declaration() {
    // C: Global function pointer variable
    let c_code = "int (*callback)(int, int);";

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    // Should generate: let callback: fn(i32, i32) -> i32;
    // Or: static mut callback: Option<fn(i32, i32) -> i32> = None;
    assert!(
        rust_code.contains("callback"),
        "Should include callback variable"
    );
    assert!(
        rust_code.contains("fn(i32, i32) -> i32"),
        "Should generate Rust fn type"
    );
}

#[test]
fn test_transpile_function_pointer_void_return() {
    let c_code = "void (*handler)(int);";

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    // Should generate: fn(i32) (no return type for void)
    assert!(
        rust_code.contains("handler"),
        "Should include handler variable"
    );
    assert!(
        rust_code.contains("fn(i32)"),
        "Should generate fn type with no return"
    );
}

#[test]
fn test_transpile_function_pointer_no_params() {
    let c_code = "int (*get_value)(void);";

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    assert!(rust_code.contains("get_value"));
    assert!(
        rust_code.contains("fn() -> i32"),
        "Should generate fn() -> i32"
    );
}

#[test]
fn test_transpile_multiple_function_pointers() {
    let c_code = r#"
        int (*add)(int, int);
        int (*subtract)(int, int);
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    assert!(rust_code.contains("add"));
    assert!(rust_code.contains("subtract"));
    assert!(rust_code.matches("fn(i32, i32) -> i32").count() >= 2);
}

#[test]
fn test_transpile_function_pointer_with_function() {
    let c_code = r#"
        int add(int a, int b) {
            return a + b;
        }

        int (*operation)(int, int);
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    // Should have both the function and the function pointer
    assert!(rust_code.contains("fn add"));
    assert!(rust_code.contains("operation"));
    assert!(rust_code.contains("fn(i32, i32) -> i32"));
}

#[test]
fn test_transpile_function_pointer_typedef() {
    let c_code = "typedef int (*Callback)(int, int);";

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    // Should generate: type Callback = fn(i32, i32) -> i32;
    assert!(rust_code.contains("type Callback"));
    assert!(rust_code.contains("fn(i32, i32) -> i32"));
}

#[test]
fn test_transpile_struct_with_function_pointer_field() {
    let c_code = r#"
        struct Handler {
            void (*on_event)(int);
        };
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    assert!(rust_code.contains("struct Handler"));
    assert!(rust_code.contains("on_event"));
    assert!(rust_code.contains("fn(i32)"));
}

#[test]
fn test_transpile_callback_pattern() {
    // Common C pattern: callback typedef and usage
    let c_code = r#"
        typedef void (*EventCallback)(int);

        struct EventHandler {
            EventCallback callback;
        };
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    assert!(rust_code.contains("type EventCallback"));
    assert!(rust_code.contains("struct EventHandler"));
    assert!(rust_code.contains("callback"));
}

#[test]
fn test_transpile_function_pointer_with_pointer_params() {
    let c_code = "void (*process)(int*, char*);";

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    assert!(rust_code.contains("process"));
    // Should have fn type with pointer parameters
    assert!(rust_code.contains("fn("));
}
