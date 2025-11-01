//! Use-After-Free Safety Integration Tests
//!
//! **RED PHASE**: Comprehensive tests for C use-after-free → Safe Rust
//!
//! This validates that dangerous C use-after-free patterns are transpiled
//! to safe Rust with proper lifetime management and no dangling pointers.
//!
//! **Pattern**: EXTREME TDD - Test-First Development
//! **Reference**: ISO C99 §7.22.3 - accessing freed memory is undefined behavior
//!
//! **Safety Goal**: ≤100 unsafe blocks per 1000 LOC
//! **Validation**: No dangling pointers, lifetime-safe, RAII patterns

use decy_core::transpile;

// ============================================================================
// RED PHASE: Basic Use-After-Free
// ============================================================================

#[test]
fn test_simple_use_after_free() {
    // Classic use-after-free pattern
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* ptr = (int*)malloc(sizeof(int));

            if (ptr != 0) {
                *ptr = 42;
                int value = *ptr;
                free(ptr);
                // ptr is now dangling (not accessed)
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
        "Use-after-free prevention should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_use_after_free_prevented() {
    // Use-after-free prevented by not accessing after free
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* ptr = (int*)malloc(sizeof(int));

            if (ptr != 0) {
                *ptr = 42;
                free(ptr);
                ptr = 0;  // Set to NULL after free
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "Nulling pointer after free should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Double-Free
// ============================================================================

#[test]
fn test_double_free_prevented() {
    // Double-free is undefined behavior
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* ptr = (int*)malloc(sizeof(int));

            if (ptr != 0) {
                *ptr = 42;
                free(ptr);
                ptr = 0;  // Prevents double-free

                if (ptr != 0) {
                    free(ptr);  // Won't execute
                }
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 6,
        "Double-free prevention should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Dangling Pointer via Return
// ============================================================================

#[test]
fn test_dangling_pointer_local_variable() {
    // Returning pointer to local variable
    let c_code = r#"
        int* get_pointer() {
            int value = 42;
            return &value;  // Dangling pointer!
        }

        int main() {
            int* ptr = get_pointer();
            // ptr is dangling (not dereferenced here)
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");
    assert!(
        result.contains("fn get_pointer"),
        "Should have get_pointer function"
    );

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Dangling pointer should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Use-After-Free in Loop
// ============================================================================

#[test]
fn test_use_after_free_in_loop() {
    // Free inside loop, must not use after
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* ptr = (int*)malloc(sizeof(int));

            if (ptr != 0) {
                *ptr = 42;

                for (int i = 0; i < 1; i++) {
                    int value = *ptr;
                    free(ptr);
                    ptr = 0;
                    // ptr not used after free
                    return value;
                }
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 6,
        "Use-after-free in loop should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Conditional Free
// ============================================================================

#[test]
fn test_conditional_free() {
    // Free in one branch but not another
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* ptr = (int*)malloc(sizeof(int));

            if (ptr != 0) {
                *ptr = 42;

                int condition = 1;
                if (condition) {
                    int value = *ptr;
                    free(ptr);
                    return value;
                } else {
                    int value = *ptr;
                    free(ptr);
                    return value;
                }
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 7,
        "Conditional free should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Linked List Use-After-Free
// ============================================================================

#[test]
fn test_linked_list_use_after_free() {
    // Linked list node free
    let c_code = r#"
        #include <stdlib.h>

        struct Node {
            int value;
            struct Node* next;
        };

        int main() {
            struct Node* node = (struct Node*)malloc(sizeof(struct Node));

            if (node != 0) {
                node->value = 42;
                node->next = 0;

                int value = node->value;
                free(node);
                node = 0;

                return value;
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 6,
        "Linked list use-after-free should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Array of Pointers Use-After-Free
// ============================================================================

#[test]
fn test_array_of_pointers_free() {
    // Array of allocated pointers
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* array[3];

            for (int i = 0; i < 3; i++) {
                array[i] = (int*)malloc(sizeof(int));
                if (array[i] != 0) {
                    *array[i] = i;
                }
            }

            int sum = 0;
            for (int i = 0; i < 3; i++) {
                if (array[i] != 0) {
                    sum += *array[i];
                    free(array[i]);
                    array[i] = 0;
                }
            }

            return sum;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 10,
        "Array of pointers should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Realloc Use-After-Free
// ============================================================================

#[test]
fn test_realloc_invalidates_old_pointer() {
    // realloc may invalidate old pointer
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* ptr = (int*)malloc(sizeof(int) * 5);

            if (ptr != 0) {
                ptr[0] = 42;

                int* new_ptr = (int*)realloc(ptr, sizeof(int) * 10);

                if (new_ptr != 0) {
                    // Use new_ptr, not ptr
                    int value = new_ptr[0];
                    free(new_ptr);
                    return value;
                }

                free(ptr);
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 8,
        "Realloc should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Function Argument Use-After-Free
// ============================================================================

#[test]
fn test_function_frees_argument() {
    // Function frees its argument
    let c_code = r#"
        #include <stdlib.h>

        void process_and_free(int* ptr) {
            if (ptr != 0) {
                int value = *ptr;
                free(ptr);
            }
        }

        int main() {
            int* ptr = (int*)malloc(sizeof(int));

            if (ptr != 0) {
                *ptr = 42;
                process_and_free(ptr);
                // ptr is now freed (don't use)
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");
    assert!(
        result.contains("fn process_and_free"),
        "Should have process_and_free function"
    );

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 6,
        "Function freeing argument should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Struct Member Use-After-Free
// ============================================================================

#[test]
fn test_struct_member_use_after_free() {
    // Struct containing allocated pointer
    let c_code = r#"
        #include <stdlib.h>

        struct Container {
            int* data;
            int size;
        };

        int main() {
            struct Container container;
            container.data = (int*)malloc(sizeof(int) * 10);
            container.size = 10;

            if (container.data != 0) {
                container.data[0] = 42;
                int value = container.data[0];
                free(container.data);
                container.data = 0;

                return value;
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 7,
        "Struct member use-after-free should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Global Pointer Use-After-Free
// ============================================================================

#[test]
fn test_global_pointer_lifetime() {
    // Global pointer management
    let c_code = r#"
        #include <stdlib.h>

        int* global_ptr = 0;

        void cleanup() {
            if (global_ptr != 0) {
                free(global_ptr);
                global_ptr = 0;
            }
        }

        int main() {
            global_ptr = (int*)malloc(sizeof(int));

            if (global_ptr != 0) {
                *global_ptr = 42;
                int value = *global_ptr;
                cleanup();
                // global_ptr is now NULL
                return value;
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");
    assert!(
        result.contains("fn cleanup"),
        "Should have cleanup function"
    );

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 7,
        "Global pointer should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: RAII Pattern (Constructor/Destructor)
// ============================================================================

#[test]
fn test_raii_pattern() {
    // RAII-like pattern in C (manual)
    let c_code = r#"
        #include <stdlib.h>

        struct Resource {
            int* data;
        };

        struct Resource create_resource() {
            struct Resource res;
            res.data = (int*)malloc(sizeof(int));
            if (res.data != 0) {
                *res.data = 42;
            }
            return res;
        }

        void destroy_resource(struct Resource* res) {
            if (res != 0 && res->data != 0) {
                free(res->data);
                res->data = 0;
            }
        }

        int main() {
            struct Resource res = create_resource();

            int value = 0;
            if (res.data != 0) {
                value = *res.data;
            }

            destroy_resource(&res);

            return value;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");
    assert!(
        result.contains("fn create_resource"),
        "Should have create_resource function"
    );
    assert!(
        result.contains("fn destroy_resource"),
        "Should have destroy_resource function"
    );

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 10,
        "RAII pattern should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Unsafe Density Target
// ============================================================================

#[test]
fn test_unsafe_block_count_target() {
    // CRITICAL: Validate overall unsafe minimization for use-after-free
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* ptr = (int*)malloc(sizeof(int));

            if (ptr != 0) {
                *ptr = 42;
                int value = *ptr;
                free(ptr);
                ptr = 0;
                return value;
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

    // Target: <=100 unsafe per 1000 LOC for use-after-free
    assert!(
        unsafe_per_1000 <= 100.0,
        "Use-after-free handling should minimize unsafe (got {:.2} per 1000 LOC, want <=100)",
        unsafe_per_1000
    );

    // Should have main function
    assert!(result.contains("fn main"), "Should generate main function");
}

// ============================================================================
// RED PHASE: Compilation and Correctness
// ============================================================================

#[test]
fn test_transpiled_lifetime_code_compiles() {
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
fn test_use_after_free_safety_documentation() {
    // Validate generated code quality
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* ptr = (int*)malloc(sizeof(int));

            if (ptr != 0) {
                *ptr = 42;
                int value = *ptr;
                free(ptr);
                return value;
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
