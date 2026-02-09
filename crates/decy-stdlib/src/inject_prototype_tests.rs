//! Comprehensive tests for inject_prototypes_for_header coverage.
//!
//! These tests exercise every branch in inject_prototypes_for_header,
//! including all StdHeader match arms, function filtering, sorting,
//! and function pointer parameter skipping.

use super::*;

// ============================================================================
// Common preamble verification
// ============================================================================

#[test]
fn test_preamble_iso_c99_section_reference() {
    let stdlib = StdlibPrototypes::new();
    for header in all_headers() {
        let result = stdlib.inject_prototypes_for_header(header);
        assert!(
            result.contains("(ISO C99 §7)"),
            "{:?} missing ISO C99 section reference in comment",
            header
        );
    }
}

#[test]
fn test_preamble_null_macro_definition() {
    let stdlib = StdlibPrototypes::new();
    // NULL should be defined as 0 (not ((void*)0)) to avoid parser issues
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);
    assert!(result.contains("#define NULL 0\n"));
    // Verify it's not the complex form
    assert!(!result.contains("((void*)0)"));
}

#[test]
fn test_preamble_types_for_every_header() {
    let stdlib = StdlibPrototypes::new();
    for header in all_headers() {
        let result = stdlib.inject_prototypes_for_header(header);
        // Every header gets these common type definitions
        assert!(
            result.contains("typedef unsigned long size_t;\n"),
            "{:?} missing size_t typedef",
            header
        );
        assert!(
            result.contains("typedef long ssize_t;\n"),
            "{:?} missing ssize_t typedef",
            header
        );
        assert!(
            result.contains("typedef long ptrdiff_t;\n"),
            "{:?} missing ptrdiff_t typedef",
            header
        );
    }
}

// ============================================================================
// Stdio header (StdHeader::Stdio) - extensive coverage
// ============================================================================

#[test]
fn test_stdio_file_type_and_streams() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdio);
    // Struct and typedef
    assert!(result.contains("struct _IO_FILE;"));
    assert!(result.contains("typedef struct _IO_FILE FILE;"));
    // Standard streams (DECY-239)
    assert!(result.contains("extern FILE* stdin;"));
    assert!(result.contains("extern FILE* stdout;"));
    assert!(result.contains("extern FILE* stderr;"));
}

#[test]
fn test_stdio_all_macros_present() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdio);
    let expected_macros = [
        "#define EOF (-1)",
        "#define SEEK_SET 0",
        "#define SEEK_CUR 1",
        "#define SEEK_END 2",
        "#define BUFSIZ 8192",
        "#define L_tmpnam 20",
        "#define _IONBF 2",
        "#define _IOLBF 1",
        "#define _IOFBF 0",
    ];
    for m in &expected_macros {
        assert!(
            result.contains(m),
            "Stdio missing macro: {}",
            m
        );
    }
}

#[test]
fn test_stdio_formatted_io_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdio);
    // printf family
    assert!(result.contains("int printf(const char* format, ...);"));
    assert!(result.contains("int fprintf(FILE* stream, const char* format, ...);"));
    assert!(result.contains("int sprintf(char* str, const char* format, ...);"));
    assert!(result.contains("int snprintf(char* str, size_t size, const char* format, ...);"));
    // scanf family
    assert!(result.contains("int scanf(const char* format, ...);"));
    assert!(result.contains("int fscanf(FILE* stream, const char* format, ...);"));
    assert!(result.contains("int sscanf(const char* str, const char* format, ...);"));
}

#[test]
fn test_stdio_character_io_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdio);
    assert!(result.contains("int getchar(void);"));
    assert!(result.contains("int putchar(int c);"));
    assert!(result.contains("int fgetc(FILE* stream);"));
    assert!(result.contains("int fputc(int c, FILE* stream);"));
    assert!(result.contains("int puts(const char* s);"));
}

#[test]
fn test_stdio_file_operation_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdio);
    assert!(result.contains("int fclose(FILE* stream);"));
    assert!(result.contains("long ftell(FILE* stream);"));
    assert!(result.contains("void rewind(FILE* stream);"));
}

#[test]
fn test_stdio_no_stdlib_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdio);
    assert!(!result.contains("void* malloc("));
    assert!(!result.contains("void free("));
    assert!(!result.contains("int atoi("));
    assert!(!result.contains("void exit("));
}

