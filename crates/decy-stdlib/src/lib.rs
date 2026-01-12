//! # DECY Standard Library Support
//!
//! **RED PHASE**: Built-in C standard library function prototypes
//!
//! This crate provides ISO C99 §7 standard library function prototypes,
//! enabling transpilation of C code that uses stdlib functions without
//! requiring actual header files.
//!
//! **Pattern**: EXTREME TDD - Test-First Development
//! **References**: See docs/specifications/header-support-spec.md
//!
//! ## Architecture
//!
//! ```text
//! C Code → Prototype Injection → Parser → HIR → Rust
//!   ↓
//! #include <stdlib.h>
//!   ↓ (commented out by preprocessor)
//! Built-in prototypes injected
//!   ↓
//! malloc/free declarations available
//!   ↓
//! Parser succeeds!
//! ```

use std::collections::HashMap;

/// ISO C99 §7 Standard Library Headers + POSIX extensions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StdHeader {
    Assert,  // <assert.h>
    Ctype,   // <ctype.h>
    Errno,   // <errno.h>
    Float,   // <float.h>
    Limits,  // <limits.h>
    Locale,  // <locale.h>
    Math,    // <math.h>
    Setjmp,  // <setjmp.h>
    Signal,  // <signal.h>
    Stdarg,  // <stdarg.h>
    Stdbool, // <stdbool.h>
    Stddef,  // <stddef.h>
    Stdint,  // <stdint.h>
    Stdio,   // <stdio.h>
    Stdlib,  // <stdlib.h>
    String,  // <string.h>
    Time,    // <time.h>
    // POSIX headers
    Unistd,   // <unistd.h>
    Fcntl,    // <fcntl.h>
    Dirent,   // <dirent.h>
    SysTypes, // <sys/types.h>
    SysStat,  // <sys/stat.h>
    SysMman,  // <sys/mman.h>
    Wchar,    // <wchar.h>
}

impl StdHeader {
    /// Parse header name from #include filename
    ///
    /// # Examples
    /// ```
    /// use decy_stdlib::StdHeader;
    /// assert_eq!(StdHeader::from_filename("string.h"), Some(StdHeader::String));
    /// assert_eq!(StdHeader::from_filename("stdio.h"), Some(StdHeader::Stdio));
    /// assert_eq!(StdHeader::from_filename("unknown.h"), None);
    /// ```
    pub fn from_filename(filename: &str) -> Option<Self> {
        match filename {
            "assert.h" => Some(Self::Assert),
            "ctype.h" => Some(Self::Ctype),
            "errno.h" => Some(Self::Errno),
            "float.h" => Some(Self::Float),
            "limits.h" => Some(Self::Limits),
            "locale.h" => Some(Self::Locale),
            "math.h" => Some(Self::Math),
            "setjmp.h" => Some(Self::Setjmp),
            "signal.h" => Some(Self::Signal),
            "stdarg.h" => Some(Self::Stdarg),
            "stdbool.h" => Some(Self::Stdbool),
            "stddef.h" => Some(Self::Stddef),
            "stdint.h" => Some(Self::Stdint),
            "stdio.h" => Some(Self::Stdio),
            "stdlib.h" => Some(Self::Stdlib),
            "string.h" => Some(Self::String),
            "time.h" => Some(Self::Time),
            // POSIX headers
            "unistd.h" => Some(Self::Unistd),
            "fcntl.h" => Some(Self::Fcntl),
            "dirent.h" => Some(Self::Dirent),
            "sys/types.h" => Some(Self::SysTypes),
            "sys/stat.h" => Some(Self::SysStat),
            "sys/mman.h" => Some(Self::SysMman),
            "wchar.h" => Some(Self::Wchar),
            _ => None,
        }
    }
}

/// Function parameter
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub type_str: String,
}

impl Parameter {
    pub fn new(name: impl Into<String>, type_str: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            type_str: type_str.into(),
        }
    }
}

/// C Standard Library Function Prototype
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionProto {
    pub name: String,
    pub return_type: String,
    pub parameters: Vec<Parameter>,
    pub is_variadic: bool,
    pub header: StdHeader,
    pub c99_section: String,
}

impl FunctionProto {
    /// Convert to C function declaration
    pub fn to_c_declaration(&self) -> String {
        let params = if self.parameters.is_empty() {
            "void".to_string()
        } else {
            let mut p = self
                .parameters
                .iter()
                .map(|param| format!("{} {}", param.type_str, param.name))
                .collect::<Vec<_>>()
                .join(", ");

            if self.is_variadic {
                p.push_str(", ...");
            }

            p
        };

        format!("{} {}({});", self.return_type, self.name, params)
    }
}

/// Built-in C Standard Library Prototype Database
///
/// Contains all 150+ functions from ISO C99 §7
pub struct StdlibPrototypes {
    functions: HashMap<String, FunctionProto>,
}

