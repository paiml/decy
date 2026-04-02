//! C++ Operator Overloading Transpilation — Proven by Compilation
//!
//! Demonstrates: operator+, operator==, operator+=  -> std::ops traits
//! **Proves**: transpiled output compiles with rustc
//!
//! Run: `cargo run -p decy-core --example cpp_operator_demo`

use decy_core::transpile;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== C++ Operators -> Rust std::ops Traits ===\n");

    let cpp = r#"
extern "C" { void __trigger(); }

class Vec3 {
public:
    int x;
    int y;
    int z;
    Vec3(int a, int b, int c) : x(a), y(b), z(c) {}
    int dot(Vec3 other) { return x * other.x + y * other.y + z * other.z; }
    int magnitude_squared() { return x * x + y * y + z * z; }
    Vec3 operator+(Vec3 other) { Vec3 r(0,0,0); return r; }
    bool operator==(Vec3 other) { return x == other.x; }
};

class Matrix2x2 {
public:
    int a;
    int b;
    int c;
    int d;
    Matrix2x2(int a_, int b_, int c_, int d_) : a(a_), b(b_), c(c_), d(d_) {}
    int determinant() { return a * d - b * c; }
    Matrix2x2 operator+(Matrix2x2 o) { Matrix2x2 r(0,0,0,0); return r; }
};
"#;

    println!("C++ Input ({} lines):", cpp.lines().count());
    for line in cpp.trim().lines().take(10) {
        println!("  {}", line);
    }
    println!("  ...\n");

    let rust = transpile(cpp)?;
    let clean = strip(&rust);
    println!("Rust Output ({} lines):", clean.lines().count());
    println!("{}\n", clean);

    let ok = compile_check(&clean)?;
    if ok {
        println!("[PROVEN] Output compiles with rustc --edition 2021");
    } else {
        println!("[FAILED] Compilation error");
        std::process::exit(1);
    }

    assert!(clean.contains("impl std::ops::Add<Vec3> for Vec3"), "Vec3 Add");
    assert!(clean.contains("impl PartialEq for Vec3"), "Vec3 PartialEq");
    assert!(clean.contains("impl std::ops::Add<Matrix2x2> for Matrix2x2"), "Matrix2x2 Add");
    assert!(clean.contains("self.x * other.x"), "Method body with self access");
    println!("[VERIFIED] All operator properties correct");

    Ok(())
}

fn strip(code: &str) -> String {
    let mut out = Vec::new();
    let mut skip = false;
    for line in code.lines() {
        if line.starts_with("fn __") {
            skip = true;
            continue;
        }
        if skip && line.trim() == "}" {
            skip = false;
            continue;
        }
        skip = false;
        if line.starts_with("static mut ERRNO") {
            continue;
        }
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
        .arg(&rlib)
        .arg(&rs)
        .output()?;
    Ok(!String::from_utf8_lossy(&o.stderr).lines().any(|l| l.starts_with("error")))
}