// ============================================================================
// Errno header (StdHeader::Errno)
// ============================================================================

#[test]
fn test_errno_all_error_codes() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Errno);
    assert!(result.contains("extern int errno;"));
    assert!(result.contains("#define EACCES 13"));
    assert!(result.contains("#define ENOENT 2"));
    assert!(result.contains("#define EINVAL 22"));
    assert!(result.contains("#define ENOMEM 12"));
    assert!(result.contains("#define ERANGE 34"));
}

#[test]
fn test_errno_no_functions_registered() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Errno);
    // Only macros and extern declarations, no function prototypes
    let lines: Vec<&str> = result.lines().collect();
    for line in &lines {
        if line.contains('(') && line.contains(')') && !line.starts_with("//") && !line.starts_with("#") && !line.starts_with("typedef") {
            panic!("Errno should have no function declarations, found: {}", line);
        }
    }
}

// ============================================================================
// Time header (StdHeader::Time)
// ============================================================================

#[test]
fn test_time_type_definitions_complete() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Time);
    assert!(result.contains("typedef long time_t;"));
    assert!(result.contains("typedef long clock_t;"));
    assert!(result.contains("struct tm;"));
    assert!(result.contains("#define CLOCKS_PER_SEC 1000000"));
}

#[test]
fn test_time_functions_complete() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Time);
    assert!(result.contains("clock_t clock(void);"));
    assert!(result.contains("time_t time(time_t* timer);"));
}

#[test]
fn test_time_no_stdio_types() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Time);
    assert!(!result.contains("typedef struct _IO_FILE FILE;"));
    assert!(!result.contains("extern FILE*"));
}

// ============================================================================
// Stdarg header (StdHeader::Stdarg)
// ============================================================================

#[test]
fn test_stdarg_va_list_type() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdarg);
    assert!(result.contains("typedef void* va_list;"));
}

#[test]
fn test_stdarg_va_macros() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdarg);
    assert!(result.contains("#define va_start(ap, last) ((void)0)"));
    assert!(result.contains("#define va_end(ap) ((void)0)"));
    assert!(result.contains("#define va_arg(ap, type) (*(type*)0)"));
}

// ============================================================================
// Stdbool header (StdHeader::Stdbool)
// ============================================================================

#[test]
fn test_stdbool_all_definitions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdbool);
    assert!(result.contains("typedef _Bool bool;"));
    assert!(result.contains("#define true 1"));
    assert!(result.contains("#define false 0"));
}

#[test]
fn test_stdbool_no_function_prototypes() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdbool);
    // Stdbool has no registered functions
    assert!(!result.contains("int printf("));
    assert!(!result.contains("void* malloc("));
}

// ============================================================================
// Stdint header (StdHeader::Stdint)
// ============================================================================

#[test]
fn test_stdint_signed_types() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdint);
    assert!(result.contains("typedef signed char int8_t;"));
    assert!(result.contains("typedef short int16_t;"));
    assert!(result.contains("typedef int int32_t;"));
    assert!(result.contains("typedef long long int64_t;"));
}

#[test]
fn test_stdint_unsigned_types() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdint);
    assert!(result.contains("typedef unsigned char uint8_t;"));
    assert!(result.contains("typedef unsigned short uint16_t;"));
    assert!(result.contains("typedef unsigned int uint32_t;"));
    assert!(result.contains("typedef unsigned long long uint64_t;"));
}

#[test]
fn test_stdint_pointer_types() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdint);
    assert!(result.contains("typedef long intptr_t;"));
    assert!(result.contains("typedef unsigned long uintptr_t;"));
}

// ============================================================================
// Unistd header (StdHeader::Unistd)
// ============================================================================

#[test]
fn test_unistd_posix_types() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Unistd);
    assert!(result.contains("typedef int pid_t;"));
    assert!(result.contains("typedef long off_t;"));
    assert!(result.contains("typedef unsigned int uid_t;"));
    assert!(result.contains("typedef unsigned int gid_t;"));
}

#[test]
fn test_unistd_fileno_macros() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Unistd);
    assert!(result.contains("#define STDIN_FILENO 0"));
    assert!(result.contains("#define STDOUT_FILENO 1"));
    assert!(result.contains("#define STDERR_FILENO 2"));
}

