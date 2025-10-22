# Quick Start

Coming soon! See [USER_GUIDE.md](../docs/USER_GUIDE.md) for now.

```rust
// Example: Using Decy programmatically
use decy_core;

fn main() {
    let c_code = "int add(int a, int b) { return a + b; }";
    let rust_code = decy_core::transpile(c_code).unwrap();
    println!("{}", rust_code);
}

#[test]
fn test_transpile() {
    let c_code = "int add(int a, int b) { return a + b; }";
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}
```

Next: [Your First Transpilation](./first-transpilation.md)
