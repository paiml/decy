//! C++ Namespace Transpilation — Proven by Compilation
//!
//! Demonstrates: namespaces -> pub mod, nested namespaces, class-in-namespace
//! **Proves**: transpiled output compiles with rustc
//!
//! Run: `cargo run -p decy-core --example cpp_namespace_demo`

use decy_core::transpile;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== C++ Namespaces -> Rust Modules ===\n");

    let cpp = r#"
extern "C" { void __trigger(); }

namespace math {
    int square(int x) { return x * x; }
    int cube(int x) { return x * x * x; }
    int abs(int x) { return x < 0 ? -x : x; }

    struct Point {
        int x;
        int y;
    };

    namespace constants {
        int zero() { return 0; }
        int one() { return 1; }
    }
}

namespace collections {
    class Stack {
    public:
        int top;
        Stack() : top(0) {}
        void push(int val) { top = top + 1; }
        int pop() {
            if (top > 0) { top = top - 1; return top; }
            return -1;
        }
        bool empty() { return top == 0; }
    };
}
"#;

    println!("C++ Input: 2 namespaces (1 nested), 1 class\n");

    let rust = transpile(cpp)?;
    let clean = strip(&rust);
    println!("Rust Output:");
    println!("{}\n", clean);

    let ok = compile_check(&clean)?;
    if ok {
        println!("[PROVEN] Output compiles with rustc --edition 2021");
    } else {
        println!("[FAILED] Compilation error");
        std::process::exit(1);
    }

    assert!(clean.contains("pub mod math"), "math module");
    assert!(clean.contains("pub mod constants"), "nested constants module");
    assert!(clean.contains("pub mod collections"), "collections module");
    assert!(clean.contains("pub struct Point"), "Point struct in math");
    assert!(clean.contains("pub struct Stack"), "Stack class in collections");
    assert!(clean.contains("fn square("), "square function");
    assert!(clean.contains("fn push("), "Stack push method");
    println!("[VERIFIED] All namespace properties correct");

    Ok(())
}

fn strip(code: &str) -> String {
    let mut out = Vec::new();
    let mut skip = false;
    for line in code.lines() {
        if line.starts_with("fn __") { skip = true; continue; }
        if skip && line.trim() == "}" { skip = false; continue; }
        skip = false;
        if line.starts_with("static mut ERRNO") { continue; }
        out.push(line);
    }
    out.join("\n")
}

fn compile_check(code: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let rs = dir.path().join("out.rs");
    let rlib = dir.path().join("out.rlib");
    std::fs::write(&rs, code)?;
    let o = Command::new("rustc")
        .args(["--edition", "2021", "--crate-type=lib", "-o"])
        .arg(&rlib).arg(&rs).output()?;
    Ok(!String::from_utf8_lossy(&o.stderr).lines().any(|l| l.starts_with("error")))
}
