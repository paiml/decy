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

pub struct StdlibPrototypes {
    functions: HashMap<String, FunctionProto>,
}

    pub fn inject_prototypes_for_header(&self, header: StdHeader) -> String {
        let mut result = String::new();

        // Type definitions (always needed)
        result.push_str(&format!("// Built-in prototypes for {:?} (ISO C99 §7)\n", header));
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
                !p.parameters.iter().any(|param| param.type_str.contains("(*"))
            })
            .collect();
        protos.sort_by_key(|p| &p.name);

        for proto in protos {
            result.push_str(&proto.to_c_declaration());
            result.push('\n');
        }

        result
    }

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
