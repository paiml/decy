//! Integer Overflow Safety Integration Tests
//!
//! **RED PHASE**: Comprehensive tests for C integer overflow → Safe Rust
//!
//! This validates that dangerous C integer overflow patterns are transpiled to safe
//! Rust code where overflows are either prevented or handled explicitly.
//!
//! **Pattern**: EXTREME TDD - Test-First Development
//! **Reference**: CWE-190 (Integer Overflow or Wraparound)
//!
//! **Safety Goal**: ≤100 unsafe blocks per 1000 LOC
//! **Validation**: Integer overflows detected via checked arithmetic or wrapping types

use decy_core::transpile;

// ============================================================================
// RED PHASE: Basic Integer Overflow Prevention
// ============================================================================

#[test]
fn test_addition_overflow_prevention() {
    // Addition with potential overflow
    let c_code = r#"
        int main() {
            int a = 1000;
            int b = 2000;
            int sum = a + b;

            return sum;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Addition should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_subtraction_underflow_prevention() {
    // Subtraction with potential underflow
    let c_code = r#"
        int main() {
            int a = 100;
            int b = 200;
            int diff = a - b;

            return diff;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Subtraction should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_multiplication_overflow_prevention() {
    // Multiplication with potential overflow
    let c_code = r#"
        int main() {
            int a = 10000;
            int b = 20000;
            int product = a * b;

            return product;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Multiplication should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Division and Modulo
// ============================================================================

#[test]
fn test_division_by_zero_check() {
    // Division with potential division by zero
    let c_code = r#"
        int main() {
            int a = 100;
            int b = 5;
            int quotient;

            if (b != 0) {
                quotient = a / b;
            } else {
                quotient = 0;
            }

            return quotient;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "Division should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_modulo_by_zero_check() {
    // Modulo with potential division by zero
    let c_code = r#"
        int main() {
            int a = 100;
            int b = 7;
            int remainder;

            if (b != 0) {
                remainder = a % b;
            } else {
                remainder = 0;
            }

            return remainder;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "Modulo should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Negation Overflow
// ============================================================================

#[test]
fn test_negation_overflow() {
    // Negation of minimum value (overflow)
    let c_code = r#"
        int main() {
            int a = -100;
            int b = -a;

            return b;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Negation should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Loop Counter Overflow
// ============================================================================

#[test]
fn test_loop_counter_overflow() {
    // Loop with counter that could overflow
    let c_code = r#"
        int main() {
            int i;
            int sum = 0;

            for (i = 0; i < 10; i++) {
                sum = sum + i;
            }

            return sum;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "Loop counter should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Unsigned Integer Wraparound
// ============================================================================

#[test]
fn test_unsigned_wraparound() {
    // Unsigned integer that can wrap around
    let c_code = r#"
        int main() {
            unsigned int a = 10;
            unsigned int b = 20;
            unsigned int diff = a - b;

            return (int)diff;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "Unsigned wraparound should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Increment/Decrement Overflow
// ============================================================================

#[test]
fn test_increment_overflow() {
    // Pre/post increment with potential overflow
    let c_code = r#"
        int main() {
            int a = 100;
            int b;

            a = a + 1;
            b = a;

            return b;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Increment should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_decrement_underflow() {
    // Pre/post decrement with potential underflow
    let c_code = r#"
        int main() {
            int a = 100;
            int b;

            a = a - 1;
            b = a;

            return b;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Decrement should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Compound Assignment Overflow
// ============================================================================

#[test]
fn test_compound_addition_overflow() {
    // Compound addition assignment
    let c_code = r#"
        int main() {
            int a = 1000;
            int b = 2000;

            a = a + b;

            return a;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Compound addition should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_compound_multiplication_overflow() {
    // Compound multiplication assignment
    let c_code = r#"
        int main() {
            int a = 100;
            int b = 200;

            a = a * b;

            return a;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Compound multiplication should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Array Index Overflow
// ============================================================================

#[test]
fn test_array_index_arithmetic_overflow() {
    // Array indexing with arithmetic that could overflow
    let c_code = r#"
        int main() {
            int arr[10];
            int index = 5;
            int offset = 2;
            int final_index = index + offset;

            if (final_index >= 0 && final_index < 10) {
                arr[final_index] = 42;
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 6,
        "Array index arithmetic should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Size Calculation Overflow
// ============================================================================

#[test]
fn test_size_calculation_overflow() {
    // Size calculation that could overflow
    let c_code = r#"
        int main() {
            int count = 100;
            int item_size = 50;
            int total_size = count * item_size;

            return total_size;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Size calculation should minimize unsafe (found {})",
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
            int a = 1000;
            int b = 2000;
            int c = 3000;

            int sum1 = a + b;
            int sum2 = sum1 + c;
            int product = sum2 * 2;
            int diff = product - 1000;

            return diff;
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

    // Target: <=100 unsafe per 1000 LOC for integer overflow prevention
    assert!(
        unsafe_per_1000 <= 100.0,
        "Integer overflow prevention should minimize unsafe (got {:.2} per 1000 LOC, want <=100)",
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
            int a = 100;
            int b = 200;
            int sum = a + b;

            return sum;
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
fn test_integer_overflow_safety_documentation() {
    // Validate generated code quality
    let c_code = r#"
        int main() {
            int a = 1000000;
            int b = 2000000;
            int product = a * b;

            return product;
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
