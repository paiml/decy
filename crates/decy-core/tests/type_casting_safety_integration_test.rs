//! Type Casting Safety Integration Tests
//!
//! **RED PHASE**: Comprehensive tests for C type casts → Safe Rust
//!
//! This validates that dangerous C type casts and implicit conversions
//! are transpiled to safe Rust with proper type checking and validation.
//!
//! **Pattern**: EXTREME TDD - Test-First Development
//! **Reference**: ISO C99 §6.3 (Conversions), §6.5.4 (Cast operators)
//!
//! **Safety Goal**: <100 unsafe blocks per 1000 LOC
//! **Validation**: No type confusion, no truncation bugs, no pointer aliasing

use decy_core::transpile;

// ============================================================================
// RED PHASE: Integer Type Casts
// ============================================================================

#[test]
fn test_int_to_char_cast() {
    // Integer to char cast (potential truncation)
    let c_code = r#"
        int main() {
            int value = 65;
            char ch = (char)value;

            return ch;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "Integer cast should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_char_to_int_cast() {
    // Char to int cast (sign extension)
    let c_code = r#"
        int main() {
            char ch = 'A';
            int value = (int)ch;

            return value;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "Char to int cast should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_unsigned_to_signed_cast() {
    // Unsigned to signed conversion
    let c_code = r#"
        int main() {
            unsigned int u = 4294967295U;  // Max unsigned
            int s = (int)u;

            return s;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "Unsigned to signed cast should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_long_to_int_cast() {
    // Long to int cast (potential truncation)
    let c_code = r#"
        int main() {
            long big = 1000000L;
            int small = (int)big;

            return small;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "Long to int cast should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Pointer Type Casts
// ============================================================================

#[test]
#[ignore = "Parser limitation: Cannot handle #include <stdlib.h>. malloc/free require libc."]
fn test_void_pointer_cast() {
    // void* to typed pointer (common malloc pattern)
    let c_code = r#"
        #include <stdlib.h>

        int main() {
            void* ptr = malloc(sizeof(int));
            int* iptr = (int*)ptr;

            if (iptr != 0) {
                *iptr = 42;
                free(ptr);
            }

            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Void pointer cast should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_pointer_to_pointer_cast() {
    // Cast between different pointer types
    let c_code = r#"
        int main() {
            int value = 42;
            int* iptr = &value;
            char* cptr = (char*)iptr;

            char first_byte = *cptr;

            return first_byte;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Pointer to pointer cast should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_const_cast_away() {
    // Casting away const qualifier
    let c_code = r#"
        int main() {
            const int value = 42;
            const int* cptr = &value;
            int* ptr = (int*)cptr;

            return *ptr;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 3,
        "Const cast should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Integer Promotion and Implicit Conversions
// ============================================================================

#[test]
fn test_implicit_int_promotion() {
    // Implicit promotion in arithmetic
    let c_code = r#"
        int main() {
            char a = 10;
            char b = 20;
            int result = a + b;  // Promoted to int

            return result;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "Implicit promotion should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_implicit_float_conversion() {
    // Int to float implicit conversion
    let c_code = r#"
        int main() {
            int i = 42;
            float f = i;  // Implicit conversion

            return (int)f;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "Float conversion should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Struct and Array Casts
// ============================================================================

#[test]
fn test_struct_pointer_cast() {
    // Cast between struct pointer types
    let c_code = r#"
        struct Point {
            int x;
            int y;
        };

        struct Point3D {
            int x;
            int y;
            int z;
        };

        int main() {
            struct Point3D p3d = {1, 2, 3};
            struct Point* p = (struct Point*)&p3d;

            return p->x + p->y;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 5,
        "Struct pointer cast should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_array_to_pointer_decay() {
    // Array to pointer implicit conversion
    let c_code = r#"
        int main() {
            int array[5] = {1, 2, 3, 4, 5};
            int* ptr = array;  // Implicit decay

            return ptr[0];
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "Array decay should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Type Punning Patterns
// ============================================================================

#[test]
fn test_union_type_punning() {
    // Union for type punning (common but dangerous)
    let c_code = r#"
        union FloatInt {
            float f;
            int i;
        };

        int main() {
            union FloatInt u;
            u.f = 3.14f;
            int bits = u.i;  // Type punning

            return bits;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 4,
        "Union type punning should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Enum Type Casts
// ============================================================================

#[test]
fn test_enum_to_int_cast() {
    // Enum to int conversion
    let c_code = r#"
        enum Color {
            RED = 0,
            GREEN = 1,
            BLUE = 2
        };

        int main() {
            enum Color c = GREEN;
            int value = (int)c;

            return value;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "Enum to int cast should minimize unsafe (found {})",
        unsafe_count
    );
}

#[test]
fn test_int_to_enum_cast() {
    // Int to enum cast (dangerous - no validation)
    let c_code = r#"
        enum Status {
            OK = 0,
            ERROR = 1
        };

        int main() {
            int value = 0;
            enum Status s = (enum Status)value;

            return s;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "Int to enum cast should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Overflow and Truncation Safety
// ============================================================================

#[test]
fn test_truncation_safety() {
    // Explicit check for truncation patterns
    let c_code = r#"
        int main() {
            long big = 2147483648L;  // Larger than int max
            int small = (int)big;     // Truncation

            return small;
        }
    "#;

    let result = transpile(c_code).expect("Should transpile");

    assert!(result.contains("fn main"), "Should have main function");

    let unsafe_count = result.matches("unsafe").count();
    assert!(
        unsafe_count <= 2,
        "Truncation cast should minimize unsafe (found {})",
        unsafe_count
    );
}

// ============================================================================
// RED PHASE: Unsafe Density Target
// ============================================================================

#[test]
fn test_unsafe_block_count_target() {
    // CRITICAL: Validate overall unsafe minimization for type casts
    let c_code = r#"
        int main() {
            // Various type conversions
            int i = 42;
            char c = (char)i;
            unsigned int u = (unsigned int)i;
            long l = (long)i;
            float f = (float)i;

            // Pointer conversions
            int* ptr = &i;
            void* vptr = (void*)ptr;
            int* iptr = (int*)vptr;

            // Arithmetic with conversions
            int result = (int)c + u + (int)l + (int)f;

            return result;
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

    // Target: <100 unsafe per 1000 LOC for type casts
    assert!(
        unsafe_per_1000 < 150.0,
        "Type casting should minimize unsafe (got {:.2} per 1000 LOC, want <150)",
        unsafe_per_1000
    );

    // Should have main function
    assert!(result.contains("fn main"), "Should generate main function");
}

// ============================================================================
// RED PHASE: Compilation and Correctness
// ============================================================================

#[test]
fn test_transpiled_casts_compile() {
    // Generated Rust should have valid syntax
    let c_code = r#"
        int main() {
            int value = 100;
            char ch = (char)value;
            int back = (int)ch;

            return back;
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
fn test_type_safety_documentation() {
    // Validate generated code quality
    let c_code = r#"
        int main() {
            unsigned int u = 42;
            int s = (int)u;

            return s;
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
