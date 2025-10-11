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

    // Extract function body by visiting children
    let mut body = Vec::new();
    let body_ptr = &mut body as *mut Vec<Statement>;

    unsafe {
        clang_visitChildren(cursor, visit_statement, body_ptr as CXClientData);
    }

    Some(Function::new_with_body(name, return_type, parameters, body))
}

/// Visitor callback for extracting statements from function body.
///
/// # Safety
///
/// This function is called by clang_visitChildren and must follow C calling conventions.
#[allow(non_upper_case_globals)]
extern "C" fn visit_statement(
    cursor: CXCursor,
    _parent: CXCursor,
    client_data: CXClientData,
) -> CXChildVisitResult {
    // SAFETY: Converting client data back to statement vector pointer
    let statements = unsafe { &mut *(client_data as *mut Vec<Statement>) };

    // SAFETY: Getting cursor kind
    let kind = unsafe { clang_getCursorKind(cursor) };

    match kind {
        CXCursor_CompoundStmt => {
            // Compound statement (function body) - recurse into it
            CXChildVisit_Recurse
        }
        CXCursor_DeclStmt => {
            // Declaration statement - visit its children to get the actual declaration
            CXChildVisit_Recurse
        }
        CXCursor_VarDecl => {
            // Variable declaration
            if let Some(stmt) = extract_var_decl(cursor) {
                statements.push(stmt);
            }
            CXChildVisit_Continue
        }
        CXCursor_ReturnStmt => {
            // Return statement
            if let Some(stmt) = extract_return_stmt(cursor) {
                statements.push(stmt);
            }
            CXChildVisit_Continue
        }
        CXCursor_BinaryOperator => {
            // Could be an assignment statement (x = 42)
            if let Some(stmt) = extract_assignment_stmt(cursor) {
                statements.push(stmt);
            }
            CXChildVisit_Continue
        }
        _ => CXChildVisit_Continue,
    }
}

/// Extract a variable declaration statement.
fn extract_var_decl(cursor: CXCursor) -> Option<Statement> {
    // Get variable name
    let name_cxstring = unsafe { clang_getCursorSpelling(cursor) };
    let name = unsafe {
        let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
        let name = c_str.to_string_lossy().into_owned();
        clang_disposeString(name_cxstring);
        name
    };

    // Get variable type
    let cx_type = unsafe { clang_getCursorType(cursor) };
    let var_type = convert_type(cx_type)?;

    // Extract initializer by visiting children
    let mut initializer: Option<Expression> = None;
    let init_ptr = &mut initializer as *mut Option<Expression>;

    unsafe {
        clang_visitChildren(cursor, visit_expression, init_ptr as CXClientData);
    }

    Some(Statement::VariableDeclaration {
        name,
        var_type,
        initializer,
    })
}

/// Extract a return statement.
fn extract_return_stmt(cursor: CXCursor) -> Option<Statement> {
    // Extract return expression by visiting children
    let mut return_expr: Option<Expression> = None;
    let expr_ptr = &mut return_expr as *mut Option<Expression>;

    unsafe {
        clang_visitChildren(cursor, visit_expression, expr_ptr as CXClientData);
    }

    Some(Statement::Return(return_expr))
}

