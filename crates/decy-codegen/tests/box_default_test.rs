// =============================================================================
// DECY-141: Replace mem::zeroed with Box::default() for structs with Default
// =============================================================================
// Tests for generating Box::default() instead of Box::new(unsafe { mem::zeroed() })
// when allocating structs that derive Default.
//
// Current (unsafe):
//   let mut entry: Box<Entry> = Box::new(unsafe { std::mem::zeroed::<Entry>() });
//
// Expected (safe):
//   let mut entry: Box<Entry> = Box::default();
//
// This reduces unsafe block count while maintaining identical behavior.

use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

/// Helper to transpile C code and return the generated Rust
fn transpile_c(c_code: &str) -> String {
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(c_code.as_bytes())
        .expect("Failed to write C code");

    let output = Command::new("cargo")
        .args(["run", "-p", "decy", "--quiet", "--", "transpile"])
        .arg(temp_file.path())
        .output()
        .expect("Failed to run decy transpile");

    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Helper to compile Rust code and return success/failure
fn compile_rust(rust_code: &str) -> (bool, String) {
    let mut temp_file = NamedTempFile::with_suffix(".rs").expect("Failed to create temp file");
    temp_file
        .write_all(rust_code.as_bytes())
        .expect("Failed to write Rust code");

    let output = Command::new("rustc")
        .args([
            "--crate-type=lib",
            "--edition=2021",
            "--crate-name=test_box_default",
            "-A",
            "warnings",
        ])
        .arg(temp_file.path())
        .arg("-o")
        .arg("/tmp/test_box_default_compile")
        .output()
        .expect("Failed to run rustc");

    let success = output.status.success();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (success, stderr)
}

/// Count unsafe blocks in code
fn count_unsafe_blocks(code: &str) -> usize {
    code.matches("unsafe {").count() + code.matches("unsafe{").count()
}

/// Test that malloc(sizeof(T)) for a struct generates Box::default() instead of mem::zeroed.
/// The struct derives Default, so we should use safe initialization.
#[test]
fn test_struct_malloc_uses_box_default() {
    let c_code = r#"
typedef struct Point {
    int x;
    int y;
} Point;

Point* create_point(void) {
    Point* p = (Point*)malloc(sizeof(Point));
    return p;
}
"#;

    let rust_code = transpile_c(c_code);

    // Should contain Box allocation
    assert!(
        rust_code.contains("Box"),
        "Should generate Box allocation, got: {}",
        rust_code
    );

    // Should use Box::default() instead of mem::zeroed
    // DECY-141: This is the key assertion
    assert!(
        rust_code.contains("Box::default()")
            || rust_code.contains("Box::<Point>::default()"),
        "DECY-141: Should use Box::default() instead of mem::zeroed for structs with Default. Got:\n{}",
        rust_code
    );

    // Should NOT contain mem::zeroed for this simple struct
    let has_zeroed = rust_code.contains("mem::zeroed");
    assert!(
        !has_zeroed,
        "DECY-141: Should NOT use mem::zeroed when Default is available. Got:\n{}",
        rust_code
    );
}

/// Test that struct with #[derive(Default)] uses Box::default().
#[test]
fn test_struct_with_default_derive_uses_box_default() {
    let c_code = r#"
typedef struct Rectangle {
    int width;
    int height;
    int area;
} Rectangle;

Rectangle* create_rectangle(int w, int h) {
    Rectangle* r = (Rectangle*)malloc(sizeof(Rectangle));
    r->width = w;
    r->height = h;
    r->area = w * h;
    return r;
}
"#;

    let rust_code = transpile_c(c_code);

    // Verify generated struct has Default derive
    assert!(
        rust_code.contains("#[derive(") && rust_code.contains("Default"),
        "Generated struct should derive Default, got: {}",
        rust_code
    );

    // Should use Box::default() for allocation
    assert!(
        rust_code.contains("Box::default()") || rust_code.contains("Box::<Rectangle>::default()"),
        "DECY-141: Should use Box::default() for structs with Default derive. Got:\n{}",
        rust_code
    );
}

/// Test that unsafe block count is reduced by using Box::default().
#[test]
fn test_box_default_reduces_unsafe_count() {
    let c_code = r#"
typedef struct Node {
    int data;
    struct Node* left;
    struct Node* right;
} Node;

Node* create_node(int data) {
    Node* n = (Node*)malloc(sizeof(Node));
    n->data = data;
    n->left = NULL;
    n->right = NULL;
    return n;
}
"#;

    let rust_code = transpile_c(c_code);

    // Count unsafe blocks
    let unsafe_count = count_unsafe_blocks(&rust_code);

    // With Box::default(), we should have fewer unsafe blocks
    // The create_node function should have 0 unsafe blocks for allocation
    // (only potentially for pointer operations if any)
    assert!(
        unsafe_count < 2,
        "DECY-141: Box::default() should reduce unsafe blocks. Found {} unsafe blocks in:\n{}",
        unsafe_count,
        rust_code
    );
}

/// Test that generated code still compiles after Box::default() transformation.
#[test]
fn test_box_default_code_compiles() {
    let c_code = r#"
typedef struct Rectangle {
    int width;
    int height;
} Rectangle;

Rectangle* create_rect(int w, int h) {
    Rectangle* r = (Rectangle*)malloc(sizeof(Rectangle));
    r->width = w;
    r->height = h;
    return r;
}
"#;

    let rust_code = transpile_c(c_code);

    // Should compile successfully
    let (success, stderr) = compile_rust(&rust_code);

    assert!(
        success,
        "Generated code with Box::default() should compile.\nCode:\n{}\nErrors:\n{}",
        rust_code, stderr
    );
}

/// Test that hash_table.c still compiles with Box::default() optimization.
#[test]
fn test_hash_table_compiles_with_box_default() {
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "decy",
            "--quiet",
            "--",
            "transpile",
            "examples/data_structures/hash_table.c",
        ])
        .current_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap())
        .output()
        .expect("Failed to run decy transpile");

    let rust_code = String::from_utf8_lossy(&output.stdout).to_string();

    // Write to temp file and compile
    let mut temp_file = NamedTempFile::with_suffix(".rs").expect("Failed to create temp file");
    temp_file
        .write_all(rust_code.as_bytes())
        .expect("Failed to write Rust code");

    let output = Command::new("rustc")
        .args([
            "--crate-type=lib",
            "--edition=2021",
            "--crate-name=hash_table",
            "-A",
            "warnings",
        ])
        .arg(temp_file.path())
        .arg("-o")
        .arg("/tmp/test_hash_table_box_default")
        .output()
        .expect("Failed to run rustc");

    let success = output.status.success();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    assert!(
        success,
        "hash_table.c should still compile with Box::default() optimization.\nErrors:\n{}",
        stderr
    );

    // Verify reduced unsafe count
    let unsafe_count = count_unsafe_blocks(&rust_code);

    // Document current count - with Box::default() it should be lower
    println!("hash_table.c unsafe block count: {}", unsafe_count);
}

/// Test that binary_tree.c still compiles with Box::default() optimization.
#[test]
fn test_binary_tree_compiles_with_box_default() {
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "decy",
            "--quiet",
            "--",
            "transpile",
            "examples/data_structures/binary_tree.c",
        ])
        .current_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap())
        .output()
        .expect("Failed to run decy transpile");

    let rust_code = String::from_utf8_lossy(&output.stdout).to_string();

    // Write to temp file and compile
    let mut temp_file = NamedTempFile::with_suffix(".rs").expect("Failed to create temp file");
    temp_file
        .write_all(rust_code.as_bytes())
        .expect("Failed to write Rust code");

    let output = Command::new("rustc")
        .args([
            "--crate-type=lib",
            "--edition=2021",
            "--crate-name=binary_tree",
            "-A",
            "warnings",
        ])
        .arg(temp_file.path())
        .arg("-o")
        .arg("/tmp/test_binary_tree_box_default")
        .output()
        .expect("Failed to run rustc");

    let success = output.status.success();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    assert!(
        success,
        "binary_tree.c should still compile with Box::default() optimization.\nErrors:\n{}",
        stderr
    );
}