#[test]
fn test_unistd_access_macros() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Unistd);
    assert!(result.contains("#define F_OK 0"));
    assert!(result.contains("#define R_OK 4"));
    assert!(result.contains("#define W_OK 2"));
    assert!(result.contains("#define X_OK 1"));
}

#[test]
fn test_unistd_sysconf_macros() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Unistd);
    assert!(result.contains("#define _SC_OPEN_MAX 4"));
    assert!(result.contains("#define _SC_PAGESIZE 30"));
}

#[test]
fn test_unistd_io_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Unistd);
    assert!(result.contains("ssize_t read(int fd, void* buf, size_t count);"));
    assert!(result.contains("ssize_t write(int fd, const void* buf, size_t count);"));
    assert!(result.contains("int close(int fd);"));
    assert!(result.contains("off_t lseek(int fd, off_t offset, int whence);"));
}

#[test]
fn test_unistd_process_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Unistd);
    assert!(result.contains("pid_t fork(void);"));
    assert!(result.contains("int pipe(int* pipefd);"));
    assert!(result.contains("int dup(int oldfd);"));
    assert!(result.contains("int dup2(int oldfd, int newfd);"));
}

// ============================================================================
// Fcntl header (StdHeader::Fcntl)
// ============================================================================

#[test]
fn test_fcntl_file_mode_macros() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Fcntl);
    assert!(result.contains("#define O_RDONLY 0"));
    assert!(result.contains("#define O_WRONLY 1"));
    assert!(result.contains("#define O_RDWR 2"));
}

#[test]
fn test_fcntl_creation_macros() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Fcntl);
    assert!(result.contains("#define O_CREAT 0100"));
    assert!(result.contains("#define O_TRUNC 01000"));
    assert!(result.contains("#define O_APPEND 02000"));
    assert!(result.contains("#define O_NONBLOCK 04000"));
}

#[test]
fn test_fcntl_lock_macros() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Fcntl);
    assert!(result.contains("#define LOCK_SH 1"));
    assert!(result.contains("#define LOCK_EX 2"));
    assert!(result.contains("#define LOCK_UN 8"));
}

#[test]
fn test_fcntl_open_function() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Fcntl);
    assert!(result.contains("int open(const char* pathname, int flags, ...);"));
}

// ============================================================================
// Dirent header (StdHeader::Dirent)
// ============================================================================

#[test]
fn test_dirent_type_definitions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Dirent);
    assert!(result.contains("struct dirent { char d_name[256]; };"));
    assert!(result.contains("typedef struct __dirstream DIR;"));
}

#[test]
fn test_dirent_all_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Dirent);
    assert!(result.contains("DIR* opendir(const char* name);"));
    assert!(result.contains("struct dirent* readdir(DIR* dirp);"));
    assert!(result.contains("int closedir(DIR* dirp);"));
}

// ============================================================================
// SysTypes header (StdHeader::SysTypes)
// ============================================================================

#[test]
fn test_sys_types_all_typedefs() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::SysTypes);
    assert!(result.contains("typedef int pid_t;"));
    assert!(result.contains("typedef long off_t;"));
    assert!(result.contains("typedef unsigned int mode_t;"));
    assert!(result.contains("typedef long ssize_t;"));
}

#[test]
fn test_sys_types_no_extra_definitions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::SysTypes);
    // SysTypes only has type definitions, no special macros
    assert!(!result.contains("#define EOF"));
    assert!(!result.contains("extern FILE*"));
}

// ============================================================================
// SysStat header (StdHeader::SysStat)
// ============================================================================

#[test]
fn test_sys_stat_struct_definition() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::SysStat);
    assert!(result.contains("struct stat { long st_size; int st_mode; };"));
}

#[test]
fn test_sys_stat_file_mode_macros() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::SysStat);
    assert!(result.contains("#define S_ISREG(m) (((m) & 0170000) == 0100000)"));
    assert!(result.contains("#define S_ISDIR(m) (((m) & 0170000) == 0040000)"));
}

// ============================================================================
// SysMman header (StdHeader::SysMman)
// ============================================================================

