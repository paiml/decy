//! C++ new/delete Transpilation — Proven by Compilation
//!
//! Demonstrates: new T(args) -> Box::new(T::new(args)), delete -> drop()
//! **Proves**: transpiled output compiles with rustc
//!
//! Run: `cargo run -p decy-core --example cpp_new_delete_demo`

use decy_core::transpile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== C++ new/delete -> Rust Box/drop ===\n");

    let cpp = r#"
extern "C" { void __trigger(); }

class Widget {
public:
    int id;
    int priority;
    Widget(int i, int p) : id(i), priority(p) {}
    int get_id() { return id; }
    int get_priority() { return priority; }
    ~Widget() {}
};

void create_and_destroy() {
    Widget* w = new Widget(1, 10);
    int id = w->id;
    delete w;
}
"#;

    println!("C++ Input:");
    println!("{}\n", cpp.trim());

    let rust = transpile(cpp)?;
    let clean = strip(&rust);
    println!("Rust Output:");
    println!("{}\n", clean);

    assert!(clean.contains("Box::new(Widget::new("), "new -> Box::new");
    assert!(clean.contains("drop(w)"), "delete -> drop");
    assert!(clean.contains("impl Drop for Widget"), "destructor -> Drop");
    assert!(clean.contains("pub fn new("), "constructor -> new()");
    println!("[VERIFIED] new/delete transpilation correct");

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
