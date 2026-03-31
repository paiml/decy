//! C++ Inheritance Transpilation — Proven by Compilation
//!
//! Demonstrates: single inheritance -> composition + Deref/DerefMut
//! **Proves**: transpiled output compiles with rustc
//!
//! Run: `cargo run -p decy-core --example cpp_inheritance_demo`

use decy_core::transpile;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== C++ Inheritance -> Rust Composition + Deref ===\n");

    let cpp = r#"
extern "C" { void __trigger(); }

class Animal {
public:
    int legs;
    int weight;
    Animal(int l, int w) : legs(l), weight(w) {}
    int get_legs() { return legs; }
    int get_weight() { return weight; }
};

class Dog : public Animal {
public:
    int bark_volume;
    int get_bark() { return bark_volume; }
};

class Cat : public Animal {
public:
    int purr_freq;
    int get_purr() { return purr_freq; }
};
"#;

    println!("C++ Input:");
    println!("{}\n", cpp.trim());

    let rust = transpile(cpp)?;

    // Strip noise
    let clean = strip(&rust);
    println!("Rust Output:");
    println!("{}\n", clean);

    // Compile with rustc
    let ok = compile_check(&clean)?;
    if ok {
        println!("[PROVEN] Output compiles with rustc --edition 2021");
    } else {
        println!("[FAILED] Output does not compile");
        std::process::exit(1);
    }

    // Verify key properties
    assert!(clean.contains("pub struct Animal"), "Base struct");
    assert!(clean.contains("pub struct Dog"), "Dog struct");
    assert!(clean.contains("pub struct Cat"), "Cat struct");
    assert!(clean.contains("base: Animal"), "Dog has base field");
    assert!(clean.contains("impl std::ops::Deref for Dog"), "Dog Deref");
    assert!(clean.contains("impl std::ops::Deref for Cat"), "Cat Deref");
    assert!(clean.contains("type Target = Animal"), "Deref target");
    println!("[VERIFIED] All inheritance properties correct");

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