#[test]
fn test_sys_mman_protection_macros() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::SysMman);
    assert!(result.contains("#define PROT_NONE 0"));
    assert!(result.contains("#define PROT_READ 1"));
    assert!(result.contains("#define PROT_WRITE 2"));
    assert!(result.contains("#define PROT_EXEC 4"));
}

#[test]
fn test_sys_mman_map_macros() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::SysMman);
    assert!(result.contains("#define MAP_SHARED 1"));
    assert!(result.contains("#define MAP_PRIVATE 2"));
    assert!(result.contains("#define MAP_ANONYMOUS 0x20"));
    assert!(result.contains("#define MAP_FAILED ((void*)-1)"));
}

#[test]
fn test_sys_mman_no_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::SysMman);
    // No functions are registered under SysMman
    assert!(!result.contains("int printf("));
    assert!(!result.contains("void* malloc("));
}

// ============================================================================
// Wchar header (StdHeader::Wchar)
// ============================================================================

#[test]
fn test_wchar_type_definitions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Wchar);
    assert!(result.contains("typedef int wchar_t;"));
    assert!(result.contains("typedef int wint_t;"));
}

#[test]
fn test_wchar_eof_macro() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Wchar);
    assert!(result.contains("#define WEOF (-1)"));
}

// ============================================================================
// Signal header (StdHeader::Signal)
// ============================================================================

#[test]
fn test_signal_handler_type() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Signal);
    assert!(result.contains("typedef void (*sighandler_t)(int);"));
}

#[test]
fn test_signal_number_macros() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Signal);
    assert!(result.contains("#define SIGINT 2"));
    assert!(result.contains("#define SIGTERM 15"));
}

// ============================================================================
// Limits header (StdHeader::Limits)
// ============================================================================

#[test]
fn test_limits_char_macros() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Limits);
    assert!(result.contains("#define CHAR_BIT 8"));
    assert!(result.contains("#define CHAR_MIN (-128)"));
    assert!(result.contains("#define CHAR_MAX 127"));
}

#[test]
fn test_limits_short_macros() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Limits);
    assert!(result.contains("#define SHRT_MIN (-32768)"));
    assert!(result.contains("#define SHRT_MAX 32767"));
}

#[test]
fn test_limits_int_macros() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Limits);
    assert!(result.contains("#define INT_MIN (-2147483647-1)"));
    assert!(result.contains("#define INT_MAX 2147483647"));
    assert!(result.contains("#define UINT_MAX 4294967295U"));
}

#[test]
fn test_limits_long_macros() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Limits);
    assert!(result.contains("#define LONG_MIN (-9223372036854775807L-1)"));
    assert!(result.contains("#define LONG_MAX 9223372036854775807L"));
}

#[test]
fn test_limits_path_max() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Limits);
    assert!(result.contains("#define PATH_MAX 4096"));
}

// ============================================================================
// Ctype header (StdHeader::Ctype)
// ============================================================================

#[test]
fn test_ctype_classification_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Ctype);
    assert!(result.contains("int isspace(int c);"));
    assert!(result.contains("int isdigit(int c);"));
    assert!(result.contains("int isalpha(int c);"));
    assert!(result.contains("int isalnum(int c);"));
    assert!(result.contains("int isupper(int c);"));
    assert!(result.contains("int islower(int c);"));
}

#[test]
fn test_ctype_conversion_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Ctype);
    assert!(result.contains("int tolower(int c);"));
    assert!(result.contains("int toupper(int c);"));
}

#[test]
fn test_ctype_no_extra_header_types() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Ctype);
    // Ctype has no header-specific type definitions
    assert!(!result.contains("typedef struct _IO_FILE FILE;"));
    assert!(!result.contains("typedef long time_t;"));
    assert!(!result.contains("extern int errno;"));
    assert!(!result.contains("typedef void* va_list;"));
}

// ============================================================================
// Math header (StdHeader::Math)
// ============================================================================

#[test]
fn test_math_constant_macros() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Math);
    assert!(result.contains("#define M_PI 3.14159265358979323846"));
    assert!(result.contains("#define M_E 2.71828182845904523536"));
    assert!(result.contains("#define INFINITY (1.0/0.0)"));
    assert!(result.contains("#define NAN (0.0/0.0)"));
}

