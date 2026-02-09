//! Coverage improvement tests for decy-stdlib
//!
//! Targets all uncovered branches in lib.rs including:
//! - inject_prototypes_for_header() for every StdHeader variant
//! - Individual function prototype details
//! - to_c_declaration() edge cases
//! - Default trait implementation
//! - Function pointer parameter filtering in injection

use decy_stdlib::{FunctionProto, Parameter, StdHeader, StdlibPrototypes};

// ============================================================================
// Default trait implementation
// ============================================================================

#[test]
fn test_stdlib_prototypes_default_trait() {
    let stdlib = StdlibPrototypes::default();
    assert!(!stdlib.is_empty());
    assert!(stdlib.len() >= 50);
    // Default and new should produce equivalent databases
    let stdlib_new = StdlibPrototypes::new();
    assert_eq!(stdlib.len(), stdlib_new.len());
}

// ============================================================================
// to_c_declaration edge cases
// ============================================================================

#[test]
fn test_to_c_declaration_multi_params() {
    let proto = FunctionProto {
        name: "memcpy".to_string(),
        return_type: "void*".to_string(),
        parameters: vec![
            Parameter::new("dest", "void*"),
            Parameter::new("src", "const void*"),
            Parameter::new("n", "size_t"),
        ],
        is_variadic: false,
        header: StdHeader::String,
        c99_section: "test".to_string(),
    };
    assert_eq!(
        proto.to_c_declaration(),
        "void* memcpy(void* dest, const void* src, size_t n);"
    );
}

#[test]
fn test_to_c_declaration_variadic_with_multiple_params() {
    let proto = FunctionProto {
        name: "snprintf".to_string(),
        return_type: "int".to_string(),
        parameters: vec![
            Parameter::new("str", "char*"),
            Parameter::new("size", "size_t"),
            Parameter::new("format", "const char*"),
        ],
        is_variadic: true,
        header: StdHeader::Stdio,
        c99_section: "test".to_string(),
    };
    let decl = proto.to_c_declaration();
    assert!(decl.contains("..."));
    assert!(decl.starts_with("int snprintf("));
    assert!(decl.ends_with(");"));
}

#[test]
fn test_to_c_declaration_void_return() {
    let proto = FunctionProto {
        name: "free".to_string(),
        return_type: "void".to_string(),
        parameters: vec![Parameter::new("ptr", "void*")],
        is_variadic: false,
        header: StdHeader::Stdlib,
        c99_section: "test".to_string(),
    };
    assert_eq!(proto.to_c_declaration(), "void free(void* ptr);");
}

// ============================================================================
// inject_prototypes_for_header -- Stdio branch
// ============================================================================

#[test]
fn test_inject_header_stdio_type_definitions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdio);

    assert!(result.contains("struct _IO_FILE;"));
    assert!(result.contains("typedef struct _IO_FILE FILE;"));
    assert!(result.contains("extern FILE* stdin;"));
    assert!(result.contains("extern FILE* stdout;"));
    assert!(result.contains("extern FILE* stderr;"));
    assert!(result.contains("#define EOF (-1)"));
    assert!(result.contains("#define SEEK_SET 0"));
    assert!(result.contains("#define SEEK_CUR 1"));
    assert!(result.contains("#define SEEK_END 2"));
    assert!(result.contains("#define BUFSIZ 8192"));
    assert!(result.contains("#define L_tmpnam 20"));
    assert!(result.contains("#define _IONBF 2"));
    assert!(result.contains("#define _IOLBF 1"));
    assert!(result.contains("#define _IOFBF 0"));
}

#[test]
fn test_inject_header_stdio_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdio);

    assert!(result.contains("printf"));
    assert!(result.contains("fprintf"));
    assert!(result.contains("sprintf"));
    assert!(result.contains("snprintf"));
    assert!(result.contains("scanf"));
    assert!(result.contains("fscanf"));
    assert!(result.contains("sscanf"));
    assert!(result.contains("fopen"));
    assert!(result.contains("fclose"));
    assert!(result.contains("fread"));
    assert!(result.contains("fwrite"));
    assert!(result.contains("fseek"));
    assert!(result.contains("ftell"));
    assert!(result.contains("rewind"));
    assert!(result.contains("getchar"));
    assert!(result.contains("putchar"));
    assert!(result.contains("fgetc"));
    assert!(result.contains("fputc"));
    assert!(result.contains("fgets"));
    assert!(result.contains("fputs"));
    assert!(result.contains("puts"));
    // stdio functions should NOT include string.h functions
    assert!(!result.contains("void* memcpy("));
}

