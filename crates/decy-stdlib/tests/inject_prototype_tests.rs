//! Integration tests for inject_prototypes_for_header coverage.
//!
//! These tests target all remaining uncovered lines in inject_prototypes_for_header,
//! exercising every branch of the match statement, function filtering logic,
//! function pointer parameter skipping, output structure verification, and
//! cross-header isolation guarantees.

use decy_stdlib::{FunctionProto, Parameter, StdHeader, StdlibPrototypes};

// ============================================================================
// Helper
// ============================================================================

fn all_headers() -> Vec<StdHeader> {
    vec![
        StdHeader::Assert,
        StdHeader::Ctype,
        StdHeader::Errno,
        StdHeader::Float,
        StdHeader::Limits,
        StdHeader::Locale,
        StdHeader::Math,
        StdHeader::Setjmp,
        StdHeader::Signal,
        StdHeader::Stdarg,
        StdHeader::Stdbool,
        StdHeader::Stddef,
        StdHeader::Stdint,
        StdHeader::Stdio,
        StdHeader::Stdlib,
        StdHeader::String,
        StdHeader::Time,
        StdHeader::Unistd,
        StdHeader::Fcntl,
        StdHeader::Dirent,
        StdHeader::SysTypes,
        StdHeader::SysStat,
        StdHeader::SysMman,
        StdHeader::Wchar,
    ]
}

// ============================================================================
// 1. Every header produces non-empty output with common preamble
// ============================================================================

#[test]
fn test_every_header_has_common_preamble_size_t() {
    let stdlib = StdlibPrototypes::new();
    for header in all_headers() {
        let result = stdlib.inject_prototypes_for_header(header);
        assert!(
            result.contains("typedef unsigned long size_t;"),
            "{:?} missing size_t",
            header
        );
    }
}

#[test]
fn test_every_header_has_common_preamble_ssize_t() {
    let stdlib = StdlibPrototypes::new();
    for header in all_headers() {
        let result = stdlib.inject_prototypes_for_header(header);
        assert!(
            result.contains("typedef long ssize_t;"),
            "{:?} missing ssize_t",
            header
        );
    }
}

#[test]
fn test_every_header_has_common_preamble_ptrdiff_t() {
    let stdlib = StdlibPrototypes::new();
    for header in all_headers() {
        let result = stdlib.inject_prototypes_for_header(header);
        assert!(
            result.contains("typedef long ptrdiff_t;"),
            "{:?} missing ptrdiff_t",
            header
        );
    }
}

#[test]
fn test_every_header_has_null_macro() {
    let stdlib = StdlibPrototypes::new();
    for header in all_headers() {
        let result = stdlib.inject_prototypes_for_header(header);
        assert!(
            result.contains("#define NULL 0"),
            "{:?} missing NULL macro",
            header
        );
    }
}

#[test]
fn test_every_header_has_iso_c99_comment() {
    let stdlib = StdlibPrototypes::new();
    for header in all_headers() {
        let result = stdlib.inject_prototypes_for_header(header);
        assert!(
            result.contains("ISO C99"),
            "{:?} missing ISO C99 reference",
            header
        );
    }
}

// ============================================================================
// 2. Stdio branch -- complete macro and type coverage
// ============================================================================

#[test]
fn test_stdio_bufsiz_macro() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdio);
    assert!(result.contains("#define BUFSIZ 8192"));
}

#[test]
fn test_stdio_l_tmpnam_macro() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdio);
    assert!(result.contains("#define L_tmpnam 20"));
}

#[test]
fn test_stdio_buffering_macros() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdio);
    assert!(result.contains("#define _IONBF 2"));
    assert!(result.contains("#define _IOLBF 1"));
    assert!(result.contains("#define _IOFBF 0"));
}

#[test]
fn test_stdio_fread_fwrite_declarations() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdio);
    assert!(result.contains("fread"));
    assert!(result.contains("fwrite"));
}

