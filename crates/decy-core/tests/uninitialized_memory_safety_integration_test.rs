//! Uninitialized Memory Safety Integration Tests
//!
//! **RED PHASE**: Comprehensive tests for C uninitialized memory → Safe Rust
//!
//! This validates that dangerous C uninitialized memory patterns are transpiled
//! to safe Rust with proper initialization and no undefined behavior.
//!
//! **Pattern**: EXTREME TDD - Test-First Development
//! **Reference**: ISO C99 §6.7.9 - uninitialized objects have indeterminate values
//!
//! **Safety Goal**: ≤50 unsafe blocks per 1000 LOC
//! **Validation**: All variables initialized, no undefined reads

use decy_core::transpile;

// ============================================================================
// RED PHASE: Uninitialized Local Variables
// ============================================================================

#[test]
fn test_initialized_local_variable() {
    // Properly initialized local variable
    let c_code = r#"
        int main() {
            int value = 42;
            return value;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "Initialized variable should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_uninitialized_local_variable() {
    // Uninitialized local variable (undefined behavior if read)
    let c_code = r#"
        int main() {
            int value;
            value = 42;  // Initialized before use
            return value;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "Variable initialized before use should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Uninitialized Arrays
// ============================================================================

#[test]
fn test_initialized_array() {
    // Array with initializer
    let c_code = r#"
        int main() {
            int array[5] = {1, 2, 3, 4, 5};
            return array[0];
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Initialized array should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_zero_initialized_array() {
    // Array initialized to zero
    let c_code = r#"
        int main() {
            int array[5] = {0};
            return array[0];
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Zero-initialized array should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_loop_initialized_array() {
    // Array initialized in loop
    let c_code = r#"
        int main() {
            int array[5];

            for (int i = 0; i < 5; i++) {
                array[i] = i;
            }

            return array[0];
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Loop-initialized array should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Uninitialized Structs
// ============================================================================

#[test]
fn test_initialized_struct() {
    // Struct with initializer
    let c_code = r#"
        struct Point {
            int x;
            int y;
        };

        int main() {
            struct Point p = {10, 20};
            return p.x;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Initialized struct should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_partially_initialized_struct() {
    // Struct with partial initializer (rest zeroed)
    let c_code = r#"
        struct Point {
            int x;
            int y;
            int z;
        };

        int main() {
            struct Point p = {10, 20};  // z is zero-initialized
            return p.x + p.y + p.z;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Partially initialized struct should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_field_by_field_initialized_struct() {
    // Struct initialized field by field
    let c_code = r#"
        struct Point {
            int x;
            int y;
        };

        int main() {
            struct Point p;
            p.x = 10;
            p.y = 20;
            return p.x + p.y;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Field-initialized struct should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Conditional Initialization
// ============================================================================

#[test]
fn test_conditional_initialization() {
    // Variable initialized in both branches
    let c_code = r#"
        int main() {
            int value;
            int condition = 1;

            if (condition) {
                value = 42;
            } else {
                value = 0;
            }

            return value;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Conditionally initialized should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Heap Memory Initialization
// ============================================================================

#[test]
fn test_malloc_uninitialized() {
    // malloc returns uninitialized memory
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* ptr = (int*)malloc(sizeof(int));

            if (ptr != 0) {
                *ptr = 42;  // Initialize before use
                int value = *ptr;
                free(ptr);
                return value;
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "malloc with initialization should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_calloc_zero_initialized() {
    // calloc returns zero-initialized memory
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* ptr = (int*)calloc(1, sizeof(int));

            if (ptr != 0) {
                int value = *ptr;  // Safe: calloc zeroes memory
                free(ptr);
                return value;
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "calloc should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Static and Global Variables
// ============================================================================

#[test]
fn test_static_variable_initialization() {
    // Static variables are zero-initialized by default
    let c_code = r#"
        int main() {
            static int counter = 0;
            counter++;
            return counter;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Static variable should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_global_variable_initialization() {
    // Global variables are zero-initialized by default
    let c_code = r#"
        int global_value = 42;

        int main() {
            return global_value;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Global variable should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Function Parameters
// ============================================================================

#[test]
fn test_function_parameter_passed() {
    // Function parameters are always initialized by caller
    let c_code = r#"
        int double_value(int x) {
            return x * 2;
        }

        int main() {
            int value = 21;
            return double_value(value);
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");
    assert!(
        result.contains("fn double_value"),
        "Should have double_value function"
    );

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Function parameter should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Return Value Initialization
// ============================================================================

#[test]
fn test_function_return_value() {
    // Function must initialize return value
    let c_code = r#"
        int get_value() {
            int result = 42;
            return result;
        }

        int main() {
            return get_value();
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");
    assert!(
        result.contains("fn get_value"),
        "Should have get_value function"
    );

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Return value should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Complex Initialization Patterns
// ============================================================================

#[test]
fn test_nested_struct_initialization() {
    // Nested structs
    let c_code = r#"
        struct Inner {
            int value;
        };

        struct Outer {
            struct Inner inner;
            int count;
        };

        int main() {
            struct Outer outer = {{42}, 10};
            return outer.inner.value + outer.count;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Nested struct should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_array_of_structs_initialization() {
    // Array of structs
    let c_code = r#"
        struct Point {
            int x;
            int y;
        };

        int main() {
            struct Point points[3] = {
                {1, 2},
                {3, 4},
                {5, 6}
            };

            return points[1].x;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Array of structs should minimize unsafe (found {})",
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
            int a = 10;
            int b = 20;
            int sum = a + b;

            int array[3] = {1, 2, 3};
            int product = array[0] * array[1];

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

    // Target: <=50 unsafe per 1000 LOC for initialization
    assert!(
        unsafe_per_1000 <= 50.0,
        "Initialization should minimize unsafe (got {:.2} per 1000 LOC, want <=50)",
        unsafe_per_1000
    );

    // Should have main function
    assert!(result.contains("fn main"), "Should generate main function");
}

// ============================================================================
// RED PHASE: Compilation and Correctness
// ============================================================================

#[test]
fn test_transpiled_initialization_compiles() {
    // Generated Rust should have valid syntax
    let c_code = r#"
        int main() {
            int value = 42;
            int array[3] = {1, 2, 3};

            return value + array[0];
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
fn test_initialization_safety_documentation() {
    // Validate generated code quality
    let c_code = r#"
        int main() {
            int value = 42;
            return value;
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