// ============================================================================
// inject_prototypes_for_header -- Errno branch
// ============================================================================

#[test]
fn test_inject_header_errno() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Errno);

    assert!(result.contains("extern int errno;"));
    assert!(result.contains("#define EACCES 13"));
    assert!(result.contains("#define ENOENT 2"));
    assert!(result.contains("#define EINVAL 22"));
    assert!(result.contains("#define ENOMEM 12"));
    assert!(result.contains("#define ERANGE 34"));
}

// ============================================================================
// inject_prototypes_for_header -- Time branch
// ============================================================================

#[test]
fn test_inject_header_time() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Time);

    assert!(result.contains("typedef long time_t;"));
    assert!(result.contains("typedef long clock_t;"));
    assert!(result.contains("struct tm;"));
    assert!(result.contains("#define CLOCKS_PER_SEC 1000000"));
    // Should contain time-related function prototypes
    assert!(result.contains("clock"));
    assert!(result.contains("time"));
}

// ============================================================================
// inject_prototypes_for_header -- Stdarg branch
// ============================================================================

#[test]
fn test_inject_header_stdarg() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdarg);

    assert!(result.contains("typedef void* va_list;"));
    assert!(result.contains("#define va_start(ap, last) ((void)0)"));
    assert!(result.contains("#define va_end(ap) ((void)0)"));
    assert!(result.contains("#define va_arg(ap, type) (*(type*)0)"));
}

// ============================================================================
// inject_prototypes_for_header -- Stdbool branch
// ============================================================================

#[test]
fn test_inject_header_stdbool() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdbool);

    assert!(result.contains("typedef _Bool bool;"));
    assert!(result.contains("#define true 1"));
    assert!(result.contains("#define false 0"));
}

// ============================================================================
// inject_prototypes_for_header -- Stdint branch
// ============================================================================

#[test]
fn test_inject_header_stdint() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdint);

    assert!(result.contains("typedef signed char int8_t;"));
    assert!(result.contains("typedef short int16_t;"));
    assert!(result.contains("typedef int int32_t;"));
    assert!(result.contains("typedef long long int64_t;"));
    assert!(result.contains("typedef unsigned char uint8_t;"));
    assert!(result.contains("typedef unsigned short uint16_t;"));
    assert!(result.contains("typedef unsigned int uint32_t;"));
    assert!(result.contains("typedef unsigned long long uint64_t;"));
    assert!(result.contains("typedef long intptr_t;"));
    assert!(result.contains("typedef unsigned long uintptr_t;"));
}

// ============================================================================
// inject_prototypes_for_header -- Unistd branch
// ============================================================================

#[test]
fn test_inject_header_unistd() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Unistd);

    assert!(result.contains("typedef int pid_t;"));
    assert!(result.contains("typedef long off_t;"));
    assert!(result.contains("typedef unsigned int uid_t;"));
    assert!(result.contains("typedef unsigned int gid_t;"));
    assert!(result.contains("#define STDIN_FILENO 0"));
    assert!(result.contains("#define STDOUT_FILENO 1"));
    assert!(result.contains("#define STDERR_FILENO 2"));
    assert!(result.contains("#define F_OK 0"));
    assert!(result.contains("#define R_OK 4"));
    assert!(result.contains("#define W_OK 2"));
    assert!(result.contains("#define X_OK 1"));
    assert!(result.contains("#define _SC_OPEN_MAX 4"));
    assert!(result.contains("#define _SC_PAGESIZE 30"));
    // Should include POSIX functions
    assert!(result.contains("pipe"));
    assert!(result.contains("fork"));
    assert!(result.contains("close"));
    assert!(result.contains("dup"));
}

// ============================================================================
// inject_prototypes_for_header -- Fcntl branch
// ============================================================================

#[test]
fn test_inject_header_fcntl() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Fcntl);

    assert!(result.contains("#define O_RDONLY 0"));
    assert!(result.contains("#define O_WRONLY 1"));
    assert!(result.contains("#define O_RDWR 2"));
    assert!(result.contains("#define O_CREAT 0100"));
    assert!(result.contains("#define O_TRUNC 01000"));
    assert!(result.contains("#define O_APPEND 02000"));
    assert!(result.contains("#define O_NONBLOCK 04000"));
    assert!(result.contains("#define LOCK_SH 1"));
    assert!(result.contains("#define LOCK_EX 2"));
    assert!(result.contains("#define LOCK_UN 8"));
    // Should include open() function
    assert!(result.contains("open"));
}

