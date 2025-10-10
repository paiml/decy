//! C parser implementation using clang-sys.
//!
//! This module provides the core parsing functionality to convert C source code
//! into an AST representation using LLVM/Clang bindings.

use anyhow::Result;
use std::path::Path;

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
    // Will be implemented with clang::Clang, clang::Index
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
        // RED phase: not yet implemented
        Ok(Self {})
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
    pub fn parse(&self, _source: &str) -> Result<Ast> {
        // RED phase: not yet implemented
        Err(anyhow::anyhow!("Not implemented yet"))
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