#[test]
fn test_stdio_fgets_fputs_declarations() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdio);
    assert!(result.contains("fgets"));
    assert!(result.contains("fputs"));
}

#[test]
fn test_stdio_fopen_declaration() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdio);
    assert!(result.contains("fopen"));
}

#[test]
fn test_stdio_fseek_declaration() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdio);
    assert!(result.contains("fseek"));
}

// ============================================================================
// 3. Errno branch -- verify no function protos leak
// ============================================================================

#[test]
fn test_errno_no_stdio_types() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Errno);
    assert!(!result.contains("typedef struct _IO_FILE FILE;"));
    assert!(!result.contains("extern FILE*"));
}

#[test]
fn test_errno_no_stdlib_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Errno);
    assert!(!result.contains("void* malloc("));
    assert!(!result.contains("int printf("));
}

// ============================================================================
// 4. Time branch
// ============================================================================

#[test]
fn test_time_struct_tm_declaration() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Time);
    assert!(result.contains("struct tm;"));
}

#[test]
fn test_time_clocks_per_sec() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Time);
    assert!(result.contains("#define CLOCKS_PER_SEC 1000000"));
}

#[test]
fn test_time_clock_function() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Time);
    assert!(result.contains("clock_t clock(void);"));
}

// ============================================================================
// 5. Stdint branch -- pointer-width types
// ============================================================================

#[test]
fn test_stdint_intptr_uintptr() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdint);
    assert!(result.contains("typedef long intptr_t;"));
    assert!(result.contains("typedef unsigned long uintptr_t;"));
}

// ============================================================================
// 6. Unistd branch -- POSIX type coverage
// ============================================================================

#[test]
fn test_unistd_uid_gid_types() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Unistd);
    assert!(result.contains("typedef unsigned int uid_t;"));
    assert!(result.contains("typedef unsigned int gid_t;"));
}

#[test]
fn test_unistd_sc_pagesize() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Unistd);
    assert!(result.contains("#define _SC_PAGESIZE 30"));
}

#[test]
fn test_unistd_lseek_function() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Unistd);
    assert!(result.contains("lseek"));
}

// ============================================================================
// 7. Fcntl branch
// ============================================================================

#[test]
fn test_fcntl_nonblock_macro() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Fcntl);
    assert!(result.contains("#define O_NONBLOCK 04000"));
}

// ============================================================================
// 8. Dirent branch
// ============================================================================

#[test]
fn test_dirent_dir_typedef() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Dirent);
    assert!(result.contains("typedef struct __dirstream DIR;"));
}

// ============================================================================
// 9. SysTypes branch
// ============================================================================

#[test]
fn test_sys_types_mode_t() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::SysTypes);
    assert!(result.contains("typedef unsigned int mode_t;"));
}

// ============================================================================
// 10. SysStat branch
// ============================================================================

#[test]
fn test_sys_stat_s_isreg_macro() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::SysStat);
    assert!(result.contains("#define S_ISREG(m) (((m) & 0170000) == 0100000)"));
}

#[test]
fn test_sys_stat_s_isdir_macro() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::SysStat);
    assert!(result.contains("#define S_ISDIR(m) (((m) & 0170000) == 0040000)"));
}

// ============================================================================
// 11. SysMman branch
// ============================================================================

#[test]
fn test_sys_mman_map_anonymous() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::SysMman);
    assert!(result.contains("#define MAP_ANONYMOUS 0x20"));
}

#[test]
fn test_sys_mman_map_failed() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::SysMman);
    assert!(result.contains("#define MAP_FAILED ((void*)-1)"));
}

// ============================================================================
// 12. Wchar branch
// ============================================================================

#[test]
fn test_wchar_wint_t_type() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Wchar);
    assert!(result.contains("typedef int wint_t;"));
}

#[test]
fn test_wchar_weof_macro() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Wchar);
    assert!(result.contains("#define WEOF (-1)"));
}

// ============================================================================
// 13. Signal branch
// ============================================================================