// ============================================================================
// inject_prototypes_for_header -- Dirent branch
// ============================================================================

#[test]
fn test_inject_header_dirent() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Dirent);

    assert!(result.contains("struct dirent { char d_name[256]; };"));
    assert!(result.contains("typedef struct __dirstream DIR;"));
    assert!(result.contains("opendir"));
    assert!(result.contains("readdir"));
    assert!(result.contains("closedir"));
}

// ============================================================================
// inject_prototypes_for_header -- SysTypes branch
// ============================================================================

#[test]
fn test_inject_header_sys_types() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::SysTypes);

    assert!(result.contains("typedef int pid_t;"));
    assert!(result.contains("typedef long off_t;"));
    assert!(result.contains("typedef unsigned int mode_t;"));
    assert!(result.contains("typedef long ssize_t;"));
}

// ============================================================================
// inject_prototypes_for_header -- SysStat branch
// ============================================================================

#[test]
fn test_inject_header_sys_stat() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::SysStat);

    assert!(result.contains("struct stat { long st_size; int st_mode; };"));
    assert!(result.contains("#define S_ISREG(m)"));
    assert!(result.contains("#define S_ISDIR(m)"));
}

// ============================================================================
// inject_prototypes_for_header -- SysMman branch
// ============================================================================

#[test]
fn test_inject_header_sys_mman() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::SysMman);

    assert!(result.contains("#define PROT_NONE 0"));
    assert!(result.contains("#define PROT_READ 1"));
    assert!(result.contains("#define PROT_WRITE 2"));
    assert!(result.contains("#define PROT_EXEC 4"));
    assert!(result.contains("#define MAP_SHARED 1"));
    assert!(result.contains("#define MAP_PRIVATE 2"));
    assert!(result.contains("#define MAP_ANONYMOUS 0x20"));
    assert!(result.contains("#define MAP_FAILED ((void*)-1)"));
}

// ============================================================================
// inject_prototypes_for_header -- Wchar branch
// ============================================================================

#[test]
fn test_inject_header_wchar() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Wchar);

    assert!(result.contains("typedef int wchar_t;"));
    assert!(result.contains("typedef int wint_t;"));
    assert!(result.contains("#define WEOF (-1)"));
}

// ============================================================================
// inject_prototypes_for_header -- Signal branch
// ============================================================================

#[test]
fn test_inject_header_signal() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Signal);

    assert!(result.contains("typedef void (*sighandler_t)(int);"));
    assert!(result.contains("#define SIGINT 2"));
    assert!(result.contains("#define SIGTERM 15"));
}

// ============================================================================
// inject_prototypes_for_header -- Limits branch
// ============================================================================

#[test]
fn test_inject_header_limits() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Limits);

    assert!(result.contains("#define CHAR_BIT 8"));
    assert!(result.contains("#define CHAR_MIN (-128)"));
    assert!(result.contains("#define CHAR_MAX 127"));
    assert!(result.contains("#define SHRT_MIN (-32768)"));
    assert!(result.contains("#define SHRT_MAX 32767"));
    assert!(result.contains("#define INT_MIN (-2147483647-1)"));
    assert!(result.contains("#define INT_MAX 2147483647"));
    assert!(result.contains("#define UINT_MAX 4294967295U"));
    assert!(result.contains("#define LONG_MIN (-9223372036854775807L-1)"));
    assert!(result.contains("#define LONG_MAX 9223372036854775807L"));
    assert!(result.contains("#define PATH_MAX 4096"));
}

// ============================================================================
// inject_prototypes_for_header -- Ctype branch (empty body)
// ============================================================================

#[test]
fn test_inject_header_ctype() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Ctype);

    // Ctype has no special type defs, but common typedefs are always present
    assert!(result.contains("typedef unsigned long size_t;"));
    // Should include ctype functions
    assert!(result.contains("isspace"));
    assert!(result.contains("isdigit"));
    assert!(result.contains("isalpha"));
    assert!(result.contains("isalnum"));
    assert!(result.contains("isupper"));
    assert!(result.contains("islower"));
    assert!(result.contains("tolower"));
    assert!(result.contains("toupper"));
}

// ============================================================================
// inject_prototypes_for_header -- Math branch
// ============================================================================

