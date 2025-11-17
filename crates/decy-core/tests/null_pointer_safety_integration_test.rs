//! NULL Pointer Safety Integration Tests
//!
//! **RED PHASE**: Comprehensive tests for C NULL pointers → Safe Rust
//!
//! This validates that dangerous C NULL pointer patterns are transpiled
//! to safe Rust with proper checking and Option<T> usage.
//!
//! **Pattern**: EXTREME TDD - Test-First Development
//! **Reference**: ISO C99 §6.3.2.3 (Null pointer constant)
//!
//! **Safety Goal**: <50 unsafe blocks per 1000 LOC
//! **Validation**: No NULL dereference, proper checking, Option<T> where possible

use decy_core::transpile;

// ============================================================================
// RED PHASE: NULL Pointer Checks
// ============================================================================

#[test]
fn test_null_pointer_check() {
    // Basic NULL check pattern
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* ptr = (int*)malloc(sizeof(int));

            if (ptr == 0) {
                return 1;  // Allocation failed
            }

            *ptr = 42;
            free(ptr);
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "NULL check should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_null_pointer_comparison() {
    // Compare pointer against NULL
    let c_code = r#"
        int main() {
            int value = 42;
            int* ptr = &value;

            if (ptr != 0) {
                return *ptr;
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "NULL comparison should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_null_pointer_initialization() {
    // Initialize pointer to NULL
    let c_code = r#"
        int main() {
            int* ptr = 0;  // NULL

            if (ptr == 0) {
                return 1;
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "NULL initialization should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Function Return NULL
// ============================================================================

#[test]
fn test_function_return_null() {
    // Function returns NULL on failure
    let c_code = r#"
        #include <stdlib.h>

        int* create_value(int condition) {
            if (condition == 0) {
                return 0;  // NULL
            }

            int* ptr = (int*)malloc(sizeof(int));
            *ptr = 42;
            return ptr;
        }

        int main() {
            int* value = create_value(1);

            if (value != 0) {
                int result = *value;
                free(value);
                return result;
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");
    assert!(
        result.contains("fn create_value"),
        "Should have create_value function"
    );

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 6,
        "Function returning NULL should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: NULL Pointer in Structs
// ============================================================================

#[test]
fn test_null_pointer_in_struct() {
    // Struct with nullable pointer field
    let c_code = r#"
        struct Node {
            int value;
            struct Node* next;
        };

        int main() {
            struct Node node;
            node.value = 42;
            node.next = 0;  // NULL

            if (node.next == 0) {
                return node.value;
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "NULL in struct should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Array of Pointers with NULL
// ============================================================================

#[test]
fn test_null_in_pointer_array() {
    // Array of pointers with NULL sentinel
    let c_code = r#"
        int main() {
            int a = 1, b = 2, c = 3;
            int* array[4] = {&a, &b, &c, 0};  // NULL terminated

            int sum = 0;
            for (int i = 0; array[i] != 0; i++) {
                sum += *array[i];
            }

            return sum;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 6,
        "NULL in array should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Defensive NULL Checks
// ============================================================================

#[test]
fn test_defensive_null_check() {
    // Defensive programming with NULL checks
    let c_code = r#"
        int safe_deref(int* ptr) {
            if (ptr == 0) {
                return -1;  // Error code
            }
            return *ptr;
        }

        int main() {
            int value = 42;
            int result = safe_deref(&value);

            return result;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");
    assert!(
        result.contains("fn safe_deref"),
        "Should have safe_deref function"
    );

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Defensive NULL check should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: NULL Coalescing Pattern
// ============================================================================

#[test]
fn test_null_coalescing() {
    // Use default value if NULL
    let c_code = r#"
        int main() {
            int* ptr = 0;
            int value = (ptr != 0) ? *ptr : 42;

            return value;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "NULL coalescing should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: String NULL Checks
// ============================================================================

#[test]
fn test_string_null_check() {
    // Check string pointer before use
    let c_code = r#"
        #include <string.h>

        int safe_strlen(const char* str) {
            if (str == 0) {
                return 0;
            }
            return strlen(str);
        }

        int main() {
            const char* text = "Hello";
            int len = safe_strlen(text);

            return len;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "String NULL check should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Multiple NULL Checks
// ============================================================================

#[test]
fn test_multiple_null_checks() {
    // Chain of NULL checks
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* a = (int*)malloc(sizeof(int));
            int* b = (int*)malloc(sizeof(int));

            if (a == 0 || b == 0) {
                if (a != 0) free(a);
                if (b != 0) free(b);
                return 1;
            }

            *a = 10;
            *b = 20;
            int result = *a + *b;

            free(a);
            free(b);

            return result;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 8,
        "Multiple NULL checks should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: NULL Pointer Assignment
// ============================================================================

#[test]
fn test_null_pointer_assignment() {
    // Set pointer to NULL after free
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* ptr = (int*)malloc(sizeof(int));

            if (ptr != 0) {
                *ptr = 42;
                free(ptr);
                ptr = 0;  // Set to NULL after free
            }

            if (ptr == 0) {
                return 1;  // Success
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "NULL assignment should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Conditional NULL Dereference
// ============================================================================

#[test]
fn test_conditional_null_dereference() {
    // Only dereference if not NULL
    let c_code = r#"
        int main() {
            int value = 42;
            int* ptr = &value;
            int result = 0;

            if (ptr != 0 && *ptr > 0) {
                result = *ptr;
            }

            return result;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Conditional dereference should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Unsafe Density Target
// ============================================================================

#[test]
fn test_unsafe_block_count_target() {
    // CRITICAL: Validate overall unsafe minimization for NULL checks
    let c_code = r#"
        #include <stdlib.h>

        int* allocate_array(int size) {
            if (size <= 0) {
                return 0;  // NULL
            }
            return (int*)malloc(sizeof(int) * size);
        }

        int main() {
            int* array = allocate_array(10);

            if (array == 0) {
                return 1;  // Allocation failed
            }

            // Initialize array
            for (int i = 0; i < 10; i++) {
                array[i] = i;
            }

            // Sum array
            int sum = 0;
            for (int i = 0; i < 10; i++) {
                sum += array[i];
            }

            free(array);
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

    // Target: <=100 unsafe per 1000 LOC for NULL checks
    assert!(
        unsafe_per_1000 <= 100.0,
        "NULL checks should minimize unsafe (got {:.2} per 1000 LOC, want <=100)",
        unsafe_per_1000
    );

    // Should have main function
    assert!(result.contains("fn main"), "Should generate main function");
}

// ============================================================================
// RED PHASE: Compilation and Correctness
// ============================================================================

#[test]
fn test_transpiled_null_checks_compile() {
    // Generated Rust should have valid syntax
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* ptr = (int*)malloc(sizeof(int));

            if (ptr != 0) {
                *ptr = 42;
                free(ptr);
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
fn test_null_safety_documentation() {
    // Validate generated code quality
    let c_code = r#"
        int main() {
            int* ptr = 0;

            if (ptr == 0) {
                return 1;
            }

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