/// Extract an assignment statement.
fn extract_assignment_stmt(cursor: CXCursor) -> Option<Statement> {
    // Check if this binary operator is an assignment '=' (not '==', '!=', etc.)
    // Get the translation unit
    let tu = unsafe { clang_Cursor_getTranslationUnit(cursor) };
    if tu.is_null() {
        return None;
    }

    // Get the extent (source range) of the cursor
    let extent = unsafe { clang_getCursorExtent(cursor) };

    // Tokenize to find the operator
    let mut tokens = ptr::null_mut();
    let mut num_tokens = 0;

    unsafe {
        clang_tokenize(tu, extent, &mut tokens, &mut num_tokens);
    }

    let mut is_assignment = false;

    // Look through tokens to find '=' (and make sure it's not '==', '!=', etc.)
    for i in 0..num_tokens {
        unsafe {
            let token = *tokens.add(i as usize);
            let token_kind = clang_getTokenKind(token);

            if token_kind == CXToken_Punctuation {
                let token_cxstring = clang_getTokenSpelling(tu, token);
                let c_str = CStr::from_ptr(clang_getCString(token_cxstring));
                if let Ok(token_str) = c_str.to_str() {
                    // Only accept single '=' for assignment
                    if token_str == "=" {
                        is_assignment = true;
                        clang_disposeString(token_cxstring);
                        break;
                    } else if token_str == "=="
                        || token_str == "!="
                        || token_str == "<="
                        || token_str == ">="
                    {
                        // This is a comparison operator, not assignment
                        clang_disposeString(token_cxstring);
                        break;
                    }
                }
                clang_disposeString(token_cxstring);
            }
        }
    }

    unsafe {
        clang_disposeTokens(tu, tokens, num_tokens);
    }

    if !is_assignment {
        return None;
    }

    // Extract left side (target) and right side (value)
    let mut operands: Vec<Expression> = Vec::new();
    let operands_ptr = &mut operands as *mut Vec<Expression>;

    unsafe {
        clang_visitChildren(cursor, visit_binary_operand, operands_ptr as CXClientData);
    }

    // Assignment should have exactly 2 operands
    if operands.len() != 2 {
        return None;
    }

    // Left side must be a variable reference
    let target = match &operands[0] {
        Expression::Variable(name) => name.clone(),
        _ => return None, // Can't assign to non-variables (yet)
    };

    Some(Statement::Assignment {
        target,
        value: operands[1].clone(),
    })
}

/// Visitor callback for extracting expressions.
///
/// # Safety
///
/// This function is called by clang_visitChildren and must follow C calling conventions.
#[allow(non_upper_case_globals)]
extern "C" fn visit_expression(
    cursor: CXCursor,
    _parent: CXCursor,
    client_data: CXClientData,
) -> CXChildVisitResult {
    // SAFETY: Converting client data back to expression option pointer
    let expr_opt = unsafe { &mut *(client_data as *mut Option<Expression>) };

    // SAFETY: Getting cursor kind
    let kind = unsafe { clang_getCursorKind(cursor) };

    match kind {
        CXCursor_IntegerLiteral => {
            // Integer literal
            if let Some(expr) = extract_int_literal(cursor) {
                *expr_opt = Some(expr);
            }
            CXChildVisit_Continue
        }
        CXCursor_DeclRefExpr => {
            // Variable reference (e.g., "a" or "b" in "a + b")
            if let Some(expr) = extract_variable_ref(cursor) {
                *expr_opt = Some(expr);
            }
            CXChildVisit_Continue
        }
        CXCursor_BinaryOperator => {
            // Binary operation (e.g., a + b)
            if let Some(expr) = extract_binary_op(cursor) {
                *expr_opt = Some(expr);
            }
            CXChildVisit_Continue
        }
        CXCursor_CallExpr => {
            // Function call
            if let Some(expr) = extract_function_call(cursor) {
                *expr_opt = Some(expr);
            }
            CXChildVisit_Continue
        }
        _ => CXChildVisit_Recurse,
    }
}

/// Extract an integer literal expression.
fn extract_int_literal(cursor: CXCursor) -> Option<Expression> {
    // SAFETY: Get the extent (source range) of the cursor
    let extent = unsafe { clang_getCursorExtent(cursor) };

    // SAFETY: Get the translation unit from the cursor
    let tu = unsafe {
        let loc = clang_getCursorLocation(cursor);
        let mut file = ptr::null_mut();
        let mut line = 0;
        let mut column = 0;
        let mut offset = 0;
        clang_getFileLocation(loc, &mut file, &mut line, &mut column, &mut offset);

        // Get the translation unit containing this cursor
        // We need to traverse up to get it, but for now use a different approach
        clang_Cursor_getTranslationUnit(cursor)
    };

    if tu.is_null() {
        return Some(Expression::IntLiteral(0));
    }

    // SAFETY: Tokenize the extent
    let mut tokens = ptr::null_mut();
    let mut num_tokens = 0;

    unsafe {
        clang_tokenize(tu, extent, &mut tokens, &mut num_tokens);
    }

    let mut value = 0;

    if num_tokens > 0 {
        // SAFETY: Get the spelling of the first token
        unsafe {
            let token_cxstring = clang_getTokenSpelling(tu, *tokens);
            let c_str = CStr::from_ptr(clang_getCString(token_cxstring));
            if let Ok(token_str) = c_str.to_str() {
                value = token_str.parse().unwrap_or(0);
            }
            clang_disposeString(token_cxstring);

            // SAFETY: Dispose tokens
            clang_disposeTokens(tu, tokens, num_tokens);
        }
    }

    Some(Expression::IntLiteral(value))
}

