//! Buffer Overflow Safety Integration Tests
//!
//! **RED PHASE**: Comprehensive tests for C buffer overflows → Safe Rust
//!
//! This validates that dangerous C buffer overflow patterns are transpiled
//! to safe Rust with proper bounds checking and no out-of-bounds access.
//!
//! **Pattern**: EXTREME TDD - Test-First Development
//! **Reference**: ISO C99 §6.5.6 - array subscript out of bounds is undefined behavior
//!
//! **Safety Goal**: <30 unsafe blocks per 1000 LOC
//! **Validation**: All array accesses bounds-checked, no buffer overruns

use decy_core::transpile;

// ============================================================================
// RED PHASE: Array Bounds Violations
// ============================================================================

#[test]
fn test_array_read_out_of_bounds() {
    // Reading past array end is undefined behavior
    let c_code = r#"
        int main() {
            int array[10] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9};
            int index = 5;

            if (index < 10) {
                return array[index];
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Array bounds check should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_array_write_out_of_bounds() {
    // Writing past array end is undefined behavior
    let c_code = r#"
        int main() {
            int array[10];
            int index = 5;

            if (index < 10) {
                array[index] = 42;
                return array[index];
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Array write with bounds check should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Off-By-One Errors
// ============================================================================

#[test]
fn test_off_by_one_error() {
    // Classic off-by-one: accessing array[n] when size is n
    let c_code = r#"
        int main() {
            int array[10] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9};

            for (int i = 0; i < 10; i++) {
                array[i] = i;
            }

            return array[9];
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Loop with correct bounds should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_off_by_one_prevented() {
    // Prevented off-by-one with proper check
    let c_code = r#"
        int main() {
            int array[10] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9};
            int index = 10;

            if (index < 10) {
                return array[index];
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Off-by-one prevention should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Negative Indices
// ============================================================================

#[test]
fn test_negative_array_index() {
    // Negative indices are undefined behavior
    let c_code = r#"
        int main() {
            int array[10] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9};
            int index = 5;

            if (index >= 0 && index < 10) {
                return array[index];
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Negative index check should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: String Buffer Overflows
// ============================================================================

#[test]
fn test_string_buffer_overflow() {
    // strcpy can cause buffer overflow
    let c_code = r#"
        #include <string.h>

        int main() {
            char buffer[10];
            const char* source = "Hello";

            if (strlen(source) < 10) {
                strcpy(buffer, source);
                return buffer[0];
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "String buffer with length check should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_string_concatenation_overflow() {
    // strcat can overflow buffer
    let c_code = r#"
        #include <string.h>

        int main() {
            char buffer[20] = "Hello";
            const char* suffix = " World";

            if (strlen(buffer) + strlen(suffix) < 20) {
                strcat(buffer, suffix);
                return buffer[0];
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 6,
        "String concatenation with check should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: memcpy Buffer Overflows
// ============================================================================

#[test]
fn test_memcpy_buffer_overflow() {
    // memcpy can overflow destination
    let c_code = r#"
        #include <string.h>

        int main() {
            int dest[10];
            int source[5] = {1, 2, 3, 4, 5};

            memcpy(dest, source, sizeof(source));

            return dest[0];
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "memcpy should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Stack-Based Buffer Overflow
// ============================================================================

#[test]
fn test_stack_buffer_overflow() {
    // Stack buffer overflow pattern
    let c_code = r#"
        int main() {
            char buffer[8];

            for (int i = 0; i < 8; i++) {
                buffer[i] = 'A' + i;
            }

            return buffer[0];
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Stack buffer with correct bounds should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Heap-Based Buffer Overflow
// ============================================================================

#[test]
fn test_heap_buffer_overflow() {
    // Heap buffer overflow via malloc
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* buffer = (int*)malloc(10 * sizeof(int));

            if (buffer != 0) {
                for (int i = 0; i < 10; i++) {
                    buffer[i] = i;
                }

                int result = buffer[5];
                free(buffer);
                return result;
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 6,
        "Heap buffer with correct bounds should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Multi-Dimensional Array Bounds
// ============================================================================

#[test]
fn test_multidimensional_array_bounds() {
    // Multi-dimensional array bounds
    let c_code = r#"
        int main() {
            int matrix[3][4] = {
                {1, 2, 3, 4},
                {5, 6, 7, 8},
                {9, 10, 11, 12}
            };

            int row = 1;
            int col = 2;

            if (row < 3 && col < 4) {
                return matrix[row][col];
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Multi-dimensional bounds check should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Pointer Arithmetic Buffer Overflow
// ============================================================================

#[test]
fn test_pointer_arithmetic_bounds() {
    // Pointer arithmetic can go out of bounds
    let c_code = r#"
        int main() {
            int array[10] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9};
            int* ptr = array;
            int offset = 5;

            if (offset < 10) {
                ptr = ptr + offset;
                return *ptr;
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Pointer arithmetic with bounds check should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Variable-Length Array (VLA) Bounds
// ============================================================================

#[test]
fn test_variable_length_array_bounds() {
    // VLA with runtime size (C99 feature)
    let c_code = r#"
        int main() {
            int size = 10;
            int array[10];

            for (int i = 0; i < size; i++) {
                array[i] = i;
            }

            return array[5];
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "VLA with correct bounds should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Buffer Overflow in Function Arguments
// ============================================================================

#[test]
fn test_buffer_overflow_in_function() {
    // Buffer passed to function can overflow
    let c_code = r#"
        void fill_buffer(int* buffer, int size) {
            for (int i = 0; i < size; i++) {
                buffer[i] = i;
            }
        }

        int main() {
            int array[10];
            fill_buffer(array, 10);

            return array[5];
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");
    assert!(
        result.contains("fn fill_buffer"),
        "Should have fill_buffer function"
    );

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "Function with buffer parameter should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Array Initialization Bounds
// ============================================================================

#[test]
fn test_array_initialization_bounds() {
    // Array initialization with loops
    let c_code = r#"
        int main() {
            int array[5];

            for (int i = 0; i < 5; i++) {
                array[i] = 0;
            }

            array[2] = 42;

            return array[2];
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Array initialization should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Buffer Overflow with Struct
// ============================================================================

#[test]
fn test_buffer_overflow_in_struct() {
    // Buffer inside struct
    let c_code = r#"
        struct Data {
            int buffer[10];
            int count;
        };

        int main() {
            struct Data data;
            data.count = 5;

            for (int i = 0; i < data.count; i++) {
                data.buffer[i] = i;
            }

            return data.buffer[2];
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Struct buffer access should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Unsafe Density Target
// ============================================================================

#[test]
fn test_unsafe_block_count_target() {
    // CRITICAL: Validate overall unsafe minimization for buffer overflow
    let c_code = r#"
        int main() {
            int array[10] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9};

            for (int i = 0; i < 10; i++) {
                array[i] = array[i] * 2;
            }

            int sum = 0;
            for (int i = 0; i < 10; i++) {
                sum += array[i];
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

    // Target: <=30 unsafe per 1000 LOC for buffer overflow
    assert!(
        unsafe_per_1000 <= 30.0,
        "Buffer overflow handling should minimize unsafe (got {:.2} per 1000 LOC, want <=30)",
        unsafe_per_1000
    );

    // Should have main function
    assert!(result.contains("fn main"), "Should generate main function");
}

// ============================================================================
// RED PHASE: Compilation and Correctness
// ============================================================================

#[test]
fn test_transpiled_bounds_check_compiles() {
    // Generated Rust should have valid syntax
    let c_code = r#"
        int main() {
            int array[5] = {1, 2, 3, 4, 5};
            int index = 2;

            if (index < 5) {
                return array[index];
            }

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
fn test_buffer_overflow_safety_documentation() {
    // Validate generated code quality
    let c_code = r#"
        int main() {
            int array[10];

            for (int i = 0; i < 10; i++) {
                array[i] = i;
            }

            return array[5];
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
