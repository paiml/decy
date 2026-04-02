//! Integration test: Verify transpiled C++ code compiles with rustc (DECY-223)
//!
//! This is the most rigorous test — transpile C++ through the full pipeline
//! and verify the output actually compiles with `rustc --edition 2021`.
//!
//! **Dogfood principle**: If it doesn't compile, it doesn't count.

use decy_core::transpile;
use std::process::Command;

/// Helper: write Rust code to a temp file and compile with rustc.
/// Returns Ok(()) if compilation succeeds, Err(stderr) if it fails.
fn compile_rust_code(code: &str) -> Result<(), String> {
    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let rs_path = dir.path().join("test_output.rs");
    let rlib_path = dir.path().join("test_output.rlib");

    std::fs::write(&rs_path, code).map_err(|e| e.to_string())?;

    let output = Command::new("rustc")
        .args([
            "--edition",
            "2021",
            "--crate-type=lib",
            "-o",
            rlib_path.to_str().unwrap(),
            rs_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to run rustc: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Filter out warnings — only fail on errors
        let errors: Vec<&str> = stderr.lines().filter(|l| l.starts_with("error")).collect();
        if errors.is_empty() {
            Ok(()) // Warnings only
        } else {
            Err(errors.join("\n"))
        }
    }
}

/// Strip the info note, dummy extern "C" trigger functions, and ERRNO from output.
fn strip_noise(code: &str) -> String {
    let lines: Vec<&str> = code.lines().collect();
    let mut result = Vec::new();
    let mut skip_next_brace = false;

    for line in &lines {
        // Skip info notes
        if line.contains("ℹ Note:") || line.contains("rustc --crate-type") {
            continue;
        }
        // Skip dummy trigger functions and their closing braces
        if line.starts_with("fn __") {
            skip_next_brace = true;
            continue;
        }
        if skip_next_brace && line.trim() == "}" {
            skip_next_brace = false;
            continue;
        }
        skip_next_brace = false;
        // Skip static mut ERRNO (C compatibility shim)
        if line.starts_with("static mut ERRNO") {
            continue;
        }
        result.push(*line);
    }

    result.join("\n")
}

#[test]
fn test_cpp_class_compiles_with_rustc() {
    let cpp_code = r#"
extern "C" { void __mode(); }
class Counter {
public:
    int count;
    Counter(int initial) : count(initial) {}
    int get() { return count; }
    void increment() { count = count + 1; }
    ~Counter() {}
};
"#;

    let rust_code = transpile(cpp_code).expect("Transpilation failed");
    let clean = strip_noise(&rust_code);
    let result = compile_rust_code(&clean);
    assert!(
        result.is_ok(),
        "Transpiled class should compile:\n{}\nErrors: {:?}",
        clean,
        result.err()
    );
}

#[test]
fn test_cpp_namespace_compiles_with_rustc() {
    let cpp_code = r#"
extern "C" { void __mode(); }
namespace math {
    int square(int x) { return x * x; }
    struct Point { int x; int y; };
}
"#;

    let rust_code = transpile(cpp_code).expect("Transpilation failed");
    let clean = strip_noise(&rust_code);
    let result = compile_rust_code(&clean);
    assert!(
        result.is_ok(),
        "Transpiled namespace should compile:\n{}\nErrors: {:?}",
        clean,
        result.err()
    );
}

#[test]
fn test_cpp_operator_overload_compiles_with_rustc() {
    let cpp_code = r#"
extern "C" { void __mode(); }
class Vec2 {
public:
    int x;
    int y;
    Vec2(int a, int b) : x(a), y(b) {}
    Vec2 operator+(Vec2 other) { Vec2 r(0,0); return r; }
    bool operator==(Vec2 other) { return x == other.x; }
};
"#;

    let rust_code = transpile(cpp_code).expect("Transpilation failed");
    let clean = strip_noise(&rust_code);
    let result = compile_rust_code(&clean);
    assert!(
        result.is_ok(),
        "Transpiled operators should compile:\n{}\nErrors: {:?}",
        clean,
        result.err()
    );
}

#[test]
fn test_cpp_inheritance_compiles_with_rustc() {
    let cpp_code = r#"
extern "C" { void __mode(); }
class Shape {
public:
    int color;
    int get_color() { return color; }
};
class Circle : public Shape {
public:
    int radius;
    int area() { return 3 * radius * radius; }
};
"#;

    let rust_code = transpile(cpp_code).expect("Transpilation failed");
    let clean = strip_noise(&rust_code);
    let result = compile_rust_code(&clean);
    assert!(
        result.is_ok(),
        "Transpiled inheritance should compile:\n{}\nErrors: {:?}",
        clean,
        result.err()
    );
}

#[test]
fn test_cpp_full_program_compiles_with_rustc() {
    let cpp_code = r#"
extern "C" { void __mode(); }

class StringBuilder {
public:
    int pos;
    StringBuilder() : pos(0) {}
    void append(int c) {
        pos = pos + 1;
    }
    int size() { return pos; }
};

namespace collections {
    class Stack {
    public:
        int top;
        Stack() : top(0) {}
        void push(int val) {
            top = top + 1;
        }
        int pop() {
            if (top > 0) {
                top = top - 1;
                return top;
            }
            return -1;
        }
        bool is_empty() { return top == 0; }
    };
}
"#;

    let rust_code = transpile(cpp_code).expect("Transpilation failed");
    let clean = strip_noise(&rust_code);
    let result = compile_rust_code(&clean);
    assert!(
        result.is_ok(),
        "Full C++ program should compile:\n{}\nErrors: {:?}",
        clean,
        result.err()
    );
}

#[test]
#[ignore = "Known limitation: new returns Box<T> but var type is *mut T — needs ownership inference upgrade"]
fn test_cpp_new_delete_compiles_with_rustc() {
    let cpp_code = r#"
extern "C" { void __m(); }
class Resource {
public:
    int handle;
    Resource(int h) : handle(h) {}
    int get() { return handle; }
    ~Resource() {}
};
void use_resource() {
    Resource* r = new Resource(42);
    delete r;
}
"#;

    let rust_code = transpile(cpp_code).expect("Transpilation failed");
    let clean = strip_noise(&rust_code);
    let result = compile_rust_code(&clean);
    assert!(
        result.is_ok(),
        "new/delete transpilation should compile:\n{}\nErrors: {:?}",
        clean,
        result.err()
    );
}
