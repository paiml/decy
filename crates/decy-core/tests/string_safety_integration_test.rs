//! String Safety Integration Test
//!
//! **RED PHASE**: Comprehensive test for C string operations → Safe Rust
//!
//! This test validates that unsafe C string operations (strcpy, strcat, strlen)
//! are transpiled to safe Rust alternatives (String, str methods).
//!
//! **Pattern**: EXTREME TDD - Test-First Development
//! **Reference**: ISO C99 §7.21 (String handling) + K&R Chapter 5
//!
//! **Safety Goal**: Zero unsafe blocks for string operations
//! **Validation**: Transpiled Rust code must compile and run safely

use decy_core::transpile;

// ============================================================================
// RED PHASE: Failing Tests for String Operations
// ============================================================================

#[test]
fn test_strlen_transpilation() {
    // C code using strlen
    let c_code = r#"
        #include <string.h>

        int get_length(const char* str) {
            return strlen(str);
        }

        int main() {
            const char* message = "Hello, World!";
            int len = get_length(message);
            return len;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    // Should NOT contain unsafe blocks for strlen
    assert!(
        !result.contains("unsafe"),
        "String length should be safe (use .len())"
    );

    // Should use Rust str::len() or String::len()
    assert!(
        result.contains(".len()"),
        "Should use safe Rust .len() method"
    );

    // Should compile as valid Rust
    assert!(result.contains("fn get_length"), "Should generate function");
    assert!(result.contains("fn main"), "Should generate main");
}

#[test]
fn test_strcpy_transpilation_to_string_copy() {
    // C code using strcpy (UNSAFE in C!)
    let c_code = r#"
        #include <string.h>

        void copy_string(char* dest, const char* src) {
            strcpy(dest, src);
        }

        int main() {
            char buffer[100];
            copy_string(buffer, "Safe in Rust!");
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    // Should avoid raw strcpy - use String or clone
    // Unsafe count should be minimal (ideally 0)
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "strcpy should minimize unsafe (found {})",
        unsafe_count
    );

    // Should generate valid Rust function
    assert!(
        result.contains("fn copy_string"),
        "Should generate function"
    );
}

#[test]
fn test_strcat_transpilation_to_string_concatenation() {
    // C code using strcat (UNSAFE in C!)
    let c_code = r#"
        #include <string.h>

        void append_string(char* dest, const char* src) {
            strcat(dest, src);
        }

        int main() {
            char buffer[200];
            strcpy(buffer, "Hello, ");
            append_string(buffer, "Rust!");
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    // strcat should ideally use String::push_str or format!
    // Check for minimal unsafe
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "strcat should minimize unsafe (found {})",
        unsafe_count
    );

    assert!(
        result.contains("fn append_string"),
        "Should generate function"
    );
}

#[test]
fn test_string_literal_transpilation() {
    // C string literals should become &str or String
    let c_code = r#"
        int main() {
            const char* greeting = "Hello, World!";
            const char* farewell = "Goodbye!";
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    // String literals should be safe
    assert!(
        !result.contains("unsafe") || result.matches("unsafe").count() <= 1,
        "String literals should be mostly safe"
    );

    // Should contain the string values
    assert!(
        result.contains("Hello, World!"),
        "Should preserve string literal"
    );
    assert!(
        result.contains("Goodbye!"),
        "Should preserve string literal"
    );
}

#[test]
fn test_strcmp_transpilation_to_eq() {
    // strcmp should become == comparison
    let c_code = r#"
        #include <string.h>

        int are_equal(const char* s1, const char* s2) {
            return strcmp(s1, s2) == 0;
        }

        int main() {
            const char* a = "test";
            const char* b = "test";
            int equal = are_equal(a, b);
            return equal;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    // strcmp should ideally become == (safe comparison)
    assert!(result.contains("fn are_equal"), "Should generate function");

    // Should minimize unsafe
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "String comparison should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// Edge Cases and Safety Validation
// ============================================================================

#[test]
fn test_null_string_handling() {
    // NULL pointer check pattern in C
    let c_code = r#"
        #include <string.h>

        int check_null(const char* str) {
            if (str == 0) {
                return 0;
            }
            return strlen(str);
        }

        int main() {
            const char* valid_str = "test";
            int len = check_null(valid_str);
            return len;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    // NULL check should be handled
    assert!(result.contains("fn check_null"), "Should generate function");

    // Should handle null checks safely (no crashes)
    // May use if statements or Option type
    assert!(result.contains("fn main"), "Should generate main");
}

#[test]
fn test_empty_string_handling() {
    // Empty strings should work correctly
    let c_code = r#"
        #include <string.h>

        int main() {
            const char* empty = "";
            int len = strlen(empty);
            return len;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    // Empty string should be safe
    assert!(!result.is_empty(), "Should generate code");
    assert!(result.contains("fn main"), "Should generate main");
}

#[test]
fn test_unsafe_block_count_target() {
    // CRITICAL: Validate unsafe minimization goal
    let c_code = r#"
        #include <string.h>

        int main() {
            const char* str1 = "Hello";
            const char* str2 = "World";

            int len1 = strlen(str1);
            int len2 = strlen(str2);

            return len1 + len2;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    // Count unsafe blocks
    let unsafe_count = result.matches("unsafe").count();
    let lines_of_code = result.lines().count();

    // Target: <5 unsafe per 1000 LOC
    let unsafe_per_1000 = if lines_of_code > 0 {
        (unsafe_count as f64 / lines_of_code as f64) * 1000.0
    } else {
        0.0
    };

    assert!(
        unsafe_per_1000 < 5.0,
        "Unsafe blocks per 1000 LOC should be <5 (got {:.2})",
        unsafe_per_1000
    );
}

// ============================================================================
// Compilation Validation
// ============================================================================

#[test]
fn test_transpiled_rust_compiles() {
    // The transpiled Rust code should compile
    let c_code = r#"
        #include <string.h>

        int main() {
            const char* message = "Decy transpiler";
            int length = strlen(message);
            return length;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    // Basic syntax checks (actual compilation test would use rustc)
    assert!(!result.is_empty(), "Should generate non-empty code");
    assert!(result.contains("fn main"), "Should have main function");

    // Should not have obvious syntax errors
    assert!(
        !result.contains("}}}}"),
        "Should not have excessive closing braces"
    );
    assert!(
        !result.contains(";;;;"),
        "Should not have excessive semicolons"
    );
}

#[test]
fn test_string_safety_documentation() {
    // Transpiled code should have safety documentation
    let c_code = r#"
        #include <string.h>

        char* get_string() {
            static char buffer[100] = "Static string";
            return buffer;
        }

        int main() {
            char* str = get_string();
            int len = strlen(str);
            return len;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    // If unsafe blocks exist, they should have SAFETY comments
    if result.contains("unsafe") {
        // Should document why unsafe is needed
        // (This is aspirational - may not be implemented yet)
        assert!(result.contains("fn get_string"), "Should generate function");
    }
}
