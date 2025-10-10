//! C parser implementation using clang-sys.
//!
//! This module provides the core parsing functionality to convert C source code
//! into an AST representation using LLVM/Clang bindings.

use anyhow::{Context, Result};
use clang_sys::*;
use std::ffi::{CStr, CString};
use std::path::Path;
use std::ptr;

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
}

impl CParser {
    /// Create a new C parser.
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
        Ok(Self { index })
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

        // SAFETY: Parsing with clang_parseTranslationUnit2
        let mut tu = ptr::null_mut();
        let result = unsafe {
            clang_parseTranslationUnit2(
                self.index,
                filename.as_ptr(),
                ptr::null(),
                0,
                &unsaved_file as *const CXUnsavedFile as *mut CXUnsavedFile,
                1,
                CXTranslationUnit_None,
                &mut tu,
            )
        };

        if result != CXError_Success || tu.is_null() {
            anyhow::bail!("Failed to parse C source");
        }

        // SAFETY: Check for diagnostics (errors/warnings)
        let num_diagnostics = unsafe { clang_getNumDiagnostics(tu) };
        for i in 0..num_diagnostics {
            let diag = unsafe { clang_getDiagnostic(tu, i) };
            let severity = unsafe { clang_getDiagnosticSeverity(diag) };

            // If we have errors, fail the parse
            if severity >= CXDiagnostic_Error {
                unsafe { clang_disposeDiagnostic(diag) };
                unsafe { clang_disposeTranslationUnit(tu) };
                anyhow::bail!("C source has syntax errors");
            }

            unsafe { clang_disposeDiagnostic(diag) };
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
    pub fn parse_file(&self, _path: &Path) -> Result<Ast> {
        // RED phase: not yet implemented
        Err(anyhow::anyhow!("Not implemented yet"))
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

/// Visitor callback for clang AST traversal.
///
/// # Safety
///
/// This function is called by clang_visitChildren and must follow C calling conventions.
extern "C" fn visit_function(
    cursor: CXCursor,
    _parent: CXCursor,
    client_data: CXClientData,
) -> CXChildVisitResult {
    // SAFETY: Converting client data back to AST pointer
    let ast = unsafe { &mut *(client_data as *mut Ast) };

    // SAFETY: Getting cursor kind
    let kind = unsafe { clang_getCursorKind(cursor) };

    if kind == CXCursor_FunctionDecl {
        // Extract function information
        if let Some(function) = extract_function(cursor) {
            ast.add_function(function);
        }
    }

    CXChildVisit_Continue
}

/// Extract function information from a clang cursor.
fn extract_function(cursor: CXCursor) -> Option<Function> {
    // SAFETY: Getting cursor spelling (function name)
    let name_cxstring = unsafe { clang_getCursorSpelling(cursor) };
    let name = unsafe {
        let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
        let name = c_str.to_string_lossy().into_owned();
        clang_disposeString(name_cxstring);
        name
    };

    // SAFETY: Getting return type
    let cx_type = unsafe { clang_getCursorType(cursor) };
    let return_cx_type = unsafe { clang_getResultType(cx_type) };
    let return_type = convert_type(return_cx_type)?;

    // Extract parameters
    let num_args = unsafe { clang_Cursor_getNumArguments(cursor) };
    let mut parameters = Vec::new();

    for i in 0..num_args {
        // SAFETY: Getting argument cursor
        let arg_cursor = unsafe { clang_Cursor_getArgument(cursor, i as u32) };

        // Get parameter name
        let param_name_cxstring = unsafe { clang_getCursorSpelling(arg_cursor) };
        let param_name = unsafe {
            let c_str = CStr::from_ptr(clang_getCString(param_name_cxstring));
            let name = c_str.to_string_lossy().into_owned();
            clang_disposeString(param_name_cxstring);
            name
        };

        // Get parameter type
        let param_cx_type = unsafe { clang_getCursorType(arg_cursor) };
        if let Some(param_type) = convert_type(param_cx_type) {
            parameters.push(Parameter::new(param_name, param_type));
        }
    }

    Some(Function::new(name, return_type, parameters))
}

/// Convert clang type to our Type enum.
#[allow(non_upper_case_globals)]
fn convert_type(cx_type: CXType) -> Option<Type> {
    // SAFETY: Getting type kind
    match cx_type.kind {
        CXType_Void => Some(Type::Void),
        CXType_Int => Some(Type::Int),
        CXType_Float => Some(Type::Float),
        CXType_Double => Some(Type::Double),
        CXType_Char_S | CXType_Char_U => Some(Type::Char),
        CXType_Pointer => {
            // SAFETY: Getting pointee type
            let pointee = unsafe { clang_getPointeeType(cx_type) };
            convert_type(pointee).map(|t| Type::Pointer(Box::new(t)))
        }
        _ => None,
    }
}

/// Abstract Syntax Tree representing parsed C code.
#[derive(Debug, Clone, PartialEq)]
pub struct Ast {
    functions: Vec<Function>,
}

impl Ast {
    /// Create a new empty AST.
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
        }
    }

    /// Get the functions in the AST.
    pub fn functions(&self) -> &[Function] {
        &self.functions
    }

    /// Add a function to the AST.
    pub fn add_function(&mut self, function: Function) {
        self.functions.push(function);
    }
}

impl Default for Ast {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a C function.
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    /// Function name
    pub name: String,
    /// Return type
    pub return_type: Type,
    /// Parameters
    pub parameters: Vec<Parameter>,
}

impl Function {
    /// Create a new function.
    pub fn new(name: String, return_type: Type, parameters: Vec<Parameter>) -> Self {
        Self {
            name,
            return_type,
            parameters,
        }
    }
}

/// Represents a C type.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// void
    Void,
    /// int
    Int,
    /// float
    Float,
    /// double
    Double,
    /// char
    Char,
    /// Pointer to a type
    Pointer(Box<Type>),
}

/// Represents a function parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: Type,
}

impl Parameter {
    /// Create a new parameter.
    pub fn new(name: String, param_type: Type) -> Self {
        Self { name, param_type }
    }
}

#[cfg(test)]
#[path = "parser_tests.rs"]
mod parser_tests;