/// Extract a variable reference expression.
fn extract_variable_ref(cursor: CXCursor) -> Option<Expression> {
    // Get variable name
    let name_cxstring = unsafe { clang_getCursorSpelling(cursor) };
    let name = unsafe {
        let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
        let var_name = c_str.to_string_lossy().into_owned();
        clang_disposeString(name_cxstring);
        var_name
    };

    Some(Expression::Variable(name))
}

/// Extract a binary operation expression.
fn extract_binary_op(cursor: CXCursor) -> Option<Expression> {
    // Extract operator by tokenizing
    let op = extract_binary_operator(cursor)?;

    // Extract left and right operands by visiting children
    let mut operands: Vec<Expression> = Vec::new();
    let operands_ptr = &mut operands as *mut Vec<Expression>;

    unsafe {
        clang_visitChildren(cursor, visit_binary_operand, operands_ptr as CXClientData);
    }

    // Binary operators should have exactly 2 operands
    if operands.len() != 2 {
        return None;
    }

    Some(Expression::BinaryOp {
        op,
        left: Box::new(operands[0].clone()),
        right: Box::new(operands[1].clone()),
    })
}

/// Visitor callback for binary operator operands.
#[allow(non_upper_case_globals)]
extern "C" fn visit_binary_operand(
    cursor: CXCursor,
    _parent: CXCursor,
    client_data: CXClientData,
) -> CXChildVisitResult {
    let operands = unsafe { &mut *(client_data as *mut Vec<Expression>) };
    let kind = unsafe { clang_getCursorKind(cursor) };

    match kind {
        CXCursor_IntegerLiteral => {
            if let Some(expr) = extract_int_literal(cursor) {
                operands.push(expr);
            }
            CXChildVisit_Continue
        }
        CXCursor_DeclRefExpr => {
            if let Some(expr) = extract_variable_ref(cursor) {
                operands.push(expr);
            }
            CXChildVisit_Continue
        }
        CXCursor_BinaryOperator => {
            // Nested binary operation
            if let Some(expr) = extract_binary_op(cursor) {
                operands.push(expr);
            }
            CXChildVisit_Continue
        }
        _ => CXChildVisit_Recurse,
    }
}

/// Extract the binary operator from a cursor by tokenizing.
#[allow(non_upper_case_globals)]
fn extract_binary_operator(cursor: CXCursor) -> Option<BinaryOperator> {
    // Get the translation unit
    let tu = unsafe { clang_Cursor_getTranslationUnit(cursor) };
    if tu.is_null() {
        return None;
    }

    // Get the extent (source range) of the cursor
    let extent = unsafe { clang_getCursorExtent(cursor) };

    // Tokenize to find the operator
    let mut tokens = ptr::null_mut();
    let mut num_tokens = 0;

    unsafe {
        clang_tokenize(tu, extent, &mut tokens, &mut num_tokens);
    }

    let mut operator = None;

    // Look through tokens to find the operator
    for i in 0..num_tokens {
        unsafe {
            let token = *tokens.add(i as usize);
            let token_kind = clang_getTokenKind(token);

            // Only look at punctuation tokens
            if token_kind == CXToken_Punctuation {
                let token_cxstring = clang_getTokenSpelling(tu, token);
                let c_str = CStr::from_ptr(clang_getCString(token_cxstring));
                if let Ok(token_str) = c_str.to_str() {
                    operator = match token_str {
                        "+" => Some(BinaryOperator::Add),
                        "-" => Some(BinaryOperator::Subtract),
                        "*" => Some(BinaryOperator::Multiply),
                        "/" => Some(BinaryOperator::Divide),
                        "%" => Some(BinaryOperator::Modulo),
                        "==" => Some(BinaryOperator::Equal),
                        "!=" => Some(BinaryOperator::NotEqual),
                        "<" => Some(BinaryOperator::LessThan),
                        ">" => Some(BinaryOperator::GreaterThan),
                        "<=" => Some(BinaryOperator::LessEqual),
                        ">=" => Some(BinaryOperator::GreaterEqual),
                        _ => None,
                    };
                    if operator.is_some() {
                        clang_disposeString(token_cxstring);
                        break;
                    }
                }
                clang_disposeString(token_cxstring);
            }
        }
    }

    unsafe {
        clang_disposeTokens(tu, tokens, num_tokens);
    }

    operator
}

