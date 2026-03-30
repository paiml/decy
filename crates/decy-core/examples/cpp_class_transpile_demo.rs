//! C++ Class Transpilation Demonstration
//!
//! This example shows how Decy transpiles C++ classes, namespaces,
//! operator overloading, and single inheritance to idiomatic Rust.
//!
//! Run with: `cargo run -p decy-core --example cpp_class_transpile_demo`

use decy_core::transpile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Decy C++ Transpilation Demo ===\n");

    demo_class_with_methods()?;
    demo_namespace()?;
    demo_operator_overloading()?;
    demo_inheritance()?;

    println!("\n=== All C++ demos completed successfully ===");
    Ok(())
}

fn demo_class_with_methods() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Demo 1: C++ Class -> Rust struct + impl ---\n");

    let cpp_code = r#"
extern "C" { void __trigger_cpp_mode(); }
class Counter {
public:
    int count;
    Counter(int initial) : count(initial) {}
    int get() { return count; }
    void increment() { count = count + 1; }
    ~Counter() {}
};
"#;

    println!("C++ Input:");
    println!("{}", cpp_code.trim());
    println!();

    let rust_code = transpile(cpp_code)?;
    println!("Rust Output:");
    println!("{}", rust_code);
    println!();

    assert!(rust_code.contains("pub struct Counter"), "Should generate struct");
    assert!(rust_code.contains("pub fn new("), "Should generate constructor");
    assert!(rust_code.contains("impl Drop"), "Should generate Drop");
    println!("  [PASS] Class transpilation verified\n");
    Ok(())
}

fn demo_namespace() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Demo 2: C++ Namespace -> Rust mod ---\n");

    let cpp_code = r#"
extern "C" { void __trigger_cpp_mode(); }
namespace math {
    int square(int x) { return x * x; }
    struct Vec2 { int x; int y; };
}
"#;

    println!("C++ Input:");
    println!("{}", cpp_code.trim());
    println!();

    let rust_code = transpile(cpp_code)?;
    println!("Rust Output:");
    println!("{}", rust_code);
    println!();

    assert!(rust_code.contains("pub mod math"), "Should generate module");
    assert!(rust_code.contains("fn square("), "Should contain function");
    println!("  [PASS] Namespace transpilation verified\n");
    Ok(())
}

fn demo_operator_overloading() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Demo 3: C++ operator+ -> Rust impl Add ---\n");

    let cpp_code = r#"
extern "C" { void __trigger_cpp_mode(); }
class Vec2 {
public:
    int x;
    int y;
    Vec2(int a, int b) : x(a), y(b) {}
    int dot(Vec2 other) { return x * other.x + y * other.y; }
    Vec2 operator+(Vec2 other) { Vec2 r(0,0); r.x = x + other.x; r.y = y + other.y; return r; }
};
"#;

    println!("C++ Input:");
    println!("{}", cpp_code.trim());
    println!();

    let rust_code = transpile(cpp_code)?;
    println!("Rust Output:");
    println!("{}", rust_code);
    println!();

    assert!(rust_code.contains("impl std::ops::Add"), "Should generate Add trait");
    assert!(rust_code.contains("type Output"), "Should have Output type");
    assert!(rust_code.contains("self.x * other.x"), "Method body with self access");
    println!("  [PASS] Operator overloading transpilation verified\n");
    Ok(())
}

fn demo_inheritance() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Demo 4: C++ Inheritance -> Rust Composition + Deref ---\n");

    let cpp_code = r#"
extern "C" { void __trigger_cpp_mode(); }
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

    println!("C++ Input:");
    println!("{}", cpp_code.trim());
    println!();

    let rust_code = transpile(cpp_code)?;
    println!("Rust Output:");
    println!("{}", rust_code);
    println!();

    assert!(rust_code.contains("pub struct Shape"), "Should have base struct");
    assert!(rust_code.contains("base: Shape"), "Derived should embed base");
    assert!(
        rust_code.contains("impl std::ops::Deref for Circle"),
        "Should generate Deref"
    );
    println!("  [PASS] Inheritance transpilation verified\n");
    Ok(())
}