impl StdlibPrototypes {
    /// Create new prototype database with all C99 §7 functions
    pub fn new() -> Self {
        let mut functions = HashMap::new();

        // ====================================================================
        // ISO C99 §7.22 - General utilities <stdlib.h>
        // ====================================================================

        // §7.22.3 - Memory management functions
        functions.insert(
            "malloc".to_string(),
            FunctionProto {
                name: "malloc".to_string(),
                return_type: "void*".to_string(),
                parameters: vec![Parameter::new("size", "size_t")],
                is_variadic: false,
                header: StdHeader::Stdlib,
                c99_section: "§7.22.3.4".to_string(),
            },
        );

        functions.insert(
            "calloc".to_string(),
            FunctionProto {
                name: "calloc".to_string(),
                return_type: "void*".to_string(),
                parameters: vec![
                    Parameter::new("nmemb", "size_t"),
                    Parameter::new("size", "size_t"),
                ],
                is_variadic: false,
                header: StdHeader::Stdlib,
                c99_section: "§7.22.3.2".to_string(),
            },
        );

        functions.insert(
            "realloc".to_string(),
            FunctionProto {
                name: "realloc".to_string(),
                return_type: "void*".to_string(),
                parameters: vec![
                    Parameter::new("ptr", "void*"),
                    Parameter::new("size", "size_t"),
                ],
                is_variadic: false,
                header: StdHeader::Stdlib,
                c99_section: "§7.22.3.5".to_string(),
            },
        );

        functions.insert(
            "free".to_string(),
            FunctionProto {
                name: "free".to_string(),
                return_type: "void".to_string(),
                parameters: vec![Parameter::new("ptr", "void*")],
                is_variadic: false,
                header: StdHeader::Stdlib,
                c99_section: "§7.22.3.3".to_string(),
            },
        );

        // §7.22.2 - Pseudo-random sequence generation
        functions.insert(
            "rand".to_string(),
            FunctionProto {
                name: "rand".to_string(),
                return_type: "int".to_string(),
                parameters: vec![],
                is_variadic: false,
                header: StdHeader::Stdlib,
                c99_section: "§7.22.2.1".to_string(),
            },
        );

        functions.insert(
            "srand".to_string(),
            FunctionProto {
                name: "srand".to_string(),
                return_type: "void".to_string(),
                parameters: vec![Parameter::new("seed", "unsigned int")],
                is_variadic: false,
                header: StdHeader::Stdlib,
                c99_section: "§7.22.2.2".to_string(),
            },
        );

        // §7.22.1 - Numeric conversion functions
        functions.insert(
            "atoi".to_string(),
            FunctionProto {
                name: "atoi".to_string(),
                return_type: "int".to_string(),
                parameters: vec![Parameter::new("nptr", "const char*")],
                is_variadic: false,
                header: StdHeader::Stdlib,
                c99_section: "§7.22.1.2".to_string(),
            },
        );

        functions.insert(
            "atol".to_string(),
            FunctionProto {
                name: "atol".to_string(),
                return_type: "long".to_string(),
                parameters: vec![Parameter::new("nptr", "const char*")],
                is_variadic: false,
                header: StdHeader::Stdlib,
                c99_section: "§7.22.1.3".to_string(),
            },
        );

        functions.insert(
            "atof".to_string(),
            FunctionProto {
                name: "atof".to_string(),
                return_type: "double".to_string(),
                parameters: vec![Parameter::new("nptr", "const char*")],
                is_variadic: false,
                header: StdHeader::Stdlib,
                c99_section: "§7.22.1.1".to_string(),
            },
        );

        functions.insert(
            "strtol".to_string(),
            FunctionProto {
                name: "strtol".to_string(),
                return_type: "long".to_string(),
                parameters: vec![
                    Parameter::new("nptr", "const char*"),
                    Parameter::new("endptr", "char**"),
                    Parameter::new("base", "int"),
                ],
                is_variadic: false,
                header: StdHeader::Stdlib,
                c99_section: "§7.22.1.4".to_string(),
            },
        );

        functions.insert(
            "strtod".to_string(),
            FunctionProto {
                name: "strtod".to_string(),
                return_type: "double".to_string(),
                parameters: vec![
                    Parameter::new("nptr", "const char*"),
                    Parameter::new("endptr", "char**"),
                ],
                is_variadic: false,
                header: StdHeader::Stdlib,
                c99_section: "§7.22.1.3".to_string(),
            },
        );

        // §7.22.4 - Communication with the environment
        functions.insert(
            "exit".to_string(),
            FunctionProto {
                name: "exit".to_string(),
                return_type: "void".to_string(),
                parameters: vec![Parameter::new("status", "int")],
                is_variadic: false,
                header: StdHeader::Stdlib,
                c99_section: "§7.22.4.4".to_string(),
            },
        );

        functions.insert(
            "abort".to_string(),
            FunctionProto {
                name: "abort".to_string(),
                return_type: "void".to_string(),
                parameters: vec![],
                is_variadic: false,
                header: StdHeader::Stdlib,
                c99_section: "§7.22.4.1".to_string(),
            },
        );

        functions.insert(
            "getenv".to_string(),
            FunctionProto {
                name: "getenv".to_string(),
                return_type: "char*".to_string(),
                parameters: vec![Parameter::new("name", "const char*")],
                is_variadic: false,
                header: StdHeader::Stdlib,
                c99_section: "§7.22.4.6".to_string(),
            },
        );

        functions.insert(
            "system".to_string(),
            FunctionProto {
                name: "system".to_string(),
                return_type: "int".to_string(),
                parameters: vec![Parameter::new("command", "const char*")],
                is_variadic: false,
                header: StdHeader::Stdlib,
                c99_section: "§7.22.4.8".to_string(),
            },
        );

        // §7.22.5 - Searching and sorting
        functions.insert(
            "qsort".to_string(),
            FunctionProto {
                name: "qsort".to_string(),
                return_type: "void".to_string(),
                parameters: vec![
                    Parameter::new("base", "void*"),
                    Parameter::new("nmemb", "size_t"),
                    Parameter::new("size", "size_t"),
                    Parameter::new("compar", "int (*)(const void*, const void*)"),
                ],
                is_variadic: false,
                header: StdHeader::Stdlib,
                c99_section: "§7.22.5.2".to_string(),
            },
        );

        functions.insert(
            "bsearch".to_string(),
            FunctionProto {
                name: "bsearch".to_string(),
                return_type: "void*".to_string(),
                parameters: vec![
                    Parameter::new("key", "const void*"),
                    Parameter::new("base", "const void*"),
                    Parameter::new("nmemb", "size_t"),
                    Parameter::new("size", "size_t"),
                    Parameter::new("compar", "int (*)(const void*, const void*)"),
                ],
                is_variadic: false,
                header: StdHeader::Stdlib,
                c99_section: "§7.22.5.1".to_string(),
            },
        );

        // §7.22.6 - Integer arithmetic functions
        functions.insert(
            "abs".to_string(),
            FunctionProto {
                name: "abs".to_string(),
                return_type: "int".to_string(),
                parameters: vec![Parameter::new("j", "int")],
                is_variadic: false,
                header: StdHeader::Stdlib,
                c99_section: "§7.22.6.1".to_string(),
            },
        );

        functions.insert(
            "labs".to_string(),
            FunctionProto {
                name: "labs".to_string(),
                return_type: "long".to_string(),
                parameters: vec![Parameter::new("j", "long")],
                is_variadic: false,
                header: StdHeader::Stdlib,
                c99_section: "§7.22.6.1".to_string(),
            },
        );

        // ====================================================================
        // ISO C99 §7.21 - Input/output <stdio.h>
        // ====================================================================

        // §7.21.6 - Formatted output functions
        functions.insert(
            "printf".to_string(),
            FunctionProto {
                name: "printf".to_string(),
                return_type: "int".to_string(),
                parameters: vec![Parameter::new("format", "const char*")],
                is_variadic: true,
                header: StdHeader::Stdio,
                c99_section: "§7.21.6.1".to_string(),
            },
        );

        functions.insert(
            "fprintf".to_string(),
            FunctionProto {
                name: "fprintf".to_string(),
                return_type: "int".to_string(),
                parameters: vec![
                    Parameter::new("stream", "FILE*"),
                    Parameter::new("format", "const char*"),
                ],
                is_variadic: true,
                header: StdHeader::Stdio,
                c99_section: "§7.21.6.1".to_string(),
            },
        );

        functions.insert(
            "sprintf".to_string(),
            FunctionProto {
                name: "sprintf".to_string(),
                return_type: "int".to_string(),
                parameters: vec![
                    Parameter::new("str", "char*"),
                    Parameter::new("format", "const char*"),
                ],
                is_variadic: true,
                header: StdHeader::Stdio,
                c99_section: "§7.21.6.5".to_string(),
            },
        );

        functions.insert(
            "snprintf".to_string(),
            FunctionProto {
                name: "snprintf".to_string(),
                return_type: "int".to_string(),
                parameters: vec![
                    Parameter::new("str", "char*"),
                    Parameter::new("size", "size_t"),
                    Parameter::new("format", "const char*"),
                ],
                is_variadic: true,
                header: StdHeader::Stdio,
                c99_section: "§7.21.6.5".to_string(),
            },
        );

        // §7.21.6 - Formatted input functions
        functions.insert(
            "scanf".to_string(),
            FunctionProto {
                name: "scanf".to_string(),
                return_type: "int".to_string(),
                parameters: vec![Parameter::new("format", "const char*")],
                is_variadic: true,
                header: StdHeader::Stdio,
                c99_section: "§7.21.6.2".to_string(),
            },
        );

        functions.insert(
            "fscanf".to_string(),
            FunctionProto {
                name: "fscanf".to_string(),
                return_type: "int".to_string(),
                parameters: vec![
                    Parameter::new("stream", "FILE*"),
                    Parameter::new("format", "const char*"),
                ],
                is_variadic: true,
                header: StdHeader::Stdio,
                c99_section: "§7.21.6.2".to_string(),
            },
        );

        functions.insert(
            "sscanf".to_string(),
            FunctionProto {
                name: "sscanf".to_string(),
                return_type: "int".to_string(),
                parameters: vec![
                    Parameter::new("str", "const char*"),
                    Parameter::new("format", "const char*"),
                ],
                is_variadic: true,
                header: StdHeader::Stdio,
                c99_section: "§7.21.6.4".to_string(),
            },
        );

        // §7.21.5 - File operations
        functions.insert(
            "fopen".to_string(),
            FunctionProto {
                name: "fopen".to_string(),
                return_type: "FILE*".to_string(),
                parameters: vec![
                    Parameter::new("filename", "const char*"),
                    Parameter::new("mode", "const char*"),
                ],
                is_variadic: false,
                header: StdHeader::Stdio,
                c99_section: "§7.21.5.3".to_string(),
            },
        );

        functions.insert(
            "fclose".to_string(),
            FunctionProto {
                name: "fclose".to_string(),
                return_type: "int".to_string(),
                parameters: vec![Parameter::new("stream", "FILE*")],
                is_variadic: false,
                header: StdHeader::Stdio,
                c99_section: "§7.21.5.1".to_string(),
            },
        );

        functions.insert(
            "fread".to_string(),
            FunctionProto {
                name: "fread".to_string(),
                return_type: "size_t".to_string(),
                parameters: vec![
                    Parameter::new("ptr", "void*"),
                    Parameter::new("size", "size_t"),
                    Parameter::new("nmemb", "size_t"),
                    Parameter::new("stream", "FILE*"),
                ],
                is_variadic: false,
                header: StdHeader::Stdio,
                c99_section: "§7.21.8.1".to_string(),
            },
        );

        functions.insert(
            "fwrite".to_string(),
            FunctionProto {
                name: "fwrite".to_string(),
                return_type: "size_t".to_string(),
                parameters: vec![
                    Parameter::new("ptr", "const void*"),
                    Parameter::new("size", "size_t"),
                    Parameter::new("nmemb", "size_t"),
                    Parameter::new("stream", "FILE*"),
                ],
                is_variadic: false,
                header: StdHeader::Stdio,
                c99_section: "§7.21.8.2".to_string(),
            },
        );

        functions.insert(
            "fseek".to_string(),
            FunctionProto {
                name: "fseek".to_string(),
                return_type: "int".to_string(),
                parameters: vec![
                    Parameter::new("stream", "FILE*"),
                    Parameter::new("offset", "long"),
                    Parameter::new("whence", "int"),
                ],
                is_variadic: false,
                header: StdHeader::Stdio,
                c99_section: "§7.21.9.2".to_string(),
            },
        );

        functions.insert(
            "ftell".to_string(),
            FunctionProto {
                name: "ftell".to_string(),
                return_type: "long".to_string(),
                parameters: vec![Parameter::new("stream", "FILE*")],
                is_variadic: false,
                header: StdHeader::Stdio,
                c99_section: "§7.21.9.4".to_string(),
            },
        );

        functions.insert(
            "rewind".to_string(),
            FunctionProto {
                name: "rewind".to_string(),
                return_type: "void".to_string(),
                parameters: vec![Parameter::new("stream", "FILE*")],
                is_variadic: false,
                header: StdHeader::Stdio,
                c99_section: "§7.21.9.3".to_string(),
            },
        );

        // §7.21.7 - Character I/O
        functions.insert(
            "getchar".to_string(),
            FunctionProto {
                name: "getchar".to_string(),
                return_type: "int".to_string(),
                parameters: vec![],
                is_variadic: false,
                header: StdHeader::Stdio,
                c99_section: "§7.21.7.6".to_string(),
            },
        );

        functions.insert(
            "putchar".to_string(),
            FunctionProto {
                name: "putchar".to_string(),
                return_type: "int".to_string(),
                parameters: vec![Parameter::new("c", "int")],
                is_variadic: false,
                header: StdHeader::Stdio,
                c99_section: "§7.21.7.8".to_string(),
            },
        );

        functions.insert(
            "fgetc".to_string(),
            FunctionProto {
                name: "fgetc".to_string(),
                return_type: "int".to_string(),
                parameters: vec![Parameter::new("stream", "FILE*")],
                is_variadic: false,
                header: StdHeader::Stdio,
                c99_section: "§7.21.7.1".to_string(),
            },
        );

        functions.insert(
            "fputc".to_string(),
            FunctionProto {
                name: "fputc".to_string(),
                return_type: "int".to_string(),
                parameters: vec![
                    Parameter::new("c", "int"),
                    Parameter::new("stream", "FILE*"),
                ],
                is_variadic: false,
                header: StdHeader::Stdio,
                c99_section: "§7.21.7.3".to_string(),
            },
        );

        functions.insert(
            "fgets".to_string(),
            FunctionProto {
                name: "fgets".to_string(),
                return_type: "char*".to_string(),
                parameters: vec![
                    Parameter::new("s", "char*"),
                    Parameter::new("size", "int"),
                    Parameter::new("stream", "FILE*"),
                ],
                is_variadic: false,
                header: StdHeader::Stdio,
                c99_section: "§7.21.7.2".to_string(),
            },
        );

        functions.insert(
            "fputs".to_string(),
            FunctionProto {
                name: "fputs".to_string(),
                return_type: "int".to_string(),
                parameters: vec![
                    Parameter::new("s", "const char*"),
                    Parameter::new("stream", "FILE*"),
                ],
                is_variadic: false,
                header: StdHeader::Stdio,
                c99_section: "§7.21.7.4".to_string(),
            },
        );

        functions.insert(
            "puts".to_string(),
            FunctionProto {
                name: "puts".to_string(),
                return_type: "int".to_string(),
                parameters: vec![Parameter::new("s", "const char*")],
                is_variadic: false,
                header: StdHeader::Stdio,
                c99_section: "§7.21.7.9".to_string(),
            },
        );

        // ====================================================================
        // ISO C99 §7.23 - String handling <string.h>
        // ====================================================================

        // §7.23.2 - Copying functions
        functions.insert(
            "memcpy".to_string(),
            FunctionProto {
                name: "memcpy".to_string(),
                return_type: "void*".to_string(),
                parameters: vec![
                    Parameter::new("dest", "void*"),
                    Parameter::new("src", "const void*"),
                    Parameter::new("n", "size_t"),
                ],
                is_variadic: false,
                header: StdHeader::String,
                c99_section: "§7.23.2.1".to_string(),
            },
        );

        functions.insert(
            "memmove".to_string(),
            FunctionProto {
                name: "memmove".to_string(),
                return_type: "void*".to_string(),
                parameters: vec![
                    Parameter::new("dest", "void*"),
                    Parameter::new("src", "const void*"),
                    Parameter::new("n", "size_t"),
                ],
                is_variadic: false,
                header: StdHeader::String,
                c99_section: "§7.23.2.2".to_string(),
            },
        );

        functions.insert(
            "strcpy".to_string(),
            FunctionProto {
                name: "strcpy".to_string(),
                return_type: "char*".to_string(),
                parameters: vec![
                    Parameter::new("dest", "char*"),
                    Parameter::new("src", "const char*"),
                ],
                is_variadic: false,
                header: StdHeader::String,
                c99_section: "§7.23.2.3".to_string(),
            },
        );

        functions.insert(
            "strncpy".to_string(),
            FunctionProto {
                name: "strncpy".to_string(),
                return_type: "char*".to_string(),
                parameters: vec![
                    Parameter::new("dest", "char*"),
                    Parameter::new("src", "const char*"),
                    Parameter::new("n", "size_t"),
                ],
                is_variadic: false,
                header: StdHeader::String,
                c99_section: "§7.23.2.4".to_string(),
            },
        );

        // §7.23.3 - Concatenation functions
        functions.insert(
            "strcat".to_string(),
            FunctionProto {
                name: "strcat".to_string(),
                return_type: "char*".to_string(),
                parameters: vec![
                    Parameter::new("dest", "char*"),
                    Parameter::new("src", "const char*"),
                ],
                is_variadic: false,
                header: StdHeader::String,
                c99_section: "§7.23.3.1".to_string(),
            },
        );

        functions.insert(
            "strncat".to_string(),
            FunctionProto {
                name: "strncat".to_string(),
                return_type: "char*".to_string(),
                parameters: vec![
                    Parameter::new("dest", "char*"),
                    Parameter::new("src", "const char*"),
                    Parameter::new("n", "size_t"),
                ],
                is_variadic: false,
                header: StdHeader::String,
                c99_section: "§7.23.3.2".to_string(),
            },
        );

        // §7.23.4 - Comparison functions
        functions.insert(
            "memcmp".to_string(),
            FunctionProto {
                name: "memcmp".to_string(),
                return_type: "int".to_string(),
                parameters: vec![
                    Parameter::new("s1", "const void*"),
                    Parameter::new("s2", "const void*"),
                    Parameter::new("n", "size_t"),
                ],
                is_variadic: false,
                header: StdHeader::String,
                c99_section: "§7.23.4.1".to_string(),
            },
        );

        functions.insert(
            "strcmp".to_string(),
            FunctionProto {
                name: "strcmp".to_string(),
                return_type: "int".to_string(),
                parameters: vec![
                    Parameter::new("s1", "const char*"),
                    Parameter::new("s2", "const char*"),
                ],
                is_variadic: false,
                header: StdHeader::String,
                c99_section: "§7.23.4.2".to_string(),
            },
        );

        functions.insert(
            "strncmp".to_string(),
            FunctionProto {
                name: "strncmp".to_string(),
                return_type: "int".to_string(),
                parameters: vec![
                    Parameter::new("s1", "const char*"),
                    Parameter::new("s2", "const char*"),
                    Parameter::new("n", "size_t"),
                ],
                is_variadic: false,
                header: StdHeader::String,
                c99_section: "§7.23.4.4".to_string(),
            },
        );

        // §7.23.5 - Search functions
        functions.insert(
            "memchr".to_string(),
            FunctionProto {
                name: "memchr".to_string(),
                return_type: "void*".to_string(),
                parameters: vec![
                    Parameter::new("s", "const void*"),
                    Parameter::new("c", "int"),
                    Parameter::new("n", "size_t"),
                ],
                is_variadic: false,
                header: StdHeader::String,
                c99_section: "§7.23.5.1".to_string(),
            },
        );

        functions.insert(
            "strchr".to_string(),
            FunctionProto {
                name: "strchr".to_string(),
                return_type: "char*".to_string(),
                parameters: vec![
                    Parameter::new("s", "const char*"),
                    Parameter::new("c", "int"),
                ],
                is_variadic: false,
                header: StdHeader::String,
                c99_section: "§7.23.5.2".to_string(),
            },
        );

        functions.insert(
            "strrchr".to_string(),
            FunctionProto {
                name: "strrchr".to_string(),
                return_type: "char*".to_string(),
                parameters: vec![
                    Parameter::new("s", "const char*"),
                    Parameter::new("c", "int"),
                ],
                is_variadic: false,
                header: StdHeader::String,
                c99_section: "§7.23.5.5".to_string(),
            },
        );

        functions.insert(
            "strstr".to_string(),
            FunctionProto {
                name: "strstr".to_string(),
                return_type: "char*".to_string(),
                parameters: vec![
                    Parameter::new("haystack", "const char*"),
                    Parameter::new("needle", "const char*"),
                ],
                is_variadic: false,
                header: StdHeader::String,
                c99_section: "§7.23.5.7".to_string(),
            },
        );

        functions.insert(
            "strtok".to_string(),
            FunctionProto {
                name: "strtok".to_string(),
                return_type: "char*".to_string(),
                parameters: vec![
                    Parameter::new("str", "char*"),
                    Parameter::new("delim", "const char*"),
                ],
                is_variadic: false,
                header: StdHeader::String,
                c99_section: "§7.23.5.8".to_string(),
            },
        );

        // §7.23.6 - Miscellaneous functions
        functions.insert(
            "memset".to_string(),
            FunctionProto {
                name: "memset".to_string(),
                return_type: "void*".to_string(),
                parameters: vec![
                    Parameter::new("s", "void*"),
                    Parameter::new("c", "int"),
                    Parameter::new("n", "size_t"),
                ],
                is_variadic: false,
                header: StdHeader::String,
                c99_section: "§7.23.6.1".to_string(),
            },
        );

        functions.insert(
            "strlen".to_string(),
            FunctionProto {
                name: "strlen".to_string(),
                return_type: "size_t".to_string(),
                parameters: vec![Parameter::new("s", "const char*")],
                is_variadic: false,
                header: StdHeader::String,
                c99_section: "§7.23.6.3".to_string(),
            },
        );

        functions.insert(
            "strdup".to_string(),
            FunctionProto {
                name: "strdup".to_string(),
                return_type: "char*".to_string(),
                parameters: vec![Parameter::new("s", "const char*")],
                is_variadic: false,
                header: StdHeader::String,
                c99_section: "POSIX".to_string(),
            },
        );

        // ====================================================================
        // ISO C99 §7.4 - Character handling <ctype.h>
        // ====================================================================

        functions.insert(
            "isspace".to_string(),
            FunctionProto {
                name: "isspace".to_string(),
                return_type: "int".to_string(),
                parameters: vec![Parameter::new("c", "int")],
                is_variadic: false,
                header: StdHeader::Ctype,
                c99_section: "§7.4.1.10".to_string(),
            },
        );

        functions.insert(
            "isdigit".to_string(),
            FunctionProto {
                name: "isdigit".to_string(),
                return_type: "int".to_string(),
                parameters: vec![Parameter::new("c", "int")],
                is_variadic: false,
                header: StdHeader::Ctype,
                c99_section: "§7.4.1.5".to_string(),
            },
        );

        functions.insert(
            "isalpha".to_string(),
            FunctionProto {
                name: "isalpha".to_string(),
                return_type: "int".to_string(),
                parameters: vec![Parameter::new("c", "int")],
                is_variadic: false,
                header: StdHeader::Ctype,
                c99_section: "§7.4.1.2".to_string(),
            },
        );

        functions.insert(
            "isalnum".to_string(),
            FunctionProto {
                name: "isalnum".to_string(),
                return_type: "int".to_string(),
                parameters: vec![Parameter::new("c", "int")],
                is_variadic: false,
                header: StdHeader::Ctype,
                c99_section: "§7.4.1.1".to_string(),
            },
        );

        functions.insert(
            "isupper".to_string(),
            FunctionProto {
                name: "isupper".to_string(),
                return_type: "int".to_string(),
                parameters: vec![Parameter::new("c", "int")],
                is_variadic: false,
                header: StdHeader::Ctype,
                c99_section: "§7.4.1.11".to_string(),
            },
        );

        functions.insert(
            "islower".to_string(),
            FunctionProto {
                name: "islower".to_string(),
                return_type: "int".to_string(),
                parameters: vec![Parameter::new("c", "int")],
                is_variadic: false,
                header: StdHeader::Ctype,
                c99_section: "§7.4.1.7".to_string(),
            },
        );

        functions.insert(
            "tolower".to_string(),
            FunctionProto {
                name: "tolower".to_string(),
                return_type: "int".to_string(),
                parameters: vec![Parameter::new("c", "int")],
                is_variadic: false,
                header: StdHeader::Ctype,
                c99_section: "§7.4.2.1".to_string(),
            },
        );

        functions.insert(
            "toupper".to_string(),
            FunctionProto {
                name: "toupper".to_string(),
                return_type: "int".to_string(),
                parameters: vec![Parameter::new("c", "int")],
                is_variadic: false,
                header: StdHeader::Ctype,
                c99_section: "§7.4.2.2".to_string(),
            },
        );

        // ====================================================================
        // ISO C99 §7.23 - Date and time <time.h>
        // ====================================================================

        functions.insert(
            "clock".to_string(),
            FunctionProto {
                name: "clock".to_string(),
                return_type: "clock_t".to_string(),
                parameters: vec![],
                is_variadic: false,
                header: StdHeader::Time,
                c99_section: "§7.23.2.1".to_string(),
            },
        );

        functions.insert(
            "time".to_string(),
            FunctionProto {
                name: "time".to_string(),
                return_type: "time_t".to_string(),
                parameters: vec![Parameter::new("timer", "time_t*")],
                is_variadic: false,
                header: StdHeader::Time,
                c99_section: "§7.23.2.4".to_string(),
            },
        );

        // ====================================================================
        // ISO C99 §7.12 - Mathematics <math.h>
        // ====================================================================

        functions.insert(
            "sqrt".to_string(),
            FunctionProto {
                name: "sqrt".to_string(),
                return_type: "double".to_string(),
                parameters: vec![Parameter::new("x", "double")],
                is_variadic: false,
                header: StdHeader::Math,
                c99_section: "§7.12.7.5".to_string(),
            },
        );

        functions.insert(
            "sin".to_string(),
            FunctionProto {
                name: "sin".to_string(),
                return_type: "double".to_string(),
                parameters: vec![Parameter::new("x", "double")],
                is_variadic: false,
                header: StdHeader::Math,
                c99_section: "§7.12.4.6".to_string(),
            },
        );

        functions.insert(
            "cos".to_string(),
            FunctionProto {
                name: "cos".to_string(),
                return_type: "double".to_string(),
                parameters: vec![Parameter::new("x", "double")],
                is_variadic: false,
                header: StdHeader::Math,
                c99_section: "§7.12.4.5".to_string(),
            },
        );

        functions.insert(
            "pow".to_string(),
            FunctionProto {
                name: "pow".to_string(),
                return_type: "double".to_string(),
                parameters: vec![Parameter::new("x", "double"), Parameter::new("y", "double")],
                is_variadic: false,
                header: StdHeader::Math,
                c99_section: "§7.12.7.4".to_string(),
            },
        );

        functions.insert(
            "fabs".to_string(),
            FunctionProto {
                name: "fabs".to_string(),
                return_type: "double".to_string(),
                parameters: vec![Parameter::new("x", "double")],
                is_variadic: false,
                header: StdHeader::Math,
                c99_section: "§7.12.7.2".to_string(),
            },
        );

        functions.insert(
            "ceil".to_string(),
            FunctionProto {
                name: "ceil".to_string(),
                return_type: "double".to_string(),
                parameters: vec![Parameter::new("x", "double")],
                is_variadic: false,
                header: StdHeader::Math,
                c99_section: "§7.12.9.1".to_string(),
            },
        );

        functions.insert(
            "floor".to_string(),
            FunctionProto {
                name: "floor".to_string(),
                return_type: "double".to_string(),
                parameters: vec![Parameter::new("x", "double")],
                is_variadic: false,
                header: StdHeader::Math,
                c99_section: "§7.12.9.2".to_string(),
            },
        );

        functions.insert(
            "round".to_string(),
            FunctionProto {
                name: "round".to_string(),
                return_type: "double".to_string(),
                parameters: vec![Parameter::new("x", "double")],
                is_variadic: false,
                header: StdHeader::Math,
                c99_section: "§7.12.9.6".to_string(),
            },
        );

        functions.insert(
            "trunc".to_string(),
            FunctionProto {
                name: "trunc".to_string(),
                return_type: "double".to_string(),
                parameters: vec![Parameter::new("x", "double")],
                is_variadic: false,
                header: StdHeader::Math,
                c99_section: "§7.12.9.8".to_string(),
            },
        );

        functions.insert(
            "exp".to_string(),
            FunctionProto {
                name: "exp".to_string(),
                return_type: "double".to_string(),
                parameters: vec![Parameter::new("x", "double")],
                is_variadic: false,
                header: StdHeader::Math,
                c99_section: "§7.12.6.1".to_string(),
            },
        );

        functions.insert(
            "log".to_string(),
            FunctionProto {
                name: "log".to_string(),
                return_type: "double".to_string(),
                parameters: vec![Parameter::new("x", "double")],
                is_variadic: false,
                header: StdHeader::Math,
                c99_section: "§7.12.6.7".to_string(),
            },
        );

        functions.insert(
            "log10".to_string(),
            FunctionProto {
                name: "log10".to_string(),
                return_type: "double".to_string(),
                parameters: vec![Parameter::new("x", "double")],
                is_variadic: false,
                header: StdHeader::Math,
                c99_section: "§7.12.6.8".to_string(),
            },
        );

        functions.insert(
            "tan".to_string(),
            FunctionProto {
                name: "tan".to_string(),
                return_type: "double".to_string(),
                parameters: vec![Parameter::new("x", "double")],
                is_variadic: false,
                header: StdHeader::Math,
                c99_section: "§7.12.4.7".to_string(),
            },
        );

        functions.insert(
            "asin".to_string(),
            FunctionProto {
                name: "asin".to_string(),
                return_type: "double".to_string(),
                parameters: vec![Parameter::new("x", "double")],
                is_variadic: false,
                header: StdHeader::Math,
                c99_section: "§7.12.4.2".to_string(),
            },
        );

        functions.insert(
            "acos".to_string(),
            FunctionProto {
                name: "acos".to_string(),
                return_type: "double".to_string(),
                parameters: vec![Parameter::new("x", "double")],
                is_variadic: false,
                header: StdHeader::Math,
                c99_section: "§7.12.4.1".to_string(),
            },
        );

        functions.insert(
            "atan".to_string(),
            FunctionProto {
                name: "atan".to_string(),
                return_type: "double".to_string(),
                parameters: vec![Parameter::new("x", "double")],
                is_variadic: false,
                header: StdHeader::Math,
                c99_section: "§7.12.4.3".to_string(),
            },
        );

        functions.insert(
            "atan2".to_string(),
            FunctionProto {
                name: "atan2".to_string(),
                return_type: "double".to_string(),
                parameters: vec![Parameter::new("y", "double"), Parameter::new("x", "double")],
                is_variadic: false,
                header: StdHeader::Math,
                c99_section: "§7.12.4.4".to_string(),
            },
        );

        functions.insert(
            "fmod".to_string(),
            FunctionProto {
                name: "fmod".to_string(),
                return_type: "double".to_string(),
                parameters: vec![Parameter::new("x", "double"), Parameter::new("y", "double")],
                is_variadic: false,
                header: StdHeader::Math,
                c99_section: "§7.12.10.1".to_string(),
            },
        );

        // ====================================================================
        // POSIX - unistd.h additional functions
        // ====================================================================

        functions.insert(
            "pipe".to_string(),
            FunctionProto {
                name: "pipe".to_string(),
                return_type: "int".to_string(),
                parameters: vec![Parameter::new("pipefd", "int*")],
                is_variadic: false,
                header: StdHeader::Unistd,
                c99_section: "POSIX".to_string(),
            },
        );

        functions.insert(
            "fork".to_string(),
            FunctionProto {
                name: "fork".to_string(),
                return_type: "pid_t".to_string(),
                parameters: vec![],
                is_variadic: false,
                header: StdHeader::Unistd,
                c99_section: "POSIX".to_string(),
            },
        );

        functions.insert(
            "read".to_string(),
            FunctionProto {
                name: "read".to_string(),
                return_type: "ssize_t".to_string(),
                parameters: vec![
                    Parameter::new("fd", "int"),
                    Parameter::new("buf", "void*"),
                    Parameter::new("count", "size_t"),
                ],
                is_variadic: false,
                header: StdHeader::Unistd,
                c99_section: "POSIX".to_string(),
            },
        );

        functions.insert(
            "write".to_string(),
            FunctionProto {
                name: "write".to_string(),
                return_type: "ssize_t".to_string(),
                parameters: vec![
                    Parameter::new("fd", "int"),
                    Parameter::new("buf", "const void*"),
                    Parameter::new("count", "size_t"),
                ],
                is_variadic: false,
                header: StdHeader::Unistd,
                c99_section: "POSIX".to_string(),
            },
        );

        functions.insert(
            "close".to_string(),
            FunctionProto {
                name: "close".to_string(),
                return_type: "int".to_string(),
                parameters: vec![Parameter::new("fd", "int")],
                is_variadic: false,
                header: StdHeader::Unistd,
                c99_section: "POSIX".to_string(),
            },
        );

        functions.insert(
            "lseek".to_string(),
            FunctionProto {
                name: "lseek".to_string(),
                return_type: "off_t".to_string(),
                parameters: vec![
                    Parameter::new("fd", "int"),
                    Parameter::new("offset", "off_t"),
                    Parameter::new("whence", "int"),
                ],
                is_variadic: false,
                header: StdHeader::Unistd,
                c99_section: "POSIX".to_string(),
            },
        );

        functions.insert(
            "open".to_string(),
            FunctionProto {
                name: "open".to_string(),
                return_type: "int".to_string(),
                parameters: vec![
                    Parameter::new("pathname", "const char*"),
                    Parameter::new("flags", "int"),
                ],
                is_variadic: true, // Optional mode parameter
                header: StdHeader::Fcntl,
                c99_section: "POSIX".to_string(),
            },
        );

        functions.insert(
            "dup".to_string(),
            FunctionProto {
                name: "dup".to_string(),
                return_type: "int".to_string(),
                parameters: vec![Parameter::new("oldfd", "int")],
                is_variadic: false,
                header: StdHeader::Unistd,
                c99_section: "POSIX".to_string(),
            },
        );

        functions.insert(
            "dup2".to_string(),
            FunctionProto {
                name: "dup2".to_string(),
                return_type: "int".to_string(),
                parameters: vec![
                    Parameter::new("oldfd", "int"),
                    Parameter::new("newfd", "int"),
                ],
                is_variadic: false,
                header: StdHeader::Unistd,
                c99_section: "POSIX".to_string(),
            },
        );

        functions.insert(
            "getenv".to_string(),
            FunctionProto {
                name: "getenv".to_string(),
                return_type: "char*".to_string(),
                parameters: vec![Parameter::new("name", "const char*")],
                is_variadic: false,
                header: StdHeader::Stdlib,
                c99_section: "§7.22.4.6".to_string(),
            },
        );

        // ====================================================================
        // POSIX - dirent.h functions
        // ====================================================================

        functions.insert(
            "opendir".to_string(),
            FunctionProto {
                name: "opendir".to_string(),
                return_type: "DIR*".to_string(),
                parameters: vec![Parameter::new("name", "const char*")],
                is_variadic: false,
                header: StdHeader::Dirent,
                c99_section: "POSIX".to_string(),
            },
        );

        functions.insert(
            "readdir".to_string(),
            FunctionProto {
                name: "readdir".to_string(),
                return_type: "struct dirent*".to_string(),
                parameters: vec![Parameter::new("dirp", "DIR*")],
                is_variadic: false,
                header: StdHeader::Dirent,
                c99_section: "POSIX".to_string(),
            },
        );

        functions.insert(
            "closedir".to_string(),
            FunctionProto {
                name: "closedir".to_string(),
                return_type: "int".to_string(),
                parameters: vec![Parameter::new("dirp", "DIR*")],
                is_variadic: false,
                header: StdHeader::Dirent,
                c99_section: "POSIX".to_string(),
            },
        );

        Self { functions }
    }

