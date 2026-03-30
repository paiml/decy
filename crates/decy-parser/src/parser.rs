//! C parser implementation using clang-sys.
//!
//! This module provides the core parsing functionality to convert C source code
//! into an AST representation using LLVM/Clang bindings.

// Allow non-upper-case globals from clang-sys FFI bindings (CXCursor_* constants)
#![allow(non_upper_case_globals)]

use crate::diagnostic::{DiagnosticError, Severity};
use crate::extract::{extract_diagnostics, visit_function};
use anyhow::{Context, Result};
use clang_sys::*;
use std::ffi::CString;
use std::path::Path;
use std::process::Command;
use std::ptr;

/// Discover system include paths from the clang compiler.
///
/// This function runs `clang -E -x c - -v` to extract the system include
/// search paths, enabling parsing of code that uses standard headers.
fn discover_system_includes() -> Vec<String> {
    let mut includes = Vec::new();

    // Try to get include paths from clang
    let output = Command::new("clang")
        .args(["-E", "-x", "c", "-", "-v"])
        .stdin(std::process::Stdio::null())
        .output();

    if let Ok(output) = output {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut in_include_section = false;

        for line in stderr.lines() {
            if line.contains("#include <...> search starts here:") {
                in_include_section = true;
                continue;
            }
            if line.contains("End of search list.") {
                break;
            }
            if in_include_section {
                let path = line.trim();
                // Skip framework directories (macOS-specific)
                if !path.is_empty() && !path.contains("(framework directory)") {
                    includes.push(path.to_string());
                }
            }
        }
    }

    // Fallback paths if clang detection fails
    if includes.is_empty() {
        // macOS paths
        if cfg!(target_os = "macos") {
            includes.extend([
                "/usr/local/include".to_string(),
                "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk/usr/include".to_string(),
            ]);
        }
        // Linux paths
        if cfg!(target_os = "linux") {
            includes.extend(["/usr/include".to_string(), "/usr/local/include".to_string()]);
        }
    }

    includes
}

/// C parser using clang-sys.
///
/// # Examples
///
/// ```no_run
/// use decy_parser::parser::CParser;
///
/// let parser = CParser::new()?;
/// let ast = parser.parse("int main() { return 0; }")?;
/// assert_eq!(ast.functions().len(), 1);
/// # Ok::<(), anyhow::Error>(())
/// ```
#[derive(Debug)]
pub struct CParser {
    index: CXIndex,
    /// System include paths discovered from the compiler
    system_includes: Vec<String>,
}

impl CParser {
    /// Create a new C parser.
    ///
    /// This automatically discovers system include paths from the clang compiler.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use decy_parser::parser::CParser;
    ///
    /// let parser = CParser::new()?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new() -> Result<Self> {
        // SAFETY: clang_createIndex is safe to call with these parameters
        let index = unsafe { clang_createIndex(0, 0) };
        if index.is_null() {
            anyhow::bail!("Failed to create clang index");
        }

        // Discover system include paths for standard header support
        let system_includes = discover_system_includes();

