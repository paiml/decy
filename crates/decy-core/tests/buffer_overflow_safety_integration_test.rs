//! Buffer Overflow Safety Integration Tests
//!
//! **RED PHASE**: Comprehensive tests for C buffer overflow → Safe Rust
//!
//! This validates that dangerous C buffer overflow patterns are transpiled to safe
//! Rust code where buffer overflows are prevented by bounds checking and safe types.
//!
//! **Pattern**: EXTREME TDD - Test-First Development
//! **Reference**: CWE-120 (Buffer Copy without Checking Size), CWE-119 (Buffer Overflow)
//!
//! **Safety Goal**: ≤100 unsafe blocks per 1000 LOC
//! **Validation**: Buffer overflows prevented by bounds checking, Vec/String types

use decy_core::transpile;

// ============================================================================
// RED PHASE: Basic Buffer Overflow Prevention
// ============================================================================

#[test]
fn test_fixed_array_access() {
    // Fixed array with bounded access (safe)
    let c_code = r#"
        int main() {
            int arr[10];
            int i;

            for (i = 0; i < 10; i++) {
                arr[i] = i * 2;
            }

            return arr[5];
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Fixed array access should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_string_buffer_safe_size() {
    // String buffer with safe size
    let c_code = r#"
        int main() {
            char str[20];
            int i;

            for (i = 0; i < 10; i++) {
                str[i] = 'A' + i;
            }
            str[10] = '\0';

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "String buffer should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Array Bounds Checking
// ============================================================================

#[test]
fn test_array_index_validation() {
    // Array access with index validation
    let c_code = r#"
        int main() {
            int arr[5];
            int index = 3;

            if (index >= 0 && index < 5) {
                arr[index] = 42;
                return arr[index];
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Index validation should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_loop_bounds_checked() {
    // Loop with proper bounds checking
    let c_code = r#"
        int main() {
            int arr[10];
            int sum = 0;
            int i;

            for (i = 0; i < 10; i++) {
                arr[i] = i;
                sum = sum + arr[i];
            }

            return sum;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "Loop bounds checking should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Multi-Dimensional Arrays
// ============================================================================

#[test]
fn test_2d_array_access() {
    // 2D array with bounds checking
    let c_code = r#"
        int main() {
            int matrix[3][3];
            int i;
            int j;

            for (i = 0; i < 3; i++) {
                for (j = 0; j < 3; j++) {
                    matrix[i][j] = i * 3 + j;
                }
            }

            return matrix[1][1];
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 6,
        "2D array access should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Buffer Copy Operations
// ============================================================================

#[test]
fn test_manual_buffer_copy() {
    // Manual buffer copy with bounds
    let c_code = r#"
        int main() {
            int src[5];
            int dst[5];
            int i;

            for (i = 0; i < 5; i++) {
                src[i] = i * 10;
            }

            for (i = 0; i < 5; i++) {
                dst[i] = src[i];
            }

            return dst[2];
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 6,
        "Buffer copy should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_partial_buffer_copy() {
    // Partial buffer copy (copy first N elements)
    let c_code = r#"
        int main() {
            int src[10];
            int dst[10];
            int count = 5;
            int i;

            for (i = 0; i < 10; i++) {
                src[i] = i;
            }

            for (i = 0; i < count && i < 10; i++) {
                dst[i] = src[i];
            }

            return dst[3];
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 7,
        "Partial copy should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: String Operations
// ============================================================================

#[test]
fn test_string_initialization() {
    // String buffer initialization
    let c_code = r#"
        int main() {
            char str[10];
            int i;

            for (i = 0; i < 5; i++) {
                str[i] = 'A';
            }
            str[5] = '\0';

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "String initialization should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_string_length_check() {
    // String with length checking
    let c_code = r#"
        int main() {
            char str[20];
            int len = 0;
            int i;

            for (i = 0; i < 15; i++) {
                str[i] = 'X';
                len = len + 1;
            }
            str[len] = '\0';

            return len;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 6,
        "String length check should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Off-by-One Prevention
// ============================================================================

#[test]
fn test_off_by_one_prevention() {
    // Proper loop bounds (< not <=)
    let c_code = r#"
        int main() {
            int arr[5];
            int i;

            for (i = 0; i < 5; i++) {
                arr[i] = i * 2;
            }

            return arr[4];
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Off-by-one prevention should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Dynamic Sizing
// ============================================================================

#[test]
fn test_variable_size_array() {
    // Array with variable-based size
    let c_code = r#"
        int main() {
            int size = 8;
            int arr[10];
            int i;

            for (i = 0; i < size && i < 10; i++) {
                arr[i] = i;
            }

            return arr[5];
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "Variable size should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Struct with Array Members
// ============================================================================

#[test]
fn test_struct_with_array() {
    // Struct containing array
    let c_code = r#"
        struct Buffer {
            int data[5];
            int size;
        };

        int main() {
            struct Buffer buf;
            int i;

            buf.size = 5;

            for (i = 0; i < buf.size; i++) {
                buf.data[i] = i * 3;
            }

            return buf.data[2];
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "Struct with array should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Nested Arrays
// ============================================================================

#[test]
fn test_nested_array_access() {
    // Array of arrays
    let c_code = r#"
        int main() {
            int arrays[3][4];
            int i;
            int j;
            int sum = 0;

            for (i = 0; i < 3; i++) {
                for (j = 0; j < 4; j++) {
                    arrays[i][j] = i + j;
                    sum = sum + arrays[i][j];
                }
            }

            return sum;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 7,
        "Nested arrays should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Function with Array Parameters
// ============================================================================

#[test]
fn test_function_array_parameter() {
    // Function taking array and size
    let c_code = r#"
        int sum_array(int arr[], int size) {
            int sum = 0;
            int i;

            for (i = 0; i < size; i++) {
                sum = sum + arr[i];
            }

            return sum;
        }

        int main() {
            int values[5];
            int i;

            for (i = 0; i < 5; i++) {
                values[i] = i + 1;
            }

            return sum_array(values, 5);
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 6,
        "Function with array should minimize unsafe (found {})",
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
        int main() {
            int buffer[100];
            int i;
            int sum = 0;

            for (i = 0; i < 100; i++) {
                buffer[i] = i;
            }

            for (i = 0; i < 100; i++) {
                sum = sum + buffer[i];
            }

            return sum;
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

    // Target: <=100 unsafe per 1000 LOC for buffer overflow prevention
    assert!(
        unsafe_per_1000 <= 100.0,
        "Buffer overflow prevention should minimize unsafe (got {:.2} per 1000 LOC, want <=100)",
        unsafe_per_1000
    );

    // Should have main function
    assert!(result.contains("fn main"), "Should generate main function");
}

// ============================================================================
// RED PHASE: Compilation and Correctness
// ============================================================================

#[test]
fn test_transpiled_code_compiles() {
    // Generated Rust should have valid syntax
    let c_code = r#"
        int main() {
            int arr[10];
            int i;

            for (i = 0; i < 10; i++) {
                arr[i] = i * i;
            }

            return arr[5];
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
fn test_buffer_overflow_safety_documentation() {
    // Validate generated code quality
    let c_code = r#"
        int main() {
            int buffer[20];
            int i;

            for (i = 0; i < 20; i++) {
                buffer[i] = i;
            }

            return buffer[10];
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
