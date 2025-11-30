//! DECY-132: FILE* to std::fs::File transformation tests.
//!
//! C FILE* operations should transform to safe Rust file I/O:
//! - fopen(name, mode) → std::fs::File::open/create
//! - fgetc(f) → f.read_byte()
//! - fclose(f) → drop(f) or automatic RAII

use decy_core::transpile;

/// Test that fopen gets transformed to File::open.
///
/// C: fopen("file.txt", "r")
/// Expected: std::fs::File::open("file.txt") or similar
#[test]
fn test_fopen_transformation() {
    // Note: Using forward declaration to avoid #include <stdio.h>
    let c_code = r#"
        typedef struct _IO_FILE FILE;
        FILE *fopen(const char *filename, const char *mode);
        int fclose(FILE *f);

        int main() {
            FILE *f = fopen("test.txt", "r");
            if (f != 0) {
                fclose(f);
            }
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should transform fopen to File::open
    assert!(
        result.contains("File::open") || result.contains("std::fs::File"),
        "Should transform fopen to File::open\nGenerated:\n{}",
        result
    );
}

/// Test that fclose gets transformed to drop or RAII comment.
///
/// C: fclose(f)
/// Expected: drop(f) or comment about RAII
#[test]
fn test_fclose_transformation() {
    let c_code = r#"
        typedef struct _IO_FILE FILE;
        int fclose(FILE *f);

        int close_file(FILE *f) {
            fclose(f);
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should transform fclose to drop or RAII
    assert!(
        result.contains("drop") || result.contains("RAII") || !result.contains("fclose"),
        "Should transform fclose to drop or RAII\nGenerated:\n{}",
        result
    );
}

/// Test that fgetc gets transformed to read operation.
///
/// C: fgetc(f)
/// Expected: Some form of read operation
#[test]
fn test_fgetc_transformation() {
    let c_code = r#"
        typedef struct _IO_FILE FILE;
        int fgetc(FILE *f);

        int read_char(FILE *f) {
            return fgetc(f);
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should transform fgetc to a read operation
    assert!(
        result.contains("read") || result.contains("Read") || !result.contains("fgetc("),
        "Should transform fgetc to read operation\nGenerated:\n{}",
        result
    );
}

/// Test that printf gets transformed to print! macro.
///
/// C: printf("Hello")
/// Expected: print!(...) or println!(...)
#[test]
fn test_printf_transformation() {
    let c_code = r#"
        int printf(const char *format, ...);

        int main() {
            printf("Hello");
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should transform printf to print!
    assert!(
        result.contains("print!") || result.contains("println!"),
        "Should transform printf to print! macro\nGenerated:\n{}",
        result
    );
}
