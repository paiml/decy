//! Double Free Safety Integration Tests
//!
//! **RED PHASE**: Comprehensive tests for C double free → Safe Rust
//!
//! This validates that dangerous C double free patterns are transpiled to safe
//! Rust code where double frees are prevented by the ownership system.
//!
//! **Pattern**: EXTREME TDD - Test-First Development
//! **Reference**: CWE-415 (Double Free)
//!
//! **Safety Goal**: ≤100 unsafe blocks per 1000 LOC
//! **Validation**: Double frees impossible through ownership, RAII ensures single free

use decy_core::transpile;

// ============================================================================
// RED PHASE: Basic Double Free Prevention
// ============================================================================

#[test]
fn test_simple_malloc_free() {
    // Single malloc/free (safe)
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

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Single malloc/free should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_double_free_prevented_by_null_check() {
    // Double free prevented by NULL check (defensive C)
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* ptr = (int*)malloc(sizeof(int));
            if (ptr != 0) {
                *ptr = 42;
                free(ptr);
                ptr = 0;  // Set to NULL after free

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
        unsafe_count <= 4,
        "NULL check pattern should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Conditional Free Patterns
// ============================================================================

#[test]
fn test_conditional_free() {
    // Conditional free based on flag
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* ptr = (int*)malloc(sizeof(int));
            int should_free = 1;

            if (ptr != 0 && should_free) {
                *ptr = 42;
                free(ptr);
                should_free = 0;
            }

            if (should_free && ptr != 0) {
                free(ptr);  // Won't execute
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "Conditional free should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Array of Pointers
// ============================================================================

#[test]
fn test_array_of_pointers_free() {
    // Array of pointers, each freed once
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* array[3];
            int i;

            for (i = 0; i < 3; i++) {
                array[i] = (int*)malloc(sizeof(int));
                if (array[i] != 0) {
                    *array[i] = i;
                }
            }

            for (i = 0; i < 3; i++) {
                if (array[i] != 0) {
                    free(array[i]);
                    array[i] = 0;
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
        "Array of pointers should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Struct with Allocated Members
// ============================================================================

#[test]
fn test_struct_with_allocated_member() {
    // Struct with allocated member
    let c_code = r#"
        #include <stdlib.h>

        struct Container {
            int* data;
        };

        int main() {
            struct Container c;
            c.data = (int*)malloc(sizeof(int));

            if (c.data != 0) {
                *c.data = 42;
                free(c.data);
                c.data = 0;
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Struct member free should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Function Ownership Transfer
// ============================================================================

#[test]
fn test_function_takes_ownership() {
    // Function takes ownership and frees
    let c_code = r#"
        #include <stdlib.h>

        void cleanup(int* ptr) {
            if (ptr != 0) {
                free(ptr);
            }
        }

        int main() {
            int* ptr = (int*)malloc(sizeof(int));
            if (ptr != 0) {
                *ptr = 42;
                cleanup(ptr);  // Ownership transferred
                // ptr no longer valid
            }
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Ownership transfer should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Linked List Free
// ============================================================================

#[test]
fn test_linked_list_free() {
    // Linked list with proper free
    let c_code = r#"
        #include <stdlib.h>

        struct Node {
            int value;
            struct Node* next;
        };

        int main() {
            struct Node* head = (struct Node*)malloc(sizeof(struct Node));

            if (head != 0) {
                head->value = 1;
                head->next = (struct Node*)malloc(sizeof(struct Node));

                if (head->next != 0) {
                    head->next->value = 2;
                    head->next->next = 0;
                }

                struct Node* current = head;
                while (current != 0) {
                    struct Node* next = current->next;
                    free(current);
                    current = next;
                }
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 8,
        "Linked list free should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Error Path Free
// ============================================================================

#[test]
fn test_error_path_free() {
    // Free on error path
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* ptr1 = (int*)malloc(sizeof(int));
            int* ptr2 = (int*)malloc(sizeof(int));

            if (ptr1 == 0 || ptr2 == 0) {
                if (ptr1 != 0) {
                    free(ptr1);
                }
                if (ptr2 != 0) {
                    free(ptr2);
                }
                return 1;
            }

            *ptr1 = 42;
            *ptr2 = 84;

            free(ptr1);
            free(ptr2);

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 6,
        "Error path free should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Realloc Pattern
// ============================================================================

#[test]
fn test_realloc_pattern() {
    // realloc invalidates old pointer
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* ptr = (int*)malloc(sizeof(int) * 5);

            if (ptr != 0) {
                int* new_ptr = (int*)realloc(ptr, sizeof(int) * 10);

                if (new_ptr != 0) {
                    ptr = new_ptr;
                    free(ptr);
                } else {
                    free(ptr);
                }
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "Realloc pattern should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Multi-Free Protection
// ============================================================================

#[test]
fn test_multi_free_with_flags() {
    // Multiple potential free sites protected by flags
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* ptr = (int*)malloc(sizeof(int));
            int freed = 0;

            if (ptr != 0) {
                *ptr = 42;

                if (!freed) {
                    free(ptr);
                    freed = 1;
                }

                if (!freed) {
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
        unsafe_count <= 4,
        "Multi-free protection should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Aliased Pointer Free
// ============================================================================

#[test]
fn test_aliased_pointer() {
    // Aliased pointers (only one should be freed)
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* ptr1 = (int*)malloc(sizeof(int));
            int* ptr2 = ptr1;  // Alias

            if (ptr1 != 0) {
                *ptr1 = 42;
                free(ptr1);
                ptr1 = 0;
                ptr2 = 0;
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Aliased pointer should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: RAII Pattern
// ============================================================================

#[test]
fn test_raii_wrapper() {
    // RAII wrapper pattern
    let c_code = r#"
        #include <stdlib.h>

        struct Resource {
            int* data;
        };

        void init_resource(struct Resource* r) {
            r->data = (int*)malloc(sizeof(int));
            if (r->data != 0) {
                *r->data = 42;
            }
        }

        void cleanup_resource(struct Resource* r) {
            if (r->data != 0) {
                free(r->data);
                r->data = 0;
            }
        }

        int main() {
            struct Resource r;
            r.data = 0;

            init_resource(&r);
            cleanup_resource(&r);

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "RAII wrapper should minimize unsafe (found {})",
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

    // Target: <=100 unsafe per 1000 LOC for double free prevention
    assert!(
        unsafe_per_1000 <= 100.0,
        "Double free prevention should minimize unsafe (got {:.2} per 1000 LOC, want <=100)",
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
fn test_double_free_safety_documentation() {
    // Validate generated code quality
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            int* ptr = (int*)malloc(sizeof(int));
            if (ptr != 0) {
                free(ptr);
                ptr = 0;
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
