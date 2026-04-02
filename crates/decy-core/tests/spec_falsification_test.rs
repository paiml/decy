//! Spec Falsification Tests (Popperian Methodology)
//!
//! Each test makes a **falsifiable prediction** from the spec and attempts
//! to refute it. Tests that PASS confirm the spec claim. Tests that FAIL
//! falsify it and must be marked `#[ignore = "FALSIFIED: reason"]`.
//!
//! Reference: docs/specifications/README.md

use decy_core::transpile;

// =========================================================================
// Section 6.2: C++ Support — Class -> struct + impl
// =========================================================================

#[test]
fn falsify_class_to_struct_field_count_preserved() {
    // Prediction: |fields(class)| = |fields(struct)|
    let cpp = r#"
extern "C" { void __m(); }
class Triple { public: int a; int b; int c; };
"#;
    let rust = transpile(cpp).unwrap();
    assert!(rust.contains("pub a: i32"), "Field a missing");
    assert!(rust.contains("pub b: i32"), "Field b missing");
    assert!(rust.contains("pub c: i32"), "Field c missing");
}

#[test]
fn falsify_constructor_maps_to_new() {
    // Prediction: Constructor(int a, int b) -> pub fn new(a: i32, b: i32) -> Self
    let cpp = r#"
extern "C" { void __m(); }
class Pair { public: int x; int y; Pair(int a, int b) : x(a), y(b) {} };
"#;
    let rust = transpile(cpp).unwrap();
    assert!(rust.contains("pub fn new(a: i32, b: i32) -> Self"), "Constructor not mapped to new()");
}

#[test]
fn falsify_destructor_maps_to_drop() {
    // Prediction: ~ClassName() -> impl Drop for ClassName
    let cpp = r#"
extern "C" { void __m(); }
class Res { public: int h; ~Res() {} };
"#;
    let rust = transpile(cpp).unwrap();
    assert!(rust.contains("impl Drop for Res"), "Destructor not mapped to Drop");
}

#[test]
fn falsify_no_destructor_no_drop() {
    // Prediction: class without ~() should NOT have impl Drop
    let cpp = r#"
extern "C" { void __m(); }
class Simple { public: int x; };
"#;
    let rust = transpile(cpp).unwrap();
    assert!(!rust.contains("impl Drop"), "Should not have Drop without destructor");
}

// =========================================================================
// Section 6.2: C++ Support — Namespace -> mod
// =========================================================================

#[test]
fn falsify_namespace_to_mod() {
    // Prediction: namespace foo { ... } -> pub mod foo { ... }
    let cpp = r#"
extern "C" { void __m(); }
namespace utils { int helper(int x) { return x + 1; } }
"#;
    let rust = transpile(cpp).unwrap();
    assert!(rust.contains("pub mod utils"), "Namespace not mapped to mod");
    assert!(rust.contains("fn helper("), "Function not inside mod");
}

#[test]
fn falsify_nested_namespace() {
    // Prediction: nested namespaces become nested modules
    let cpp = r#"
extern "C" { void __m(); }
namespace a { namespace b { int f() { return 1; } } }
"#;
    let rust = transpile(cpp).unwrap();
    assert!(rust.contains("pub mod a"), "Outer namespace missing");
    assert!(rust.contains("pub mod b"), "Inner namespace missing");
}

// =========================================================================
// Section 6.2: C++ Support — Operator overloading -> std::ops
// =========================================================================

#[test]
fn falsify_operator_plus_maps_to_add() {
    // Prediction: operator+ -> impl std::ops::Add
    let cpp = r#"
extern "C" { void __m(); }
class V { public: int x; V operator+(V o) { V r; return r; } };
"#;
    let rust = transpile(cpp).unwrap();
    assert!(rust.contains("impl std::ops::Add"), "operator+ not mapped to Add trait");
    assert!(rust.contains("type Output"), "Missing Output associated type");
}

#[test]
fn falsify_operator_eq_maps_to_partial_eq() {
    // Prediction: operator== -> impl PartialEq
    let cpp = r#"
extern "C" { void __m(); }
class P { public: int x; bool operator==(P o) { return x == o.x; } };
"#;
    let rust = transpile(cpp).unwrap();
    assert!(rust.contains("impl PartialEq for P"), "operator== not mapped to PartialEq");
}

// =========================================================================
// Section 6.2: C++ Support — Inheritance -> Composition + Deref
// =========================================================================

#[test]
fn falsify_inheritance_embeds_base() {
    // Prediction: class D : public B -> struct D { base: B, ... }
    let cpp = r#"
extern "C" { void __m(); }
class B { public: int id; };
class D : public B { public: int extra; };
"#;
    let rust = transpile(cpp).unwrap();
    assert!(rust.contains("base: B"), "Base class not embedded as field");
}

#[test]
fn falsify_inheritance_generates_deref() {
    // Prediction: derived class gets impl Deref with Target = Base
    let cpp = r#"
extern "C" { void __m(); }
class B { public: int id; };
class D : public B { public: int extra; };
"#;
    let rust = transpile(cpp).unwrap();
    assert!(rust.contains("impl std::ops::Deref for D"), "Missing Deref impl");
    assert!(rust.contains("type Target = B"), "Wrong Deref target");
}

// =========================================================================
// Section 6.2: C++ Support — new/delete
// =========================================================================

