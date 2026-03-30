//! Dogfood Validation: Transpile C++ and verify output compiles with rustc
//!
//! This example demonstrates Decy's C++ transpilation capabilities on
//! realistic code patterns and validates the output actually compiles.
//!
//! Run with: `cargo run -p decy-core --example dogfood_validation_demo`

use decy_core::transpile;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Decy Dogfood Validation ===\n");

    let mut total = 0;
    let mut passed = 0;

    passed += dogfood_test("Class with methods", r#"
extern "C" { void __m(); }
class Calculator {
public:
    int result;
    Calculator(int init) : result(init) {}
    int get() { return result; }
    void add(int x) { result = result + x; }
    void multiply(int x) { result = result * x; }
    ~Calculator() {}
};
"#, &mut total)?;

    passed += dogfood_test("Namespace with nested struct", r#"
extern "C" { void __m(); }
namespace geometry {
    struct Point { int x; int y; };
    int distance_squared(int x1, int y1, int x2, int y2) {
        int dx = x2 - x1;
        int dy = y2 - y1;
        return dx * dx + dy * dy;
    }
}
"#, &mut total)?;

    passed += dogfood_test("Operator overloading", r#"
extern "C" { void __m(); }
class Pair {
public:
    int first;
    int second;
    Pair(int a, int b) : first(a), second(b) {}
    Pair operator+(Pair other) { Pair r(0,0); return r; }
    bool operator==(Pair other) { return first == other.first; }
};
"#, &mut total)?;

    passed += dogfood_test("Inheritance", r#"
extern "C" { void __m(); }
class Base {
public:
    int id;
    int get_id() { return id; }
};
class Derived : public Base {
public:
    int extra;
    int get_extra() { return extra; }
};
"#, &mut total)?;

    passed += dogfood_test("CUDA kernel", r#"
__global__ void saxpy(float* y, float a, float* x, int n) {
    int i = 0;
}
void host_setup(int n) {
    int blocks = (n + 255) / 256;
}
"#, &mut total)?;

    println!("\n=== Results: {}/{} passed ===", passed, total);
    if passed == total {
        println!("All dogfood tests passed!");
    } else {
        println!("Some tests failed — see output above for details.");
    }

    Ok(())
}

fn dogfood_test(name: &str, cpp_code: &str, total: &mut usize) -> Result<usize, Box<dyn std::error::Error>> {
    *total += 1;
    print!("  {:40} ", name);

    let rust_code = transpile(cpp_code)?;

    // Strip noise: dummy trigger functions, ERRNO, info notes
    let lines: Vec<&str> = rust_code.lines().collect();
    let mut clean_lines = Vec::new();
    let mut skip_next_brace = false;
    for line in &lines {
        if line.contains("ℹ Note:") || line.contains("rustc --crate-type") {
            continue;
        }
        if line.starts_with("fn __") {
            skip_next_brace = true;
            continue;
        }
        if skip_next_brace && line.trim() == "}" {
            skip_next_brace = false;
            continue;
        }
        skip_next_brace = false;
        if line.starts_with("static mut ERRNO") {
            continue;
        }
        clean_lines.push(*line);
    }
    let clean = clean_lines.join("\n");

    // Try to compile
    let dir = tempfile::tempdir()?;
    let rs_path = dir.path().join("test.rs");
    let rlib_path = dir.path().join("test.rlib");
    std::fs::write(&rs_path, &clean)?;

    let output = Command::new("rustc")
        .args(["--edition", "2021", "--crate-type=lib", "-o"])
        .arg(&rlib_path)
        .arg(&rs_path)
        .output()?;

    let has_errors = String::from_utf8_lossy(&output.stderr)
        .lines()
        .any(|l| l.starts_with("error"));

    if has_errors {
        println!("[FAIL]");
        let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();
        let errors: Vec<&str> = stderr_str
            .lines()
            .filter(|l| l.starts_with("error"))
            .collect();
        for e in errors.iter().take(3) {
            println!("    {}", e);
        }
        Ok(0)
    } else {
        println!("[PASS]");
        Ok(1)
    }
}