#[test]
fn test_math_trig_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Math);
    assert!(result.contains("double sin(double x);"));
    assert!(result.contains("double cos(double x);"));
    assert!(result.contains("double tan(double x);"));
    assert!(result.contains("double asin(double x);"));
    assert!(result.contains("double acos(double x);"));
    assert!(result.contains("double atan(double x);"));
    assert!(result.contains("double atan2(double y, double x);"));
}

#[test]
fn test_math_exponential_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Math);
    assert!(result.contains("double exp(double x);"));
    assert!(result.contains("double log(double x);"));
    assert!(result.contains("double log10(double x);"));
    assert!(result.contains("double pow(double x, double y);"));
    assert!(result.contains("double sqrt(double x);"));
}

#[test]
fn test_math_rounding_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Math);
    assert!(result.contains("double fabs(double x);"));
    assert!(result.contains("double ceil(double x);"));
    assert!(result.contains("double floor(double x);"));
    assert!(result.contains("double round(double x);"));
    assert!(result.contains("double trunc(double x);"));
    assert!(result.contains("double fmod(double x, double y);"));
}

#[test]
fn test_math_no_stdio_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Math);
    assert!(!result.contains("int printf("));
    assert!(!result.contains("FILE*"));
}

// ============================================================================
// Stdlib header (StdHeader::Stdlib)
// ============================================================================

#[test]
fn test_stdlib_memory_management() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);
    assert!(result.contains("void* malloc(size_t size);"));
    assert!(result.contains("void* calloc(size_t nmemb, size_t size);"));
    assert!(result.contains("void* realloc(void* ptr, size_t size);"));
    assert!(result.contains("void free(void* ptr);"));
}

#[test]
fn test_stdlib_string_conversion() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);
    assert!(result.contains("int atoi(const char* nptr);"));
    assert!(result.contains("long atol(const char* nptr);"));
    assert!(result.contains("double atof(const char* nptr);"));
    assert!(result.contains("long strtol(const char* nptr, char** endptr, int base);"));
    assert!(result.contains("double strtod(const char* nptr, char** endptr);"));
}

#[test]
fn test_stdlib_process_control() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);
    assert!(result.contains("void exit(int status);"));
    assert!(result.contains("void abort(void);"));
    assert!(result.contains("char* getenv(const char* name);"));
    assert!(result.contains("int system(const char* command);"));
}

#[test]
fn test_stdlib_random_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);
    assert!(result.contains("int rand(void);"));
    assert!(result.contains("void srand(unsigned int seed);"));
}

#[test]
fn test_stdlib_arithmetic_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);
    assert!(result.contains("int abs(int j);"));
    assert!(result.contains("long labs(long j);"));
}

#[test]
fn test_stdlib_function_pointer_params_skipped() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);
    // qsort and bsearch have function pointer parameters (contain "(*") and should be filtered
    assert!(
        !result.contains("void qsort("),
        "qsort with function pointer param should be filtered"
    );
    assert!(
        !result.contains("void* bsearch("),
        "bsearch with function pointer param should be filtered"
    );
}

// ============================================================================
// String header (StdHeader::String)
// ============================================================================

#[test]
fn test_string_memory_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::String);
    assert!(result.contains("void* memcpy(void* dest, const void* src, size_t n);"));
    assert!(result.contains("void* memmove(void* dest, const void* src, size_t n);"));
    assert!(result.contains("void* memset(void* s, int c, size_t n);"));
    assert!(result.contains("void* memchr(const void* s, int c, size_t n);"));
    assert!(result.contains("int memcmp(const void* s1, const void* s2, size_t n);"));
}

#[test]
fn test_string_copy_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::String);
    assert!(result.contains("char* strcpy(char* dest, const char* src);"));
    assert!(result.contains("char* strncpy(char* dest, const char* src, size_t n);"));
}

#[test]
fn test_string_concat_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::String);
    assert!(result.contains("char* strcat(char* dest, const char* src);"));
    assert!(result.contains("char* strncat(char* dest, const char* src, size_t n);"));
}

#[test]
fn test_string_comparison_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::String);
    assert!(result.contains("int strcmp(const char* s1, const char* s2);"));
    assert!(result.contains("int strncmp(const char* s1, const char* s2, size_t n);"));
}

