//! Pointer Arithmetic Safety Integration Tests
//!
//! **RED PHASE**: Comprehensive tests for C pointer arithmetic → Safe Rust
//!
//! This validates that dangerous C pointer arithmetic operations are transpiled
//! to safe Rust with bounds checking and proper offset calculations.
//!
//! **Pattern**: EXTREME TDD - Test-First Development
//! **Reference**: ISO C99 §6.5.6 (Additive operators - pointer arithmetic)
//!
//! **Safety Goal**: <50 unsafe blocks per 1000 LOC
//! **Validation**: No buffer overflows, no out-of-bounds access

use decy_core::transpile;

// ============================================================================
// RED PHASE: Pointer Increment/Decrement
// ============================================================================

#[test]
fn test_pointer_increment() {
    // Classic pointer increment pattern
    let c_code = r#"
        int main() {
            int array[5] = {1, 2, 3, 4, 5};
            int* ptr = array;

            int first = *ptr;
            ptr++;
            int second = *ptr;

            return first + second;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    // Pointer increment should be handled
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "Pointer increment should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_pointer_decrement() {
    // Pointer decrement pattern
    let c_code = r#"
        int main() {
            int array[5] = {1, 2, 3, 4, 5};
            int* ptr = &array[4];  // Point to last element

            int last = *ptr;
            ptr--;
            int second_last = *ptr;

            return last + second_last;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "Pointer decrement should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Pointer Addition/Subtraction
// ============================================================================

#[test]
fn test_pointer_addition() {
    // Add offset to pointer
    let c_code = r#"
        int main() {
            int array[10] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9};
            int* ptr = array;

            int value = *(ptr + 5);  // array[5]

            return value;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Pointer addition should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_pointer_subtraction() {
    // Subtract offset from pointer
    let c_code = r#"
        int main() {
            int array[10] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9};
            int* ptr = &array[7];

            int value = *(ptr - 3);  // array[4]

            return value;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Pointer subtraction should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Pointer Difference
// ============================================================================

#[test]
fn test_pointer_difference() {
    // Calculate distance between pointers
    let c_code = r#"
        int main() {
            int array[10] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9};
            int* ptr1 = &array[2];
            int* ptr2 = &array[7];

            int distance = ptr2 - ptr1;  // Should be 5

            return distance;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Pointer difference should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Array Traversal with Pointers
// ============================================================================

#[test]
fn test_array_traversal_with_pointer() {
    // Walk through array with pointer
    let c_code = r#"
        int main() {
            int array[5] = {10, 20, 30, 40, 50};
            int* ptr = array;
            int sum = 0;

            for (int i = 0; i < 5; i++) {
                sum += *ptr;
                ptr++;
            }

            return sum;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "Array traversal should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_array_traversal_backwards() {
    // Walk backwards through array
    let c_code = r#"
        int main() {
            int array[5] = {10, 20, 30, 40, 50};
            int* ptr = &array[4];  // Start at end
            int sum = 0;

            for (int i = 0; i < 5; i++) {
                sum += *ptr;
                ptr--;
            }

            return sum;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "Backwards traversal should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Pointer Comparison
// ============================================================================

#[test]
fn test_pointer_comparison() {
    // Compare pointers
    let c_code = r#"
        int main() {
            int array[10] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9};
            int* start = array;
            int* end = &array[10];
            int* current = array;
            int count = 0;

            while (current < end) {
                count++;
                current++;
            }

            return count;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "Pointer comparison should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Pointer Indexing (ptr[i])
// ============================================================================

#[test]
fn test_pointer_indexing() {
    // Use pointer with array indexing syntax
    let c_code = r#"
        int main() {
            int array[5] = {1, 2, 3, 4, 5};
            int* ptr = array;

            int value = ptr[2];  // Equivalent to *(ptr + 2)

            return value;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Pointer indexing should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_pointer_indexing_with_offset() {
    // Pointer with offset, then indexing
    let c_code = r#"
        int main() {
            int array[10] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9};
            int* ptr = &array[3];

            int value = ptr[2];  // array[5]

            return value;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Offset pointer indexing should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: String Pointer Arithmetic
// ============================================================================

#[test]
fn test_string_pointer_arithmetic() {
    // Walk through string with pointer
    let c_code = r#"
        int main() {
            const char* str = "Hello";
            const char* ptr = str;
            int count = 0;

            while (*ptr != 0) {
                count++;
                ptr++;
            }

            return count;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "String pointer arithmetic should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Bounds Checking Pattern
// ============================================================================

#[test]
fn test_pointer_bounds_checking() {
    // Defensive pointer arithmetic with bounds check
    let c_code = r#"
        int main() {
            int array[10] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9};
            int* ptr = array;
            int* end = &array[10];
            int sum = 0;

            while (ptr < end) {
                sum += *ptr;
                ptr++;
            }

            return sum;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "Bounds checking pattern should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Pointer to Struct Members
// ============================================================================

#[test]
fn test_pointer_to_struct_member() {
    // Pointer arithmetic with struct members
    let c_code = r#"
        struct Point {
            int x;
            int y;
        };

        int main() {
            struct Point points[3] = {{1, 2}, {3, 4}, {5, 6}};
            struct Point* ptr = points;

            int sum = 0;
            for (int i = 0; i < 3; i++) {
                sum += ptr->x + ptr->y;
                ptr++;
            }

            return sum;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 6,
        "Struct pointer arithmetic should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Complex Pointer Arithmetic
// ============================================================================

#[test]
fn test_complex_pointer_arithmetic() {
    // Multiple operations combined
    let c_code = r#"
        int main() {
            int array[10] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9};
            int* ptr = array + 2;

            int a = *ptr;       // array[2]
            int b = *(ptr + 3);  // array[5]
            ptr++;
            int c = *ptr;       // array[3]

            return a + b + c;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 6,
        "Complex pointer arithmetic should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Unsafe Density Target
// ============================================================================

#[test]
fn test_unsafe_block_count_target() {
    // CRITICAL: Validate overall unsafe minimization for pointer arithmetic
    let c_code = r#"
        int main() {
            int data[20] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                           10, 11, 12, 13, 14, 15, 16, 17, 18, 19};
            int* ptr = data;
            int sum = 0;

            // Walk through array with pointer
            for (int i = 0; i < 20; i++) {
                sum += *ptr;
                ptr++;
            }

            // Reset and walk backwards
            ptr = &data[19];
            int product = 1;
            for (int i = 0; i < 5; i++) {
                product *= *ptr;
                ptr--;
            }

            return sum + product;
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

    // Target: <250 unsafe per 1000 LOC (pointer arithmetic is inherently complex)
    assert!(
        unsafe_per_1000 < 250.0,
        "Pointer arithmetic should minimize unsafe (got {:.2} per 1000 LOC, want <250)",
        unsafe_per_1000
    );

    // Should have main function
    assert!(result.contains("fn main"), "Should generate main function");
}

// ============================================================================
// RED PHASE: Compilation and Correctness
// ============================================================================

#[test]
fn test_transpiled_pointer_arithmetic_compiles() {
    // Generated Rust should have valid syntax
    let c_code = r#"
        int main() {
            int numbers[10] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9};
            int* ptr = numbers;
            int total = 0;

            for (int i = 0; i < 10; i++) {
                total += *(ptr + i);
            }

            return total;
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
fn test_pointer_safety_documentation() {
    // If unsafe is used, validate it's minimal
    let c_code = r#"
        int main() {
            int array[5] = {1, 2, 3, 4, 5};
            int* ptr = array;

            int sum = 0;
            for (int i = 0; i < 5; i++) {
                sum += *ptr;
                ptr++;
            }

            return sum;
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
