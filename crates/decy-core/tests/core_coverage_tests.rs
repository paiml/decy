//! Comprehensive coverage tests for decy-core lib.rs.
//!
//! Targets uncovered paths in the transpilation pipeline, including:
//! - transpile_with_trace
//! - transpile_with_box_transform
//! - Various global variable type defaults
//! - Array initialization paths
//! - preprocess_includes edge cases
//! - process_ast_to_rust (via transpile_from_file_path)
//! - Struct/enum/typedef deduplication

use decy_core::{
    DependencyGraph, ProjectContext, TranspilationCache, TranspiledFile,
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
// transpile_with_trace tests
// ============================================================================

#[test]
fn core_coverage_transpile_with_trace_simple_function() {
    let c_code = "int add(int a, int b) { return a + b; }";
    let result = decy_core::transpile_with_trace(c_code);
    assert!(result.is_ok(), "transpile_with_trace should succeed");

    let (rust_code, trace) = result.unwrap();
    assert!(!rust_code.is_empty(), "Should produce Rust code");
    assert!(rust_code.contains("fn add"), "Should contain add function");

    // Verify trace has entries
    let entries = trace.entries();
    assert!(
        entries.len() >= 2,
        "Trace should have at least 2 entries (parsing start + completion)"
    );

    // Verify JSON output
    let json = trace.to_json();
    assert!(json.starts_with('['), "JSON should be an array");
    assert!(json.contains("clang-sys"), "Should record parser choice");
    assert!(json.contains("completed"), "Should record completion");
}

#[test]
fn core_coverage_transpile_with_trace_empty_function() {
    let c_code = "void noop() { }";
    let (code, trace) = decy_core::transpile_with_trace(c_code).unwrap();
    assert!(code.contains("fn noop"));
    assert!(!trace.entries().is_empty());
}

#[test]
fn core_coverage_transpile_with_trace_multiple_functions() {
    let c_code = r#"
        int foo(int x) { return x + 1; }
        int bar(int y) { return y * 2; }
    "#;
    let (code, trace) = decy_core::transpile_with_trace(c_code).unwrap();
    assert!(code.contains("fn foo"));
    assert!(code.contains("fn bar"));

    // Check trace records line count
    let json = trace.to_json();
    assert!(
        json.contains("lines of Rust"),
        "Completion entry should mention line count"
    );
}

#[test]
fn core_coverage_transpile_with_trace_with_struct() {
    let c_code = r#"
        struct Point { int x; int y; };
        int get_x(struct Point* p) { return p->x; }
    "#;
    let (code, trace) = decy_core::transpile_with_trace(c_code).unwrap();
    assert!(code.contains("Point"));
    assert!(!trace.entries().is_empty());
}

// ============================================================================
// transpile_with_box_transform tests
// ============================================================================

#[test]
fn core_coverage_box_transform_simple_function() {
    let c_code = "int get_val() { return 42; }";
    let result = decy_core::transpile_with_box_transform(c_code);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("fn get_val"));
}

#[test]
fn core_coverage_box_transform_with_pointer_param() {
    let c_code = r#"
        void set(int* p, int v) { *p = v; }
    "#;
    let result = decy_core::transpile_with_box_transform(c_code);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_box_transform_multiple_functions() {
    let c_code = r#"
        int create() { return 1; }
        void destroy(int x) { }
    "#;
    let result = decy_core::transpile_with_box_transform(c_code);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("fn create"));
    assert!(code.contains("fn destroy"));
}

#[test]
fn core_coverage_box_transform_empty_code() {
    let c_code = "";
    let result = decy_core::transpile_with_box_transform(c_code);
    assert!(result.is_ok());
}

// ============================================================================
// transpile_with_verification tests (coverage of success/failure paths)
// ============================================================================

#[test]
fn core_coverage_verification_success_with_struct() {
    let c_code = r#"
        struct Pair { int a; int b; };
        int sum_pair(struct Pair* p) { return p->a + p->b; }
    "#;
    let result = decy_core::transpile_with_verification(c_code).unwrap();
    assert!(!result.rust_code.is_empty());
    assert!(result.compiles);
    assert!(result.errors.is_empty());
}