#[test]
fn test_signal_sighandler_t() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Signal);
    assert!(result.contains("typedef void (*sighandler_t)(int);"));
}

// ============================================================================
// 14. Limits branch
// ============================================================================

#[test]
fn test_limits_all_long_macros() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Limits);
    assert!(result.contains("#define LONG_MIN (-9223372036854775807L-1)"));
    assert!(result.contains("#define LONG_MAX 9223372036854775807L"));
}

#[test]
fn test_limits_path_max_value() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Limits);
    assert!(result.contains("#define PATH_MAX 4096"));
}

// ============================================================================
// 15. Ctype branch (empty body - no header-specific type defs)
// ============================================================================

#[test]
fn test_ctype_no_header_specific_types() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Ctype);
    // Ctype has no header-specific type definitions (empty match arm body)
    assert!(!result.contains("typedef struct _IO_FILE FILE;"));
    assert!(!result.contains("extern int errno;"));
    assert!(!result.contains("typedef long time_t;"));
}

#[test]
fn test_ctype_has_classification_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Ctype);
    assert!(result.contains("isspace"));
    assert!(result.contains("isdigit"));
    assert!(result.contains("isalpha"));
    assert!(result.contains("toupper"));
    assert!(result.contains("tolower"));
}

// ============================================================================
// 16. Math branch
// ============================================================================

#[test]
fn test_math_infinity_nan_macros() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Math);
    assert!(result.contains("#define INFINITY (1.0/0.0)"));
    assert!(result.contains("#define NAN (0.0/0.0)"));
}

#[test]
fn test_math_m_pi_m_e_macros() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Math);
    assert!(result.contains("#define M_PI 3.14159265358979323846"));
    assert!(result.contains("#define M_E 2.71828182845904523536"));
}

// ============================================================================
// 17. Wildcard arm headers (Assert, Float, Locale, Setjmp, Stddef)
// ============================================================================

#[test]
fn test_assert_wildcard_has_preamble_only() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Assert);
    assert!(result.contains("typedef unsigned long size_t;"));
    assert!(!result.contains("extern int errno;"));
    assert!(!result.contains("struct _IO_FILE;"));
}

#[test]
fn test_float_wildcard_has_preamble_only() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Float);
    assert!(result.contains("typedef unsigned long size_t;"));
    assert!(!result.contains("double sin("));
}

#[test]
fn test_locale_wildcard_has_preamble_only() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Locale);
    assert!(result.contains("typedef unsigned long size_t;"));
}

#[test]
fn test_setjmp_wildcard_has_preamble_only() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Setjmp);
    assert!(result.contains("typedef unsigned long size_t;"));
}

#[test]
fn test_stddef_wildcard_has_preamble_only() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stddef);
    assert!(result.contains("typedef unsigned long size_t;"));
    assert!(result.contains("typedef long ptrdiff_t;"));
}

// ============================================================================
// 18. Function pointer parameter filtering
// ============================================================================

#[test]
fn test_function_pointer_params_filtered_for_stdlib() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);
    // qsort has function pointer param with "(*" in type_str -- should be skipped
    assert!(
        !result.contains("void qsort("),
        "qsort with function pointer param should be filtered out"
    );
}

#[test]
fn test_function_pointer_params_filtered_for_stdlib_bsearch() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);
    assert!(
        !result.contains("void* bsearch("),
        "bsearch with function pointer param should be filtered out"
    );
}

// ============================================================================
// 19. Alphabetical sorting
// ============================================================================

#[test]
fn test_stdlib_functions_sorted() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);
    let abs_pos = result.find("int abs(").unwrap();
    let free_pos = result.find("void free(").unwrap();
    let malloc_pos = result.find("void* malloc(").unwrap();
    assert!(abs_pos < free_pos, "abs should come before free");
    assert!(free_pos < malloc_pos, "free should come before malloc");
}

