//! C standard library prototype initialization.
//! Split from lib.rs for PMAT File Health compliance.

use super::*;

impl StdlibPrototypes {
    pub(crate) fn init_stdlib(functions: &mut HashMap<String, FunctionProto>) {
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
                parameters: vec![Parameter::new("ptr", "void*"), Parameter::new("size", "size_t")],
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
    }

    /// ISO C99 §7.21 - Input/output <stdio.h>
    pub(crate) fn init_stdio(functions: &mut HashMap<String, FunctionProto>) {
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
                parameters: vec![Parameter::new("c", "int"), Parameter::new("stream", "FILE*")],
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
    }

    /// ISO C99 §7.23 - String handling <string.h>
    pub(crate) fn init_string(functions: &mut HashMap<String, FunctionProto>) {
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
                parameters: vec![Parameter::new("s", "const char*"), Parameter::new("c", "int")],
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
                parameters: vec![Parameter::new("s", "const char*"), Parameter::new("c", "int")],
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
    }

    /// ISO C99 §7.4 - Character handling <ctype.h>
    pub(crate) fn init_ctype(functions: &mut HashMap<String, FunctionProto>) {
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
    }

    /// ISO C99 §7.23 - Date and time <time.h>
    pub(crate) fn init_time(functions: &mut HashMap<String, FunctionProto>) {
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
    }

    /// ISO C99 §7.12 - Mathematics <math.h>
    pub(crate) fn init_math(functions: &mut HashMap<String, FunctionProto>) {
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
    }

    /// POSIX - unistd.h additional functions
    pub(crate) fn init_posix_unistd(functions: &mut HashMap<String, FunctionProto>) {
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
                parameters: vec![Parameter::new("oldfd", "int"), Parameter::new("newfd", "int")],
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
    }

    /// POSIX - dirent.h functions
    pub(crate) fn init_posix_dirent(functions: &mut HashMap<String, FunctionProto>) {
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
    }
}