#[test]
fn test_inject_header_math() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Math);

    assert!(result.contains("#define M_PI 3.14159265358979323846"));
    assert!(result.contains("#define M_E 2.71828182845904523536"));
    assert!(result.contains("#define INFINITY (1.0/0.0)"));
    assert!(result.contains("#define NAN (0.0/0.0)"));
    // Should include math functions
    assert!(result.contains("sqrt"));
    assert!(result.contains("sin"));
    assert!(result.contains("cos"));
    assert!(result.contains("pow"));
    assert!(result.contains("fabs"));
    assert!(result.contains("ceil"));
    assert!(result.contains("floor"));
    assert!(result.contains("round"));
    assert!(result.contains("trunc"));
    assert!(result.contains("exp"));
    assert!(result.contains("log10"));
    assert!(result.contains("tan"));
    assert!(result.contains("asin"));
    assert!(result.contains("acos"));
    assert!(result.contains("atan"));
    assert!(result.contains("atan2"));
    assert!(result.contains("fmod"));
}

// ============================================================================
// inject_prototypes_for_header -- wildcard/fallback branches
// (headers with no special type defs: Assert, Stddef, Float, Locale, Setjmp)
// ============================================================================

#[test]
fn test_inject_header_assert_fallback() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Assert);

    // Should still have common typedefs
    assert!(result.contains("typedef unsigned long size_t;"));
    assert!(result.contains("typedef long ssize_t;"));
    assert!(result.contains("#define NULL 0"));
}

#[test]
fn test_inject_header_stddef_fallback() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stddef);

    assert!(result.contains("typedef unsigned long size_t;"));
    assert!(result.contains("typedef long ptrdiff_t;"));
}

#[test]
fn test_inject_header_float_fallback() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Float);

    // Float header has no functions in db, so just typedefs
    assert!(result.contains("typedef unsigned long size_t;"));
}

#[test]
fn test_inject_header_locale_fallback() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Locale);
    assert!(result.contains("typedef unsigned long size_t;"));
}

#[test]
fn test_inject_header_setjmp_fallback() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Setjmp);
    assert!(result.contains("typedef unsigned long size_t;"));
}

// ============================================================================
// inject_prototypes_for_header -- function pointer parameter filtering
// ============================================================================

#[test]
fn test_inject_stdlib_skips_function_pointer_params() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);

    // qsort and bsearch have function pointer parameters
    // They should be filtered OUT of the injection
    assert!(
        !result.contains("void qsort("),
        "qsort should be filtered out due to function pointer parameter"
    );
    assert!(
        !result.contains("void* bsearch("),
        "bsearch should be filtered out due to function pointer parameter"
    );

    // But other stdlib functions should be present
    assert!(result.contains("malloc"));
    assert!(result.contains("calloc"));
    assert!(result.contains("realloc"));
    assert!(result.contains("free"));
    assert!(result.contains("atoi"));
    assert!(result.contains("exit"));
    assert!(result.contains("abort"));
}

// ============================================================================
// inject_prototypes_for_header -- Stdlib branch functions
// ============================================================================

#[test]
fn test_inject_header_stdlib_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);

    // Memory functions
    assert!(result.contains("void* malloc(size_t size);"));
    assert!(result.contains("void* calloc(size_t nmemb, size_t size);"));
    assert!(result.contains("void* realloc(void* ptr, size_t size);"));
    assert!(result.contains("void free(void* ptr);"));

    // Numeric conversion functions
    assert!(result.contains("atoi"));
    assert!(result.contains("atol"));
    assert!(result.contains("atof"));
    assert!(result.contains("strtol"));
    assert!(result.contains("strtod"));

    // Environment functions
    assert!(result.contains("exit"));
    assert!(result.contains("abort"));
    assert!(result.contains("getenv"));
    assert!(result.contains("system"));

    // Integer arithmetic
    assert!(result.contains("abs"));
    assert!(result.contains("labs"));

    // Random number generation
    assert!(result.contains("rand"));
    assert!(result.contains("srand"));
}

// ============================================================================
// inject_prototypes_for_header -- String header
// ============================================================================

#[test]
fn test_inject_header_string_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::String);

    assert!(result.contains("memcpy"));
    assert!(result.contains("memmove"));
    assert!(result.contains("strcpy"));
    assert!(result.contains("strncpy"));
    assert!(result.contains("strcat"));
    assert!(result.contains("strncat"));
    assert!(result.contains("memcmp"));
    assert!(result.contains("strcmp"));
    assert!(result.contains("strncmp"));
    assert!(result.contains("memchr"));
    assert!(result.contains("strchr"));
    assert!(result.contains("strrchr"));
    assert!(result.contains("strstr"));
    assert!(result.contains("strtok"));
    assert!(result.contains("memset"));
    assert!(result.contains("strlen"));
    assert!(result.contains("strdup"));
}

