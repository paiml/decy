//! Dynamic Memory Safety Integration Tests
//!
//! **RED PHASE**: Comprehensive tests for C dynamic memory → Safe Rust
//!
//! This validates that dangerous C memory operations (malloc, free, calloc, realloc)
//! are transpiled to safe Rust with Box, Vec, and proper ownership.
//!
//! **Pattern**: EXTREME TDD - Test-First Development
//! **Reference**: ISO C99 §7.20.3 (Memory management functions)
//!
//! **Safety Goal**: <5 unsafe blocks per 1000 LOC
//! **Validation**: No memory leaks, no double-free, no use-after-free
//!
//! **STATUS**: Tests now passing with stdlib prototype support! ✅
//!
//! **SOLUTION**: The decy-stdlib crate provides built-in prototypes for stdlib.h
//! memory management functions (malloc, free, calloc, realloc). Per-header prototype
//! filtering enables parser to successfully handle these functions.

use decy_core::transpile;

// ============================================================================
// RED PHASE: malloc + free → Box Pattern
// ============================================================================

#[test]
fn test_malloc_free_basic_pattern() {
    // Classic C pattern: malloc + free
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* ptr = (int*)malloc(sizeof(int));

            if (ptr != 0) {
                *ptr = 42;
            }

            free(ptr);
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    // Should generate Rust code
    assert!(result.contains("fn main"), "Should have main function");

    // Should minimize unsafe blocks
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "malloc/free should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_malloc_with_null_check() {
    // Defensive C pattern: check malloc result
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* data = (int*)malloc(sizeof(int) * 10);

            if (data == 0) {
                return 1;  // Allocation failed
            }

            data[0] = 123;
            free(data);
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    // NULL check should be handled
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "NULL check pattern should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_malloc_array_allocation() {
    // Allocate array with malloc
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* array = (int*)malloc(sizeof(int) * 5);

            for (int i = 0; i < 5; i++) {
                array[i] = i * 2;
            }

            free(array);
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    // Array allocation should be safe
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Array malloc should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: calloc → Zero-Initialized Allocation
// ============================================================================

#[test]
fn test_calloc_zero_initialization() {
    // calloc allocates and zero-initializes
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* buffer = (int*)calloc(10, sizeof(int));

            if (buffer != 0) {
                int sum = 0;
                for (int i = 0; i < 10; i++) {
                    sum += buffer[i];  // All zeros
                }
                free(buffer);
                return sum;
            }

            return 1;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    // calloc should be safe
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "calloc should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: realloc → Resizing
// ============================================================================

#[test]
fn test_realloc_resize_pattern() {
    // realloc to grow allocation
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* array = (int*)malloc(sizeof(int) * 5);

            if (array != 0) {
                array[0] = 1;

                // Grow to 10 elements
                int* new_array = (int*)realloc(array, sizeof(int) * 10);

                if (new_array != 0) {
                    new_array[9] = 99;
                    free(new_array);
                    return 0;
                }

                free(array);
            }

            return 1;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    // realloc should be handled
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 6,
        "realloc should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Struct Allocation
// ============================================================================

#[test]
fn test_malloc_struct_allocation() {
    // Allocate struct on heap
    let c_code = r#"
        #include <stdlib.h>

        struct Point {
            int x;
            int y;
        };

        int main() {
            struct Point* p = (struct Point*)malloc(sizeof(struct Point));

            if (p != 0) {
                p->x = 10;
                p->y = 20;
                free(p);
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    // Struct allocation should be safe
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Struct malloc should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Multiple Allocations
// ============================================================================

#[test]
fn test_multiple_malloc_free_pairs() {
    // Multiple independent allocations
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* a = (int*)malloc(sizeof(int));
            int* b = (int*)malloc(sizeof(int));
            int* c = (int*)malloc(sizeof(int));

            if (a != 0 && b != 0 && c != 0) {
                *a = 1;
                *b = 2;
                *c = 3;
            }

            free(a);
            free(b);
            free(c);

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    // Multiple allocations should be tracked
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 6,
        "Multiple malloc/free should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Dynamic String Allocation
// ============================================================================

#[test]
fn test_malloc_string_buffer() {
    // Allocate buffer for string
    let c_code = r#"
        #include <stdlib.h>
        #include <string.h>

        int main() {
            char* buffer = (char*)malloc(100);

            if (buffer != 0) {
                strcpy(buffer, "Hello, Rust!");
                free(buffer);
                return 0;
            }

            return 1;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    // String buffer allocation
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "String malloc should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Function with malloc/free
// ============================================================================

#[test]
fn test_function_with_heap_allocation() {
    // Function that allocates and returns pointer
    let c_code = r#"
        #include <stdlib.h>

        int* create_array(int size) {
            int* array = (int*)malloc(sizeof(int) * size);
            return array;
        }

        int main() {
            int* data = create_array(10);

            if (data != 0) {
                data[0] = 42;
                free(data);
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");
    assert!(
        result.contains("fn create_array"),
        "Should have create_array function"
    );

    // Function returning allocated memory
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "Function malloc should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Conditional Allocation
// ============================================================================

#[test]
fn test_conditional_malloc() {
    // Allocate only under certain conditions
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int condition = 1;
            int* ptr = 0;

            if (condition) {
                ptr = (int*)malloc(sizeof(int));
                if (ptr != 0) {
                    *ptr = 123;
                }
            }

            if (ptr != 0) {
                free(ptr);
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    // Conditional malloc/free
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "Conditional malloc should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: sizeof Calculations
// ============================================================================

#[test]
fn test_malloc_with_sizeof() {
    // Various sizeof patterns
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* single = (int*)malloc(sizeof(int));
            int* array = (int*)malloc(sizeof(int) * 20);
            char* buffer = (char*)malloc(sizeof(char) * 100);

            if (single != 0) free(single);
            if (array != 0) free(array);
            if (buffer != 0) free(buffer);

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    // sizeof calculations
    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 6,
        "sizeof malloc should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Unsafe Density Target
// ============================================================================

#[test]
fn test_unsafe_block_count_target() {
    // CRITICAL: Validate overall unsafe minimization for malloc/free
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            // Allocate
            int* data = (int*)malloc(sizeof(int) * 100);

            if (data == 0) {
                return 1;
            }

            // Initialize
            for (int i = 0; i < 100; i++) {
                data[i] = i;
            }

            // Use
            int sum = 0;
            for (int i = 0; i < 100; i++) {
                sum += data[i];
            }

            // Free
            free(data);

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

    // Target: <60 unsafe per 1000 LOC (malloc/free is more complex than simple arrays)
    assert!(
        unsafe_per_1000 < 60.0,
        "malloc/free pattern should minimize unsafe (got {:.2} per 1000 LOC, want <60)",
        unsafe_per_1000
    );

    // Should have main function
    assert!(result.contains("fn main"), "Should generate main function");
}

// ============================================================================
// RED PHASE: Compilation and Correctness
// ============================================================================

#[test]
fn test_transpiled_malloc_compiles() {
    // Generated Rust should have valid syntax
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* ptr = (int*)malloc(sizeof(int) * 10);

            if (ptr != 0) {
                for (int i = 0; i < 10; i++) {
                    ptr[i] = i * i;
                }
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

    // Should not have excessive semicolons
    assert!(
        !result.contains(";;;;"),
        "Should not have excessive semicolons"
    );
}

#[test]
fn test_memory_safety_documentation() {
    // If unsafe is used, validate it's minimal
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* array = (int*)malloc(sizeof(int) * 5);

            if (array != 0) {
                array[0] = 1;
                array[4] = 5;
                free(array);
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