    /// Get prototype for a stdlib function by name
    pub fn get_prototype(&self, name: &str) -> Option<&FunctionProto> {
        self.functions.get(name)
    }

    /// Inject prototypes for a specific header
    ///
    /// Only injects function declarations for the specified header.
    /// This prevents parser overload from injecting all 55+ prototypes at once.
    ///
    /// # Examples
    /// ```
    /// use decy_stdlib::{StdlibPrototypes, StdHeader};
    /// let stdlib = StdlibPrototypes::new();
    /// let string_protos = stdlib.inject_prototypes_for_header(StdHeader::String);
    /// assert!(string_protos.contains("strlen"));
    /// assert!(!string_protos.contains("printf")); // stdio function, not string
    /// ```
    pub fn inject_prototypes_for_header(&self, header: StdHeader) -> String {
        let mut result = String::new();

        // Type definitions (always needed)
        result.push_str(&format!(
            "// Built-in prototypes for {:?} (ISO C99 §7)\n",
            header
        ));
        result.push_str("typedef unsigned long size_t;\n");
        result.push_str("typedef long ssize_t;\n");
        result.push_str("typedef long ptrdiff_t;\n");
        // NULL macro (ISO C99 §7.17) - use simple 0 to avoid parser issues
        result.push_str("#define NULL 0\n");

        // Add header-specific type definitions
        match header {
            StdHeader::Stdio => {
                result.push_str("struct _IO_FILE;\n");
                result.push_str("typedef struct _IO_FILE FILE;\n");
                // DECY-239: Add standard streams as extern declarations
                result.push_str("extern FILE* stdin;\n");
                result.push_str("extern FILE* stdout;\n");
                result.push_str("extern FILE* stderr;\n");
                // Common stdio macros
                result.push_str("#define EOF (-1)\n");
                result.push_str("#define SEEK_SET 0\n");
                result.push_str("#define SEEK_CUR 1\n");
                result.push_str("#define SEEK_END 2\n");
                result.push_str("#define BUFSIZ 8192\n");
                result.push_str("#define L_tmpnam 20\n");
                result.push_str("#define _IONBF 2\n");
                result.push_str("#define _IOLBF 1\n");
                result.push_str("#define _IOFBF 0\n");
            }
            StdHeader::Errno => {
                result.push_str("extern int errno;\n");
                result.push_str("#define EACCES 13\n");
                result.push_str("#define ENOENT 2\n");
                result.push_str("#define EINVAL 22\n");
                result.push_str("#define ENOMEM 12\n");
                result.push_str("#define ERANGE 34\n");
            }
            StdHeader::Time => {
                result.push_str("typedef long time_t;\n");
                result.push_str("typedef long clock_t;\n");
                result.push_str("struct tm;\n");
                result.push_str("#define CLOCKS_PER_SEC 1000000\n");
            }
            StdHeader::Stdarg => {
                // va_list is typically a pointer or array type
                result.push_str("typedef void* va_list;\n");
                result.push_str("#define va_start(ap, last) ((void)0)\n");
                result.push_str("#define va_end(ap) ((void)0)\n");
                result.push_str("#define va_arg(ap, type) (*(type*)0)\n");
            }
            StdHeader::Stdbool => {
                result.push_str("typedef _Bool bool;\n");
                result.push_str("#define true 1\n");
                result.push_str("#define false 0\n");
            }
            StdHeader::Stdint => {
                result.push_str("typedef signed char int8_t;\n");
                result.push_str("typedef short int16_t;\n");
                result.push_str("typedef int int32_t;\n");
                result.push_str("typedef long long int64_t;\n");
                result.push_str("typedef unsigned char uint8_t;\n");
                result.push_str("typedef unsigned short uint16_t;\n");
                result.push_str("typedef unsigned int uint32_t;\n");
                result.push_str("typedef unsigned long long uint64_t;\n");
                result.push_str("typedef long intptr_t;\n");
                result.push_str("typedef unsigned long uintptr_t;\n");
            }
            StdHeader::Unistd => {
                // POSIX types and file descriptor macros
                result.push_str("typedef int pid_t;\n");
                result.push_str("typedef long off_t;\n");
                result.push_str("typedef unsigned int uid_t;\n");
                result.push_str("typedef unsigned int gid_t;\n");
                result.push_str("#define STDIN_FILENO 0\n");
                result.push_str("#define STDOUT_FILENO 1\n");
                result.push_str("#define STDERR_FILENO 2\n");
                // Access mode flags
                result.push_str("#define F_OK 0\n");
                result.push_str("#define R_OK 4\n");
                result.push_str("#define W_OK 2\n");
                result.push_str("#define X_OK 1\n");
                // sysconf names
                result.push_str("#define _SC_OPEN_MAX 4\n");
                result.push_str("#define _SC_PAGESIZE 30\n");
            }
            StdHeader::Fcntl => {
                // File access mode flags
                result.push_str("#define O_RDONLY 0\n");
                result.push_str("#define O_WRONLY 1\n");
                result.push_str("#define O_RDWR 2\n");
                result.push_str("#define O_CREAT 0100\n");
                result.push_str("#define O_TRUNC 01000\n");
                result.push_str("#define O_APPEND 02000\n");
                result.push_str("#define O_NONBLOCK 04000\n");
                // File lock types (from flock)
                result.push_str("#define LOCK_SH 1\n");
                result.push_str("#define LOCK_EX 2\n");
                result.push_str("#define LOCK_UN 8\n");
            }
            StdHeader::Dirent => {
                result.push_str("struct dirent { char d_name[256]; };\n");
                result.push_str("typedef struct __dirstream DIR;\n");
            }
            StdHeader::SysTypes => {
                result.push_str("typedef int pid_t;\n");
                result.push_str("typedef long off_t;\n");
                result.push_str("typedef unsigned int mode_t;\n");
                result.push_str("typedef long ssize_t;\n");
            }
            StdHeader::SysStat => {
                result.push_str("struct stat { long st_size; int st_mode; };\n");
                result.push_str("#define S_ISREG(m) (((m) & 0170000) == 0100000)\n");
                result.push_str("#define S_ISDIR(m) (((m) & 0170000) == 0040000)\n");
            }
            StdHeader::SysMman => {
                // Memory protection flags
                result.push_str("#define PROT_NONE 0\n");
                result.push_str("#define PROT_READ 1\n");
                result.push_str("#define PROT_WRITE 2\n");
                result.push_str("#define PROT_EXEC 4\n");
                // Map flags
                result.push_str("#define MAP_SHARED 1\n");
                result.push_str("#define MAP_PRIVATE 2\n");
                result.push_str("#define MAP_ANONYMOUS 0x20\n");
                result.push_str("#define MAP_FAILED ((void*)-1)\n");
            }
            StdHeader::Wchar => {
                result.push_str("typedef int wchar_t;\n");
                result.push_str("typedef int wint_t;\n");
                result.push_str("#define WEOF (-1)\n");
            }
            StdHeader::Signal => {
                result.push_str("typedef void (*sighandler_t)(int);\n");
                result.push_str("#define SIGINT 2\n");
                result.push_str("#define SIGTERM 15\n");
            }
            StdHeader::Limits => {
                result.push_str("#define CHAR_BIT 8\n");
                result.push_str("#define CHAR_MIN (-128)\n");
                result.push_str("#define CHAR_MAX 127\n");
                result.push_str("#define SHRT_MIN (-32768)\n");
                result.push_str("#define SHRT_MAX 32767\n");
                result.push_str("#define INT_MIN (-2147483647-1)\n");
                result.push_str("#define INT_MAX 2147483647\n");
                result.push_str("#define UINT_MAX 4294967295U\n");
                result.push_str("#define LONG_MIN (-9223372036854775807L-1)\n");
                result.push_str("#define LONG_MAX 9223372036854775807L\n");
                result.push_str("#define PATH_MAX 4096\n");
            }
            StdHeader::Ctype => {
                // Character classification functions - ISO C99 §7.4
                // All return non-zero if true, 0 if false
            }
            StdHeader::Math => {
                // Math functions - ISO C99 §7.12
                result.push_str("#define M_PI 3.14159265358979323846\n");
                result.push_str("#define M_E 2.71828182845904523536\n");
                result.push_str("#define INFINITY (1.0/0.0)\n");
                result.push_str("#define NAN (0.0/0.0)\n");
            }
            _ => {}
        }

        result.push('\n');

        // Filter functions by header and inject
        // NOTE: Functions with function pointer parameters are currently skipped.
        // Function pointer syntax like `int (*comp)(const void*, const void*)`
        // needs special handling in to_c_declaration() - name goes inside (*name)
        let mut protos: Vec<_> = self
            .functions
            .values()
            .filter(|p| p.header == header)
            .filter(|p| {
                // Skip functions with function pointer parameters (contain "(*" in type)
                !p.parameters
                    .iter()
                    .any(|param| param.type_str.contains("(*"))
            })
            .collect();
        protos.sort_by_key(|p| &p.name);

        for proto in protos {
            result.push_str(&proto.to_c_declaration());
            result.push('\n');
        }

        result
    }

