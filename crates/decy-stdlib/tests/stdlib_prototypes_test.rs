//! # DECY Standard Library Prototype Tests
//!
//! **RED PHASE**: These tests WILL FAIL initially!
//!
//! Following EXTREME TDD:
//! 1. RED: Write failing tests (this file)
//! 2. GREEN: Minimal implementation to pass tests
//! 3. REFACTOR: Clean up and optimize
//!
//! **Reference**: docs/specifications/header-support-spec.md

use decy_stdlib::{StdHeader, StdlibPrototypes};

// ============================================================================
// RED PHASE: Prototype Database Tests
// ============================================================================

#[test]
fn test_malloc_prototype_exists() {
    // RED: This will FAIL - database is empty!
    let stdlib = StdlibPrototypes::new();

    let malloc = stdlib.get_prototype("malloc");
    assert!(malloc.is_some(), "malloc prototype should exist");

    let proto = malloc.unwrap();
    assert_eq!(proto.name, "malloc");
    assert_eq!(proto.return_type, "void*");
    assert_eq!(proto.parameters.len(), 1);
    assert_eq!(proto.parameters[0].type_str, "size_t");
    assert!(!proto.is_variadic);
    assert_eq!(proto.header, StdHeader::Stdlib);
}

#[test]
fn test_free_prototype_exists() {
    // RED: This will FAIL
    let stdlib = StdlibPrototypes::new();

    let free = stdlib.get_prototype("free");
    assert!(free.is_some(), "free prototype should exist");

    let proto = free.unwrap();
    assert_eq!(proto.name, "free");
    assert_eq!(proto.return_type, "void");
    assert_eq!(proto.parameters.len(), 1);
    assert_eq!(proto.parameters[0].type_str, "void*");
}

#[test]
fn test_printf_prototype_exists() {
    // RED: This will FAIL
    let stdlib = StdlibPrototypes::new();

    let printf = stdlib.get_prototype("printf");
    assert!(printf.is_some(), "printf prototype should exist");

    let proto = printf.unwrap();
    assert_eq!(proto.name, "printf");
    assert_eq!(proto.return_type, "int");
    assert!(proto.is_variadic, "printf is variadic");
    assert_eq!(proto.header, StdHeader::Stdio);
}

#[test]
fn test_strlen_prototype_exists() {
    // RED: This will FAIL
    let stdlib = StdlibPrototypes::new();

    let strlen = stdlib.get_prototype("strlen");
    assert!(strlen.is_some(), "strlen prototype should exist");

    let proto = strlen.unwrap();
    assert_eq!(proto.name, "strlen");
    assert_eq!(proto.return_type, "size_t");
    assert_eq!(proto.parameters.len(), 1);
    assert_eq!(proto.parameters[0].type_str, "const char*");
    assert_eq!(proto.header, StdHeader::String);
}

#[test]
fn test_database_has_minimum_functions() {
    // RED: This will FAIL - need at least 50 core functions
    let stdlib = StdlibPrototypes::new();

    assert!(
        stdlib.len() >= 50,
        "Should have at least 50 core stdlib functions, got {}",
        stdlib.len()
    );
}

#[test]
fn test_all_stdlib_memory_functions() {
    // RED: All will FAIL
    let stdlib = StdlibPrototypes::new();

    // ISO C99 §7.22.3 - Memory management functions
    assert!(stdlib.get_prototype("malloc").is_some());
    assert!(stdlib.get_prototype("calloc").is_some());
    assert!(stdlib.get_prototype("realloc").is_some());
    assert!(stdlib.get_prototype("free").is_some());
}

#[test]
fn test_all_stdio_functions() {
    // RED: All will FAIL
    let stdlib = StdlibPrototypes::new();

    // ISO C99 §7.21 - Input/output functions
    assert!(stdlib.get_prototype("printf").is_some());
    assert!(stdlib.get_prototype("fprintf").is_some());
    assert!(stdlib.get_prototype("sprintf").is_some());
    assert!(stdlib.get_prototype("snprintf").is_some());
    assert!(stdlib.get_prototype("scanf").is_some());
    assert!(stdlib.get_prototype("fscanf").is_some());
    assert!(stdlib.get_prototype("sscanf").is_some());
    assert!(stdlib.get_prototype("fopen").is_some());
    assert!(stdlib.get_prototype("fclose").is_some());
    assert!(stdlib.get_prototype("fread").is_some());
    assert!(stdlib.get_prototype("fwrite").is_some());
}