#[test]
fn test_string_search_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::String);
    assert!(result.contains("char* strchr(const char* s, int c);"));
    assert!(result.contains("char* strrchr(const char* s, int c);"));
    assert!(result.contains("char* strstr(const char* haystack, const char* needle);"));
    assert!(result.contains("char* strtok(char* str, const char* delim);"));
}

#[test]
fn test_string_utility_functions() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::String);
    assert!(result.contains("size_t strlen(const char* s);"));
    assert!(result.contains("char* strdup(const char* s);"));
}

// ============================================================================
// Wildcard arm headers (Assert, Float, Locale, Setjmp, Stddef)
// ============================================================================

#[test]
fn test_assert_wildcard_arm() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Assert);
    assert!(result.contains("// Built-in prototypes for Assert"));
    // Only common preamble, no header-specific definitions
    assert!(!result.contains("typedef struct _IO_FILE FILE;"));
    assert!(!result.contains("extern int errno;"));
}

#[test]
fn test_float_wildcard_arm() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Float);
    assert!(result.contains("// Built-in prototypes for Float"));
    assert!(!result.contains("double sin("));
}

#[test]
fn test_locale_wildcard_arm() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Locale);
    assert!(result.contains("// Built-in prototypes for Locale"));
}

#[test]
fn test_setjmp_wildcard_arm() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Setjmp);
    assert!(result.contains("// Built-in prototypes for Setjmp"));
}

#[test]
fn test_stddef_wildcard_arm() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stddef);
    assert!(result.contains("// Built-in prototypes for Stddef"));
    // Even wildcard arms get common preamble
    assert!(result.contains("typedef unsigned long size_t;"));
    assert!(result.contains("typedef long ptrdiff_t;"));
}

// ============================================================================
// Cross-header isolation tests
// ============================================================================

#[test]
fn test_header_isolation_stdio_vs_stdlib() {
    let stdlib = StdlibPrototypes::new();
    let stdio = stdlib.inject_prototypes_for_header(StdHeader::Stdio);
    let stdlib_result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);

    // stdio should not have stdlib functions
    assert!(!stdio.contains("void* malloc("));
    assert!(!stdio.contains("void free("));
    assert!(!stdio.contains("int atoi("));

    // stdlib should not have stdio functions
    assert!(!stdlib_result.contains("int printf("));
    assert!(!stdlib_result.contains("FILE* fopen("));
}

#[test]
fn test_header_isolation_string_vs_math() {
    let stdlib = StdlibPrototypes::new();
    let string_result = stdlib.inject_prototypes_for_header(StdHeader::String);
    let math_result = stdlib.inject_prototypes_for_header(StdHeader::Math);

    assert!(!string_result.contains("double sin("));
    assert!(!string_result.contains("#define M_PI"));
    assert!(!math_result.contains("size_t strlen("));
    assert!(!math_result.contains("void* memcpy("));
}

#[test]
fn test_header_isolation_ctype_vs_time() {
    let stdlib = StdlibPrototypes::new();
    let ctype = stdlib.inject_prototypes_for_header(StdHeader::Ctype);
    let time = stdlib.inject_prototypes_for_header(StdHeader::Time);

    assert!(!ctype.contains("clock_t clock("));
    assert!(!ctype.contains("typedef long time_t;"));
    assert!(!time.contains("int isspace("));
    assert!(!time.contains("int toupper("));
}

// ============================================================================
// Function sorting and declaration format
// ============================================================================

#[test]
fn test_functions_sorted_alphabetically_in_stdlib() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);

    let abs_pos = result.find("int abs(").unwrap();
    let atof_pos = result.find("double atof(").unwrap();
    let atoi_pos = result.find("int atoi(").unwrap();
    let exit_pos = result.find("void exit(").unwrap();
    let malloc_pos = result.find("void* malloc(").unwrap();

    assert!(abs_pos < atof_pos);
    assert!(atof_pos < atoi_pos);
    assert!(atoi_pos < exit_pos);
    assert!(exit_pos < malloc_pos);
}

#[test]
fn test_functions_sorted_alphabetically_in_math() {
    let stdlib = StdlibPrototypes::new();
    let result = stdlib.inject_prototypes_for_header(StdHeader::Math);

    let acos_pos = result.find("double acos(").unwrap();
    let asin_pos = result.find("double asin(").unwrap();
    let cos_pos = result.find("double cos(").unwrap();
    let sin_pos = result.find("double sin(").unwrap();

    assert!(acos_pos < asin_pos);
    assert!(asin_pos < cos_pos);
    assert!(cos_pos < sin_pos);
}