#[test]
fn falsify_new_generates_box_new() {
    // Prediction: new T(args) -> Box::new(T::new(args))
    let cpp = r#"
extern "C" { void __m(); }
class Obj { public: int v; Obj(int x) : v(x) {} ~Obj() {} };
void test() { Obj* o = new Obj(7); delete o; }
"#;
    let rust = transpile(cpp).unwrap();
    assert!(rust.contains("Box::new(Obj::new(7))"), "new not mapped to Box::new");
}

#[test]
fn falsify_delete_generates_drop() {
    // Prediction: delete ptr -> drop(ptr)
    let cpp = r#"
extern "C" { void __m(); }
class Obj { public: int v; Obj(int x) : v(x) {} ~Obj() {} };
void test() { Obj* o = new Obj(7); delete o; }
"#;
    let rust = transpile(cpp).unwrap();
    assert!(rust.contains("drop(o)"), "delete not mapped to drop()");
}

// =========================================================================
// Section 6.3: CUDA Support — __global__ kernel FFI
// =========================================================================

#[test]
fn falsify_cuda_global_generates_extern_c() {
    // Prediction: __global__ void kernel() -> extern "C" { fn kernel(); }
    let cuda = r#"
__global__ void my_kernel(int* data, int n) {
    int i = 0;
}
void host() { int x = 1; }
"#;
    let rust = transpile(cuda).unwrap();
    assert!(rust.contains("extern \"C\""), "Kernel should generate extern C");
    assert!(rust.contains("fn my_kernel("), "Kernel name not preserved");
}

#[test]
fn falsify_cuda_host_transpiles_normally() {
    // Prediction: host function in .cu transpiles as normal Rust (not inside extern "C")
    let cuda = r#"
__global__ void k(int n) { int i = 0; }
void host_add(int a, int b) { int c = a + b; }
"#;
    let rust = transpile(cuda).unwrap();
    assert!(rust.contains("fn host_add("), "Host function should transpile normally");
    // host_add should appear as a regular fn, not inside extern "C" block
    // Find the line with host_add and check it's not preceded by extern "C"
    let host_line_idx = rust.lines().position(|l| l.contains("fn host_add("));
    let extern_line_idx = rust.lines().position(|l| l.contains("extern \"C\""));
    if let (Some(host), Some(ext)) = (host_line_idx, extern_line_idx) {
        assert!(host < ext || host > ext + 5, "host_add should not be inside extern C block");
    }
}

// =========================================================================
// Section 5: Ownership Inference — CROWN 3-qualifier model
// =========================================================================

#[test]
fn falsify_malloc_maps_to_box() {
    // Prediction: malloc(sizeof(T)) -> Box::new(T::default())
    let c = r#"
#include <stdlib.h>
int* create() {
    int* p = (int*)malloc(sizeof(int));
    *p = 42;
    return p;
}
"#;
    let rust = transpile(c).unwrap();
    // malloc should map to Box::new or vec!
    assert!(
        rust.contains("Box::new") || rust.contains("vec!") || rust.contains("Vec::"),
        "malloc not mapped to safe Rust allocation, got:\n{}",
        rust
    );
}

// =========================================================================
// Section 7: Safety — implicit this -> self
// =========================================================================

#[test]
fn falsify_implicit_this_maps_to_self() {
    // Prediction: member access via implicit this -> self.field
    let cpp = r#"
extern "C" { void __m(); }
class C { public: int val; int get() { return val; } };
"#;
    let rust = transpile(cpp).unwrap();
    assert!(rust.contains("self.val"), "Implicit this.val should become self.val");
}

// =========================================================================
// Section 8: Provable Contracts — Copy + Drop conflict
// =========================================================================

#[test]
fn falsify_no_copy_with_drop() {
    // Prediction: class with destructor should NOT derive Copy
    let cpp = r#"
extern "C" { void __m(); }
class R { public: int h; R(int x) : h(x) {} ~R() {} };
"#;
    let rust = transpile(cpp).unwrap();
    assert!(!rust.contains("Copy"), "Should not derive Copy when Drop is implemented");
}

#[test]
fn falsify_no_conflicting_partial_eq() {
    // Prediction: class with operator== should NOT also derive PartialEq
    let cpp = r#"
extern "C" { void __m(); }
class P { public: int x; bool operator==(P o) { return x == o.x; } };
"#;
    let rust = transpile(cpp).unwrap();
    // Should have impl PartialEq but not derive PartialEq
    assert!(rust.contains("impl PartialEq"), "Should have PartialEq impl");
    let derive_line = rust.lines().find(|l| l.contains("#[derive("));
    if let Some(derive) = derive_line {
        assert!(
            !derive.contains("PartialEq"),
            "Should NOT derive PartialEq when operator== exists"
        );
    }
}

// =========================================================================
// Section 13: Metrics — constructor positional mapping
// =========================================================================

#[test]
fn falsify_constructor_positional_fallback() {
    // Prediction: when param names don't match fields, use positional mapping
    let cpp = r#"
extern "C" { void __m(); }
class V { public: int x; int y; V(int a, int b) : x(a), y(b) {} };
"#;
    let rust = transpile(cpp).unwrap();
    // Params a,b don't match fields x,y — positional: a->x, b->y
    assert!(rust.contains("x: a"), "Positional mapping x: a failed");
    assert!(rust.contains("y: b"), "Positional mapping y: b failed");
}
