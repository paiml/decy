//! Integration test: C++ class and namespace transpilation (DECY-206)
//!
//! **Test Category**: Integration (End-to-End)
//! **Purpose**: Verify C++ classes and namespaces flow through the full
//! transpilation pipeline: C++ Source -> Parser -> HIR -> Codegen -> Rust Output
//!
//! **Tickets**: DECY-198 through DECY-205

use decy_core::transpile;

#[test]
fn test_cpp_class_transpiles_to_struct_impl() {
    // C++ class with extern "C" to trigger C++ mode
    let cpp_code = r#"
extern "C" { void __dummy(); }
class Counter {
public:
    int count;
    Counter(int initial) : count(initial) {}
    int get() { return count; }
    ~Counter() {}
};
"#;

    let result = transpile(cpp_code);
    assert!(result.is_ok(), "C++ class transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Verify struct generation
    assert!(
        rust_code.contains("pub struct Counter"),
        "Expected struct Counter, got:\n{}",
        rust_code
    );

    // Verify impl block
    assert!(
        rust_code.contains("impl Counter"),
        "Expected impl Counter, got:\n{}",
        rust_code
    );

    // Verify constructor maps to new()
    assert!(
        rust_code.contains("pub fn new("),
        "Expected constructor mapped to new(), got:\n{}",
        rust_code
    );

    // Verify destructor maps to Drop
    assert!(
        rust_code.contains("impl Drop for Counter"),
        "Expected Drop impl for destructor, got:\n{}",
        rust_code
    );

    // Verify field
    assert!(
        rust_code.contains("count: i32") || rust_code.contains("count:i32"),
        "Expected count field as i32, got:\n{}",
        rust_code
    );
}

#[test]
fn test_cpp_namespace_transpiles_to_mod() {
    let cpp_code = r#"
extern "C" { void __dummy(); }
namespace utils {
    int helper(int x) { return x + 1; }
}
"#;

    let result = transpile(cpp_code);
    assert!(result.is_ok(), "Namespace transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    assert!(
        rust_code.contains("pub mod utils"),
        "Expected mod utils, got:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains("fn helper("),
        "Expected helper function in module, got:\n{}",
        rust_code
    );
}

#[test]
fn test_cpp_class_inside_namespace() {
    let cpp_code = r#"
extern "C" { void __dummy(); }
namespace shapes {
    class Circle {
    public:
        int radius;
        int area() { return 3 * radius * radius; }
    };
}
"#;

    let result = transpile(cpp_code);
    assert!(result.is_ok(), "Class-in-namespace transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    assert!(
        rust_code.contains("pub mod shapes"),
        "Expected mod shapes, got:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains("pub struct Circle"),
        "Expected struct Circle inside module, got:\n{}",
        rust_code
    );
}

#[test]
fn test_cpp_class_no_destructor_no_drop() {
    let cpp_code = r#"
extern "C" { void __dummy(); }
class Point {
public:
    int x;
    int y;
};
"#;

    let result = transpile(cpp_code);
    assert!(result.is_ok(), "Simple class transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    assert!(
        rust_code.contains("pub struct Point"),
        "Expected struct Point, got:\n{}",
        rust_code
    );
    assert!(
        !rust_code.contains("impl Drop for Point"),
        "Should NOT have Drop impl without destructor, got:\n{}",
        rust_code
    );
}

#[test]
fn test_mixed_c_and_cpp() {
    // Mix C functions with C++ classes
    let cpp_code = r#"
extern "C" { void __dummy(); }
int add(int a, int b) { return a + b; }
class Adder {
public:
    int base;
    Adder(int b) : base(b) {}
    int add_to(int x) { return base + x; }
};
"#;

    let result = transpile(cpp_code);
    assert!(result.is_ok(), "Mixed C/C++ transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Both C function and C++ class should be present
    assert!(
        rust_code.contains("fn add("),
        "Expected C function add(), got:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains("pub struct Adder"),
        "Expected class Adder as struct, got:\n{}",
        rust_code
    );
}

// =========================================================================
// DECY-210: Phase 2 - operator overloading E2E tests
// =========================================================================

#[test]
fn test_cpp_operator_overloading_add() {
    let cpp_code = r#"
extern "C" { void __dummy(); }
class Vec2 {
public:
    int x;
    int y;
    Vec2 operator+(Vec2 other) { Vec2 result; result.x = x + other.x; result.y = y + other.y; return result; }
};
"#;

    let result = transpile(cpp_code);
    assert!(result.is_ok(), "Operator overloading transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    assert!(
        rust_code.contains("pub struct Vec2"),
        "Expected struct Vec2, got:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains("impl std::ops::Add"),
        "Expected Add trait impl for operator+, got:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains("type Output"),
        "Expected Output associated type, got:\n{}",
        rust_code
    );
}

#[test]
fn test_cpp_operator_equality() {
    let cpp_code = r#"
extern "C" { void __dummy(); }
class Point {
public:
    int x;
    int y;
    int get_x() { return x; }
    bool operator==(Point other) { return x == other.x && y == other.y; }
};
"#;

    let result = transpile(cpp_code);
    assert!(result.is_ok(), "Equality operator transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Regular method should be in impl block
    assert!(
        rust_code.contains("fn get_x("),
        "Expected regular method get_x, got:\n{}",
        rust_code
    );
    // Operator== should generate PartialEq
    assert!(
        rust_code.contains("impl PartialEq for Point"),
        "Expected PartialEq impl for operator==, got:\n{}",
        rust_code
    );
}

// =========================================================================
// DECY-225: new/delete E2E test
// =========================================================================

#[test]
fn test_cpp_new_generates_box_new() {
    let cpp_code = r#"
extern "C" { void __m(); }
class Obj {
public:
    int val;
    Obj(int v) : val(v) {}
    ~Obj() {}
};
void test() {
    Obj* o = new Obj(42);
    delete o;
}
"#;

    let result = transpile(cpp_code);
    assert!(result.is_ok(), "new/delete transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    assert!(
        rust_code.contains("Box::new(Obj::new(42))"),
        "new Obj(42) should become Box::new(Obj::new(42)), got:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains("drop(o)"),
        "delete o should become drop(o), got:\n{}",
        rust_code
    );
}

// =========================================================================
// DECY-209: Single inheritance E2E test
// =========================================================================

#[test]
fn test_cpp_single_inheritance() {
    let cpp_code = r#"
extern "C" { void __dummy(); }
class Animal {
public:
    int legs;
    int get_legs() { return legs; }
};
class Dog : public Animal {
public:
    int bark_volume;
    int get_bark() { return bark_volume; }
};
"#;

    let result = transpile(cpp_code);
    assert!(result.is_ok(), "Inheritance transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Base class should be a struct
    assert!(
        rust_code.contains("pub struct Animal"),
        "Expected base struct Animal, got:\n{}",
        rust_code
    );

    // Derived class should have base field
    assert!(
        rust_code.contains("pub struct Dog"),
        "Expected derived struct Dog, got:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains("base: Animal"),
        "Expected base field in Dog, got:\n{}",
        rust_code
    );

    // Should generate Deref impl
    assert!(
        rust_code.contains("impl std::ops::Deref for Dog"),
        "Expected Deref impl for inheritance, got:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains("type Target = Animal"),
        "Expected Target = Animal, got:\n{}",
        rust_code
    );
}

// =========================================================================
// DECY-211: CUDA kernel FFI E2E test
// =========================================================================

#[test]
fn test_cuda_host_function_transpiles_normally() {
    // A .cu file with just host code should transpile like C
    let cuda_code = r#"
int add(int a, int b) { return a + b; }
"#;

    let result = transpile(cuda_code);
    assert!(result.is_ok(), "CUDA host code failed: {:?}", result.err());

    let rust_code = result.unwrap();
    assert!(
        rust_code.contains("fn add("),
        "Host function should transpile normally, got:\n{}",
        rust_code
    );
}