#[test]
fn test_all_declarations_end_with_semicolons() {
    let stdlib = StdlibPrototypes::new();
    for header in all_headers() {
        let result = stdlib.inject_prototypes_for_header(header);
        for line in result.lines() {
            // Skip comments, macros, typedefs, struct defs, extern decls, and empty lines
            if line.starts_with("//")
                || line.starts_with("#")
                || line.starts_with("typedef")
                || line.starts_with("struct")
                || line.starts_with("extern")
                || line.is_empty()
            {
                continue;
            }
            // Function declarations must end with ;
            if line.contains('(') && line.contains(')') {
                assert!(
                    line.ends_with(';'),
                    "{:?}: Function line should end with semicolon: {}",
                    header,
                    line
                );
            }
        }
    }
}

// ============================================================================
// get_prototype and inject_all_prototypes
// ============================================================================

#[test]
fn test_get_prototype_existing_function() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("malloc");
    assert!(proto.is_some());
    let proto = proto.unwrap();
    assert_eq!(proto.name, "malloc");
    assert_eq!(proto.return_type, "void*");
    assert_eq!(proto.header, StdHeader::Stdlib);
}

#[test]
fn test_get_prototype_nonexistent_function() {
    let stdlib = StdlibPrototypes::new();
    let proto = stdlib.get_prototype("nonexistent_function");
    assert!(proto.is_none());
}

#[test]
fn test_get_prototype_each_header_has_functions() {
    let stdlib = StdlibPrototypes::new();
    // Verify some representative functions from key headers
    assert!(stdlib.get_prototype("printf").is_some());
    assert!(stdlib.get_prototype("strlen").is_some());
    assert!(stdlib.get_prototype("sin").is_some());
    assert!(stdlib.get_prototype("isalpha").is_some());
    assert!(stdlib.get_prototype("fork").is_some());
    assert!(stdlib.get_prototype("opendir").is_some());
}

#[test]
fn test_inject_all_prototypes_contains_all_headers() {
    let stdlib = StdlibPrototypes::new();
    let all = stdlib.inject_all_prototypes();
    // Should contain functions from multiple headers
    assert!(all.contains("int printf("));
    assert!(all.contains("void* malloc("));
    assert!(all.contains("size_t strlen("));
    assert!(all.contains("double sin("));
    assert!(all.contains("int isspace("));
    assert!(all.contains("pid_t fork("));
}

#[test]
fn test_inject_all_prototypes_has_common_types() {
    let stdlib = StdlibPrototypes::new();
    let all = stdlib.inject_all_prototypes();
    assert!(all.contains("typedef unsigned long size_t;"));
    assert!(all.contains("typedef long ssize_t;"));
    assert!(all.contains("typedef long ptrdiff_t;"));
    assert!(all.contains("#define NULL 0"));
    assert!(all.contains("struct _IO_FILE;"));
    assert!(all.contains("typedef struct _IO_FILE FILE;"));
}

#[test]
fn test_inject_all_prototypes_has_posix_types() {
    let stdlib = StdlibPrototypes::new();
    let all = stdlib.inject_all_prototypes();
    assert!(all.contains("typedef int pid_t;"));
    assert!(all.contains("typedef long off_t;"));
    assert!(all.contains("typedef long time_t;"));
    assert!(all.contains("typedef long clock_t;"));
    assert!(all.contains("typedef int wchar_t;"));
    assert!(all.contains("extern int errno;"));
}

#[test]
fn test_inject_all_prototypes_sorted() {
    let stdlib = StdlibPrototypes::new();
    let all = stdlib.inject_all_prototypes();
    // Functions should be sorted alphabetically
    if let (Some(abs_pos), Some(malloc_pos)) = (all.find("int abs("), all.find("void* malloc(")) {
        assert!(abs_pos < malloc_pos);
    }
}

// ============================================================================
// Default trait implementation
// ============================================================================

#[test]
fn test_stdlib_prototypes_default() {
    let stdlib = StdlibPrototypes::default();
    // Default should produce the same result as new()
    let proto = stdlib.get_prototype("malloc");
    assert!(proto.is_some());
}

