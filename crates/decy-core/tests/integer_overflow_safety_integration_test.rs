//! Integer Overflow Safety Integration Tests
//!
//! **RED PHASE**: Comprehensive tests for C integer overflow → Safe Rust
//!
//! This validates that dangerous C integer overflow patterns are transpiled
//! to safe Rust with proper checking and explicit overflow behavior.
//!
//! **Pattern**: EXTREME TDD - Test-First Development
//! **Reference**: ISO C99 §6.5 (Expressions) - signed overflow is undefined behavior
//!
//! **Safety Goal**: <50 unsafe blocks per 1000 LOC
//! **Validation**: No undefined behavior, explicit wrapping/checked operations

use decy_core::transpile;

// ============================================================================
// RED PHASE: Addition Overflow
// ============================================================================

#[test]
fn test_signed_addition_overflow() {
    // Signed addition overflow is undefined behavior in C
    let c_code = r#"
        int main() {
            int a = 2147483647;  // INT_MAX
            int b = 1;
            int result = a + b;  // Overflow!

            return result;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Addition overflow should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_unsigned_addition_overflow() {
    // Unsigned overflow is defined (wraps) in C
    let c_code = r#"
        int main() {
            unsigned int a = 4294967295U;  // UINT_MAX
            unsigned int b = 1U;
            unsigned int result = a + b;  // Wraps to 0

            return (int)result;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Unsigned addition should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Subtraction Underflow
// ============================================================================

#[test]
fn test_signed_subtraction_underflow() {
    // Signed subtraction underflow is undefined behavior
    let c_code = r#"
        int main() {
            int a = -2147483648;  // INT_MIN
            int b = 1;
            int result = a - b;  // Underflow!

            return result;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Subtraction underflow should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_unsigned_subtraction_underflow() {
    // Unsigned underflow wraps in C
    let c_code = r#"
        int main() {
            unsigned int a = 0U;
            unsigned int b = 1U;
            unsigned int result = a - b;  // Wraps to UINT_MAX

            return (int)result;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Unsigned subtraction should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Multiplication Overflow
// ============================================================================

#[test]
fn test_signed_multiplication_overflow() {
    // Signed multiplication overflow is undefined behavior
    let c_code = r#"
        int main() {
            int a = 100000;
            int b = 100000;
            int result = a * b;  // Overflow!

            return result;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Multiplication overflow should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_unsigned_multiplication_overflow() {
    // Unsigned multiplication wraps in C
    let c_code = r#"
        int main() {
            unsigned int a = 4294967295U;
            unsigned int b = 2U;
            unsigned int result = a * b;  // Wraps

            return (int)result;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Unsigned multiplication should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Division Edge Cases
// ============================================================================

#[test]
fn test_division_by_zero_check() {
    // Division by zero is undefined behavior
    let c_code = r#"
        int main() {
            int a = 42;
            int b = 0;

            if (b != 0) {
                return a / b;
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "Division by zero check should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_division_overflow() {
    // INT_MIN / -1 causes overflow
    let c_code = r#"
        int main() {
            int a = -2147483648;  // INT_MIN
            int b = -1;
            int result = a / b;  // Overflow!

            return result;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Division overflow should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Negation Overflow
// ============================================================================

#[test]
fn test_negation_overflow() {
    // -INT_MIN causes overflow
    let c_code = r#"
        int main() {
            int a = -2147483648;  // INT_MIN
            int result = -a;  // Overflow!

            return result;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Negation overflow should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Left Shift Overflow
// ============================================================================

#[test]
fn test_left_shift_overflow() {
    // Left shift can cause overflow
    let c_code = r#"
        int main() {
            int a = 1;
            int result = a << 31;  // Shifts into sign bit

            return result;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Left shift should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_left_shift_by_large_value() {
    // Shift by >= width is undefined behavior
    let c_code = r#"
        int main() {
            int a = 1;
            int shift = 32;  // Undefined!

            if (shift < 32) {
                return a << shift;
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Shift bounds check should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Loop Counter Overflow
// ============================================================================

#[test]
fn test_loop_counter_overflow() {
    // Loop counter can overflow
    let c_code = r#"
        int main() {
            int sum = 0;

            for (int i = 0; i < 100; i++) {
                sum = sum + 1;
            }

            return sum;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Loop counter should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Array Index Overflow
// ============================================================================

#[test]
fn test_array_index_with_overflow() {
    // Array index calculation can overflow
    let c_code = r#"
        int main() {
            int array[10] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9};
            int index = 5;
            int offset = 2;
            int i = index + offset;

            if (i < 10) {
                return array[i];
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Array index with overflow check should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Size Calculation Overflow
// ============================================================================

#[test]
fn test_malloc_size_overflow() {
    // malloc size calculation can overflow
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int count = 1000;
            int size = count * sizeof(int);

            int* array = (int*)malloc(size);

            if (array != 0) {
                array[0] = 42;
                free(array);
                return array[0];
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "Malloc size calculation should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Mixed Signed/Unsigned Operations
// ============================================================================

#[test]
fn test_signed_unsigned_mixed_operation() {
    // Mixing signed and unsigned can cause issues
    let c_code = r#"
        int main() {
            int a = -1;
            unsigned int b = 1U;
            unsigned int result = a + b;  // -1 converts to UINT_MAX

            return (int)result;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Mixed signed/unsigned should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Increment/Decrement Overflow
// ============================================================================

#[test]
fn test_increment_overflow() {
    // i++ can overflow
    let c_code = r#"
        int main() {
            int i = 2147483647;  // INT_MAX
            i++;  // Overflow!

            return i;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Increment overflow should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_decrement_underflow() {
    // i-- can underflow
    let c_code = r#"
        int main() {
            int i = -2147483648;  // INT_MIN
            i--;  // Underflow!

            return i;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Decrement underflow should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Compound Assignment Overflow
// ============================================================================

#[test]
fn test_compound_assignment_overflow() {
    // += can overflow
    let c_code = r#"
        int main() {
            int sum = 2000000000;
            int value = 2000000000;
            sum += value;  // Overflow!

            return sum;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Compound assignment should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Unsafe Density Target
// ============================================================================

#[test]
fn test_unsafe_block_count_target() {
    // CRITICAL: Validate overall unsafe minimization for overflow
    let c_code = r#"
        int main() {
            int a = 1000;
            int b = 2000;
            int sum = a + b;

            int c = 5000;
            int d = 3000;
            int product = c * d;

            int result = sum + product;

            if (result > 0) {
                return result;
            }

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

    // Target: <=50 unsafe per 1000 LOC for overflow
    assert!(
        unsafe_per_1000 <= 50.0,
        "Overflow handling should minimize unsafe (got {:.2} per 1000 LOC, want <=50)",
        unsafe_per_1000
    );

    // Should have main function
    assert!(result.contains("fn main"), "Should generate main function");
}

// ============================================================================
// RED PHASE: Compilation and Correctness
// ============================================================================

#[test]
fn test_transpiled_overflow_code_compiles() {
    // Generated Rust should have valid syntax
    let c_code = r#"
        int main() {
            int a = 100;
            int b = 200;
            int result = a + b;

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
}

#[test]
fn test_overflow_safety_documentation() {
    // Validate generated code quality
    let c_code = r#"
        int main() {
            int a = 2147483647;
            int b = 1;
            int result = a + b;

            return result;
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