// ============================================================================
// Individual prototype detail tests (targeting uncovered HashMap inserts)
// ============================================================================

#[test]
fn test_calloc_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("calloc").unwrap();
    assert_eq!(proto.return_type, "void*");
    assert_eq!(proto.parameters.len(), 2);
    assert_eq!(proto.parameters[0].name, "nmemb");
    assert_eq!(proto.parameters[1].name, "size");
    assert!(!proto.is_variadic);
    assert_eq!(proto.header, StdHeader::Stdlib);
    assert_eq!(proto.c99_section, "\u{a7}7.22.3.2");
}

#[test]
fn test_realloc_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("realloc").unwrap();
    assert_eq!(proto.return_type, "void*");
    assert_eq!(proto.parameters.len(), 2);
    assert_eq!(proto.parameters[0].type_str, "void*");
    assert_eq!(proto.parameters[1].type_str, "size_t");
}

#[test]
fn test_atoi_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("atoi").unwrap();
    assert_eq!(proto.return_type, "int");
    assert_eq!(proto.parameters.len(), 1);
    assert_eq!(proto.parameters[0].type_str, "const char*");
    assert_eq!(proto.header, StdHeader::Stdlib);
}

#[test]
fn test_atol_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("atol").unwrap();
    assert_eq!(proto.return_type, "long");
}

#[test]
fn test_atof_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("atof").unwrap();
    assert_eq!(proto.return_type, "double");
}

#[test]
fn test_strtol_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("strtol").unwrap();
    assert_eq!(proto.return_type, "long");
    assert_eq!(proto.parameters.len(), 3);
    assert_eq!(proto.parameters[2].name, "base");
    assert_eq!(proto.parameters[2].type_str, "int");
}

#[test]
fn test_strtod_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("strtod").unwrap();
    assert_eq!(proto.return_type, "double");
    assert_eq!(proto.parameters.len(), 2);
    assert_eq!(proto.parameters[1].type_str, "char**");
}

#[test]
fn test_exit_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("exit").unwrap();
    assert_eq!(proto.return_type, "void");
    assert_eq!(proto.parameters[0].name, "status");
}

#[test]
fn test_abort_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("abort").unwrap();
    assert_eq!(proto.return_type, "void");
    assert!(proto.parameters.is_empty());
}

#[test]
fn test_getenv_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("getenv").unwrap();
    assert_eq!(proto.return_type, "char*");
    assert_eq!(proto.parameters[0].type_str, "const char*");
}

#[test]
fn test_system_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("system").unwrap();
    assert_eq!(proto.return_type, "int");
    assert_eq!(proto.parameters[0].name, "command");
}

#[test]
fn test_qsort_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("qsort").unwrap();
    assert_eq!(proto.return_type, "void");
    assert_eq!(proto.parameters.len(), 4);
    assert!(proto.parameters[3].type_str.contains("(*"));
}

#[test]
fn test_bsearch_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("bsearch").unwrap();
    assert_eq!(proto.return_type, "void*");
    assert_eq!(proto.parameters.len(), 5);
    assert!(proto.parameters[4].type_str.contains("(*"));
}

#[test]
fn test_abs_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("abs").unwrap();
    assert_eq!(proto.return_type, "int");
    assert_eq!(proto.parameters[0].name, "j");
}

#[test]
fn test_labs_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("labs").unwrap();
    assert_eq!(proto.return_type, "long");
    assert_eq!(proto.parameters[0].type_str, "long");
}

#[test]
fn test_srand_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("srand").unwrap();
    assert_eq!(proto.return_type, "void");
    assert_eq!(proto.parameters[0].type_str, "unsigned int");
}

#[test]
fn test_rand_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("rand").unwrap();
    assert_eq!(proto.return_type, "int");
    assert!(proto.parameters.is_empty());
}

// ============================================================================
// Stdio function prototype details
// ============================================================================

#[test]
fn test_fprintf_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("fprintf").unwrap();
    assert_eq!(proto.parameters.len(), 2);
    assert_eq!(proto.parameters[0].type_str, "FILE*");
    assert!(proto.is_variadic);
}

#[test]
fn test_snprintf_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("snprintf").unwrap();
    assert_eq!(proto.parameters.len(), 3);
    assert!(proto.is_variadic);
}

#[test]
fn test_fscanf_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("fscanf").unwrap();
    assert!(proto.is_variadic);
    assert_eq!(proto.parameters[0].type_str, "FILE*");
}

