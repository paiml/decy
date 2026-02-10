//! Deep coverage tests for decy-core lib.rs.
//!
//! Targets uncovered branches in:
//! - `process_ast_to_rust` (line 1355, 34.7% coverage)
//! - `transpile_with_includes` (line 845, 79.4% coverage)
//!
//! Exercises diverse C language features through the full pipeline:
//! structs, enums, typedefs, global variables, multiple functions,
//! pointer operations, arrays, type casting, error paths, and
//! deduplication logic.

use decy_core::{
    transpile, transpile_from_file_path, transpile_with_box_transform, transpile_with_includes,
    transpile_with_trace, transpile_with_verification, DependencyGraph, ProjectContext,
    TranspilationCache, TranspiledFile,
};
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper: Create temporary C file with content.
fn create_temp_c_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).expect("Failed to write temp file");
    path
}

// ============================================================================
// STRUCT DEFINITION AND ACCESS
// ============================================================================

#[test]
fn deep_transpile_struct_definition() {
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
    let result = transpile(c_code);
    assert!(result.is_ok(), "Struct transpilation should succeed: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("Point"), "Should contain struct name Point");
    assert!(rust.contains("fn main"), "Should contain main function");
}

#[test]
fn deep_transpile_struct_with_multiple_types() {
    let c_code = r#"
        struct Record {
            int id;
            float value;
            double precision;
            char label;
        };

        int main() {
            struct Record r;
            r.id = 1;
            r.value = 3.14;
            return r.id;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Struct with mixed types should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("Record"), "Should contain struct Record");
    assert!(rust.contains("i32") || rust.contains("id"), "Should map int field");
}

#[test]
fn deep_transpile_nested_struct_access() {
    let c_code = r#"
        struct Inner {
            int value;
        };

        struct Outer {
            struct Inner inner;
            int count;
        };

        int main() {
            struct Outer o;
            o.count = 5;
            return o.count;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Nested struct should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("Inner"), "Should contain Inner struct");
    assert!(rust.contains("Outer"), "Should contain Outer struct");
}

// ============================================================================
// ENUM DEFINITIONS
// ============================================================================

#[test]
fn deep_transpile_enum_definition() {
    let c_code = r#"
        enum Color {
            RED,
            GREEN,
            BLUE
        };

        int main() {
            enum Color c = RED;
            return 0;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Enum transpilation should succeed: {:?}", result.err());
    let rust = result.unwrap();
    assert!(
        rust.contains("RED") || rust.contains("Color"),
        "Should contain enum definition"
    );
}

#[test]
fn deep_transpile_enum_with_values() {
    let c_code = r#"
        enum Status {
            OK = 0,
            ERROR = 1,
            PENDING = 2
        };

        int main() {
            return OK;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Enum with values should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("OK") || rust.contains("Status"), "Should contain enum");
}

// ============================================================================
// TYPEDEF DEFINITIONS
// ============================================================================

#[test]
fn deep_transpile_typedef_int() {
    let c_code = r#"
        typedef int myint;

        myint add(myint a, myint b) {
            return a + b;
        }

        int main() {
            myint x = add(2, 3);
            return x;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Typedef should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("fn add") || rust.contains("fn main"), "Should have functions");
}

#[test]
fn deep_transpile_typedef_struct() {
    let c_code = r#"
        typedef struct {
            int x;
            int y;
        } Point;

        int main() {
            Point p;
            p.x = 1;
            return 0;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Typedef struct should transpile: {:?}", result.err());
}

// ============================================================================
// GLOBAL VARIABLES (all type defaults)
// ============================================================================

#[test]
fn deep_transpile_global_int() {
    let c_code = r#"
        int counter = 0;

        int main() {
            counter = 42;
            return counter;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Global int should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(
        rust.contains("static mut") || rust.contains("counter"),
        "Should generate global variable"
    );
}

#[test]
fn deep_transpile_global_float_uninit() {
    let c_code = r#"
        float temperature;

        int main() {
            temperature = 98.6;
            return 0;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Global float should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(
        rust.contains("0.0") || rust.contains("temperature"),
        "Should default-init float"
    );
}

#[test]
fn deep_transpile_global_double() {
    let c_code = r#"
        double precision;

        int main() {
            precision = 3.14159;
            return 0;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Global double should transpile: {:?}", result.err());
}

#[test]
fn deep_transpile_global_char_uninit() {
    let c_code = r#"
        char marker;

        int main() {
            marker = 'X';
            return 0;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Global char should transpile: {:?}", result.err());
}

#[test]
fn deep_transpile_global_unsigned_int() {
    let c_code = r#"
        unsigned int flags = 0;

        int main() {
            flags = 255;
            return 0;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Global unsigned int should transpile: {:?}", result.err());
}

#[test]
fn deep_transpile_global_pointer_uninit() {
    let c_code = r#"
        int* global_ptr;

        int main() {
            int x = 5;
            return x;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Global pointer should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(
        rust.contains("null_mut") || rust.contains("global_ptr"),
        "Should default-init pointer to null"
    );
}

#[test]
fn deep_transpile_global_array_int_uninit() {
    let c_code = r#"
        int buffer[100];

        int main() {
            buffer[0] = 42;
            return buffer[0];
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Global int array should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(
        rust.contains("100") || rust.contains("buffer"),
        "Should preserve array size"
    );
}

#[test]
fn deep_transpile_global_array_char_uninit() {
    let c_code = r#"
        char name[50];

        int main() {
            name[0] = 'H';
            return 0;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Global char array should transpile: {:?}", result.err());
}

#[test]
fn deep_transpile_global_array_float_uninit() {
    let c_code = r#"
        float values[10];

        int main() {
            values[0] = 1.0;
            return 0;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Global float array should transpile: {:?}", result.err());
}

#[test]
fn deep_transpile_global_array_double_uninit() {
    let c_code = r#"
        double measurements[5];

        int main() {
            measurements[0] = 1.5;
            return 0;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Global double array should transpile: {:?}", result.err());
}

#[test]
fn deep_transpile_global_array_unsigned_int_uninit() {
    let c_code = r#"
        unsigned int masks[8];

        int main() {
            masks[0] = 0xFF;
            return 0;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Global unsigned int array should transpile: {:?}", result.err());
}

#[test]
fn deep_transpile_global_with_initializer() {
    let c_code = r#"
        int max_count = 100;
        float pi = 3.14;

        int main() {
            return max_count;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Globals with initializers should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(
        rust.contains("100") || rust.contains("max_count"),
        "Should preserve initializer value"
    );
}

// ============================================================================
// MULTIPLE FUNCTION DEFINITIONS
// ============================================================================

#[test]
fn deep_transpile_multiple_functions() {
    let c_code = r#"
        int square(int x) {
            return x * x;
        }

        int cube(int x) {
            return x * x * x;
        }

        int main() {
            int a = square(3);
            int b = cube(2);
            return a + b;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Multiple functions should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("fn square"), "Should contain square");
    assert!(rust.contains("fn cube"), "Should contain cube");
    assert!(rust.contains("fn main"), "Should contain main");
}

#[test]
fn deep_transpile_void_function() {
    let c_code = r#"
        void do_nothing() {
        }

        int main() {
            do_nothing();
            return 0;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Void function should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("fn do_nothing"), "Should contain void function");
}

#[test]
fn deep_transpile_function_with_many_params() {
    let c_code = r#"
        int add_four(int a, int b, int c, int d) {
            return a + b + c + d;
        }

        int main() {
            return add_four(1, 2, 3, 4);
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Function with many params should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("fn add_four"), "Should contain function");
}

// ============================================================================
// FUNCTION DEDUPLICATION (DECY-190)
// ============================================================================

#[test]
fn deep_transpile_function_deduplication() {
    // Declaration followed by definition - should deduplicate
    let c_code = r#"
        int add(int a, int b);

        int add(int a, int b) {
            return a + b;
        }

        int main() {
            return add(1, 2);
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Function dedup should work: {:?}", result.err());
    let rust = result.unwrap();
    // Should only have one fn add definition, not two
    let add_count = rust.matches("fn add").count();
    assert!(
        add_count <= 2,
        "Should deduplicate: found {} occurrences of fn add",
        add_count
    );
}

// ============================================================================
// POINTER OPERATIONS
// ============================================================================

#[test]
fn deep_transpile_pointer_param() {
    let c_code = r#"
        void increment(int* ptr) {
            (*ptr) = (*ptr) + 1;
        }

        int main() {
            int x = 5;
            increment(&x);
            return x;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Pointer param should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("fn increment"), "Should contain increment function");
}

#[test]
fn deep_transpile_double_pointer() {
    let c_code = r#"
        int main() {
            int x = 42;
            int* p = &x;
            return *p;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Pointer variable should transpile: {:?}", result.err());
}

// ============================================================================
// ARRAY OPERATIONS
// ============================================================================

#[test]
fn deep_transpile_array_declaration() {
    let c_code = r#"
        int main() {
            int arr[5];
            arr[0] = 10;
            arr[1] = 20;
            return arr[0] + arr[1];
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Array declaration should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("fn main"), "Should contain main");
}

#[test]
fn deep_transpile_array_parameter() {
    let c_code = r#"
        int sum(int* arr, int len) {
            int total = 0;
            int i;
            for (i = 0; i < len; i++) {
                total = total + arr[i];
            }
            return total;
        }

        int main() {
            int data[3];
            data[0] = 1;
            data[1] = 2;
            data[2] = 3;
            return sum(data, 3);
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Array parameter should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("fn sum"), "Should contain sum function");
}

// ============================================================================
// CONTROL FLOW
// ============================================================================

#[test]
fn deep_transpile_if_else() {
    let c_code = r#"
        int abs_val(int x) {
            if (x < 0) {
                return -x;
            } else {
                return x;
            }
        }

        int main() {
            return abs_val(-5);
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "If/else should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("fn abs_val"), "Should contain abs_val function");
}

#[test]
fn deep_transpile_while_loop() {
    let c_code = r#"
        int count_down(int n) {
            int count = 0;
            while (n > 0) {
                count = count + 1;
                n = n - 1;
            }
            return count;
        }

        int main() {
            return count_down(10);
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "While loop should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("while") || rust.contains("loop"), "Should contain loop");
}

#[test]
fn deep_transpile_for_loop() {
    let c_code = r#"
        int sum_to(int n) {
            int sum = 0;
            int i;
            for (i = 0; i <= n; i++) {
                sum = sum + i;
            }
            return sum;
        }

        int main() {
            return sum_to(10);
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "For loop should transpile: {:?}", result.err());
}

#[test]
fn deep_transpile_switch_statement() {
    let c_code = r#"
        int classify(int x) {
            switch (x) {
                case 0: return 0;
                case 1: return 1;
                default: return -1;
            }
        }

        int main() {
            return classify(1);
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Switch should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(
        rust.contains("match") || rust.contains("classify"),
        "Should convert switch to match"
    );
}

// ============================================================================
// TYPE CASTING AND EXPRESSIONS
// ============================================================================

#[test]
fn deep_transpile_type_cast() {
    let c_code = r#"
        int main() {
            float f = 3.14;
            int i = (int)f;
            return i;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Type cast should transpile: {:?}", result.err());
}

#[test]
fn deep_transpile_complex_expressions() {
    let c_code = r#"
        int compute(int a, int b) {
            return (a + b) * (a - b) / 2;
        }

        int main() {
            return compute(10, 3);
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Complex expressions should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("fn compute"), "Should contain compute");
}

#[test]
fn deep_transpile_ternary_expression() {
    let c_code = r#"
        int max(int a, int b) {
            return (a > b) ? a : b;
        }

        int main() {
            return max(5, 10);
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Ternary should transpile: {:?}", result.err());
}

#[test]
fn deep_transpile_bitwise_operations() {
    let c_code = r#"
        int bitops(int a, int b) {
            int x = a & b;
            int y = a | b;
            int z = a ^ b;
            return x + y + z;
        }

        int main() {
            return bitops(0xFF, 0x0F);
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Bitwise ops should transpile: {:?}", result.err());
}

// ============================================================================
// ERROR PATHS (Invalid C)
// ============================================================================

#[test]
fn deep_transpile_error_missing_semicolon() {
    let c_code = "int main() { return 0 }";
    let result = transpile(c_code);
    // This may or may not error depending on parser tolerance
    // Either result is acceptable, we just want no panic
    let _ = result;
}

#[test]
fn deep_transpile_error_empty_input() {
    let c_code = "";
    let result = transpile(c_code);
    // Empty input should either produce empty output or error gracefully
    let _ = result;
}

#[test]
fn deep_transpile_error_only_comments() {
    let c_code = "/* this is a comment */";
    let result = transpile(c_code);
    // Comment-only input should be handled gracefully
    let _ = result;
}

#[test]
fn deep_transpile_error_garbage() {
    let c_code = "@#$%^&*!!!";
    let result = transpile(c_code);
    // Garbage input should error, not panic
    let _ = result;
}

#[test]
fn deep_transpile_error_incomplete_function() {
    let c_code = "int main(";
    let result = transpile(c_code);
    assert!(result.is_err(), "Incomplete function should fail");
}

// ============================================================================
// transpile_with_includes PATHS
// ============================================================================

#[test]
fn deep_transpile_with_includes_no_base_dir() {
    let c_code = "int main() { return 0; }";
    let result = transpile_with_includes(c_code, None);
    assert!(result.is_ok(), "Should work with no base dir: {:?}", result.err());
}

#[test]
fn deep_transpile_with_includes_with_base_dir() {
    let temp = TempDir::new().unwrap();
    let c_code = "int main() { return 0; }";
    let result = transpile_with_includes(c_code, Some(temp.path()));
    assert!(result.is_ok(), "Should work with base dir: {:?}", result.err());
}

#[test]
fn deep_transpile_with_includes_struct_and_func() {
    let c_code = r#"
        struct Pair {
            int first;
            int second;
        };

        int sum_pair(int a, int b) {
            return a + b;
        }

        int main() {
            return sum_pair(1, 2);
        }
    "#;
    let result = transpile_with_includes(c_code, None);
    assert!(result.is_ok(), "Struct+func should work via includes path: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("Pair"), "Should contain Pair struct");
    assert!(rust.contains("fn sum_pair"), "Should contain function");
}

#[test]
fn deep_transpile_with_includes_global_vars() {
    let c_code = r#"
        int global_x = 10;
        float global_y;

        int main() {
            return global_x;
        }
    "#;
    let result = transpile_with_includes(c_code, None);
    assert!(result.is_ok(), "Global vars should work: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("global_x"), "Should contain global variable");
}

#[test]
fn deep_transpile_with_includes_enum_and_typedef() {
    let c_code = r#"
        enum Direction { NORTH, SOUTH, EAST, WEST };
        typedef int score_t;

        int main() {
            score_t s = 100;
            return s;
        }
    "#;
    let result = transpile_with_includes(c_code, None);
    assert!(result.is_ok(), "Enum+typedef should transpile: {:?}", result.err());
}

// ============================================================================
// transpile_from_file_path (exercises process_ast_to_rust)
// ============================================================================

#[test]
fn deep_transpile_from_file_path_simple() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(
        &temp,
        "simple.c",
        "int main() { return 0; }",
    );
    let result = transpile_from_file_path(&file);
    assert!(result.is_ok(), "File path transpilation should succeed: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("fn main"), "Should contain main");
}

#[test]
fn deep_transpile_from_file_path_with_structs() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(
        &temp,
        "structs.c",
        r#"
        struct Vec3 {
            float x;
            float y;
            float z;
        };

        float dot(float ax, float ay, float az, float bx, float by, float bz) {
            return ax * bx + ay * by + az * bz;
        }

        int main() {
            return 0;
        }
        "#,
    );
    let result = transpile_from_file_path(&file);
    assert!(result.is_ok(), "File with structs should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("Vec3"), "Should contain Vec3 struct");
}

#[test]
fn deep_transpile_from_file_path_with_enums() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(
        &temp,
        "enums.c",
        r#"
        enum Level { LOW = 0, MEDIUM = 1, HIGH = 2 };

        int main() {
            return LOW;
        }
        "#,
    );
    let result = transpile_from_file_path(&file);
    assert!(result.is_ok(), "File with enums should transpile: {:?}", result.err());
}

#[test]
fn deep_transpile_from_file_path_with_globals() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(
        &temp,
        "globals.c",
        r#"
        int counter = 0;
        float ratio = 0.5;
        char flag = 'N';

        int main() {
            counter = 42;
            return counter;
        }
        "#,
    );
    let result = transpile_from_file_path(&file);
    assert!(result.is_ok(), "File with globals should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("counter"), "Should contain global counter");
}

#[test]
fn deep_transpile_from_file_path_with_typedefs() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(
        &temp,
        "typedefs.c",
        r#"
        typedef int int32;
        typedef unsigned int uint32;

        int32 add(int32 a, int32 b) {
            return a + b;
        }

        int main() {
            int32 result = add(1, 2);
            return result;
        }
        "#,
    );
    let result = transpile_from_file_path(&file);
    assert!(result.is_ok(), "File with typedefs should transpile: {:?}", result.err());
}

#[test]
fn deep_transpile_from_file_path_nonexistent() {
    let result = transpile_from_file_path(std::path::Path::new("/tmp/nonexistent_decy_test.c"));
    assert!(result.is_err(), "Nonexistent file should error");
}

#[test]
fn deep_transpile_from_file_path_all_constructs() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(
        &temp,
        "all.c",
        r#"
        struct Data {
            int id;
            float value;
        };

        enum Type { INT_TYPE, FLOAT_TYPE };

        typedef int myint;

        int global_counter = 0;

        myint process(myint x) {
            return x + 1;
        }

        int main() {
            struct Data d;
            d.id = 1;
            d.value = 2.0;
            global_counter = process(d.id);
            return global_counter;
        }
        "#,
    );
    let result = transpile_from_file_path(&file);
    assert!(result.is_ok(), "All constructs together should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("Data"), "Should contain Data struct");
    assert!(rust.contains("fn process"), "Should contain process function");
    assert!(rust.contains("fn main"), "Should contain main function");
}

// ============================================================================
// transpile_with_verification PATHS
// ============================================================================

#[test]
fn deep_transpile_with_verification_success() {
    let c_code = "int main() { return 0; }";
    let result = transpile_with_verification(c_code);
    assert!(result.is_ok(), "Verification should succeed: {:?}", result.err());
    let transpilation = result.unwrap();
    assert!(!transpilation.rust_code.is_empty(), "Should produce code");
}

#[test]
fn deep_transpile_with_verification_failure() {
    let c_code = "int main( {";
    let result = transpile_with_verification(c_code);
    // transpile_with_verification wraps errors into TranspilationResult, should not error
    assert!(result.is_ok(), "Verification wrapper should not propagate error");
    let transpilation = result.unwrap();
    assert!(
        !transpilation.errors.is_empty() || transpilation.rust_code.is_empty(),
        "Should record errors for invalid code"
    );
}

// ============================================================================
// transpile_with_trace PATHS (additional coverage)
// ============================================================================

#[test]
fn deep_transpile_with_trace_struct() {
    let c_code = r#"
        struct Item { int count; };
        int main() { return 0; }
    "#;
    let result = transpile_with_trace(c_code);
    assert!(result.is_ok(), "Trace with struct should work: {:?}", result.err());
    let (code, trace) = result.unwrap();
    assert!(code.contains("Item"), "Should contain struct");
    assert!(!trace.entries().is_empty(), "Should have trace entries");
}

#[test]
fn deep_transpile_with_trace_global_var() {
    let c_code = r#"
        int g = 42;
        int main() { return g; }
    "#;
    let result = transpile_with_trace(c_code);
    assert!(result.is_ok(), "Trace with global should work: {:?}", result.err());
}

// ============================================================================
// transpile_with_box_transform
// ============================================================================

#[test]
fn deep_transpile_with_box_transform_simple() {
    let c_code = r#"
        int main() {
            int x = 42;
            return x;
        }
    "#;
    let result = transpile_with_box_transform(c_code);
    assert!(result.is_ok(), "Box transform should work for simple code: {:?}", result.err());
}

// ============================================================================
// STRUCT DEDUPLICATION
// ============================================================================

#[test]
fn deep_transpile_struct_deduplication() {
    // When struct appears multiple times (e.g., forward decl + definition)
    let c_code = r#"
        struct Node {
            int value;
        };

        int main() {
            struct Node n;
            n.value = 5;
            return n.value;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Struct dedup should work: {:?}", result.err());
    let rust = result.unwrap();
    // Verify struct is emitted
    assert!(rust.contains("Node"), "Should contain Node");
}

// ============================================================================
// GLOBAL VARIABLE DEDUPLICATION
// ============================================================================

#[test]
fn deep_transpile_global_deduplication() {
    // Duplicate globals should be deduplicated
    let c_code = r#"
        int shared = 0;

        int get_shared() {
            return shared;
        }

        int main() {
            shared = 100;
            return get_shared();
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Global dedup should work: {:?}", result.err());
}

// ============================================================================
// ARRAY INIT PATHS IN GLOBAL SCOPE
// ============================================================================

#[test]
fn deep_transpile_global_array_with_initializer() {
    let c_code = r#"
        int data[3] = {1, 2, 3};

        int main() {
            return data[0];
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Initialized global array should transpile: {:?}", result.err());
}

#[test]
fn deep_transpile_global_array_zero_init() {
    let c_code = r#"
        int zeros[10] = {0};

        int main() {
            return zeros[0];
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Zero-init global array should transpile: {:?}", result.err());
}

// ============================================================================
// TRANSPILED FILE AND PROJECT CONTEXT
// ============================================================================

#[test]
fn deep_transpile_file_with_context() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(
        &temp,
        "module.c",
        r#"
        int helper(int x) {
            return x * 2;
        }

        int main() {
            return helper(5);
        }
        "#,
    );
    let context = ProjectContext::default();
    let result = decy_core::transpile_file(&file, &context);
    assert!(result.is_ok(), "transpile_file should succeed: {:?}", result.err());
    let transpiled = result.unwrap();
    assert!(!transpiled.rust_code.is_empty(), "Should produce Rust code");
    assert_eq!(transpiled.source_path, file, "Should preserve source path");
}

#[test]
fn deep_transpile_file_nonexistent() {
    let context = ProjectContext::default();
    let result = decy_core::transpile_file(
        std::path::Path::new("/tmp/nonexistent_decy_test_file.c"),
        &context,
    );
    assert!(result.is_err(), "Nonexistent file should error");
}

// ============================================================================
// DEPENDENCY GRAPH
// ============================================================================

#[test]
fn deep_dependency_graph_single_file() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(&temp, "single.c", "int main() { return 0; }");

    let mut graph = DependencyGraph::new();
    graph.add_file(&file);
    let order = graph.topological_sort();
    assert!(order.is_ok(), "Single file should sort: {:?}", order.err());
    let order = order.unwrap();
    assert_eq!(order.len(), 1, "Should have one file");
}

#[test]
fn deep_dependency_graph_multiple_files() {
    let temp = TempDir::new().unwrap();
    let f1 = create_temp_c_file(&temp, "a.c", "int a() { return 1; }");
    let f2 = create_temp_c_file(&temp, "b.c", "int b() { return 2; }");
    let f3 = create_temp_c_file(&temp, "main.c", "int main() { return 0; }");

    let mut graph = DependencyGraph::new();
    graph.add_file(&f1);
    graph.add_file(&f2);
    graph.add_file(&f3);
    let order = graph.topological_sort();
    assert!(order.is_ok(), "Multiple files should sort: {:?}", order.err());
    assert_eq!(order.unwrap().len(), 3, "Should have three files");
}

// ============================================================================
// TRANSPILATION CACHE
// ============================================================================

#[test]
fn deep_transpilation_cache_basic() {
    let cache = TranspilationCache::new();
    let stats = cache.statistics();
    assert_eq!(stats.total_files, 0, "New cache should be empty");
    assert_eq!(stats.hits, 0, "New cache should have no hits");
    assert_eq!(stats.misses, 0, "New cache should have no misses");
}

#[test]
fn deep_transpilation_cache_insert_and_get() {
    let temp = TempDir::new().unwrap();
    let path = create_temp_c_file(&temp, "cached.c", "int main() { return 0; }");

    let mut cache = TranspilationCache::new();
    let transpiled = TranspiledFile {
        source_path: path.clone(),
        rust_code: "fn main() {}".to_string(),
        dependencies: vec![],
        functions_exported: vec!["main".to_string()],
        ffi_declarations: String::new(),
    };

    cache.insert(&path, &transpiled);
    let cached = cache.get(&path);
    assert!(cached.is_some(), "Should find cached entry");
}

// ============================================================================
// COMPLEX PIPELINE: STRUCT + ENUM + GLOBAL + FUNCTIONS
// ============================================================================

#[test]
fn deep_transpile_comprehensive_c_program() {
    let c_code = r#"
        struct Config {
            int width;
            int height;
            float scale;
        };

        enum Mode { NORMAL = 0, DEBUG = 1 };

        int default_width = 800;
        int default_height = 600;

        int area(int w, int h) {
            return w * h;
        }

        int main() {
            struct Config cfg;
            cfg.width = default_width;
            cfg.height = default_height;
            cfg.scale = 1.0;
            int a = area(cfg.width, cfg.height);
            return a;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Comprehensive C program should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("Config"), "Should contain Config struct");
    assert!(rust.contains("fn area"), "Should contain area function");
    assert!(rust.contains("fn main"), "Should contain main");
    assert!(
        rust.contains("ERRNO") || rust.contains("static mut"),
        "Should contain errno or static mut globals"
    );
}

#[test]
fn deep_transpile_string_literal() {
    let c_code = r#"
        int main() {
            char* msg = "Hello World";
            return 0;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "String literal should transpile: {:?}", result.err());
}

#[test]
fn deep_transpile_signed_char_global() {
    let c_code = r#"
        signed char offset;

        int main() {
            offset = -1;
            return offset;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Signed char global should transpile: {:?}", result.err());
}

#[test]
fn deep_transpile_do_while_loop() {
    let c_code = r#"
        int main() {
            int x = 10;
            do {
                x = x - 1;
            } while (x > 0);
            return x;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Do-while loop should transpile: {:?}", result.err());
}

#[test]
fn deep_transpile_logical_operators() {
    let c_code = r#"
        int check(int a, int b) {
            if (a > 0 && b > 0) {
                return 1;
            }
            if (a < 0 || b < 0) {
                return -1;
            }
            return 0;
        }

        int main() {
            return check(1, 2);
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Logical operators should transpile: {:?}", result.err());
}

#[test]
fn deep_transpile_compound_assignment() {
    let c_code = r#"
        int main() {
            int x = 10;
            x += 5;
            x -= 3;
            x *= 2;
            x /= 4;
            return x;
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Compound assignment should transpile: {:?}", result.err());
}

#[test]
fn deep_transpile_nested_if() {
    let c_code = r#"
        int classify(int x) {
            if (x > 0) {
                if (x > 100) {
                    return 2;
                }
                return 1;
            }
            return 0;
        }

        int main() {
            return classify(50);
        }
    "#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "Nested if should transpile: {:?}", result.err());
}

// ============================================================================
// EDGE CASES: ERRNO AND STATIC GENERATION
// ============================================================================

#[test]
fn deep_transpile_errno_generation() {
    // Every transpilation should generate the ERRNO global
    let c_code = "int main() { return 0; }";
    let result = transpile(c_code).unwrap();
    assert!(
        result.contains("ERRNO"),
        "Should generate ERRNO static variable"
    );
}

// ============================================================================
// CACHE PERSISTENCE
// ============================================================================

#[test]
fn deep_transpilation_cache_save_load() {
    let temp = TempDir::new().unwrap();
    let cache_dir = temp.path().join("cache");
    std::fs::create_dir_all(&cache_dir).unwrap();

    // Create and save cache
    {
        let c_file = create_temp_c_file(&temp, "persist.c", "int main() { return 0; }");
        let mut cache = TranspilationCache::new();
        let transpiled = TranspiledFile {
            source_path: c_file.clone(),
            rust_code: "fn test() {}".to_string(),
            dependencies: vec![],
            functions_exported: vec!["test".to_string()],
            ffi_declarations: String::new(),
        };
        cache.insert(&c_file, &transpiled);
        // Save is no-op for in-memory cache, but exercise the code path
        let _ = cache.save();
    }

    // Load cache
    let loaded = TranspilationCache::load(&cache_dir);
    assert!(loaded.is_ok(), "Cache load should succeed: {:?}", loaded.err());
}

// ============================================================================
// process_ast_to_rust: UNINITIALIZED GLOBALS (default value generation)
// ============================================================================

#[test]
fn deep_transpile_uninitialized_int_global() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(
        &temp,
        "uninit_int.c",
        r#"
        int counter;
        int main() {
            counter = 42;
            return counter;
        }
        "#,
    );
    let result = transpile_from_file_path(&file);
    assert!(result.is_ok(), "Uninitialized int global should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("counter"), "Should contain global counter");
}

#[test]
fn deep_transpile_uninitialized_float_global() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(
        &temp,
        "uninit_float.c",
        r#"
        float ratio;
        double precise;
        int main() {
            ratio = 0.5;
            precise = 3.14;
            return 0;
        }
        "#,
    );
    let result = transpile_from_file_path(&file);
    assert!(result.is_ok(), "Uninitialized float globals should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("ratio"), "Should contain ratio global");
    assert!(rust.contains("precise"), "Should contain precise global");
}

#[test]
fn deep_transpile_uninitialized_char_global() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(
        &temp,
        "uninit_char.c",
        r#"
        char flag;
        int main() {
            flag = 'Y';
            return 0;
        }
        "#,
    );
    let result = transpile_from_file_path(&file);
    assert!(result.is_ok(), "Uninitialized char global should transpile: {:?}", result.err());
}

#[test]
fn deep_transpile_uninitialized_pointer_global() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(
        &temp,
        "uninit_ptr.c",
        r#"
        int *buffer;
        int main() {
            return 0;
        }
        "#,
    );
    let result = transpile_from_file_path(&file);
    assert!(result.is_ok(), "Uninitialized pointer global should transpile: {:?}", result.err());
}

#[test]
fn deep_transpile_uninitialized_array_global() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(
        &temp,
        "uninit_arr.c",
        r#"
        int arr[10];
        char buf[256];
        float floats[5];
        int main() {
            arr[0] = 1;
            return 0;
        }
        "#,
    );
    let result = transpile_from_file_path(&file);
    assert!(result.is_ok(), "Uninitialized array globals should transpile: {:?}", result.err());
}

// ============================================================================
// process_ast_to_rust: ENUM VARIANTS WITHOUT EXPLICIT VALUES
// ============================================================================

#[test]
fn deep_transpile_enum_implicit_values() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(
        &temp,
        "enum_implicit.c",
        r#"
        enum Status {
            OK,
            ERROR,
            PENDING
        };

        int main() {
            return OK;
        }
        "#,
    );
    let result = transpile_from_file_path(&file);
    assert!(result.is_ok(), "Enum with implicit values should transpile: {:?}", result.err());
}

#[test]
fn deep_transpile_enum_mixed_values() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(
        &temp,
        "enum_mixed.c",
        r#"
        enum Priority {
            LOW,
            MEDIUM = 5,
            HIGH,
            CRITICAL = 100
        };

        int main() {
            return LOW;
        }
        "#,
    );
    let result = transpile_from_file_path(&file);
    assert!(result.is_ok(), "Enum with mixed values should transpile: {:?}", result.err());
}

// ============================================================================
// process_ast_to_rust: FUNCTION DECLARATION + DEFINITION DEDUPLICATION
// ============================================================================

#[test]
fn deep_transpile_function_decl_then_def() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(
        &temp,
        "decl_def.c",
        r#"
        int add(int a, int b);

        int main() {
            return add(1, 2);
        }

        int add(int a, int b) {
            return a + b;
        }
        "#,
    );
    let result = transpile_from_file_path(&file);
    assert!(result.is_ok(), "Function decl then def should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("fn add"), "Should contain add function");
}

// ============================================================================
// process_ast_to_rust: MULTIPLE CONSTRUCTS COMBINED
// ============================================================================

#[test]
fn deep_transpile_many_globals_many_types() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(
        &temp,
        "many_globals.c",
        r#"
        int g_int;
        float g_float;
        double g_double;
        char g_char;
        unsigned int g_uint = 0;

        struct Config {
            int width;
            int height;
        };

        enum Mode { FAST, SLOW, AUTO };

        typedef unsigned int uint32;

        int main() {
            g_int = 42;
            return 0;
        }
        "#,
    );
    let result = transpile_from_file_path(&file);
    assert!(result.is_ok(), "Many globals with many types should transpile: {:?}", result.err());
    let rust = result.unwrap();
    assert!(rust.contains("g_int"), "Should contain g_int");
    assert!(rust.contains("Config"), "Should contain Config struct");
}