#[test]
fn test_string_functions_sorted() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::String);
    let memchr_pos = result.find("memchr").unwrap();
    let memcpy_pos = result.find("memcpy").unwrap();
    let strlen_pos = result.find("strlen").unwrap();
    assert!(memchr_pos < memcpy_pos, "memchr should come before memcpy");
    assert!(memcpy_pos < strlen_pos, "memcpy should come before strlen");
}

// ============================================================================
// 20. Cross-header isolation -- verify no function leaks
// ============================================================================

#[test]
fn test_math_no_string_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Math);
    assert!(!result.contains("strlen"));
    assert!(!result.contains("memcpy"));
}

#[test]
fn test_string_no_math_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::String);
    assert!(!result.contains("double sin("));
    assert!(!result.contains("double cos("));
    assert!(!result.contains("#define M_PI"));
}

#[test]
fn test_time_no_stdlib_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Time);
    assert!(!result.contains("void* malloc("));
    assert!(!result.contains("void free("));
}

#[test]
fn test_ctype_no_stdio_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Ctype);
    assert!(!result.contains("int printf("));
    assert!(!result.contains("int fprintf("));
}

// ============================================================================
// 21. Output structure verification
// ============================================================================

#[test]
fn test_output_ends_with_newline_after_functions() {
    let stdlib = StdlibPrototypes::new();
    for header in all_headers() {
        let result = stdlib.inject_prototypes_for_header(header);
        assert!(
            result.ends_with('\n'),
            "{:?} output should end with newline",
            header
        );
    }
}

#[test]
fn test_every_function_decl_ends_with_semicolon_newline() {
    let stdlib = StdlibPrototypes::new();
    for header in all_headers() {
        let result = stdlib.inject_prototypes_for_header(header);
        for line in result.lines() {
            // Skip comments, macros, typedefs, struct defs, extern decls, empty
            if line.is_empty()
                || line.starts_with("//")
                || line.starts_with('#')
                || line.starts_with("typedef")
                || line.starts_with("struct")
                || line.starts_with("extern")
            {
                continue;
            }
            if line.contains('(') && line.contains(')') {
                assert!(
                    line.ends_with(';'),
                    "{:?}: function declaration should end with ';': {}",
                    header,
                    line
                );
            }
        }
    }
}

// ============================================================================
// 22. Debug format for StdHeader
// ============================================================================

#[test]
fn test_std_header_debug_format() {
    let header = StdHeader::Stdio;
    let debug = format!("{:?}", header);
    assert_eq!(debug, "Stdio");
}

#[test]
fn test_std_header_clone_and_copy() {
    let header = StdHeader::Math;
    let cloned = header;
    assert_eq!(header, cloned);
}

// ============================================================================
// 23. Stdlib-specific function coverage
// ============================================================================

#[test]
fn test_stdlib_abs_labs_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);
    assert!(result.contains("int abs(int j);"));
    assert!(result.contains("long labs(long j);"));
}

#[test]
fn test_stdlib_exit_abort_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);
    assert!(result.contains("void exit(int status);"));
    assert!(result.contains("void abort(void);"));
}

#[test]
fn test_stdlib_getenv_system_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);
    assert!(result.contains("char* getenv(const char* name);"));
    assert!(result.contains("int system(const char* command);"));
}

#[test]
fn test_stdlib_strtol_strtod_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);
    assert!(result.contains("strtol"));
    assert!(result.contains("strtod"));
}

// ============================================================================
// 24. to_c_declaration edge cases
// ============================================================================

#[test]
fn test_to_c_declaration_empty_params_produces_void() {
    let proto = FunctionProto {
        name: "getchar".to_string(),
        return_type: "int".to_string(),
        parameters: vec![],
        is_variadic: false,
        header: StdHeader::Stdio,
        c99_section: "test".to_string(),
    };
    assert_eq!(proto.to_c_declaration(), "int getchar(void);");
}