#[test]
fn test_sscanf_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("sscanf").unwrap();
    assert!(proto.is_variadic);
    assert_eq!(proto.parameters[0].type_str, "const char*");
}

#[test]
fn test_fopen_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("fopen").unwrap();
    assert_eq!(proto.return_type, "FILE*");
    assert_eq!(proto.parameters.len(), 2);
}

#[test]
fn test_fread_fwrite_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let fread = stdlib.get_prototype("fread").unwrap();
    assert_eq!(fread.parameters.len(), 4);
    assert_eq!(fread.return_type, "size_t");

    let fwrite = stdlib.get_prototype("fwrite").unwrap();
    assert_eq!(fwrite.parameters.len(), 4);
    assert_eq!(fwrite.return_type, "size_t");
}

#[test]
fn test_fseek_ftell_rewind_prototype_details() {
    let stdlib = StdlibPrototypes::new();

    let fseek = stdlib.get_prototype("fseek").unwrap();
    assert_eq!(fseek.parameters.len(), 3);
    assert_eq!(fseek.parameters[1].name, "offset");

    let ftell = stdlib.get_prototype("ftell").unwrap();
    assert_eq!(ftell.return_type, "long");

    let rewind = stdlib.get_prototype("rewind").unwrap();
    assert_eq!(rewind.return_type, "void");
}

#[test]
fn test_getchar_putchar_prototype_details() {
    let stdlib = StdlibPrototypes::new();

    let getchar = stdlib.get_prototype("getchar").unwrap();
    assert!(getchar.parameters.is_empty());
    assert_eq!(getchar.return_type, "int");

    let putchar = stdlib.get_prototype("putchar").unwrap();
    assert_eq!(putchar.parameters.len(), 1);
}

#[test]
fn test_fgetc_fputc_prototype_details() {
    let stdlib = StdlibPrototypes::new();

    let fgetc = stdlib.get_prototype("fgetc").unwrap();
    assert_eq!(fgetc.parameters[0].type_str, "FILE*");

    let fputc = stdlib.get_prototype("fputc").unwrap();
    assert_eq!(fputc.parameters.len(), 2);
    assert_eq!(fputc.parameters[0].name, "c");
}

#[test]
fn test_fgets_fputs_puts_prototype_details() {
    let stdlib = StdlibPrototypes::new();

    let fgets = stdlib.get_prototype("fgets").unwrap();
    assert_eq!(fgets.parameters.len(), 3);
    assert_eq!(fgets.return_type, "char*");

    let fputs = stdlib.get_prototype("fputs").unwrap();
    assert_eq!(fputs.parameters.len(), 2);

    let puts = stdlib.get_prototype("puts").unwrap();
    assert_eq!(puts.parameters.len(), 1);
}

// ============================================================================
// String function prototype details
// ============================================================================

#[test]
fn test_memmove_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("memmove").unwrap();
    assert_eq!(proto.return_type, "void*");
    assert_eq!(proto.parameters.len(), 3);
}

#[test]
fn test_strncpy_strncat_strncmp_details() {
    let stdlib = StdlibPrototypes::new();

    let strncpy = stdlib.get_prototype("strncpy").unwrap();
    assert_eq!(strncpy.parameters.len(), 3);
    assert_eq!(strncpy.parameters[2].type_str, "size_t");

    let strncat = stdlib.get_prototype("strncat").unwrap();
    assert_eq!(strncat.parameters.len(), 3);

    let strncmp = stdlib.get_prototype("strncmp").unwrap();
    assert_eq!(strncmp.parameters.len(), 3);
}

#[test]
fn test_memchr_strchr_strrchr_strstr_strtok_details() {
    let stdlib = StdlibPrototypes::new();

    let memchr = stdlib.get_prototype("memchr").unwrap();
    assert_eq!(memchr.return_type, "void*");
    assert_eq!(memchr.parameters.len(), 3);

    let strchr = stdlib.get_prototype("strchr").unwrap();
    assert_eq!(strchr.return_type, "char*");

    let strrchr = stdlib.get_prototype("strrchr").unwrap();
    assert_eq!(strrchr.return_type, "char*");

    let strstr = stdlib.get_prototype("strstr").unwrap();
    assert_eq!(strstr.parameters[0].name, "haystack");

    let strtok = stdlib.get_prototype("strtok").unwrap();
    assert_eq!(strtok.parameters[1].name, "delim");
}

#[test]
fn test_strdup_prototype_details() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("strdup").unwrap();
    assert_eq!(proto.return_type, "char*");
    assert_eq!(proto.c99_section, "POSIX");
}