#[test]
fn test_all_string_functions() {
    // RED: All will FAIL
    let stdlib = StdlibPrototypes::new();

    // ISO C99 §7.23 - String handling functions
    assert!(stdlib.get_prototype("strlen").is_some());
    assert!(stdlib.get_prototype("strcpy").is_some());
    assert!(stdlib.get_prototype("strncpy").is_some());
    assert!(stdlib.get_prototype("strcat").is_some());
    assert!(stdlib.get_prototype("strncat").is_some());
    assert!(stdlib.get_prototype("strcmp").is_some());
    assert!(stdlib.get_prototype("strncmp").is_some());
    assert!(stdlib.get_prototype("strchr").is_some());
    assert!(stdlib.get_prototype("strrchr").is_some());
    assert!(stdlib.get_prototype("strstr").is_some());
    assert!(stdlib.get_prototype("memcpy").is_some());
    assert!(stdlib.get_prototype("memmove").is_some());
    assert!(stdlib.get_prototype("memset").is_some());
    assert!(stdlib.get_prototype("memcmp").is_some());
}

#[test]
fn test_inject_prototypes_generates_valid_c() {
    // RED: Will generate empty string currently
    let stdlib = StdlibPrototypes::new();

    let injected = stdlib.inject_all_prototypes();

    // Should have type definitions
    assert!(injected.contains("typedef unsigned long size_t;"));
    assert!(injected.contains("typedef long ssize_t;"));

    // Should have comments
    assert!(injected.contains("// Built-in stdlib prototypes"));
}

#[test]
fn test_inject_prototypes_includes_malloc() {
    // RED: Will FAIL - no malloc yet
    let stdlib = StdlibPrototypes::new();

    let injected = stdlib.inject_all_prototypes();

    assert!(
        injected.contains("void* malloc(size_t"),
        "Should contain malloc prototype"
    );
}

#[test]
fn test_prototype_injection_is_deterministic() {
    // Prototypes should always be in same order (sorted)
    let stdlib1 = StdlibPrototypes::new();
    let stdlib2 = StdlibPrototypes::new();

    let inject1 = stdlib1.inject_all_prototypes();
    let inject2 = stdlib2.inject_all_prototypes();

    assert_eq!(
        inject1, inject2,
        "Prototype injection should be deterministic"
    );
}

// ============================================================================
// RED PHASE: Edge Case Tests
// ============================================================================

#[test]
fn test_nonexistent_function_returns_none() {
    let stdlib = StdlibPrototypes::new();

    assert!(
        stdlib.get_prototype("nonexistent_function").is_none(),
        "Should return None for functions not in database"
    );
}

#[test]
fn test_database_is_not_empty() {
    // RED: Will FAIL - database starts empty
    let stdlib = StdlibPrototypes::new();

    assert!(!stdlib.is_empty(), "Prototype database should not be empty");
}

// ============================================================================
// Additional Coverage Tests
// ============================================================================

#[test]
fn test_std_header_ctype() {
    assert_eq!(StdHeader::from_filename("ctype.h"), Some(StdHeader::Ctype));
}

#[test]
fn test_std_header_math() {
    assert_eq!(StdHeader::from_filename("math.h"), Some(StdHeader::Math));
}

#[test]
fn test_std_header_time() {
    assert_eq!(StdHeader::from_filename("time.h"), Some(StdHeader::Time));
}

#[test]
fn test_std_header_errno() {
    assert_eq!(StdHeader::from_filename("errno.h"), Some(StdHeader::Errno));
}

#[test]
fn test_std_header_limits() {
    assert_eq!(
        StdHeader::from_filename("limits.h"),
        Some(StdHeader::Limits)
    );
}

#[test]
fn test_std_header_float() {
    assert_eq!(StdHeader::from_filename("float.h"), Some(StdHeader::Float));
}

#[test]
fn test_std_header_locale() {
    assert_eq!(
        StdHeader::from_filename("locale.h"),
        Some(StdHeader::Locale)
    );
}

#[test]
fn test_std_header_setjmp() {
    assert_eq!(
        StdHeader::from_filename("setjmp.h"),
        Some(StdHeader::Setjmp)
    );
}