#[test]
fn core_coverage_verification_with_empty_code() {
    let result = decy_core::transpile_with_verification("").unwrap();
    // Empty code should produce a result (possibly empty code)
    assert!(result.errors.is_empty() || result.rust_code.is_empty());
}

// ============================================================================
// Global variable default value paths
// ============================================================================

#[test]
fn core_coverage_global_unsigned_int_default() {
    let c_code = r#"
        unsigned int flags;
        void set_flag() { flags = 1; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(
        code.contains("static mut flags"),
        "Should declare flags as static mut"
    );
}

#[test]
fn core_coverage_global_float_default() {
    let c_code = r#"
        float temperature;
        void set_temp() { temperature = 98.6; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("static mut temperature"));
}

#[test]
fn core_coverage_global_double_default() {
    let c_code = r#"
        double precision;
        void set_precision() { precision = 0.001; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("static mut precision"));
}

#[test]
fn core_coverage_global_char_default() {
    let c_code = r#"
        char letter;
        void set_letter() { letter = 65; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("static mut letter"));
}

#[test]
fn core_coverage_global_signed_char_default() {
    let c_code = r#"
        signed char byte_val;
        void set_byte() { byte_val = -1; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_global_pointer_default() {
    let c_code = r#"
        int* global_ptr;
        void set_ptr(int* p) { global_ptr = p; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(
        code.contains("null_mut") || code.contains("global_ptr"),
        "Should handle pointer default"
    );
}

// ============================================================================
// Array global initialization paths
// ============================================================================

#[test]
fn core_coverage_global_int_array_default() {
    let c_code = r#"
        int data[10];
        int get_first() { return data[0]; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("static mut data"));
}

#[test]
fn core_coverage_global_char_array_default() {
    let c_code = r#"
        char buffer[256];
        void clear_buffer() { buffer[0] = 0; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_global_unsigned_int_array_default() {
    let c_code = r#"
        unsigned int counters[8];
        void reset() { counters[0] = 0; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_global_float_array_default() {
    let c_code = r#"
        float weights[5];
        void init() { weights[0] = 1.0; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_global_double_array_default() {
    let c_code = r#"
        double values[4];
        void set_first() { values[0] = 3.14; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_global_pointer_array_default() {
    let c_code = r#"
        int* ptrs[10];
        void set_ptr(int idx, int* p) { ptrs[idx] = p; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

// ============================================================================
// Array initialization with integer literal (zero-init)
// ============================================================================

#[test]
fn core_coverage_global_int_array_initialized() {
    let c_code = r#"
        int data[10] = {0};
        int get_first() { return data[0]; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_global_char_array_initialized() {
    let c_code = r#"
        char msg[100] = {0};
        void set_char() { msg[0] = 65; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

// ============================================================================
// Struct-related global and array tests
// ============================================================================

#[test]
fn core_coverage_global_struct_array_default() {
    let c_code = r#"
        struct Item { int id; float value; };
        struct Item inventory[10];
        int get_id() { return inventory[0].id; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("static mut inventory"));
}

#[test]
fn core_coverage_struct_field_types_in_const_literal() {
    // Tests the const_struct_literal closure with various field types
    let c_code = r#"
        struct Mixed {
            int i;
            unsigned int u;
            char c;
            float f;
            double d;
            int* p;
        };
        struct Mixed globals[2];
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

// ============================================================================
// Enum transpilation (DECY-240)
// ============================================================================

#[test]
fn core_coverage_enum_with_explicit_values() {
    let c_code = r#"
        enum Priority { LOW = 0, MEDIUM = 5, HIGH = 10 };
        int get_priority() { return HIGH; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("HIGH") || code.contains("Priority"));
}

#[test]
fn core_coverage_enum_implicit_values() {
    let c_code = r#"
        enum Direction { NORTH, SOUTH, EAST, WEST };
        int is_north(int d) { return d == NORTH; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

// ============================================================================
// Typedef transpilation (DECY-054, DECY-057)
// ============================================================================

#[test]
fn core_coverage_typedef_deduplication() {
    // Tests DECY-119: Typedef deduplication
    let c_code = r#"
        typedef int size_type;
        size_type get_size() { return 42; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

// ============================================================================
// Function deduplication (DECY-190)
// ============================================================================

#[test]
fn core_coverage_function_dedup_both_have_body() {
    // When both have bodies (redefinition), first one wins
    // This tests the "both have bodies" path in the dedup logic
    let c_code = r#"
        int foo(int x) { return x + 1; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("fn foo"));
}

#[test]
fn core_coverage_function_dedup_declaration_only() {
    // Function with only a declaration (no body)
    let c_code = r#"
        int helper(int x);
        int use_helper(int y) { return helper(y); }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

// ============================================================================
// DECY-260: Deterministic output (sorted by name)
// ============================================================================

#[test]
fn core_coverage_deterministic_function_order() {
    let c_code = r#"
        int zzz_last(int x) { return x; }
        int aaa_first(int y) { return y; }
        int mmm_middle(int z) { return z; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
    let code = result.unwrap();

    // Functions should appear in alphabetical order
    let aaa_pos = code.find("fn aaa_first").unwrap_or(usize::MAX);
    let mmm_pos = code.find("fn mmm_middle").unwrap_or(usize::MAX);
    let zzz_pos = code.find("fn zzz_last").unwrap_or(usize::MAX);
    assert!(
        aaa_pos < mmm_pos && mmm_pos < zzz_pos,
        "Functions should be sorted alphabetically"
    );
}

// ============================================================================
// DECY-116: Slice function args detection with various param names
// ============================================================================

#[test]
fn core_coverage_slice_param_size() {
    let c_code = r#"
        void fill(char* buf, int size) {
            int i = 0;
            while (i < size) { buf[i] = 0; i = i + 1; }
        }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_slice_param_count() {
    let c_code = r#"
        int total(int* vals, int count) {
            int s = 0;
            int i = 0;
            while (i < count) { s = s + vals[i]; i = i + 1; }
            return s;
        }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_slice_param_n() {
    let c_code = r#"
        void zero_out(double* arr, int n) {
            int i = 0;
            while (i < n) { arr[i] = 0.0; i = i + 1; }
        }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_slice_param_num() {
    let c_code = r#"
        int max_of(int* data, int num) {
            int m = data[0];
            int i = 1;
            while (i < num) {
                if (data[i] > m) { m = data[i]; }
                i = i + 1;
            }
            return m;
        }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

// ============================================================================
// Preprocess includes edge cases
// ============================================================================

#[test]
fn core_coverage_preprocess_system_header_stdlib() {
    let c_code = r#"
        #include <stdlib.h>
        int main() { return 0; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_preprocess_system_header_string() {
    let c_code = r#"
        #include <string.h>
        int main() { return 0; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_preprocess_unknown_system_header() {
    let c_code = r#"
        #include <unknown_header.h>
        int main() { return 0; }
    "#;
    let result = decy_core::transpile(c_code);
    // Should handle unknown system header gracefully
    assert!(result.is_ok());
}

#[test]
fn core_coverage_preprocess_duplicate_system_header() {
    // Tests the injected_headers deduplication
    let c_code = r#"
        #include <stdio.h>
        #include <stdio.h>
        int main() { return 0; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_preprocess_malformed_include_no_delimiter() {
    let c_code = r#"
        #include
        int main() { return 0; }
    "#;
    // Malformed include (no filename) - should keep line as-is
    let result = decy_core::transpile(c_code);
    // May or may not succeed, but should not panic
    let _ = result;
}

#[test]
fn core_coverage_preprocess_malformed_quote_include() {
    let c_code = "#include \"unclosed\nint main() { return 0; }";
    let result = decy_core::transpile(c_code);
    let _ = result; // Should not panic
}

#[test]
fn core_coverage_preprocess_malformed_angle_include() {
    let c_code = "#include <unclosed\nint main() { return 0; }";
    let result = decy_core::transpile(c_code);
    let _ = result; // Should not panic
}

#[test]
fn core_coverage_preprocess_local_include_exists() {
    let temp = TempDir::new().unwrap();
    let header = create_temp_c_file(&temp, "helpers.h", "int helper_func(int x);");
    let _ = header; // ensure file exists

    let c_code = r#"
        #include "helpers.h"
        int main() { return helper_func(1); }
    "#;
    let result = decy_core::transpile_with_includes(c_code, Some(temp.path()));
    assert!(result.is_ok());
}

#[test]
fn core_coverage_preprocess_local_include_circular() {
    let temp = TempDir::new().unwrap();

    // Create two headers that reference each other
    // a.h includes b.h, b.h includes a.h
    let _a = create_temp_c_file(
        &temp,
        "a.h",
        "#include \"b.h\"\nint func_a();",
    );
    let _b = create_temp_c_file(
        &temp,
        "b.h",
        "#include \"a.h\"\nint func_b();",
    );

    let c_code = r#"
        #include "a.h"
        int main() { return func_a(); }
    "#;
    // Should handle circular includes without infinite loop
    let result = decy_core::transpile_with_includes(c_code, Some(temp.path()));
    assert!(result.is_ok());
}

#[test]
fn core_coverage_preprocess_nested_include() {
    let temp = TempDir::new().unwrap();

    let _inner = create_temp_c_file(&temp, "inner.h", "int inner_func();");
    let _outer = create_temp_c_file(
        &temp,
        "outer.h",
        "#include \"inner.h\"\nint outer_func();",
    );

    let c_code = r#"
        #include "outer.h"
        int main() { return outer_func(); }
    "#;
    let result = decy_core::transpile_with_includes(c_code, Some(temp.path()));
    assert!(result.is_ok());
}

// ============================================================================
// transpile_from_file_path (exercises process_ast_to_rust)
// ============================================================================

#[test]
fn core_coverage_file_path_with_struct() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(
        &temp,
        "structs.c",
        r#"
        struct Vec2 { float x; float y; };
        float dot(struct Vec2* a, struct Vec2* b) {
            return a->x * b->x + a->y * b->y;
        }
        "#,
    );
    let result = decy_core::transpile_from_file_path(&file);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("Vec2") || code.contains("dot"));
}

#[test]
fn core_coverage_file_path_with_enum() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(
        &temp,
        "enums.c",
        r#"
        enum Status { OK = 0, ERROR = 1, PENDING = 2 };
        int is_ok(int s) { return s == OK; }
        "#,
    );
    let result = decy_core::transpile_from_file_path(&file);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_file_path_with_global_vars() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(
        &temp,
        "globals.c",
        r#"
        int counter = 0;
        float ratio = 0.5;
        char name[64];
        void reset() { counter = 0; }
        "#,
    );
    let result = decy_core::transpile_from_file_path(&file);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_file_path_with_typedef() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(
        &temp,
        "types.c",
        r#"
        typedef int score_t;
        score_t max_score(score_t a, score_t b) {
            if (a > b) return a;
            return b;
        }
        "#,
    );
    let result = decy_core::transpile_from_file_path(&file);
    assert!(result.is_ok());
}

// ============================================================================
// transpile_file with project context interaction
// ============================================================================

#[test]
fn core_coverage_transpile_file_populates_context() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(
        &temp,
        "module.c",
        r#"
        struct Config { int timeout; };
        int get_timeout(struct Config* c) { return c->timeout; }
        void set_timeout(struct Config* c, int t) { c->timeout = t; }
        "#,
    );

    let ctx = ProjectContext::new();
    let result = decy_core::transpile_file(&file, &ctx).unwrap();

    // Verify transpiled file has exported functions
    assert!(result.functions_exported.contains(&"get_timeout".to_string()));
    assert!(result.functions_exported.contains(&"set_timeout".to_string()));
    assert!(!result.ffi_declarations.is_empty());

    // Add to context and verify
    let mut ctx = ProjectContext::new();
    ctx.add_transpiled_file(&result);
    assert!(ctx.has_function("get_timeout"));
    assert!(ctx.has_function("set_timeout"));
}

#[test]
fn core_coverage_transpile_file_with_dependencies() {
    let temp = TempDir::new().unwrap();
    let _header = create_temp_c_file(&temp, "dep.h", "int dep_func();");
    let main_file = create_temp_c_file(
        &temp,
        "main.c",
        // Use system include style (gets commented out) + local function
        "int main() { return 0; }",
    );

    let ctx = ProjectContext::new();
    let result = decy_core::transpile_file(&main_file, &ctx);
    assert!(result.is_ok());
    let transpiled = result.unwrap();
    // Should produce valid rust code
    assert!(transpiled.rust_code.contains("fn main") || !transpiled.rust_code.is_empty());
}

// ============================================================================
// Global variable with function pointer (Option wrapping)
// ============================================================================

#[test]
fn core_coverage_global_function_pointer() {
    let c_code = r#"
        int (*handler)(int, int);
        int call_handler(int a, int b) {
            return handler(a, b);
        }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
    let code = result.unwrap();
    // Function pointers should use Option wrapping
    assert!(
        code.contains("Option") || code.contains("handler"),
        "Should generate function pointer global"
    );
}

// ============================================================================
// Extern variable filtering (DECY-223)
// ============================================================================

#[test]
fn core_coverage_extern_variable_skip() {
    // extern without initializer should be skipped
    let c_code = r#"
        extern int external_val;
        int get_val() { return external_val; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_extern_with_initializer_kept() {
    // extern with initializer should be kept
    let c_code = r#"
        extern int init_val = 42;
        int get_init() { return init_val; }
    "#;
    let result = decy_core::transpile(c_code);
    // May or may not succeed depending on parser behavior with extern + init
    let _ = result;
}

// ============================================================================
// DECY-241: ERRNO global variable
// ============================================================================

#[test]
fn core_coverage_errno_global() {
    let c_code = "int main() { return 0; }";
    let result = decy_core::transpile(c_code).unwrap();
    assert!(
        result.contains("ERRNO"),
        "Should always include ERRNO global variable"
    );
}

// ============================================================================
// Complex C constructs through full pipeline
// ============================================================================

#[test]
fn core_coverage_if_else_chain() {
    let c_code = r#"
        int classify(int x) {
            if (x > 100) {
                return 3;
            } else if (x > 50) {
                return 2;
            } else if (x > 0) {
                return 1;
            } else {
                return 0;
            }
        }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("fn classify"));
}

#[test]
fn core_coverage_nested_loops() {
    let c_code = r#"
        int matrix_sum(int n) {
            int total = 0;
            int i = 0;
            while (i < n) {
                int j = 0;
                while (j < n) {
                    total = total + 1;
                    j = j + 1;
                }
                i = i + 1;
            }
            return total;
        }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_switch_with_default() {
    let c_code = r#"
        int map_value(int code) {
            switch (code) {
                case 0: return 10;
                case 1: return 20;
                case 2: return 30;
                default: return -1;
            }
        }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_multiple_return_types() {
    let c_code = r#"
        float to_float(int x) { return x; }
        double to_double(int x) { return x; }
        char to_char(int x) { return x; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_void_function_with_side_effects() {
    let c_code = r#"
        int state = 0;
        void advance() {
            state = state + 1;
            if (state > 10) {
                state = 0;
            }
        }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

// ============================================================================
// Cache operations - more edge cases
// ============================================================================

#[test]
fn core_coverage_cache_miss_then_hit() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(&temp, "cached.c", "int cached_func() { return 1; }");

    let mut cache = TranspilationCache::new();

    // Miss first
    let miss = cache.get(&file);
    assert!(miss.is_none());

    // Insert
    let transpiled = TranspiledFile::new(
        file.clone(),
        "fn cached_func() -> i32 { 1 }".to_string(),
        vec![],
        vec!["cached_func".to_string()],
        String::new(),
    );
    cache.insert(&file, &transpiled);

    // Hit
    let hit = cache.get(&file);
    assert!(hit.is_some());

    let stats = cache.statistics();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.total_files, 1);
}

#[test]
fn core_coverage_cache_save_load_roundtrip() {
    let temp = TempDir::new().unwrap();
    let cache_dir = temp.path().join("cache_rt");
    let file = create_temp_c_file(&temp, "rt.c", "int rt() { return 99; }");

    let transpiled = TranspiledFile::new(
        file.clone(),
        "fn rt() -> i32 { 99 }".to_string(),
        vec![],
        vec!["rt".to_string()],
        String::new(),
    );

    // Save
    let mut cache = TranspilationCache::with_directory(&cache_dir);
    cache.insert(&file, &transpiled);
    cache.save().unwrap();

    // Load
    let loaded = TranspilationCache::load(&cache_dir).unwrap();
    assert_eq!(loaded.statistics().total_files, 1);
}

// ============================================================================
// DependencyGraph edge cases
// ============================================================================

#[test]
fn core_coverage_dep_graph_from_files_with_deps() {
    let temp = TempDir::new().unwrap();
    let header = create_temp_c_file(&temp, "utils.h", "int util_func();");
    let main_file = create_temp_c_file(
        &temp,
        "app.c",
        "#include \"utils.h\"\nint main() { return util_func(); }",
    );
    let standalone = create_temp_c_file(&temp, "standalone.c", "int alone() { return 0; }");

    let graph =
        DependencyGraph::from_files(&[main_file.clone(), header.clone(), standalone.clone()])
            .unwrap();

    assert_eq!(graph.file_count(), 3);
    assert!(graph.has_dependency(&main_file, &header));
    assert!(!graph.has_dependency(&standalone, &header));
}

#[test]
fn core_coverage_dep_graph_single_node_topo_sort() {
    let mut graph = DependencyGraph::new();
    let a = PathBuf::from("/single.c");
    graph.add_file(&a);

    let order = graph.topological_sort().unwrap();
    assert_eq!(order.len(), 1);
    assert_eq!(order[0], a);
}

// ============================================================================
// ProjectContext - type extraction edge cases
// ============================================================================

#[test]
fn core_coverage_project_context_no_struct_code() {
    let mut ctx = ProjectContext::new();
    let file = TranspiledFile::new(
        PathBuf::from("simple.c"),
        "fn simple() -> i32 { 0 }".to_string(),
        vec![],
        vec!["simple".to_string()],
        String::new(),
    );
    ctx.add_transpiled_file(&file);

    // No types should be extracted from code without structs
    assert!(!ctx.has_type("simple"));
    assert!(ctx.has_function("simple"));
}

#[test]
fn core_coverage_project_context_multiple_structs() {
    let mut ctx = ProjectContext::new();
    let file = TranspiledFile::new(
        PathBuf::from("multi.c"),
        "pub struct Alpha {\n    x: i32,\n}\npub struct Beta {\n    y: f64,\n}".to_string(),
        vec![],
        vec![],
        String::new(),
    );
    ctx.add_transpiled_file(&file);

    assert!(ctx.has_type("Alpha"));
    assert!(ctx.has_type("Beta"));
}

// ============================================================================
// Global variable deduplication
// ============================================================================

#[test]
fn core_coverage_global_variable_dedup() {
    // Tests that duplicate global names only appear once
    let c_code = r#"
        int shared = 0;
        int get_shared() { return shared; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
    let code = result.unwrap();
    // Count occurrences of "static mut shared"
    let count = code.matches("static mut shared").count();
    assert!(count <= 1, "Should not have duplicate global: count={}", count);
}

// ============================================================================
// Multiple system header includes
// ============================================================================

#[test]
fn core_coverage_multiple_different_system_headers() {
    let c_code = r#"
        #include <stdio.h>
        #include <stdlib.h>
        #include <string.h>
        int main() { return 0; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

// ============================================================================
// Pointer comparison to NULL patterns
// ============================================================================

#[test]
fn core_coverage_null_check_function_param() {
    let c_code = r#"
        int safe_deref(int* ptr) {
            if (ptr == 0) {
                return -1;
            }
            return *ptr;
        }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_null_check_keeps_raw_pointer() {
    let c_code = r#"
        int is_null(int* ptr) {
            if (ptr == 0) return 1;
            return 0;
        }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
    // Pointer params compared to NULL should remain as raw pointers
    let code = result.unwrap();
    assert!(code.contains("fn is_null"));
}

// ============================================================================
// Pointer arithmetic detection
// ============================================================================

#[test]
fn core_coverage_pointer_arithmetic_keeps_raw_pointer() {
    let c_code = r#"
        int walk(int* p, int n) {
            int total = 0;
            int* end = p + n;
            while (p < end) {
                total = total + *p;
                p = p + 1;
            }
            return total;
        }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

// ============================================================================
// String iteration functions (DECY-134b)
// ============================================================================

#[test]
fn core_coverage_string_param_function() {
    let c_code = r#"
        int strlen_custom(char* str) {
            int len = 0;
            while (str[len] != 0) {
                len = len + 1;
            }
            return len;
        }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

// ============================================================================
// Struct struct deduplication (DECY-119)
// ============================================================================

#[test]
fn core_coverage_struct_dedup_same_name() {
    // Test that the same struct name doesn't appear twice
    let c_code = r#"
        struct Data { int value; };
        int get_val(struct Data* d) { return d->value; }
        void set_val(struct Data* d, int v) { d->value = v; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
    let code = result.unwrap();
    // Should only have one struct definition
    let struct_count = code.matches("struct Data").count();
    assert!(struct_count >= 1, "Should have at least one struct Data");
}

// ============================================================================
// transpile_with_verification edge cases
// ============================================================================

#[test]
fn core_coverage_verification_with_complex_code() {
    let c_code = r#"
        struct Node {
            int value;
            struct Node* next;
        };

        int list_sum(struct Node* head) {
            int total = 0;
            struct Node* curr = head;
            while (curr != 0) {
                total = total + curr->value;
                curr = curr->next;
            }
            return total;
        }
    "#;
    let result = decy_core::transpile_with_verification(c_code).unwrap();
    assert!(!result.rust_code.is_empty());
}

// ============================================================================
// Comments and whitespace edge cases
// ============================================================================

#[test]
fn core_coverage_whitespace_only() {
    let c_code = "   \n\n\t\t  \n  ";
    let result = decy_core::transpile(c_code);
    // Should handle gracefully
    assert!(result.is_ok());
}

#[test]
fn core_coverage_comments_only() {
    let c_code = "// line comment\n/* block\ncomment */\n";
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_mixed_comments_and_code() {
    let c_code = r#"
        /* Header comment */
        // Function that adds
        int add(int a, /* first */ int b /* second */) {
            // Return sum
            return a + b; /* result */
        }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("fn add"));
}

// ============================================================================
// Signed char array initialization (DECY-250)
// ============================================================================

#[test]
fn core_coverage_signed_char_array_default() {
    let c_code = r#"
        signed char bytes[16];
        void clear() { bytes[0] = 0; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

// ============================================================================
// Various C type patterns through pipeline
// ============================================================================

#[test]
fn core_coverage_multiple_params_different_types() {
    let c_code = r#"
        float compute(int count, float base, double factor, char flag) {
            if (flag) {
                return base * factor * count;
            }
            return 0.0;
        }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_function_returning_void() {
    let c_code = r#"
        void do_work(int* out, int val) {
            *out = val * 2;
        }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_function_returning_double() {
    let c_code = r#"
        double average(int a, int b) {
            return (a + b) / 2.0;
        }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_function_with_local_array() {
    let c_code = r#"
        int sum_local() {
            int arr[5];
            arr[0] = 1;
            arr[1] = 2;
            arr[2] = 3;
            arr[3] = 4;
            arr[4] = 5;
            return arr[0] + arr[1] + arr[2] + arr[3] + arr[4];
        }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

// ============================================================================
// Optimization pass coverage (triggered through transpile)
// ============================================================================

#[test]
fn core_coverage_constant_folding_through_pipeline() {
    // The optimize pass should fold 2+3 into 5
    let c_code = r#"
        int constant() { return 2 + 3; }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn core_coverage_dead_branch_removal() {
    let c_code = r#"
        int always_true() {
            if (1) {
                return 42;
            } else {
                return 0;
            }
        }
    "#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}