// ============================================================================
// Ctype function prototype details
// ============================================================================

#[test]
fn test_all_ctype_function_details() {
    let stdlib = StdlibPrototypes::new();

    for name in &[
        "isspace", "isdigit", "isalpha", "isalnum", "isupper", "islower", "tolower", "toupper",
    ] {
        let proto = stdlib.get_prototype(name).unwrap();
        assert_eq!(proto.return_type, "int", "return type for {name}");
        assert_eq!(proto.parameters.len(), 1, "param count for {name}");
        assert_eq!(proto.parameters[0].type_str, "int", "param type for {name}");
        assert_eq!(proto.header, StdHeader::Ctype, "header for {name}");
    }
}

// ============================================================================
// Time function prototype details
// ============================================================================

#[test]
fn test_clock_time_prototype_details() {
    let stdlib = StdlibPrototypes::new();

    let clock = stdlib.get_prototype("clock").unwrap();
    assert_eq!(clock.return_type, "clock_t");
    assert!(clock.parameters.is_empty());
    assert_eq!(clock.header, StdHeader::Time);

    let time = stdlib.get_prototype("time").unwrap();
    assert_eq!(time.return_type, "time_t");
    assert_eq!(time.parameters[0].type_str, "time_t*");
}

// ============================================================================
// Math function prototype details
// ============================================================================

#[test]
fn test_all_single_param_math_functions() {
    let stdlib = StdlibPrototypes::new();

    for name in &[
        "sqrt", "sin", "cos", "fabs", "ceil", "floor", "round", "trunc", "exp", "log", "log10",
        "tan", "asin", "acos", "atan",
    ] {
        let proto = stdlib.get_prototype(name).unwrap();
        assert_eq!(proto.return_type, "double", "return type for {name}");
        assert_eq!(proto.parameters.len(), 1, "param count for {name}");
        assert_eq!(
            proto.parameters[0].type_str, "double",
            "param type for {name}"
        );
        assert_eq!(proto.header, StdHeader::Math, "header for {name}");
    }
}

#[test]
fn test_dual_param_math_functions() {
    let stdlib = StdlibPrototypes::new();

    let pow = stdlib.get_prototype("pow").unwrap();
    assert_eq!(pow.parameters.len(), 2);
    assert_eq!(pow.parameters[0].name, "x");
    assert_eq!(pow.parameters[1].name, "y");

    let atan2 = stdlib.get_prototype("atan2").unwrap();
    assert_eq!(atan2.parameters.len(), 2);
    assert_eq!(atan2.parameters[0].name, "y");
    assert_eq!(atan2.parameters[1].name, "x");

    let fmod = stdlib.get_prototype("fmod").unwrap();
    assert_eq!(fmod.parameters.len(), 2);
}

// ============================================================================
// POSIX function prototype details
// ============================================================================

#[test]
fn test_posix_pipe_fork_details() {
    let stdlib = StdlibPrototypes::new();

    let pipe = stdlib.get_prototype("pipe").unwrap();
    assert_eq!(pipe.return_type, "int");
    assert_eq!(pipe.parameters[0].type_str, "int*");
    assert_eq!(pipe.header, StdHeader::Unistd);

    let fork = stdlib.get_prototype("fork").unwrap();
    assert_eq!(fork.return_type, "pid_t");
    assert!(fork.parameters.is_empty());
}

#[test]
fn test_posix_read_write_close_details() {
    let stdlib = StdlibPrototypes::new();

    let read = stdlib.get_prototype("read").unwrap();
    assert_eq!(read.return_type, "ssize_t");
    assert_eq!(read.parameters.len(), 3);

    let write = stdlib.get_prototype("write").unwrap();
    assert_eq!(write.return_type, "ssize_t");
    assert_eq!(write.parameters[1].type_str, "const void*");

    let close = stdlib.get_prototype("close").unwrap();
    assert_eq!(close.parameters[0].type_str, "int");
}

#[test]
fn test_posix_lseek_details() {
    let stdlib = StdlibPrototypes::new();
    let lseek = stdlib.get_prototype("lseek").unwrap();
    assert_eq!(lseek.return_type, "off_t");
    assert_eq!(lseek.parameters.len(), 3);
    assert_eq!(lseek.parameters[1].type_str, "off_t");
}

#[test]
fn test_posix_open_details() {
    let stdlib = StdlibPrototypes::new();
    let open = stdlib.get_prototype("open").unwrap();
    assert_eq!(open.return_type, "int");
    assert!(open.is_variadic); // Optional mode parameter
    assert_eq!(open.header, StdHeader::Fcntl);
}