/// Extract a function call expression.
fn extract_function_call(cursor: CXCursor) -> Option<Expression> {
    // Get function name
    let name_cxstring = unsafe { clang_getCursorSpelling(cursor) };
    let function = unsafe {
        let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
        let name = c_str.to_string_lossy().into_owned();
        clang_disposeString(name_cxstring);
        name
    };

    // Extract arguments by visiting children
    let mut arguments = Vec::new();
    let args_ptr = &mut arguments as *mut Vec<Expression>;

    unsafe {
        clang_visitChildren(cursor, visit_call_argument, args_ptr as CXClientData);
    }

    Some(Expression::FunctionCall {
        function,
        arguments,
    })
}

/// Visitor callback for function call arguments.
///
/// # Safety
///
/// This function is called by clang_visitChildren and must follow C calling conventions.
#[allow(non_upper_case_globals)]
extern "C" fn visit_call_argument(
    cursor: CXCursor,
    _parent: CXCursor,
    client_data: CXClientData,
) -> CXChildVisitResult {
    // SAFETY: Converting client data back to arguments vector pointer
    let arguments = unsafe { &mut *(client_data as *mut Vec<Expression>) };

    // SAFETY: Getting cursor kind
    let kind = unsafe { clang_getCursorKind(cursor) };

    match kind {
        CXCursor_IntegerLiteral => {
            if let Some(expr) = extract_int_literal(cursor) {
                arguments.push(expr);
            }
            CXChildVisit_Continue
        }
        _ => CXChildVisit_Recurse,
    }
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

/// Represents a C statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Variable declaration: `int* ptr = malloc(4);`
    VariableDeclaration {
        /// Variable name
        name: String,
        /// Variable type
        var_type: Type,
        /// Optional initializer expression
        initializer: Option<Expression>,
    },
    /// Return statement: `return expr;`
    Return(Option<Expression>),
    /// Assignment statement: `x = 42;`
    Assignment {
        /// Target variable name
        target: String,
        /// Value expression to assign
        value: Expression,
    },
}

/// Binary operators for C expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    /// Addition (+)
    Add,
    /// Subtraction (-)
    Subtract,
    /// Multiplication (*)
    Multiply,
    /// Division (/)
    Divide,
    /// Modulo (%)
    Modulo,
    /// Equality (==)
    Equal,
    /// Inequality (!=)
    NotEqual,
    /// Less than (<)
    LessThan,
    /// Greater than (>)
    GreaterThan,
    /// Less than or equal (<=)
    LessEqual,
    /// Greater than or equal (>=)
    GreaterEqual,
}

/// Represents a C expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Integer literal: `42`
    IntLiteral(i32),
    /// Variable reference: `x`
    Variable(String),
    /// Binary operation: `a + b`
    BinaryOp {
        /// Operator
        op: BinaryOperator,
        /// Left operand
        left: Box<Expression>,
        /// Right operand
        right: Box<Expression>,
    },
    /// Function call: `malloc(4)`
    FunctionCall {
        /// Function name
        function: String,
        /// Arguments
        arguments: Vec<Expression>,
    },
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
    /// Function body (statements)
    pub body: Vec<Statement>,
}

impl Function {
    /// Create a new function.
    pub fn new(name: String, return_type: Type, parameters: Vec<Parameter>) -> Self {
        Self {
            name,
            return_type,
            parameters,
            body: Vec::new(),
        }
    }

    /// Create a new function with body.
    pub fn new_with_body(
        name: String,
        return_type: Type,
        parameters: Vec<Parameter>,
        body: Vec<Statement>,
    ) -> Self {
        Self {
            name,
            return_type,
            parameters,
            body,
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