#[test]
fn test_to_c_declaration_variadic_with_one_param() {
    let proto = FunctionProto {
        name: "printf".to_string(),
        return_type: "int".to_string(),
        parameters: vec![Parameter::new("format", "const char*")],
        is_variadic: true,
        header: StdHeader::Stdio,
        c99_section: "test".to_string(),
    };
    let decl = proto.to_c_declaration();
    assert!(decl.contains("..."));
    assert_eq!(decl, "int printf(const char* format, ...);");
}

#[test]
fn test_parameter_new() {
    let p = Parameter::new("foo", "int");
    assert_eq!(p.name, "foo");
    assert_eq!(p.type_str, "int");
}

#[test]
fn test_parameter_clone() {
    let p = Parameter::new("bar", "double");
    let p2 = p.clone();
    assert_eq!(p, p2);
}

#[test]
fn test_function_proto_clone() {
    let proto = FunctionProto {
        name: "test".to_string(),
        return_type: "void".to_string(),
        parameters: vec![],
        is_variadic: false,
        header: StdHeader::Stdlib,
        c99_section: "test".to_string(),
    };
    let cloned = proto.clone();
    assert_eq!(proto, cloned);
}

// ============================================================================
// 25. inject_all_prototypes coverage
// ============================================================================

#[test]
fn test_inject_all_prototypes_contains_common_types() {
    let stdlib = StdlibPrototypes::new();
    let all = stdlib.inject_all_prototypes();
    assert!(all.contains("typedef unsigned long size_t;"));
    assert!(all.contains("#define NULL 0"));
    assert!(all.contains("struct _IO_FILE;"));
}

#[test]
fn test_inject_all_prototypes_contains_streams() {
    let stdlib = StdlibPrototypes::new();
    let all = stdlib.inject_all_prototypes();
    assert!(all.contains("extern FILE* stdin;"));
    assert!(all.contains("extern FILE* stdout;"));
    assert!(all.contains("extern FILE* stderr;"));
}

#[test]
fn test_inject_all_prototypes_contains_posix_types() {
    let stdlib = StdlibPrototypes::new();
    let all = stdlib.inject_all_prototypes();
    assert!(all.contains("typedef int pid_t;"));
    assert!(all.contains("typedef long off_t;"));
    assert!(all.contains("typedef long time_t;"));
    assert!(all.contains("typedef long clock_t;"));
}

#[test]
fn test_inject_all_prototypes_includes_functions_from_multiple_headers() {
    let stdlib = StdlibPrototypes::new();
    let all = stdlib.inject_all_prototypes();
    // Functions from different headers should all appear
    assert!(all.contains("printf"), "missing printf from stdio");
    assert!(all.contains("malloc"), "missing malloc from stdlib");
    assert!(all.contains("strlen"), "missing strlen from string");
    assert!(all.contains("sin"), "missing sin from math");
    assert!(all.contains("isspace"), "missing isspace from ctype");
}

// ============================================================================
// 26. Database size and emptiness
// ============================================================================

#[test]
fn test_stdlib_len_nonzero() {
    let stdlib = StdlibPrototypes::new();
    assert!(stdlib.len() >= 50, "Should have at least 50 function protos");
}

#[test]
fn test_stdlib_is_not_empty() {
    let stdlib = StdlibPrototypes::new();
    assert!(!stdlib.is_empty());
}

// ============================================================================
// 27. get_prototype coverage
// ============================================================================

#[test]
fn test_get_prototype_returns_correct_header() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("printf").unwrap();
    assert_eq!(proto.header, StdHeader::Stdio);
    assert_eq!(proto.return_type, "int");
    assert!(proto.is_variadic);
}

#[test]
fn test_get_prototype_strlen_header() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("strlen").unwrap();
    assert_eq!(proto.header, StdHeader::String);
}

#[test]
fn test_get_prototype_returns_none_for_unknown() {
    let stdlib = StdlibPrototypes::new();
    assert!(stdlib.get_prototype("nonexistent_func_xyz").is_none());
}
