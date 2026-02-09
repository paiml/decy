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

    // ========================================================================
    // inject_prototypes_for_header: Common preamble tests
    // ========================================================================

    #[test]
    fn test_inject_header_common_preamble_present_for_all_headers() {
        let stdlib = StdlibPrototypes::new();
        let headers = [
            StdHeader::Stdio,
            StdHeader::Errno,
            StdHeader::Time,
            StdHeader::Stdarg,
            StdHeader::Stdbool,
            StdHeader::Stdint,
            StdHeader::Unistd,
            StdHeader::Fcntl,
            StdHeader::Dirent,
            StdHeader::SysTypes,
            StdHeader::SysStat,
            StdHeader::SysMman,
            StdHeader::Wchar,
            StdHeader::Signal,
            StdHeader::Limits,
            StdHeader::Ctype,
            StdHeader::Math,
            StdHeader::Assert,
            StdHeader::Float,
            StdHeader::Locale,
            StdHeader::Setjmp,
            StdHeader::Stddef,
            StdHeader::Stdlib,
            StdHeader::String,
        ];
        for header in &headers {
            let result = stdlib.inject_prototypes_for_header(*header);
            assert!(
                result.contains("typedef unsigned long size_t;"),
                "{:?} missing size_t",
                header
            );
            assert!(
                result.contains("typedef long ssize_t;"),
                "{:?} missing ssize_t",
                header
            );
            assert!(
                result.contains("typedef long ptrdiff_t;"),
                "{:?} missing ptrdiff_t",
                header
            );
            assert!(
                result.contains("#define NULL 0"),
                "{:?} missing NULL",
                header
            );
            assert!(
                result.contains("// Built-in prototypes for"),
                "{:?} missing comment header",
                header
            );
        }
    }

    // ========================================================================
    // inject_prototypes_for_header: Stdio
    // ========================================================================

    #[test]
    fn test_inject_header_stdio_type_definitions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Stdio);
        assert!(result.contains("struct _IO_FILE;"));
        assert!(result.contains("typedef struct _IO_FILE FILE;"));
        assert!(result.contains("extern FILE* stdin;"));
        assert!(result.contains("extern FILE* stdout;"));
        assert!(result.contains("extern FILE* stderr;"));
    }

    #[test]
    fn test_inject_header_stdio_macros() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Stdio);
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
        // Formatted output
        assert!(result.contains("int printf(const char* format, ...);"));
        assert!(result.contains("int fprintf(FILE* stream, const char* format, ...);"));
        assert!(result.contains("int sprintf(char* str, const char* format, ...);"));
        assert!(result.contains("int snprintf(char* str, size_t size, const char* format, ...);"));
        // Formatted input
        assert!(result.contains("int scanf(const char* format, ...);"));
        assert!(result.contains("int fscanf(FILE* stream, const char* format, ...);"));
        assert!(result.contains("int sscanf(const char* str, const char* format, ...);"));
        // File operations
        assert!(result.contains("FILE* fopen("));
        assert!(result.contains("int fclose(FILE* stream);"));
        assert!(result.contains("size_t fread("));
        assert!(result.contains("size_t fwrite("));
        assert!(result.contains("int fseek("));
        assert!(result.contains("long ftell(FILE* stream);"));
        assert!(result.contains("void rewind(FILE* stream);"));
        // Character I/O
        assert!(result.contains("int getchar(void);"));
        assert!(result.contains("int putchar(int c);"));
        assert!(result.contains("int fgetc(FILE* stream);"));
        assert!(result.contains("int fputc(int c, FILE* stream);"));
        assert!(result.contains("char* fgets("));
        assert!(result.contains("int fputs("));
        assert!(result.contains("int puts(const char* s);"));
    }

    #[test]
    fn test_inject_header_stdio_does_not_contain_stdlib_functions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Stdio);
        // malloc is in stdlib, not stdio
        assert!(!result.contains("void* malloc("));
        assert!(!result.contains("void free("));
    }

    // ========================================================================
    // inject_prototypes_for_header: Errno
    // ========================================================================

    #[test]
    fn test_inject_header_errno_definitions() {
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
    fn test_inject_header_errno_no_functions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Errno);
        // Errno header has no registered functions, only macros/declarations
        assert!(!result.contains("int printf("));
        assert!(!result.contains("void* malloc("));
    }

    // ========================================================================
    // inject_prototypes_for_header: Time
    // ========================================================================

    #[test]
    fn test_inject_header_time_type_definitions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Time);
        assert!(result.contains("typedef long time_t;"));
        assert!(result.contains("typedef long clock_t;"));
        assert!(result.contains("struct tm;"));
        assert!(result.contains("#define CLOCKS_PER_SEC 1000000"));
    }

    #[test]
    fn test_inject_header_time_functions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Time);
        assert!(result.contains("clock_t clock(void);"));
        assert!(result.contains("time_t time(time_t* timer);"));
    }

    // ========================================================================
    // inject_prototypes_for_header: Stdarg
    // ========================================================================

    #[test]
    fn test_inject_header_stdarg_definitions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Stdarg);
        assert!(result.contains("typedef void* va_list;"));
        assert!(result.contains("#define va_start(ap, last) ((void)0)"));
        assert!(result.contains("#define va_end(ap) ((void)0)"));
        assert!(result.contains("#define va_arg(ap, type) (*(type*)0)"));
    }

    // ========================================================================
    // inject_prototypes_for_header: Stdbool
    // ========================================================================

    #[test]
    fn test_inject_header_stdbool_definitions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Stdbool);
        assert!(result.contains("typedef _Bool bool;"));
        assert!(result.contains("#define true 1"));
        assert!(result.contains("#define false 0"));
    }

    // ========================================================================
    // inject_prototypes_for_header: Stdint
    // ========================================================================

    #[test]
    fn test_inject_header_stdint_type_definitions() {
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

    // ========================================================================
    // inject_prototypes_for_header: Unistd
    // ========================================================================

    #[test]
    fn test_inject_header_unistd_type_definitions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Unistd);
        assert!(result.contains("typedef int pid_t;"));
        assert!(result.contains("typedef long off_t;"));
        assert!(result.contains("typedef unsigned int uid_t;"));
        assert!(result.contains("typedef unsigned int gid_t;"));
    }

    #[test]
    fn test_inject_header_unistd_macros() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Unistd);
        assert!(result.contains("#define STDIN_FILENO 0"));
        assert!(result.contains("#define STDOUT_FILENO 1"));
        assert!(result.contains("#define STDERR_FILENO 2"));
        assert!(result.contains("#define F_OK 0"));
        assert!(result.contains("#define R_OK 4"));
        assert!(result.contains("#define W_OK 2"));
        assert!(result.contains("#define X_OK 1"));
        assert!(result.contains("#define _SC_OPEN_MAX 4"));
        assert!(result.contains("#define _SC_PAGESIZE 30"));
    }

    #[test]
    fn test_inject_header_unistd_functions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Unistd);
        assert!(result.contains("int pipe(int* pipefd);"));
        assert!(result.contains("pid_t fork(void);"));
        assert!(result.contains("ssize_t read(int fd, void* buf, size_t count);"));
        assert!(result.contains("ssize_t write(int fd, const void* buf, size_t count);"));
        assert!(result.contains("int close(int fd);"));
        assert!(result.contains("off_t lseek(int fd, off_t offset, int whence);"));
        assert!(result.contains("int dup(int oldfd);"));
        assert!(result.contains("int dup2(int oldfd, int newfd);"));
    }

    // ========================================================================
    // inject_prototypes_for_header: Fcntl
    // ========================================================================

    #[test]
    fn test_inject_header_fcntl_macros() {
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
    }

    #[test]
    fn test_inject_header_fcntl_functions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Fcntl);
        assert!(result.contains("int open(const char* pathname, int flags, ...);"));
    }

    // ========================================================================
    // inject_prototypes_for_header: Dirent
    // ========================================================================

    #[test]
    fn test_inject_header_dirent_type_definitions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Dirent);
        assert!(result.contains("struct dirent { char d_name[256]; };"));
        assert!(result.contains("typedef struct __dirstream DIR;"));
    }

    #[test]
    fn test_inject_header_dirent_functions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Dirent);
        assert!(result.contains("DIR* opendir(const char* name);"));
        assert!(result.contains("struct dirent* readdir(DIR* dirp);"));
        assert!(result.contains("int closedir(DIR* dirp);"));
    }

    // ========================================================================
    // inject_prototypes_for_header: SysTypes
    // ========================================================================

    #[test]
    fn test_inject_header_sys_types_definitions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::SysTypes);
        assert!(result.contains("typedef int pid_t;"));
        assert!(result.contains("typedef long off_t;"));
        assert!(result.contains("typedef unsigned int mode_t;"));
        assert!(result.contains("typedef long ssize_t;"));
    }

    // ========================================================================
    // inject_prototypes_for_header: SysStat
    // ========================================================================

    #[test]
    fn test_inject_header_sys_stat_definitions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::SysStat);
        assert!(result.contains("struct stat { long st_size; int st_mode; };"));
        assert!(result.contains("#define S_ISREG(m)"));
        assert!(result.contains("#define S_ISDIR(m)"));
    }

    // ========================================================================
    // inject_prototypes_for_header: SysMman
    // ========================================================================

    #[test]
    fn test_inject_header_sys_mman_macros() {
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

    // ========================================================================
    // inject_prototypes_for_header: Wchar
    // ========================================================================

    #[test]
    fn test_inject_header_wchar_definitions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Wchar);
        assert!(result.contains("typedef int wchar_t;"));
        assert!(result.contains("typedef int wint_t;"));
        assert!(result.contains("#define WEOF (-1)"));
    }

    // ========================================================================
    // inject_prototypes_for_header: Signal
    // ========================================================================

    #[test]
    fn test_inject_header_signal_definitions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Signal);
        assert!(result.contains("typedef void (*sighandler_t)(int);"));
        assert!(result.contains("#define SIGINT 2"));
        assert!(result.contains("#define SIGTERM 15"));
    }

    // ========================================================================
    // inject_prototypes_for_header: Limits
    // ========================================================================

    #[test]
    fn test_inject_header_limits_macros() {
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

    // ========================================================================
    // inject_prototypes_for_header: Ctype
    // ========================================================================

    #[test]
    fn test_inject_header_ctype_functions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Ctype);
        assert!(result.contains("int isspace(int c);"));
        assert!(result.contains("int isdigit(int c);"));
        assert!(result.contains("int isalpha(int c);"));
        assert!(result.contains("int isalnum(int c);"));
        assert!(result.contains("int isupper(int c);"));
        assert!(result.contains("int islower(int c);"));
        assert!(result.contains("int tolower(int c);"));
        assert!(result.contains("int toupper(int c);"));
    }

    #[test]
    fn test_inject_header_ctype_no_extra_type_defs() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Ctype);
        // Ctype has no header-specific type definitions beyond the common preamble
        assert!(!result.contains("typedef struct _IO_FILE FILE;"));
        assert!(!result.contains("typedef long time_t;"));
    }

    // ========================================================================
    // inject_prototypes_for_header: Math
    // ========================================================================

    #[test]
    fn test_inject_header_math_macros() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Math);
        assert!(result.contains("#define M_PI 3.14159265358979323846"));
        assert!(result.contains("#define M_E 2.71828182845904523536"));
        assert!(result.contains("#define INFINITY (1.0/0.0)"));
        assert!(result.contains("#define NAN (0.0/0.0)"));
    }

    #[test]
    fn test_inject_header_math_trig_functions() {
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
    fn test_inject_header_math_power_and_log_functions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Math);
        assert!(result.contains("double sqrt(double x);"));
        assert!(result.contains("double pow(double x, double y);"));
        assert!(result.contains("double exp(double x);"));
        assert!(result.contains("double log(double x);"));
        assert!(result.contains("double log10(double x);"));
    }

    #[test]
    fn test_inject_header_math_rounding_functions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Math);
        assert!(result.contains("double fabs(double x);"));
        assert!(result.contains("double ceil(double x);"));
        assert!(result.contains("double floor(double x);"));
        assert!(result.contains("double round(double x);"));
        assert!(result.contains("double trunc(double x);"));
        assert!(result.contains("double fmod(double x, double y);"));
    }

    // ========================================================================
    // inject_prototypes_for_header: Assert (wildcard arm)
    // ========================================================================

    #[test]
    fn test_inject_header_assert_has_common_preamble() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Assert);
        // Assert falls through to wildcard - only common preamble
        assert!(result.contains("// Built-in prototypes for Assert"));
        assert!(result.contains("typedef unsigned long size_t;"));
        assert!(result.contains("#define NULL 0"));
    }

    #[test]
    fn test_inject_header_assert_no_header_specific_types() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Assert);
        // No functions are registered under Assert header
        assert!(!result.contains("typedef struct _IO_FILE FILE;"));
        assert!(!result.contains("typedef long time_t;"));
        assert!(!result.contains("extern int errno;"));
    }

    // ========================================================================
    // inject_prototypes_for_header: Float (wildcard arm)
    // ========================================================================

    #[test]
    fn test_inject_header_float_has_common_preamble() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Float);
        assert!(result.contains("// Built-in prototypes for Float"));
        assert!(result.contains("typedef unsigned long size_t;"));
        assert!(result.contains("#define NULL 0"));
    }

    #[test]
    fn test_inject_header_float_no_functions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Float);
        // No functions registered under Float header
        assert!(!result.contains("double sin("));
        assert!(!result.contains("int printf("));
    }

    // ========================================================================
    // inject_prototypes_for_header: Locale (wildcard arm)
    // ========================================================================

    #[test]
    fn test_inject_header_locale_has_common_preamble() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Locale);
        assert!(result.contains("// Built-in prototypes for Locale"));
        assert!(result.contains("typedef unsigned long size_t;"));
    }

    // ========================================================================
    // inject_prototypes_for_header: Setjmp (wildcard arm)
    // ========================================================================

    #[test]
    fn test_inject_header_setjmp_has_common_preamble() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Setjmp);
        assert!(result.contains("// Built-in prototypes for Setjmp"));
        assert!(result.contains("typedef unsigned long size_t;"));
    }

    // ========================================================================
    // inject_prototypes_for_header: Stddef (wildcard arm)
    // ========================================================================

    #[test]
    fn test_inject_header_stddef_has_common_preamble() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Stddef);
        assert!(result.contains("// Built-in prototypes for Stddef"));
        assert!(result.contains("typedef unsigned long size_t;"));
        assert!(result.contains("typedef long ptrdiff_t;"));
    }

    // ========================================================================
    // inject_prototypes_for_header: Stdlib
    // ========================================================================

    #[test]
    fn test_inject_header_stdlib_memory_functions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);
        assert!(result.contains("void* malloc(size_t size);"));
        assert!(result.contains("void* calloc(size_t nmemb, size_t size);"));
        assert!(result.contains("void* realloc(void* ptr, size_t size);"));
        assert!(result.contains("void free(void* ptr);"));
    }

    #[test]
    fn test_inject_header_stdlib_conversion_functions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);
        assert!(result.contains("int atoi(const char* nptr);"));
        assert!(result.contains("long atol(const char* nptr);"));
        assert!(result.contains("double atof(const char* nptr);"));
        assert!(result.contains("long strtol(const char* nptr, char** endptr, int base);"));
        assert!(result.contains("double strtod(const char* nptr, char** endptr);"));
    }

    #[test]
    fn test_inject_header_stdlib_process_functions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);
        assert!(result.contains("void exit(int status);"));
        assert!(result.contains("void abort(void);"));
        assert!(result.contains("char* getenv(const char* name);"));
        assert!(result.contains("int system(const char* command);"));
    }

    #[test]
    fn test_inject_header_stdlib_random_functions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);
        assert!(result.contains("int rand(void);"));
        assert!(result.contains("void srand(unsigned int seed);"));
    }

    #[test]
    fn test_inject_header_stdlib_arithmetic_functions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);
        assert!(result.contains("int abs(int j);"));
        assert!(result.contains("long labs(long j);"));
    }

    #[test]
    fn test_inject_header_stdlib_skips_function_pointer_params() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);
        // qsort and bsearch have function pointer parameters and should be skipped
        assert!(
            !result.contains("void qsort("),
            "qsort with function pointer param should be filtered out"
        );
        assert!(
            !result.contains("void* bsearch("),
            "bsearch with function pointer param should be filtered out"
        );
    }

    #[test]
    fn test_inject_header_stdlib_does_not_contain_stdio_functions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);
        assert!(!result.contains("int printf("));
        assert!(!result.contains("FILE* fopen("));
    }

    // ========================================================================
    // inject_prototypes_for_header: String
    // ========================================================================

    #[test]
    fn test_inject_header_string_copy_functions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::String);
        assert!(result.contains("void* memcpy(void* dest, const void* src, size_t n);"));
        assert!(result.contains("void* memmove(void* dest, const void* src, size_t n);"));
        assert!(result.contains("char* strcpy(char* dest, const char* src);"));
        assert!(result.contains("char* strncpy(char* dest, const char* src, size_t n);"));
    }

    #[test]
    fn test_inject_header_string_concat_functions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::String);
        assert!(result.contains("char* strcat(char* dest, const char* src);"));
        assert!(result.contains("char* strncat(char* dest, const char* src, size_t n);"));
    }

    #[test]
    fn test_inject_header_string_comparison_functions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::String);
        assert!(result.contains("int memcmp(const void* s1, const void* s2, size_t n);"));
        assert!(result.contains("int strcmp(const char* s1, const char* s2);"));
        assert!(result.contains("int strncmp(const char* s1, const char* s2, size_t n);"));
    }

    #[test]
    fn test_inject_header_string_search_functions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::String);
        assert!(result.contains("void* memchr(const void* s, int c, size_t n);"));
        assert!(result.contains("char* strchr(const char* s, int c);"));
        assert!(result.contains("char* strrchr(const char* s, int c);"));
        assert!(result.contains("char* strstr(const char* haystack, const char* needle);"));
        assert!(result.contains("char* strtok(char* str, const char* delim);"));
    }

    #[test]
    fn test_inject_header_string_misc_functions() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::String);
        assert!(result.contains("void* memset(void* s, int c, size_t n);"));
        assert!(result.contains("size_t strlen(const char* s);"));
        assert!(result.contains("char* strdup(const char* s);"));
    }

    #[test]
    fn test_inject_header_string_does_not_contain_math() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::String);
        assert!(!result.contains("double sin("));
        assert!(!result.contains("#define M_PI"));
    }

    // ========================================================================
    // inject_prototypes_for_header: Cross-header isolation tests
    // ========================================================================

    #[test]
    fn test_inject_header_isolation_no_cross_contamination() {
        let stdlib = StdlibPrototypes::new();

        // Each header only gets its own functions
        let stdio_result = stdlib.inject_prototypes_for_header(StdHeader::Stdio);
        let string_result = stdlib.inject_prototypes_for_header(StdHeader::String);
        let math_result = stdlib.inject_prototypes_for_header(StdHeader::Math);
        let ctype_result = stdlib.inject_prototypes_for_header(StdHeader::Ctype);

        // stdio should not have string functions
        assert!(!stdio_result.contains("size_t strlen("));
        assert!(!stdio_result.contains("void* memcpy("));

        // string should not have stdio functions
        assert!(!string_result.contains("int printf("));
        assert!(!string_result.contains("FILE* fopen("));

        // math should not have ctype functions
        assert!(!math_result.contains("int isspace("));
        assert!(!math_result.contains("int toupper("));

        // ctype should not have math functions
        assert!(!ctype_result.contains("double sqrt("));
        assert!(!ctype_result.contains("#define M_PI"));
    }

    #[test]
    fn test_inject_header_functions_are_sorted_alphabetically() {
        let stdlib = StdlibPrototypes::new();
        let result = stdlib.inject_prototypes_for_header(StdHeader::String);

        // Functions should appear in alphabetical order
        let memchr_pos = result.find("void* memchr(").unwrap();
        let memcmp_pos = result.find("int memcmp(").unwrap();
        let memcpy_pos = result.find("void* memcpy(").unwrap();
        let strlen_pos = result.find("size_t strlen(").unwrap();
        let strstr_pos = result.find("char* strstr(").unwrap();

        assert!(memchr_pos < memcmp_pos);
        assert!(memcmp_pos < memcpy_pos);
        assert!(memcpy_pos < strlen_pos);
        assert!(strlen_pos < strstr_pos);
    }

    #[test]
    fn test_inject_header_comment_contains_header_name() {
        let stdlib = StdlibPrototypes::new();

        let result = stdlib.inject_prototypes_for_header(StdHeader::Stdio);
        assert!(result.contains("// Built-in prototypes for Stdio"));

        let result = stdlib.inject_prototypes_for_header(StdHeader::Math);
        assert!(result.contains("// Built-in prototypes for Math"));

        let result = stdlib.inject_prototypes_for_header(StdHeader::SysMman);
        assert!(result.contains("// Built-in prototypes for SysMman"));
    }

    #[test]
    fn test_inject_header_result_ends_with_declarations() {
        let stdlib = StdlibPrototypes::new();
        // Verify that function declarations end with semicolons and newlines
        let result = stdlib.inject_prototypes_for_header(StdHeader::Stdlib);
        // All function lines should end with ";\n"
        for line in result.lines() {
            if line.contains('(') && line.contains(')') && !line.starts_with("//") && !line.starts_with("#") && !line.starts_with("typedef") && !line.starts_with("struct") && !line.starts_with("extern") {
                assert!(
                    line.ends_with(';'),
                    "Function declaration line should end with semicolon: {}",
                    line
                );
            }
        }
    }
}