#[test]
fn test_posix_dup_dup2_details() {
    let stdlib = StdlibPrototypes::new();

    let dup = stdlib.get_prototype("dup").unwrap();
    assert_eq!(dup.parameters.len(), 1);
    assert_eq!(dup.parameters[0].name, "oldfd");

    let dup2 = stdlib.get_prototype("dup2").unwrap();
    assert_eq!(dup2.parameters.len(), 2);
    assert_eq!(dup2.parameters[1].name, "newfd");
}

#[test]
fn test_posix_dirent_function_details() {
    let stdlib = StdlibPrototypes::new();

    let opendir = stdlib.get_prototype("opendir").unwrap();
    assert_eq!(opendir.return_type, "DIR*");
    assert_eq!(opendir.header, StdHeader::Dirent);

    let readdir = stdlib.get_prototype("readdir").unwrap();
    assert_eq!(readdir.return_type, "struct dirent*");

    let closedir = stdlib.get_prototype("closedir").unwrap();
    assert_eq!(closedir.return_type, "int");
}

// ============================================================================
// inject_all_prototypes comprehensive validation
// ============================================================================

#[test]
fn test_inject_all_prototypes_includes_posix_types() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_all_prototypes();

    assert!(result.contains("typedef int pid_t;"));
    assert!(result.contains("typedef long off_t;"));
    assert!(result.contains("typedef long time_t;"));
    assert!(result.contains("typedef long clock_t;"));
    assert!(result.contains("typedef int wchar_t;"));
    assert!(result.contains("extern int errno;"));
    assert!(result.contains("#define CLOCKS_PER_SEC 1000000"));
    assert!(result.contains("#define PATH_MAX 4096"));
}

#[test]
fn test_inject_all_prototypes_sorted_order() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_all_prototypes();

    // Find positions of some functions to verify sorted order
    let pos_abort = result.find("void abort(").unwrap();
    let pos_malloc = result.find("void* malloc(").unwrap();
    let pos_strlen = result.find("size_t strlen(").unwrap();

    assert!(pos_abort < pos_malloc, "abort should come before malloc");
    assert!(pos_malloc < pos_strlen, "malloc should come before strlen");
}

// ============================================================================
// Header-specific injection sorted order
// ============================================================================

#[test]
fn test_inject_header_functions_are_sorted() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::String);

    // Functions within a header injection should be sorted by name
    let pos_memchr = result.find("memchr").unwrap();
    let pos_memcpy = result.find("memcpy").unwrap();
    let pos_strlen = result.find("strlen").unwrap();

    assert!(pos_memchr < pos_memcpy);
    assert!(pos_memcpy < pos_strlen);
}

// ============================================================================
// Header comment injection
// ============================================================================

#[test]
fn test_inject_header_includes_debug_comment() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);

    assert!(result.contains("// Built-in prototypes for Stdlib (ISO C99"));
}

#[test]
fn test_inject_header_includes_null_macro() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);

    assert!(result.contains("#define NULL 0"));
}

// ============================================================================
// Parameter clone and equality
// ============================================================================

#[test]
fn test_parameter_clone_and_eq() {
    let p1 = Parameter::new("x", "int");
    let p2 = p1.clone();
    assert_eq!(p1, p2);
    assert_eq!(p1.name, p2.name);
    assert_eq!(p1.type_str, p2.type_str);
}

#[test]
fn test_parameter_debug() {
    let p = Parameter::new("ptr", "void*");
    let debug = format!("{:?}", p);
    assert!(debug.contains("ptr"));
    assert!(debug.contains("void*"));
}

// ============================================================================
// FunctionProto clone and equality
// ============================================================================

#[test]
fn test_function_proto_clone_and_eq() {
    let proto = FunctionProto {
        name: "test".to_string(),
        return_type: "int".to_string(),
        parameters: vec![Parameter::new("a", "int")],
        is_variadic: false,
        header: StdHeader::Stdlib,
        c99_section: "test".to_string(),
    };
    let cloned = proto.clone();
    assert_eq!(proto, cloned);
}

#[test]
fn test_function_proto_debug() {
    let proto = FunctionProto {
        name: "example".to_string(),
        return_type: "void".to_string(),
        parameters: vec![],
        is_variadic: false,
        header: StdHeader::Stdlib,
        c99_section: "test".to_string(),
    };
    let debug = format!("{:?}", proto);
    assert!(debug.contains("example"));
    assert!(debug.contains("void"));
}