    /// Inject all stdlib prototypes as C declarations
    ///
    /// **Note**: Prefer `inject_prototypes_for_header()` to avoid parser overload.
    /// This method injects ALL 55+ prototypes which may cause parsing issues.
    pub fn inject_all_prototypes(&self) -> String {
        let mut result = String::new();

        // Type definitions (ISO C99 §7.17, §7.19, §7.21)
        result.push_str("// Built-in stdlib prototypes (ISO C99 §7)\n");
        result.push_str("typedef unsigned long size_t;\n");
        result.push_str("typedef long ssize_t;\n");
        result.push_str("typedef long ptrdiff_t;\n");
        // NULL macro (ISO C99 §7.17) - use simple 0 to avoid parser issues
        result.push_str("#define NULL 0\n");
        result.push_str("struct _IO_FILE;\n");
        result.push_str("typedef struct _IO_FILE FILE;\n");
        // DECY-239: Standard streams
        result.push_str("extern FILE* stdin;\n");
        result.push_str("extern FILE* stdout;\n");
        result.push_str("extern FILE* stderr;\n");
        result.push_str("#define EOF (-1)\n");
        result.push_str("#define SEEK_SET 0\n");
        result.push_str("#define SEEK_CUR 1\n");
        result.push_str("#define SEEK_END 2\n");
        // Common POSIX types
        result.push_str("typedef int pid_t;\n");
        result.push_str("typedef long off_t;\n");
        result.push_str("typedef long time_t;\n");
        result.push_str("typedef long clock_t;\n");
        result.push_str("typedef int wchar_t;\n");
        result.push_str("extern int errno;\n");
        // Common macros
        result.push_str("#define CLOCKS_PER_SEC 1000000\n");
        result.push_str("#define PATH_MAX 4096\n");
        result.push('\n');

        // Inject function prototypes
        let mut protos: Vec<_> = self.functions.values().collect();
        protos.sort_by_key(|p| &p.name);

        for proto in protos {
            result.push_str(&proto.to_c_declaration());
            result.push('\n');
        }

        result
    }

    /// Get number of functions in database
    pub fn len(&self) -> usize {
        self.functions.len()
    }

    /// Check if database is empty
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty()
    }
}

impl Default for StdlibPrototypes {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_proto_to_c_declaration() {
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
    fn test_variadic_function_proto() {
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
    fn test_no_param_function_proto() {
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
}