// ============================================================================
// StdHeader::from_filename
// ============================================================================

#[test]
fn test_from_filename_all_standard_headers() {
    let cases = vec![
        ("assert.h", Some(StdHeader::Assert)),
        ("ctype.h", Some(StdHeader::Ctype)),
        ("errno.h", Some(StdHeader::Errno)),
        ("float.h", Some(StdHeader::Float)),
        ("limits.h", Some(StdHeader::Limits)),
        ("locale.h", Some(StdHeader::Locale)),
        ("math.h", Some(StdHeader::Math)),
        ("setjmp.h", Some(StdHeader::Setjmp)),
        ("signal.h", Some(StdHeader::Signal)),
        ("stdarg.h", Some(StdHeader::Stdarg)),
        ("stdbool.h", Some(StdHeader::Stdbool)),
        ("stddef.h", Some(StdHeader::Stddef)),
        ("stdint.h", Some(StdHeader::Stdint)),
        ("stdio.h", Some(StdHeader::Stdio)),
        ("stdlib.h", Some(StdHeader::Stdlib)),
        ("string.h", Some(StdHeader::String)),
        ("time.h", Some(StdHeader::Time)),
    ];
    for (filename, expected) in cases {
        assert_eq!(
            StdHeader::from_filename(filename),
            expected,
            "from_filename({}) failed",
            filename
        );
    }
}

#[test]
fn test_from_filename_posix_headers() {
    assert_eq!(StdHeader::from_filename("unistd.h"), Some(StdHeader::Unistd));
    assert_eq!(StdHeader::from_filename("fcntl.h"), Some(StdHeader::Fcntl));
    assert_eq!(StdHeader::from_filename("dirent.h"), Some(StdHeader::Dirent));
    assert_eq!(StdHeader::from_filename("sys/types.h"), Some(StdHeader::SysTypes));
    assert_eq!(StdHeader::from_filename("sys/stat.h"), Some(StdHeader::SysStat));
    assert_eq!(StdHeader::from_filename("sys/mman.h"), Some(StdHeader::SysMman));
    assert_eq!(StdHeader::from_filename("wchar.h"), Some(StdHeader::Wchar));
}

#[test]
fn test_from_filename_unknown_header() {
    assert_eq!(StdHeader::from_filename("unknown.h"), None);
    assert_eq!(StdHeader::from_filename(""), None);
    assert_eq!(StdHeader::from_filename("custom_header.h"), None);
    assert_eq!(StdHeader::from_filename("pthread.h"), None);
}

// ============================================================================
// FunctionProto to_c_declaration tests
// ============================================================================

#[test]
fn test_to_c_declaration_single_param() {
    let proto = FunctionProto {
        name: "malloc".to_string(),
        return_type: "void*".to_string(),
        parameters: vec![Parameter::new("size", "size_t")],
        is_variadic: false,
        header: StdHeader::Stdlib,
        c99_section: "§7.22.3.4".to_string(),
    };
    assert_eq!(proto.to_c_declaration(), "void* malloc(size_t size);");
}

#[test]
fn test_to_c_declaration_multiple_params() {
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
        c99_section: "§7.24.2.1".to_string(),
    };
    assert_eq!(
        proto.to_c_declaration(),
        "void* memcpy(void* dest, const void* src, size_t n);"
    );
}

#[test]
fn test_to_c_declaration_variadic() {
    let proto = FunctionProto {
        name: "printf".to_string(),
        return_type: "int".to_string(),
        parameters: vec![Parameter::new("format", "const char*")],
        is_variadic: true,
        header: StdHeader::Stdio,
        c99_section: "§7.21.6.1".to_string(),
    };
    assert_eq!(
        proto.to_c_declaration(),
        "int printf(const char* format, ...);"
    );
}

#[test]
fn test_to_c_declaration_no_params() {
    let proto = FunctionProto {
        name: "rand".to_string(),
        return_type: "int".to_string(),
        parameters: vec![],
        is_variadic: false,
        header: StdHeader::Stdlib,
        c99_section: "§7.22.2.1".to_string(),
    };
    assert_eq!(proto.to_c_declaration(), "int rand(void);");
}

// ============================================================================
// Helpers
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