#[test]
fn test_std_header_signal() {
    assert_eq!(
        StdHeader::from_filename("signal.h"),
        Some(StdHeader::Signal)
    );
}

#[test]
fn test_std_header_stdarg() {
    assert_eq!(
        StdHeader::from_filename("stdarg.h"),
        Some(StdHeader::Stdarg)
    );
}

#[test]
fn test_std_header_stdbool() {
    assert_eq!(
        StdHeader::from_filename("stdbool.h"),
        Some(StdHeader::Stdbool)
    );
}

#[test]
fn test_std_header_stddef() {
    assert_eq!(
        StdHeader::from_filename("stddef.h"),
        Some(StdHeader::Stddef)
    );
}

#[test]
fn test_std_header_stdint() {
    assert_eq!(
        StdHeader::from_filename("stdint.h"),
        Some(StdHeader::Stdint)
    );
}

#[test]
fn test_std_header_unistd() {
    assert_eq!(
        StdHeader::from_filename("unistd.h"),
        Some(StdHeader::Unistd)
    );
}

#[test]
fn test_std_header_fcntl() {
    assert_eq!(StdHeader::from_filename("fcntl.h"), Some(StdHeader::Fcntl));
}

#[test]
fn test_std_header_dirent() {
    assert_eq!(
        StdHeader::from_filename("dirent.h"),
        Some(StdHeader::Dirent)
    );
}

#[test]
fn test_std_header_sys_types() {
    assert_eq!(
        StdHeader::from_filename("sys/types.h"),
        Some(StdHeader::SysTypes)
    );
}

#[test]
fn test_std_header_sys_stat() {
    assert_eq!(
        StdHeader::from_filename("sys/stat.h"),
        Some(StdHeader::SysStat)
    );
}

#[test]
fn test_std_header_sys_mman() {
    assert_eq!(
        StdHeader::from_filename("sys/mman.h"),
        Some(StdHeader::SysMman)
    );
}

#[test]
fn test_std_header_wchar() {
    assert_eq!(StdHeader::from_filename("wchar.h"), Some(StdHeader::Wchar));
}

#[test]
fn test_std_header_assert() {
    assert_eq!(
        StdHeader::from_filename("assert.h"),
        Some(StdHeader::Assert)
    );
}

#[test]
fn test_stdlib_printf_exists() {
    let stdlib = StdlibPrototypes::new();
    let printf = stdlib.get_prototype("printf");
    assert!(printf.is_some());
}

#[test]
fn test_stdlib_scanf_exists() {
    let stdlib = StdlibPrototypes::new();
    let scanf = stdlib.get_prototype("scanf");
    assert!(scanf.is_some());
}

#[test]
fn test_stdlib_memset_exists() {
    let stdlib = StdlibPrototypes::new();
    let memset = stdlib.get_prototype("memset");
    assert!(memset.is_some());
}

#[test]
fn test_function_proto_parameters() {
    let stdlib = StdlibPrototypes::new();
    let memcpy = stdlib.get_prototype("memcpy");
    assert!(memcpy.is_some());
    let proto = memcpy.unwrap();
    assert_eq!(proto.parameters.len(), 3); // dest, src, n
}

#[test]
fn test_function_proto_variadic() {
    let stdlib = StdlibPrototypes::new();
    let sprintf = stdlib.get_prototype("sprintf");
    assert!(sprintf.is_some());
    let proto = sprintf.unwrap();
    assert!(proto.is_variadic);
}

#[test]
fn test_parameter_new() {
    use decy_stdlib::Parameter;
    let param = Parameter::new("test", "int");
    assert_eq!(param.name, "test");
    assert_eq!(param.type_str, "int");
}

#[test]
fn test_std_header_eq() {
    assert_eq!(StdHeader::Stdio, StdHeader::Stdio);
    assert_ne!(StdHeader::Stdio, StdHeader::Stdlib);
}

#[test]
fn test_std_header_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(StdHeader::Stdio);
    set.insert(StdHeader::Stdlib);
    assert!(set.contains(&StdHeader::Stdio));
    assert!(!set.contains(&StdHeader::Math));
}

#[test]
fn test_std_header_clone() {
    let header = StdHeader::Stdio;
    let cloned = header;
    assert_eq!(header, cloned);
}

#[test]
fn test_std_header_debug() {
    let debug = format!("{:?}", StdHeader::Stdio);
    assert!(debug.contains("Stdio"));
}
