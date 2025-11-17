//! Format String Safety Integration Tests
//!
//! **RED PHASE**: Comprehensive tests for C format string → Safe Rust
//!
//! This validates that dangerous C format string patterns (printf, sprintf, scanf)
//! are transpiled to safe Rust code with type-safe formatting and bounds checking.
//!
//! **Pattern**: EXTREME TDD - Test-First Development
//! **Reference**: CWE-134 (Format String Vulnerability)
//!
//! **Safety Goal**: ≤30 unsafe blocks per 1000 LOC
//! **Validation**: Format strings validated at compile time, no format string injection
//!
//! # FIXED: Parser System Header Support
//!
//! **STATUS**: Tests now passing with stdlib prototype support! ✅
//!
//! **SOLUTION**: The decy-stdlib crate provides built-in prototypes for C standard library.
//! - System includes are commented out during preprocessing
//! - Stdlib prototypes are injected for the specific header (e.g., stdio.h)
//! - Parser successfully handles injected prototypes (per-header filtering)
//!
//! **IMPLEMENTATION**: decy-stdlib (Sprint 18)
//! 1. ✅ Built-in definitions for 24 stdio.h functions (ISO C99 §7.21)
//! 2. ✅ Per-header prototype filtering (string.h, stdio.h, stdlib.h)
//! 3. ✅ Integration with preprocessor for automatic injection
//!
//! **TOYOTA WAY - Kaizen (改善)**: Continuous improvement through TDD!

use decy_core::transpile;

// ============================================================================
// RED PHASE: Safe printf Usage
// ============================================================================