        Ok(Self { index, system_includes })
    }

    /// Parse C source code into an AST.
    ///
    /// # Arguments
    ///
    /// * `source` - The C source code to parse
    ///
    /// # Returns
    ///
    /// * `Ok(Ast)` - The parsed AST
    /// * `Err(anyhow::Error)` - If parsing fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use decy_parser::parser::CParser;
    ///
    /// let parser = CParser::new()?;
    /// let ast = parser.parse("int add(int a, int b) { return a + b; }")?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    #[allow(clippy::disallowed_methods)] // CString::new with literals cannot fail
    pub fn parse(&self, source: &str) -> Result<Ast> {
        let filename = CString::new("input.c").context("Failed to create filename")?;
        let source_cstr = CString::new(source).context("Failed to convert source to CString")?;

        let mut ast = Ast::new();

        // Handle empty input
        if source.trim().is_empty() {
            return Ok(ast);
        }

        // SAFETY: Creating unsaved file with valid C strings
        let unsaved_file = CXUnsavedFile {
            Filename: filename.as_ptr(),
            Contents: source_cstr.as_ptr(),
            Length: source.len() as std::os::raw::c_ulong,
        };

        // Detect language mode from source content
        // DECY-198: Support C++ and CUDA language modes
        let has_extern_c = source.contains("extern \"C\"");
        let has_ifdef_guard =
            source.contains("#ifdef __cplusplus") || source.contains("#if defined(__cplusplus)");
        let needs_cpp_mode = has_extern_c && !has_ifdef_guard;

        // Build system include path arguments
        // We need to keep CStrings alive for the duration of parsing
        let isystem_flag = CString::new("-isystem").unwrap();
        let include_cstrings: Vec<CString> =
            self.system_includes.iter().map(|p| CString::new(p.as_str()).unwrap()).collect();

        // Prepare command line arguments for language mode
        let cpp_flag = CString::new("-x").unwrap();
        let cpp_lang = CString::new("c++").unwrap();

        // DECY-194: Add standard C macro definitions that might not be available
        // EOF is defined as -1 in stdio.h, NULL as 0
        let define_eof = CString::new("-DEOF=-1").unwrap();
        let define_null = CString::new("-DNULL=0").unwrap();
        // BUFSIZ from stdio.h (typical value)
        let define_bufsiz = CString::new("-DBUFSIZ=8192").unwrap();

        // Build the complete args vector
        let mut args_vec: Vec<*const std::os::raw::c_char> = Vec::new();

        // Add C++ mode flags if needed
        if needs_cpp_mode {
            args_vec.push(cpp_flag.as_ptr());
            args_vec.push(cpp_lang.as_ptr());
        }

        // Add macro definitions
        args_vec.push(define_eof.as_ptr());
        args_vec.push(define_null.as_ptr());
        args_vec.push(define_bufsiz.as_ptr());

        // Add system include paths
        for include_path in &include_cstrings {
            args_vec.push(isystem_flag.as_ptr());
            args_vec.push(include_path.as_ptr());
        }

        // SAFETY: Parsing with clang_parseTranslationUnit2
        // Enable DetailedPreprocessingRecord to capture macro definitions
        // CXTranslationUnit_DetailedPreprocessingRecord = 1
        let flags = 1;

        let mut tu = ptr::null_mut();
        let result = unsafe {
            clang_parseTranslationUnit2(
                self.index,
                filename.as_ptr(),
                if args_vec.is_empty() { ptr::null() } else { args_vec.as_ptr() },
                args_vec.len() as std::os::raw::c_int,
                &unsaved_file as *const CXUnsavedFile as *mut CXUnsavedFile,
                1,
                flags,
                &mut tu,
            )
        };

        if result != CXError_Success || tu.is_null() {
            anyhow::bail!("Failed to parse C source");
        }

        // SAFETY: Extract structured diagnostics from clang
        let diagnostics = extract_diagnostics(tu, source, None);
        if diagnostics.iter().any(|d| d.severity >= Severity::Error) {
            unsafe { clang_disposeTranslationUnit(tu) };
            return Err(DiagnosticError::new(diagnostics).into());
        }

        // SAFETY: Getting cursor from valid translation unit
        let cursor = unsafe { clang_getTranslationUnitCursor(tu) };

        // Visit children to extract functions
        let ast_ptr = &mut ast as *mut Ast;

        // SAFETY: Visiting cursor children with callback
        unsafe {
            clang_visitChildren(cursor, visit_function, ast_ptr as CXClientData);

            // Clean up
            clang_disposeTranslationUnit(tu);
        }

        Ok(ast)
    }

    /// Parse a C file into an AST.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the C file
    ///
    /// # Returns
    ///
    /// * `Ok(Ast)` - The parsed AST
    /// * `Err(anyhow::Error)` - If parsing fails
    ///
    /// Parse C source code from a file path.
    ///
    /// DECY-237: This method allows includes to resolve properly by parsing
    /// from the actual file path instead of an in-memory string.
    #[allow(clippy::disallowed_methods)] // CString::new with literals cannot fail
    pub fn parse_file(&self, path: &Path) -> Result<Ast> {
        contract_pre_parse!();
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        // Convert the path to an absolute path for clang
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };

        let filename =
            CString::new(abs_path.to_string_lossy().as_bytes()).context("Invalid path")?;
        let source_cstr = CString::new(source.as_str()).context("Invalid source")?;

        let mut ast = Ast::new();

        // Handle empty input
        if source.trim().is_empty() {
            return Ok(ast);
        }

        // SAFETY: Creating unsaved file with valid C strings
        let unsaved_file = CXUnsavedFile {
            Filename: filename.as_ptr(),
            Contents: source_cstr.as_ptr(),
            Length: source.len() as std::os::raw::c_ulong,
        };

        // DECY-198: Detect language mode from file extension and source content
        // .cu and .cpp/.cc/.cxx files use C++ mode (CUDA mode requires CUDA toolkit)
        let is_cpp_or_cuda_file = matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("cu") | Some("cpp") | Some("cc") | Some("cxx") | Some("hpp")
        );
        let has_extern_c = source.contains("extern \"C\"");
        let has_ifdef_guard =
            source.contains("#ifdef __cplusplus") || source.contains("#if defined(__cplusplus)");
        let needs_cpp_mode =
            is_cpp_or_cuda_file || (has_extern_c && !has_ifdef_guard);

        // Prepare command line arguments for language mode
        let cpp_flag = CString::new("-x").unwrap();
        let cpp_lang = CString::new("c++").unwrap();

        // Standard macro definitions
        let define_eof = CString::new("-DEOF=-1").unwrap();
        let define_null = CString::new("-DNULL=0").unwrap();
        let define_bufsiz = CString::new("-DBUFSIZ=8192").unwrap();

        // DECY-237: Add explicit defines for <limits.h> macros
        let define_char_bit = CString::new("-DCHAR_BIT=8").unwrap();
        let define_char_min = CString::new("-DCHAR_MIN=-128").unwrap();
        let define_char_max = CString::new("-DCHAR_MAX=127").unwrap();
        let define_schar_min = CString::new("-DSCHAR_MIN=-128").unwrap();
        let define_schar_max = CString::new("-DSCHAR_MAX=127").unwrap();
        let define_uchar_max = CString::new("-DUCHAR_MAX=255").unwrap();
        let define_shrt_min = CString::new("-DSHRT_MIN=-32768").unwrap();
        let define_shrt_max = CString::new("-DSHRT_MAX=32767").unwrap();
        let define_ushrt_max = CString::new("-DUSHRT_MAX=65535").unwrap();
        let define_int_min = CString::new("-DINT_MIN=-2147483648").unwrap();
        let define_int_max = CString::new("-DINT_MAX=2147483647").unwrap();
        let define_uint_max = CString::new("-DUINT_MAX=4294967295U").unwrap();
        let define_long_min = CString::new("-DLONG_MIN=-9223372036854775808L").unwrap();
        let define_long_max = CString::new("-DLONG_MAX=9223372036854775807L").unwrap();
        let define_ulong_max = CString::new("-DULONG_MAX=18446744073709551615UL").unwrap();

        // <stdlib.h> macros
        let define_exit_success = CString::new("-DEXIT_SUCCESS=0").unwrap();
        let define_exit_failure = CString::new("-DEXIT_FAILURE=1").unwrap();
        let define_rand_max = CString::new("-DRAND_MAX=2147483647").unwrap();

        // System include paths
        let include_clang = CString::new("-I/usr/lib/llvm-14/lib/clang/14.0.0/include").unwrap();
        let include_local = CString::new("-I/usr/local/include").unwrap();
        let include_arch = CString::new("-I/usr/include/x86_64-linux-gnu").unwrap();
        let include_usr = CString::new("-I/usr/include").unwrap();

        // Build the args vector with all defines
        let base_defines: Vec<*const std::os::raw::c_char> = vec![
            define_eof.as_ptr(),
            define_null.as_ptr(),
            define_bufsiz.as_ptr(),
            define_char_bit.as_ptr(),
            define_char_min.as_ptr(),
            define_char_max.as_ptr(),
            define_schar_min.as_ptr(),
            define_schar_max.as_ptr(),
            define_uchar_max.as_ptr(),
            define_shrt_min.as_ptr(),
            define_shrt_max.as_ptr(),
            define_ushrt_max.as_ptr(),
            define_int_min.as_ptr(),
            define_int_max.as_ptr(),
            define_uint_max.as_ptr(),
            define_long_min.as_ptr(),
            define_long_max.as_ptr(),
            define_ulong_max.as_ptr(),
            define_exit_success.as_ptr(),
            define_exit_failure.as_ptr(),
            define_rand_max.as_ptr(),
            include_clang.as_ptr(),
            include_local.as_ptr(),
            include_arch.as_ptr(),
            include_usr.as_ptr(),
        ];

        let args_vec: Vec<*const std::os::raw::c_char> = if needs_cpp_mode {
            let mut args = vec![cpp_flag.as_ptr(), cpp_lang.as_ptr()];
            args.extend(base_defines);
            args
        } else {
            base_defines
        };

        // Enable DetailedPreprocessingRecord to capture macro definitions
        let flags = 1;

        let mut tu = ptr::null_mut();
        let result = unsafe {
            clang_parseTranslationUnit2(
                self.index,
                filename.as_ptr(),
                if args_vec.is_empty() { ptr::null() } else { args_vec.as_ptr() },
                args_vec.len() as std::os::raw::c_int,
                &unsaved_file as *const CXUnsavedFile as *mut CXUnsavedFile,
                1,
                flags,
                &mut tu,
            )
        };

        if result != CXError_Success || tu.is_null() {
            anyhow::bail!("Failed to parse C source file: {}", path.display());
        }

        // SAFETY: Extract structured diagnostics from clang
        let file_name = path.to_string_lossy().to_string();
        let diagnostics = extract_diagnostics(tu, &source, Some(&file_name));
        if diagnostics.iter().any(|d| d.severity >= Severity::Error) {
            unsafe { clang_disposeTranslationUnit(tu) };
            return Err(DiagnosticError::new(diagnostics).into());
        }

        // Get cursor and visit
        let cursor = unsafe { clang_getTranslationUnitCursor(tu) };
        let ast_ptr = &mut ast as *mut Ast;

        unsafe {
            clang_visitChildren(cursor, visit_function, ast_ptr as CXClientData);
            clang_disposeTranslationUnit(tu);
        }

        Ok(ast)
    }
}

impl Drop for CParser {
    fn drop(&mut self) {
        // SAFETY: Disposing of valid clang index
        unsafe {
            clang_disposeIndex(self.index);
        }
    }
}

// Re-export all AST types so external users see them at parser:: level
pub use crate::ast_types::*;

#[cfg(test)]
#[path = "parser_tests.rs"]
mod parser_tests;

#[cfg(test)]
#[path = "pointer_arithmetic_tests.rs"]
mod pointer_arithmetic_tests;

#[cfg(test)]
#[path = "break_continue_tests.rs"]
mod break_continue_tests;

#[cfg(test)]
#[path = "cpp_cuda_tests.rs"]
mod cpp_cuda_tests;
