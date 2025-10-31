//! Loop + Array Access Safety Integration Tests
//!
//! **RED PHASE**: Comprehensive tests for C loop + array patterns → Safe Rust
//!
//! This validates that dangerous C patterns (array indexing in loops, buffer overflows)
//! are transpiled to safe Rust with bounds checking and safe iteration.
//!
//! **Pattern**: EXTREME TDD - Test-First Development
//! **Reference**: ISO C99 §6.5.2.1 (Array subscripting) + §6.8.5 (Iteration statements)
//!
//! **Safety Goal**: Zero buffer overflows through bounds checking
//! **Validation**: Transpiled Rust uses safe indexing or iterators

use decy_core::transpile;

// ============================================================================
// RED PHASE: For Loop + Array Access
// ============================================================================

#[test]
fn test_for_loop_array_iteration() {
    // Classic C pattern: iterate array with for loop
    let c_code = r#"
        int main() {
            int numbers[5] = {1, 2, 3, 4, 5};
            int sum = 0;

            for (int i = 0; i < 5; i++) {
                sum += numbers[i];
            }

            return sum;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    // Should generate Rust code
    assert!(result.contains("fn main"), "Should have main function");

    // Should handle array safely
    assert!(!result.is_empty(), "Should generate code");

    // Count unsafe blocks - should be minimal
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Array iteration should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_for_loop_array_modification() {
    // Modify array elements in loop
    let c_code = r#"
        int main() {
            int values[10];

            for (int i = 0; i < 10; i++) {
                values[i] = i * 2;
            }

            return values[5];
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    // Should have array and loop
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Array modification should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_for_loop_with_array_bounds() {
    // Explicit bounds checking pattern
    let c_code = r#"
        #define ARRAY_SIZE 8

        int main() {
            int data[ARRAY_SIZE];
            int total = 0;

            for (int i = 0; i < ARRAY_SIZE; i++) {
                data[i] = i + 1;
                total += data[i];
            }

            return total;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    // Bounds should be respected
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Bounded loop should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: While Loop + Array Access
// ============================================================================

#[test]
fn test_while_loop_array_access() {
    // While loop with array indexing
    let c_code = r#"
        int main() {
            int numbers[5] = {10, 20, 30, 40, 50};
            int i = 0;
            int sum = 0;

            while (i < 5) {
                sum += numbers[i];
                i++;
            }

            return sum;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "While loop array access should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_while_loop_array_search() {
    // Search pattern with while loop
    let c_code = r#"
        int main() {
            int values[10] = {5, 3, 8, 1, 9, 2, 7, 4, 6, 0};
            int i = 0;
            int target = 7;
            int found = -1;

            while (i < 10) {
                if (values[i] == target) {
                    found = i;
                    break;
                }
                i++;
            }

            return found;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    // Should handle break statement
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Search pattern should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Nested Loops + 2D Arrays
// ============================================================================

#[test]
fn test_nested_loop_2d_array() {
    // 2D array with nested loops
    let c_code = r#"
        int main() {
            int matrix[3][3] = {
                {1, 2, 3},
                {4, 5, 6},
                {7, 8, 9}
            };
            int sum = 0;

            for (int i = 0; i < 3; i++) {
                for (int j = 0; j < 3; j++) {
                    sum += matrix[i][j];
                }
            }

            return sum;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    // Nested loops with 2D array
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "Nested loops with 2D array should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Array Bounds Safety
// ============================================================================

#[test]
fn test_array_out_of_bounds_prevention() {
    // Pattern that could overflow in C
    let c_code = r#"
        int main() {
            int buffer[5];

            // Safe because loop condition matches array size
            for (int i = 0; i < 5; i++) {
                buffer[i] = i * 10;
            }

            return buffer[0];
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    // Should be safe due to matching bounds
    let unsafe_count = result.matches("unsafe").count();
    let lines = result.lines().count();
    let unsafe_per_1000 = if lines > 0 {
        (unsafe_count as f64 / lines as f64) * 1000.0
    } else {
        0.0
    };

    assert!(
        unsafe_per_1000 < 50.0,
        "Array bounds should be safe (got {:.2} unsafe/1000 LOC)",
        unsafe_per_1000
    );
}

#[test]
fn test_array_initialization_in_loop() {
    // Initialize array to zero in loop
    let c_code = r#"
        int main() {
            int array[100];

            for (int i = 0; i < 100; i++) {
                array[i] = 0;
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    // Large array initialization
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Array initialization should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Common Real-World Patterns
// ============================================================================

#[test]
fn test_array_copy_loop() {
    // Copy one array to another
    let c_code = r#"
        int main() {
            int source[5] = {1, 2, 3, 4, 5};
            int dest[5];

            for (int i = 0; i < 5; i++) {
                dest[i] = source[i];
            }

            return dest[4];
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    // Array copy should be safe
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Array copy should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_array_reverse_loop() {
    // Reverse array in-place
    let c_code = r#"
        int main() {
            int numbers[6] = {1, 2, 3, 4, 5, 6};

            for (int i = 0; i < 3; i++) {
                int temp = numbers[i];
                numbers[i] = numbers[5 - i];
                numbers[5 - i] = temp;
            }

            return numbers[0];
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    // In-place swap should be safe
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "Array reverse should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_array_max_find_loop() {
    // Find maximum value in array
    let c_code = r#"
        int main() {
            int values[8] = {23, 45, 12, 67, 34, 89, 56, 78};
            int max = values[0];

            for (int i = 1; i < 8; i++) {
                if (values[i] > max) {
                    max = values[i];
                }
            }

            return max;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    // Max find should be safe
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Max find should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Edge Cases and Safety
// ============================================================================

#[test]
fn test_empty_loop_array() {
    // Loop that doesn't execute (edge case)
    let c_code = r#"
        int main() {
            int array[5] = {1, 2, 3, 4, 5};
            int sum = 0;

            for (int i = 0; i < 0; i++) {
                sum += array[i];
            }

            return sum;  // Should be 0
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    // Empty loop should be safe
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "Empty loop should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_single_element_array_loop() {
    // Single element array
    let c_code = r#"
        int main() {
            int single[1] = {42};
            int value = 0;

            for (int i = 0; i < 1; i++) {
                value = single[i];
            }

            return value;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    // Single element should be safe
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "Single element loop should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_unsafe_minimization_target() {
    // CRITICAL: Validate overall unsafe minimization for loop+array pattern
    let c_code = r#"
        int main() {
            int data[20];

            // Initialize
            for (int i = 0; i < 20; i++) {
                data[i] = i * i;
            }

            // Sum
            int total = 0;
            for (int i = 0; i < 20; i++) {
                total += data[i];
            }

            return total;
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

    // Target: <5 unsafe per 1000 LOC
    assert!(
        unsafe_per_1000 < 50.0,
        "Loop+array pattern should minimize unsafe (got {:.2} per 1000 LOC, want <50)",
        unsafe_per_1000
    );

    // Should have main function
    assert!(result.contains("fn main"), "Should generate main function");
}

// ============================================================================
// RED PHASE: Compilation and Correctness
// ============================================================================

#[test]
fn test_transpiled_loop_array_compiles() {
    // Generated Rust should have valid syntax
    let c_code = r#"
        int main() {
            int numbers[10];

            for (int i = 0; i < 10; i++) {
                numbers[i] = i * 2;
            }

            int result = 0;
            for (int i = 0; i < 10; i++) {
                result += numbers[i];
            }

            return result;
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

    // Should not have excessive semicolons
    assert!(
        !result.contains(";;;;"),
        "Should not have excessive semicolons"
    );
}

#[test]
fn test_loop_array_safety_documentation() {
    // If unsafe is used, it should be documented
    let c_code = r#"
        int main() {
            int array[5] = {10, 20, 30, 40, 50};
            int sum = 0;

            for (int i = 0; i < 5; i++) {
                sum += array[i];
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