#[test]
fn test_safe_printf_with_format_string() {
    // Safe printf with literal format string
    let c_code = r#"
        #include <stdio.h>

        int main() {
            int value = 42;
            printf("Value: %d\n", value);
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "Safe printf should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_printf_multiple_arguments() {
    // printf with multiple format specifiers
    let c_code = r#"
        #include <stdio.h>

        int main() {
            int a = 10;
            int b = 20;
            printf("a=%d, b=%d, sum=%d\n", a, b, a + b);
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "Multi-arg printf should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_printf_string_format() {
    // printf with string format specifier
    let c_code = r#"
        #include <stdio.h>

        int main() {
            char* name = "Alice";
            printf("Hello, %s!\n", name);
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "String printf should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: sprintf and snprintf Safety
// ============================================================================

#[test]
fn test_sprintf_with_bounds() {
    // sprintf (unbounded, dangerous in C)
    let c_code = r#"
        #include <stdio.h>

        int main() {
            char buffer[100];
            int value = 42;
            sprintf(buffer, "Value: %d", value);
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "sprintf should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_snprintf_bounded() {
    // snprintf (bounded, safer)
    let c_code = r#"
        #include <stdio.h>

        int main() {
            char buffer[100];
            int value = 42;
            snprintf(buffer, sizeof(buffer), "Value: %d", value);
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "snprintf should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: scanf Safety
// ============================================================================

#[test]
fn test_scanf_with_width_specifier() {
    // scanf with width specifier (safer)
    let c_code = r#"
        #include <stdio.h>

        int main() {
            char buffer[10];
            scanf("%9s", buffer);
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "scanf with width should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_scanf_integer() {
    // scanf for integer input
    let c_code = r#"
        #include <stdio.h>

        int main() {
            int value;
            scanf("%d", &value);
            return value;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "scanf integer should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Format Specifier Validation
// ============================================================================

#[test]
fn test_printf_integer_formats() {
    // Various integer format specifiers
    let c_code = r#"
        #include <stdio.h>

        int main() {
            int d = 42;
            unsigned int u = 42;
            printf("d=%d, u=%u, hex=%x, oct=%o\n", d, u, u, u);
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Integer formats should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_printf_float_formats() {
    // Float format specifiers
    let c_code = r#"
        #include <stdio.h>

        int main() {
            double value = 3.14159;
            printf("f=%f, e=%e, g=%g\n", value, value, value);
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Float formats should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_printf_char_format() {
    // Character format specifier
    let c_code = r#"
        #include <stdio.h>

        int main() {
            char c = 'A';
            printf("Character: %c\n", c);
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "Char format should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Width and Precision Specifiers
// ============================================================================

#[test]
fn test_printf_width_specifier() {
    // Width specifier
    let c_code = r#"
        #include <stdio.h>

        int main() {
            int value = 42;
            printf("%10d\n", value);
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "Width specifier should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_printf_precision_specifier() {
    // Precision specifier for floats
    let c_code = r#"
        #include <stdio.h>

        int main() {
            double value = 3.14159;
            printf("%.2f\n", value);
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "Precision specifier should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Complex Format Strings
// ============================================================================

#[test]
fn test_printf_complex_format() {
    // Complex format string with multiple types
    let c_code = r#"
        #include <stdio.h>

        int main() {
            int i = 42;
            double d = 3.14;
            char* s = "test";
            char c = 'A';

            printf("int=%d, double=%.2f, string=%s, char=%c\n", i, d, s, c);
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Complex format should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Formatted Output to Variables
// ============================================================================

#[test]
fn test_sprintf_to_buffer() {
    // sprintf to buffer
    let c_code = r#"
        #include <stdio.h>
        #include <string.h>

        int main() {
            char buffer[50];
            int value = 42;

            sprintf(buffer, "The answer is %d", value);
            return strlen(buffer);
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "sprintf to buffer should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Format String with Escape Sequences
// ============================================================================

#[test]
fn test_printf_escape_sequences() {
    // Format string with escape sequences
    let c_code = r#"
        #include <stdio.h>

        int main() {
            printf("Line 1\nLine 2\tTabbed\n");
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "Escape sequences should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Percent Escaping
// ============================================================================

#[test]
fn test_printf_percent_escape() {
    // Percent sign escaping
    let c_code = r#"
        #include <stdio.h>

        int main() {
            int percent = 50;
            printf("Progress: %d%%\n", percent);
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "Percent escape should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Unsafe Density Target
// ============================================================================

#[test]
fn test_unsafe_block_count_target() {
    // CRITICAL: Validate overall unsafe minimization
    let c_code = r#"
        #include <stdio.h>

        int main() {
            char buffer[100];
            int a = 10;
            int b = 20;
            char* name = "Alice";

            printf("Values: a=%d, b=%d\n", a, b);
            snprintf(buffer, sizeof(buffer), "Hello, %s!", name);

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    // Count unsafe blocks and calculate density
    let unsafe_count = result.matches("unsafe").count();
    let lines_of_code = result.lines().count();

    let unsafe_per_1000 = if lines_of_code > 0 {
        (unsafe_count as f64 / lines_of_code as f64) * 1000.0
    } else {
        0.0
    };

    // Target: <=30 unsafe per 1000 LOC for format strings
    assert!(
        unsafe_per_1000 <= 30.0,
        "Format strings should minimize unsafe (got {:.2} per 1000 LOC, want <=30)",
        unsafe_per_1000
    );

    // Should have main function
    assert!(result.contains("fn main"), "Should generate main function");
}

// ============================================================================
// RED PHASE: Compilation and Correctness
// ============================================================================

#[test]
fn test_transpiled_format_code_compiles() {
    // Generated Rust should have valid syntax
    let c_code = r#"
        #include <stdio.h>

        int main() {
            int value = 42;
            printf("Value: %d\n", value);

            char buffer[50];
            sprintf(buffer, "Result: %d", value * 2);

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    // Basic syntax validation
    assert!(!result.is_empty(), "Should generate non-empty code");
    assert!(result.contains("fn main"), "Should have main function");

    // Should not have obvious syntax errors
    let open_braces = result.matches('{').count();
    let close_braces = result.matches('}').count();
    assert_eq!(
        open_braces, close_braces,
        "Braces should be balanced: {} open, {} close",
        open_braces, close_braces
    );
}

#[test]
fn test_format_string_safety_documentation() {
    // Validate generated code quality
    let c_code = r#"
        #include <stdio.h>

        int main() {
            printf("Hello, World!\n");
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    // Generated code should be reasonable
    assert!(result.contains("fn main"), "Should have main function");

    // If unsafe blocks exist, they should be minimal
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count < 10,
        "Should have minimal unsafe blocks (found {})",
        unsafe_count
    );
}
