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

        // Detect if source contains extern "C" to enable C++ mode
        // This handles C headers that use extern "C" without proper #ifdef guards
        let needs_cpp_mode = source.contains("extern \"C\"");

        // Prepare command line arguments for C++ mode if needed
        let cpp_flag = CString::new("-x").unwrap();
        let cpp_lang = CString::new("c++").unwrap();
        let args_vec: Vec<*const std::os::raw::c_char> = if needs_cpp_mode {
            vec![cpp_flag.as_ptr(), cpp_lang.as_ptr()]
        } else {
            vec![]
        };

        // SAFETY: Parsing with clang_parseTranslationUnit2
        // Use CXTranslationUnit_DetailedPreprocessingRecord to capture macro definitions
        let mut tu = ptr::null_mut();
        let result = unsafe {
            clang_parseTranslationUnit2(
                self.index,
                filename.as_ptr(),
                if args_vec.is_empty() {
                    ptr::null()
                } else {
                    args_vec.as_ptr()
                },
                args_vec.len() as std::os::raw::c_int,
                &unsaved_file as *const CXUnsavedFile as *mut CXUnsavedFile,
                1,
                CXTranslationUnit_DetailedPreprocessingRecord,
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

    // Handle extern "C" linkage specifications (DECY-055)
    // CXCursor_LinkageSpec = 23
    if kind == 23 {
        // This is extern "C" { ... } - visit its children
        // Don't process the linkage spec itself, just recurse into declarations
        unsafe {
            clang_visitChildren(cursor, visit_function, client_data);
        }
        return CXChildVisit_Continue;
    }

    if kind == CXCursor_FunctionDecl {
        // Extract function information
        if let Some(function) = extract_function(cursor) {
            ast.add_function(function);
        }
    } else if kind == CXCursor_TypedefDecl {
        // Extract typedef information
        if let Some(typedef) = extract_typedef(cursor) {
            ast.add_typedef(typedef);
        }
    } else if kind == CXCursor_StructDecl {
        // Extract struct information
        if let Some(struct_def) = extract_struct(cursor) {
            ast.add_struct(struct_def);
        }
    } else if kind == CXCursor_VarDecl {
        // Extract variable declaration
        if let Some(variable) = extract_variable(cursor) {
            ast.add_variable(variable);
        }
    } else if kind == CXCursor_MacroDefinition {
        // Extract macro definition (only from main file, not includes)
        let location = unsafe { clang_getCursorLocation(cursor) };
        let mut file: CXFile = ptr::null_mut();
        unsafe {
            clang_getFileLocation(
                location,
                &mut file,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            );
        }

        // Only process macros from the main file (not system headers)
        if !file.is_null() {
            let file_name = unsafe {
                let name_cxstring = clang_getFileName(file);
                let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
                let name = c_str.to_string_lossy().into_owned();
                clang_disposeString(name_cxstring);
                name
            };

            // Only add macros from input.c (our source file)
            if file_name.ends_with("input.c") {
                if let Some(macro_def) = extract_macro(cursor) {
                    ast.add_macro(macro_def);
                }
            }
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

/// Extract typedef information from a clang cursor.
fn extract_typedef(cursor: CXCursor) -> Option<Typedef> {
    // SAFETY: Getting typedef name
    let name_cxstring = unsafe { clang_getCursorSpelling(cursor) };
    let name = unsafe {
        let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
        let name = c_str.to_string_lossy().into_owned();
        clang_disposeString(name_cxstring);
        name
    };

    // SAFETY: Getting underlying type of typedef
    let cx_type = unsafe { clang_getTypedefDeclUnderlyingType(cursor) };
    let underlying_type = convert_type(cx_type)?;

    Some(Typedef::new(name, underlying_type))
}

/// Extract struct information from a clang cursor.
fn extract_struct(cursor: CXCursor) -> Option<Struct> {
    // SAFETY: Getting struct name
    let name_cxstring = unsafe { clang_getCursorSpelling(cursor) };
    let name = unsafe {
        let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
        let name = c_str.to_string_lossy().into_owned();
        clang_disposeString(name_cxstring);
        name
    };

    // Skip anonymous structs
    if name.is_empty() {
        return None;
    }

    // Extract struct fields by visiting children
    let mut fields = Vec::new();
    let fields_ptr = &mut fields as *mut Vec<StructField>;

    unsafe {
        clang_visitChildren(cursor, visit_struct_fields, fields_ptr as CXClientData);
    }

    Some(Struct::new(name, fields))
}

/// Extract macro definition from a clang cursor.
///
/// Extract variable declaration information from a clang cursor.
///
/// Extracts global and local variable declarations, including function pointers.
///
/// # Examples
///
/// Simple: `int x;`
/// Function pointer: `int (*callback)(int);`
fn extract_variable(cursor: CXCursor) -> Option<Variable> {
    // SAFETY: Getting cursor spelling (variable name)
    let name_cxstring = unsafe { clang_getCursorSpelling(cursor) };
    let name = unsafe {
        let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
        let name = c_str.to_string_lossy().into_owned();
        clang_disposeString(name_cxstring);
        name
    };

    // SAFETY: Getting variable type
    let cx_type = unsafe { clang_getCursorType(cursor) };
    let var_type = convert_type(cx_type)?;

    // Extract initializer by visiting children
    let mut initializer: Option<Expression> = None;
    let initializer_ptr = &mut initializer as *mut Option<Expression>;

    unsafe {
        clang_visitChildren(
            cursor,
            visit_variable_initializer,
            initializer_ptr as CXClientData,
        );
    }

    if let Some(init_expr) = initializer {
        Some(Variable::new_with_initializer(name, var_type, init_expr))
    } else {
        Some(Variable::new(name, var_type))
    }
}

/// Helper function to extract an expression from a cursor.
/// Dispatches to the appropriate extract function based on cursor kind.
#[allow(non_upper_case_globals)]
fn try_extract_expression(cursor: CXCursor) -> Option<Expression> {
    let kind = unsafe { clang_getCursorKind(cursor) };

    match kind {
        CXCursor_IntegerLiteral => extract_int_literal(cursor),
        CXCursor_StringLiteral => extract_string_literal(cursor),
        CXCursor_DeclRefExpr => extract_variable_ref(cursor),
        CXCursor_BinaryOperator => extract_binary_op(cursor),
        CXCursor_CallExpr => extract_function_call(cursor),
        CXCursor_UnaryOperator => extract_unary_op(cursor),
        CXCursor_ArraySubscriptExpr => extract_array_index(cursor),
        CXCursor_MemberRefExpr => extract_field_access(cursor),
        CXCursor_UnexposedExpr => {
            // UnexposedExpr is a wrapper - recurse into children
            let mut result: Option<Expression> = None;
            let result_ptr = &mut result as *mut Option<Expression>;
            unsafe {
                clang_visitChildren(
                    cursor,
                    visit_variable_initializer,
                    result_ptr as CXClientData,
                );
            }
            result
        }
        _ => None,
    }
}

/// Visitor callback for variable initializer expressions.
#[allow(non_upper_case_globals)]
extern "C" fn visit_variable_initializer(
    cursor: CXCursor,
    _parent: CXCursor,
    client_data: CXClientData,
) -> CXChildVisitResult {
    let initializer = unsafe { &mut *(client_data as *mut Option<Expression>) };

    // Extract the first expression found (the initializer)
    if let Some(expr) = try_extract_expression(cursor) {
        *initializer = Some(expr);
        return CXChildVisit_Break;
    }

    CXChildVisit_Continue
}

/// This function extracts #define directives, supporting both object-like and function-like macros.
///
/// # Examples
///
/// Object-like: `#define MAX 100`
/// Function-like: `#define SQR(x) ((x) * (x))`
fn extract_macro(cursor: CXCursor) -> Option<MacroDefinition> {
    // SAFETY: Getting macro name
    let name_cxstring = unsafe { clang_getCursorSpelling(cursor) };
    let name = unsafe {
        let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
        let name = c_str.to_string_lossy().into_owned();
        clang_disposeString(name_cxstring);
        name
    };

    // Skip empty macro names
    if name.is_empty() {
        return None;
    }

    // Get macro body using clang_Cursor_isMacroFunctionLike and clang token APIs
    // For now, we'll check if it's function-like and extract tokens
    let is_function_like = unsafe { clang_sys::clang_Cursor_isMacroFunctionLike(cursor) } != 0;

    // Get the source range and tokens for the macro
    let range = unsafe { clang_getCursorExtent(cursor) };
    let tu = unsafe { clang_Cursor_getTranslationUnit(cursor) };

    let mut tokens: *mut CXToken = ptr::null_mut();
    let mut num_tokens: u32 = 0;

    unsafe {
        clang_tokenize(tu, range, &mut tokens, &mut num_tokens);
    }

    // Extract macro body from tokens
    // Skip the first token (macro name) and extract the rest
    let mut parameters = Vec::new();
    let mut body_tokens = Vec::new();
    let mut in_params = false;

    for i in 0..num_tokens {
        let token = unsafe { *tokens.offset(i as isize) };
        let token_kind = unsafe { clang_getTokenKind(token) };
        let token_spelling = unsafe { clang_getTokenSpelling(tu, token) };
        let token_str = unsafe {
            let c_str = CStr::from_ptr(clang_getCString(token_spelling));
            let s = c_str.to_string_lossy().into_owned();
            clang_disposeString(token_spelling);
            s
        };

        // Skip the macro name (first token)
        if i == 0 {
            continue;
        }

        // Check for parameter list (function-like macros)
        if is_function_like && i == 1 && token_str == "(" {
            in_params = true;
            continue;
        }

        if in_params {
            if token_str == ")" {
                in_params = false;
                continue;
            } else if token_str != ","
                && (token_kind == CXToken_Identifier || token_kind == CXToken_Keyword)
            {
                // Accept both identifiers and keywords as parameter names
                // C allows keywords in macro parameter names since they're in macro scope
                parameters.push(token_str);
            }
        } else {
            body_tokens.push(token_str);
        }
    }

    // Clean up tokens
    unsafe {
        clang_disposeTokens(tu, tokens, num_tokens);
    }

    // Join body tokens without spaces (preserving original formatting)
    let body = body_tokens.join("");

    if is_function_like {
        Some(MacroDefinition::new_function_like(name, parameters, body))
    } else {
        Some(MacroDefinition::new_object_like(name, body))
    }
}

/// Visitor callback for struct fields.
///
/// # Safety
///
/// This function is called by clang_visitChildren and must follow C calling conventions.
#[allow(non_upper_case_globals)]
extern "C" fn visit_struct_fields(
    cursor: CXCursor,
    _parent: CXCursor,
    client_data: CXClientData,
) -> CXChildVisitResult {
    // SAFETY: Converting client data back to fields vector pointer
    let fields = unsafe { &mut *(client_data as *mut Vec<StructField>) };

    // SAFETY: Getting cursor kind
    let kind = unsafe { clang_getCursorKind(cursor) };

    if kind == CXCursor_FieldDecl {
        // Get field name
        let name_cxstring = unsafe { clang_getCursorSpelling(cursor) };
        let name = unsafe {
            let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
            let name = c_str.to_string_lossy().into_owned();
            clang_disposeString(name_cxstring);
            name
        };

        // Get field type
        let cx_type = unsafe { clang_getCursorType(cursor) };
        if let Some(field_type) = convert_type(cx_type) {
            fields.push(StructField::new(name, field_type));
        }
    }

    CXChildVisit_Continue
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
        CXCursor_IfStmt => {
            // If statement
            if let Some(stmt) = extract_if_stmt(cursor) {
                statements.push(stmt);
            }
            CXChildVisit_Continue
        }
        CXCursor_ForStmt => {
            // For loop
            if let Some(stmt) = extract_for_stmt(cursor) {
                statements.push(stmt);
            }
            CXChildVisit_Continue
        }
        CXCursor_WhileStmt => {
            // While loop
            if let Some(stmt) = extract_while_stmt(cursor) {
                statements.push(stmt);
            }
            CXChildVisit_Continue
        }
        CXCursor_SwitchStmt => {
            // Switch statement
            if let Some(stmt) = extract_switch_stmt(cursor) {
                statements.push(stmt);
            }
            CXChildVisit_Continue
        }
        CXCursor_BreakStmt => {
            // Break statement
            statements.push(Statement::Break);
            CXChildVisit_Continue
        }
        CXCursor_ContinueStmt => {
            // Continue statement
            statements.push(Statement::Continue);
            CXChildVisit_Continue
        }
        CXCursor_UnaryOperator => {
            // Could be ++/-- statement (ptr++, ++ptr, ptr--, --ptr)
            if let Some(stmt) = extract_inc_dec_stmt(cursor) {
                statements.push(stmt);
            }
            CXChildVisit_Continue
        }
        CXCursor_CompoundAssignOperator => {
            // Compound assignment (+=, -=, *=, /=, %=)
            if let Some(stmt) = extract_compound_assignment_stmt(cursor) {
                statements.push(stmt);
            }
            CXChildVisit_Continue
        }
        _ => CXChildVisit_Recurse, // Recurse into unknown nodes to find statements
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

    // Check if left side is a dereference (e.g., *dst = x)
    if let Expression::Dereference(inner) = &operands[0] {
        return Some(Statement::DerefAssignment {
            target: (**inner).clone(), // Extract the inner expression being dereferenced
            value: operands[1].clone(),
        });
    }

    // Check if left side is an array index (e.g., arr[i] = value)
    if let Expression::ArrayIndex { array, index } = &operands[0] {
        return Some(Statement::ArrayIndexAssignment {
            array: array.clone(),
            index: index.clone(),
            value: operands[1].clone(),
        });
    }

    // Check if left side is a field access (e.g., ptr->field = value or obj.field = value)
    if matches!(
        &operands[0],
        Expression::PointerFieldAccess { .. } | Expression::FieldAccess { .. }
    ) {
        // Extract field name from the expression
        let field = match &operands[0] {
            Expression::PointerFieldAccess { field, .. } => field.clone(),
            Expression::FieldAccess { field, .. } => field.clone(),
            _ => unreachable!(),
        };

        // Extract object from the expression
        let object = match &operands[0] {
            Expression::PointerFieldAccess { pointer, .. } => (**pointer).clone(),
            Expression::FieldAccess { object, .. } => (**object).clone(),
            _ => unreachable!(),
        };

        return Some(Statement::FieldAssignment {
            object,
            field,
            value: operands[1].clone(),
        });
    }

    // Left side must be a variable reference for regular assignment
    let target = match &operands[0] {
        Expression::Variable(name) => name.clone(),
        _ => return None, // Can't assign to non-variables (yet)
    };

    Some(Statement::Assignment {
        target,
        value: operands[1].clone(),
    })
}

/// Extract an increment/decrement statement (++, --).
fn extract_inc_dec_stmt(cursor: CXCursor) -> Option<Statement> {
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

    let mut operator: Option<String> = None;
    let mut operator_position = 0;

    // Look through tokens to find ++ or --
    for i in 0..num_tokens {
        unsafe {
            let token = *tokens.add(i as usize);
            let token_kind = clang_getTokenKind(token);

            if token_kind == CXToken_Punctuation {
                let token_cxstring = clang_getTokenSpelling(tu, token);
                let c_str = CStr::from_ptr(clang_getCString(token_cxstring));
                if let Ok(token_str) = c_str.to_str() {
                    if token_str == "++" || token_str == "--" {
                        operator = Some(token_str.to_string()); // Clone the string before disposing
                        operator_position = i;
                        clang_disposeString(token_cxstring);
                        break;
                    }
                }
                clang_disposeString(token_cxstring);
            }
        }
    }

    // Determine if this is pre or post increment/decrement
    // If operator comes before identifier, it's pre (++ptr)
    // If operator comes after identifier, it's post (ptr++)
    let is_pre = operator_position == 0;

    unsafe {
        clang_disposeTokens(tu, tokens, num_tokens);
    }

    // Extract the target variable name by visiting children
    let mut target_name: Option<String> = None;

    // Visit children to find the DeclRefExpr (variable reference)
    extern "C" fn visit_for_target(
        cursor: CXCursor,
        _parent: CXCursor,
        client_data: CXClientData,
    ) -> CXChildVisitResult {
        let target = unsafe { &mut *(client_data as *mut Option<String>) };
        let kind = unsafe { clang_getCursorKind(cursor) };

        if kind == CXCursor_DeclRefExpr {
            let name_cxstring = unsafe { clang_getCursorSpelling(cursor) };
            let name = unsafe {
                let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
                let var_name = c_str.to_string_lossy().into_owned();
                clang_disposeString(name_cxstring);
                var_name
            };
            *target = Some(name);
            CXChildVisit_Break
        } else {
            CXChildVisit_Recurse
        }
    }

    let target_ptr = &mut target_name as *mut Option<String>;
    unsafe {
        clang_visitChildren(cursor, visit_for_target, target_ptr as CXClientData);
    }

    let target = target_name?;

    match operator?.as_str() {
        "++" => {
            if is_pre {
                Some(Statement::PreIncrement { target })
            } else {
                Some(Statement::PostIncrement { target })
            }
        }
        "--" => {
            if is_pre {
                Some(Statement::PreDecrement { target })
            } else {
                Some(Statement::PostDecrement { target })
            }
        }
        _ => None,
    }
}

/// Extract a compound assignment statement (+=, -=, *=, /=, %=).
fn extract_compound_assignment_stmt(cursor: CXCursor) -> Option<Statement> {
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

    let mut operator: Option<BinaryOperator> = None;

    // Look through tokens to find compound assignment operator
    for i in 0..num_tokens {
        unsafe {
            let token = *tokens.add(i as usize);
            let token_kind = clang_getTokenKind(token);

            if token_kind == CXToken_Punctuation {
                let token_cxstring = clang_getTokenSpelling(tu, token);
                let c_str = CStr::from_ptr(clang_getCString(token_cxstring));
                if let Ok(token_str) = c_str.to_str() {
                    operator = match token_str {
                        "+=" => Some(BinaryOperator::Add),
                        "-=" => Some(BinaryOperator::Subtract),
                        "*=" => Some(BinaryOperator::Multiply),
                        "/=" => Some(BinaryOperator::Divide),
                        "%=" => Some(BinaryOperator::Modulo),
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

    let op = operator?;

    // Extract left side (target) and right side (value)
    let mut operands: Vec<Expression> = Vec::new();
    let operands_ptr = &mut operands as *mut Vec<Expression>;

    unsafe {
        clang_visitChildren(cursor, visit_binary_operand, operands_ptr as CXClientData);
    }

    // Compound assignment should have exactly 2 operands
    if operands.len() != 2 {
        return None;
    }

    // Left side must be a variable reference
    let target = match &operands[0] {
        Expression::Variable(name) => name.clone(),
        _ => return None,
    };

    Some(Statement::CompoundAssignment {
        target,
        op,
        value: operands[1].clone(),
    })
}

/// Extract an if statement.
fn extract_if_stmt(cursor: CXCursor) -> Option<Statement> {
    // An if statement has 2 or 3 children:
    // 1. Condition expression
    // 2. Then block (compound statement)
    // 3. Else block (optional compound statement)

    #[repr(C)]
    struct IfData {
        condition: Option<Expression>,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
        child_index: u32,
    }

    let mut if_data = IfData {
        condition: None,
        then_block: Vec::new(),
        else_block: None,
        child_index: 0,
    };

    let data_ptr = &mut if_data as *mut IfData;

    unsafe {
        clang_visitChildren(cursor, visit_if_children, data_ptr as CXClientData);
    }

    Some(Statement::If {
        condition: if_data.condition?,
        then_block: if_data.then_block,
        else_block: if_data.else_block,
    })
}

/// Visitor for if statement children.
#[allow(non_upper_case_globals)]
extern "C" fn visit_if_children(
    cursor: CXCursor,
    _parent: CXCursor,
    client_data: CXClientData,
) -> CXChildVisitResult {
    #[repr(C)]
    struct IfData {
        condition: Option<Expression>,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
        child_index: u32,
    }

    let if_data = unsafe { &mut *(client_data as *mut IfData) };
    let kind = unsafe { clang_getCursorKind(cursor) };

    match if_data.child_index {
        0 => {
            // First child: condition expression
            // The cursor itself IS the condition, extract it directly
            if_data.condition = match kind {
                CXCursor_BinaryOperator => extract_binary_op(cursor),
                CXCursor_IntegerLiteral => extract_int_literal(cursor),
                CXCursor_DeclRefExpr => extract_variable_ref(cursor),
                CXCursor_CallExpr => extract_function_call(cursor),
                CXCursor_UnaryOperator => extract_unary_op(cursor),
                _ => {
                    // For other expression types, try visiting children
                    let mut cond_expr: Option<Expression> = None;
                    let expr_ptr = &mut cond_expr as *mut Option<Expression>;
                    unsafe {
                        clang_visitChildren(cursor, visit_expression, expr_ptr as CXClientData);
                    }
                    cond_expr
                }
            };
            if_data.child_index += 1;
            CXChildVisit_Continue
        }
        1 => {
            // Second child: then block
            if kind == CXCursor_CompoundStmt {
                let body_ptr = &mut if_data.then_block as *mut Vec<Statement>;
                unsafe {
                    clang_visitChildren(cursor, visit_statement, body_ptr as CXClientData);
                }
            }
            if_data.child_index += 1;
            CXChildVisit_Continue
        }
        2 => {
            // Third child (optional): else block
            if kind == CXCursor_CompoundStmt || kind == CXCursor_IfStmt {
                let mut else_stmts = Vec::new();
                let body_ptr = &mut else_stmts as *mut Vec<Statement>;
                unsafe {
                    clang_visitChildren(cursor, visit_statement, body_ptr as CXClientData);
                }
                if_data.else_block = Some(else_stmts);
            }
            if_data.child_index += 1;
            CXChildVisit_Continue
        }
        _ => CXChildVisit_Continue,
    }
}

/// Extract a for loop statement.
fn extract_for_stmt(cursor: CXCursor) -> Option<Statement> {
    // A for loop has up to 4 children:
    // 1. Init statement (optional - could be DeclStmt or expression)
    // 2. Condition expression (optional)
    // 3. Increment expression (optional)
    // 4. Body (compound statement)

    #[repr(C)]
    struct ForData {
        init: Option<Box<Statement>>,
        condition: Option<Expression>,
        increment: Option<Box<Statement>>,
        body: Vec<Statement>,
        child_index: u32,
    }

    let mut for_data = ForData {
        init: None,
        condition: None,
        increment: None,
        body: Vec::new(),
        child_index: 0,
    };

    let data_ptr = &mut for_data as *mut ForData;

    unsafe {
        clang_visitChildren(cursor, visit_for_children, data_ptr as CXClientData);
    }

    Some(Statement::For {
        init: for_data.init,
        condition: for_data.condition,
        increment: for_data.increment,
        body: for_data.body,
    })
}

/// Visitor for for loop children.
#[allow(non_upper_case_globals)]
extern "C" fn visit_for_children(
    cursor: CXCursor,
    _parent: CXCursor,
    client_data: CXClientData,
) -> CXChildVisitResult {
    #[repr(C)]
    struct ForData {
        init: Option<Box<Statement>>,
        condition: Option<Expression>,
        increment: Option<Box<Statement>>,
        body: Vec<Statement>,
        child_index: u32,
    }

    let for_data = unsafe { &mut *(client_data as *mut ForData) };
    let kind = unsafe { clang_getCursorKind(cursor) };

    match for_data.child_index {
        0 => {
            // First child: init statement (could be DeclStmt or NULL)
            if kind == CXCursor_DeclStmt {
                // Visit to get the variable declaration
                let mut init_stmts = Vec::new();
                let ptr = &mut init_stmts as *mut Vec<Statement>;
                unsafe {
                    clang_visitChildren(cursor, visit_statement, ptr as CXClientData);
                }
                if let Some(stmt) = init_stmts.into_iter().next() {
                    for_data.init = Some(Box::new(stmt));
                }
            } else if kind == CXCursor_BinaryOperator {
                // Assignment in init
                if let Some(stmt) = extract_assignment_stmt(cursor) {
                    for_data.init = Some(Box::new(stmt));
                }
            }
            for_data.child_index += 1;
            CXChildVisit_Continue
        }
        1 => {
            // Second child: condition expression
            // The cursor itself IS the condition, extract it directly
            for_data.condition = match kind {
                CXCursor_BinaryOperator => extract_binary_op(cursor),
                CXCursor_IntegerLiteral => extract_int_literal(cursor),
                CXCursor_DeclRefExpr => extract_variable_ref(cursor),
                CXCursor_CallExpr => extract_function_call(cursor),
                CXCursor_UnaryOperator => extract_unary_op(cursor),
                _ => {
                    let mut cond_expr: Option<Expression> = None;
                    let expr_ptr = &mut cond_expr as *mut Option<Expression>;
                    unsafe {
                        clang_visitChildren(cursor, visit_expression, expr_ptr as CXClientData);
                    }
                    cond_expr
                }
            };
            for_data.child_index += 1;
            CXChildVisit_Continue
        }
        2 => {
            // Third child: increment statement
            if kind == CXCursor_BinaryOperator {
                if let Some(stmt) = extract_assignment_stmt(cursor) {
                    for_data.increment = Some(Box::new(stmt));
                }
            } else if kind == CXCursor_UnaryOperator {
                // Handle ++/-- in increment position
                if let Some(stmt) = extract_inc_dec_stmt(cursor) {
                    for_data.increment = Some(Box::new(stmt));
                }
            }
            for_data.child_index += 1;
            CXChildVisit_Continue
        }
        3 => {
            // Fourth child: body
            if kind == CXCursor_CompoundStmt {
                let body_ptr = &mut for_data.body as *mut Vec<Statement>;
                unsafe {
                    clang_visitChildren(cursor, visit_statement, body_ptr as CXClientData);
                }
            }
            for_data.child_index += 1;
            CXChildVisit_Continue
        }
        _ => CXChildVisit_Continue,
    }
}

/// Extract a while loop statement.
fn extract_while_stmt(cursor: CXCursor) -> Option<Statement> {
    // A while loop has 2 children:
    // 1. Condition expression
    // 2. Body (compound statement)

    #[repr(C)]
    struct WhileData {
        condition: Option<Expression>,
        body: Vec<Statement>,
        child_index: u32,
    }

    let mut while_data = WhileData {
        condition: None,
        body: Vec::new(),
        child_index: 0,
    };

    let data_ptr = &mut while_data as *mut WhileData;

    unsafe {
        clang_visitChildren(cursor, visit_while_children, data_ptr as CXClientData);
    }

    Some(Statement::While {
        condition: while_data.condition?,
        body: while_data.body,
    })
}

/// Visitor for while loop children.
#[allow(non_upper_case_globals)]
extern "C" fn visit_while_children(
    cursor: CXCursor,
    _parent: CXCursor,
    client_data: CXClientData,
) -> CXChildVisitResult {
    #[repr(C)]
    struct WhileData {
        condition: Option<Expression>,
        body: Vec<Statement>,
        child_index: u32,
    }

    let while_data = unsafe { &mut *(client_data as *mut WhileData) };
    let kind = unsafe { clang_getCursorKind(cursor) };

    match while_data.child_index {
        0 => {
            // First child: condition expression
            // The cursor itself IS the condition, extract it directly
            while_data.condition = match kind {
                CXCursor_BinaryOperator => extract_binary_op(cursor),
                CXCursor_IntegerLiteral => extract_int_literal(cursor),
                CXCursor_DeclRefExpr => extract_variable_ref(cursor),
                CXCursor_CallExpr => extract_function_call(cursor),
                CXCursor_UnaryOperator => extract_unary_op(cursor),
                _ => {
                    let mut cond_expr: Option<Expression> = None;
                    let expr_ptr = &mut cond_expr as *mut Option<Expression>;
                    unsafe {
                        clang_visitChildren(cursor, visit_expression, expr_ptr as CXClientData);
                    }
                    cond_expr
                }
            };
            while_data.child_index += 1;
            CXChildVisit_Continue
        }
        1 => {
            // Second child: body
            if kind == CXCursor_CompoundStmt {
                let body_ptr = &mut while_data.body as *mut Vec<Statement>;
                unsafe {
                    clang_visitChildren(cursor, visit_statement, body_ptr as CXClientData);
                }
            }
            while_data.child_index += 1;
            CXChildVisit_Continue
        }
        _ => CXChildVisit_Continue,
    }
}

/// Extract a switch statement from a cursor.
///
/// Parses switch statements including cases and default labels.
#[allow(non_upper_case_globals)]
fn extract_switch_stmt(cursor: CXCursor) -> Option<Statement> {
    // Switch has 2 children:
    // 1. Condition expression
    // 2. Body (compound statement containing case/default labels)

    #[repr(C)]
    struct SwitchData {
        condition: Option<Expression>,
        cases: Vec<SwitchCase>,
        default_case: Option<Vec<Statement>>,
        child_index: u32,
    }

    let mut switch_data = SwitchData {
        condition: None,
        cases: Vec::new(),
        default_case: None,
        child_index: 0,
    };

    let data_ptr = &mut switch_data as *mut SwitchData;

    unsafe {
        clang_visitChildren(cursor, visit_switch_children, data_ptr as CXClientData);
    }

    Some(Statement::Switch {
        condition: switch_data.condition?,
        cases: switch_data.cases,
        default_case: switch_data.default_case,
    })
}

/// Visitor callback for switch statement children (condition and body).
#[allow(non_upper_case_globals)]
extern "C" fn visit_switch_children(
    cursor: CXCursor,
    _parent: CXCursor,
    client_data: CXClientData,
) -> CXChildVisitResult {
    #[repr(C)]
    struct SwitchData {
        condition: Option<Expression>,
        cases: Vec<SwitchCase>,
        default_case: Option<Vec<Statement>>,
        child_index: u32,
    }

    let switch_data = unsafe { &mut *(client_data as *mut SwitchData) };
    let kind = unsafe { clang_getCursorKind(cursor) };

    match switch_data.child_index {
        0 => {
            // First child: condition expression
            if let Some(expr) = try_extract_expression(cursor) {
                switch_data.condition = Some(expr);
            }
            switch_data.child_index += 1;
            CXChildVisit_Continue
        }
        1 => {
            // Second child: compound statement body containing cases
            // Need to visit this recursively to find case/default labels
            if kind == CXCursor_CompoundStmt {
                unsafe {
                    clang_visitChildren(cursor, visit_switch_body, client_data);
                }
            }
            switch_data.child_index += 1;
            CXChildVisit_Continue
        }
        _ => CXChildVisit_Continue,
    }
}

/// Visitor callback for switch body to extract cases and default.
#[allow(non_upper_case_globals)]
extern "C" fn visit_switch_body(
    cursor: CXCursor,
    _parent: CXCursor,
    client_data: CXClientData,
) -> CXChildVisitResult {
    #[repr(C)]
    struct SwitchData {
        condition: Option<Expression>,
        cases: Vec<SwitchCase>,
        default_case: Option<Vec<Statement>>,
        child_index: u32,
    }

    let switch_data = unsafe { &mut *(client_data as *mut SwitchData) };
    let kind = unsafe { clang_getCursorKind(cursor) };

    match kind {
        CXCursor_CaseStmt => {
            // Extract case statement
            if let Some(case) = extract_case_stmt(cursor) {
                switch_data.cases.push(case);
            }
            CXChildVisit_Continue
        }
        CXCursor_DefaultStmt => {
            // Extract default statement
            if let Some(body) = extract_default_stmt(cursor) {
                switch_data.default_case = Some(body);
            }
            CXChildVisit_Continue
        }
        _ => CXChildVisit_Continue,
    }
}

/// Extract a case statement from a cursor.
fn extract_case_stmt(cursor: CXCursor) -> Option<SwitchCase> {
    // Case statement has 2 children:
    // 1. Case value expression
    // 2. Body (statements following the case label)

    #[repr(C)]
    struct CaseData {
        value: Option<Expression>,
        body: Vec<Statement>,
        child_index: u32,
    }

    let mut case_data = CaseData {
        value: None,
        body: Vec::new(),
        child_index: 0,
    };

    let data_ptr = &mut case_data as *mut CaseData;

    unsafe {
        clang_visitChildren(cursor, visit_case_children, data_ptr as CXClientData);
    }

    Some(SwitchCase {
        value: case_data.value,
        body: case_data.body,
    })
}

/// Visitor for case statement children.
#[allow(non_upper_case_globals)]
extern "C" fn visit_case_children(
    cursor: CXCursor,
    _parent: CXCursor,
    client_data: CXClientData,
) -> CXChildVisitResult {
    #[repr(C)]
    struct CaseData {
        value: Option<Expression>,
        body: Vec<Statement>,
        child_index: u32,
    }

    let case_data = unsafe { &mut *(client_data as *mut CaseData) };
    let _kind = unsafe { clang_getCursorKind(cursor) };

    match case_data.child_index {
        0 => {
            // First child: case value expression
            if let Some(expr) = try_extract_expression(cursor) {
                case_data.value = Some(expr);
            }
            case_data.child_index += 1;
            CXChildVisit_Continue
        }
        _ => {
            // Subsequent children: statements in case body
            // Extract statements until we hit another case or default
            if let Some(stmt) = extract_statement(cursor) {
                case_data.body.push(stmt);
            }
            // Continue recursing to find all statements in the case body
            CXChildVisit_Recurse
        }
    }
}

/// Extract a default statement from a cursor.
fn extract_default_stmt(cursor: CXCursor) -> Option<Vec<Statement>> {
    // Default statement has body statements as children
    let mut body: Vec<Statement> = Vec::new();
    let body_ptr = &mut body as *mut Vec<Statement>;

    unsafe {
        clang_visitChildren(cursor, visit_default_children, body_ptr as CXClientData);
    }

    Some(body)
}

/// Visitor for default statement children.
#[allow(non_upper_case_globals)]
extern "C" fn visit_default_children(
    cursor: CXCursor,
    _parent: CXCursor,
    client_data: CXClientData,
) -> CXChildVisitResult {
    let body = unsafe { &mut *(client_data as *mut Vec<Statement>) };

    // Extract all statements in default body
    if let Some(stmt) = extract_statement(cursor) {
        body.push(stmt);
    }

    CXChildVisit_Continue
}

/// Helper function to extract a statement from a cursor based on its kind.
#[allow(non_upper_case_globals)]
fn extract_statement(cursor: CXCursor) -> Option<Statement> {
    let kind = unsafe { clang_getCursorKind(cursor) };

    match kind {
        CXCursor_ReturnStmt => extract_return_stmt(cursor),
        CXCursor_VarDecl => extract_var_decl(cursor),
        CXCursor_IfStmt => extract_if_stmt(cursor),
        CXCursor_ForStmt => extract_for_stmt(cursor),
        CXCursor_WhileStmt => extract_while_stmt(cursor),
        CXCursor_BreakStmt => Some(Statement::Break),
        CXCursor_ContinueStmt => Some(Statement::Continue),
        CXCursor_UnaryOperator => extract_inc_dec_stmt(cursor),
        CXCursor_BinaryOperator => extract_assignment_stmt(cursor),
        CXCursor_CallExpr => {
            // Function call as statement
            if let Some(Expression::FunctionCall {
                function,
                arguments,
            }) = extract_function_call(cursor)
            {
                return Some(Statement::FunctionCall {
                    function,
                    arguments,
                });
            }
            None
        }
        _ => None,
    }
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
        CXCursor_StringLiteral => {
            // String literal
            if let Some(expr) = extract_string_literal(cursor) {
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
        CXCursor_UnaryOperator => {
            // Unary operator (e.g., *ptr dereference)
            if let Some(expr) = extract_unary_op(cursor) {
                *expr_opt = Some(expr);
            }
            CXChildVisit_Continue
        }
        CXCursor_ArraySubscriptExpr => {
            // Array indexing (e.g., arr[i])
            if let Some(expr) = extract_array_index(cursor) {
                *expr_opt = Some(expr);
            }
            CXChildVisit_Continue
        }
        CXCursor_MemberRefExpr => {
            // Field access (e.g., ptr->field or obj.field)
            if let Some(expr) = extract_field_access(cursor) {
                *expr_opt = Some(expr);
            }
            CXChildVisit_Continue
        }
        CXCursor_UnexposedExpr => {
            // Unexposed expressions might wrap other expressions (like ImplicitCastExpr wrapping CallExpr)
            // Recurse first to check if there's a more specific expression inside
            CXChildVisit_Recurse
        }
        CXCursor_ParenExpr => {
            // Parenthesized expressions wrap other expressions, recurse
            CXChildVisit_Recurse
        }
        136 => {
            // CXCursor_UnaryExpr - could be sizeof or other unary expr
            if let Some(expr) = extract_sizeof(cursor) {
                *expr_opt = Some(expr);
                CXChildVisit_Continue
            } else {
                // Not sizeof, recurse for other unary expressions
                CXChildVisit_Recurse
            }
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

/// Extract a string literal expression.
fn extract_string_literal(cursor: CXCursor) -> Option<Expression> {
    // SAFETY: Get the extent (source range) of the cursor
    let extent = unsafe { clang_getCursorExtent(cursor) };

    // SAFETY: Get the translation unit from the cursor
    let tu = unsafe { clang_Cursor_getTranslationUnit(cursor) };

    if tu.is_null() {
        return Some(Expression::StringLiteral(String::new()));
    }

    // SAFETY: Tokenize the extent
    let mut tokens = ptr::null_mut();
    let mut num_tokens = 0;

    unsafe {
        clang_tokenize(tu, extent, &mut tokens, &mut num_tokens);
    }

    let mut value = String::new();

    if num_tokens > 0 {
        // SAFETY: Get the spelling of the first token
        unsafe {
            let token_cxstring = clang_getTokenSpelling(tu, *tokens);
            let c_str = CStr::from_ptr(clang_getCString(token_cxstring));
            if let Ok(token_str) = c_str.to_str() {
                // Remove surrounding quotes from string literal
                value = token_str.trim_matches('"').to_string();
            }
            clang_disposeString(token_cxstring);

            // SAFETY: Dispose tokens
            clang_disposeTokens(tu, tokens, num_tokens);
        }
    }

    Some(Expression::StringLiteral(value))
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
        CXCursor_StringLiteral => {
            if let Some(expr) = extract_string_literal(cursor) {
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
        CXCursor_UnaryOperator => {
            // Unary operation (e.g., *ptr dereference)
            if let Some(expr) = extract_unary_op(cursor) {
                operands.push(expr);
            }
            CXChildVisit_Continue
        }
        CXCursor_ArraySubscriptExpr => {
            // Array indexing (e.g., arr[i])
            if let Some(expr) = extract_array_index(cursor) {
                operands.push(expr);
            }
            CXChildVisit_Continue
        }
        CXCursor_MemberRefExpr => {
            // Field access (e.g., ptr->field or obj.field)
            if let Some(expr) = extract_field_access(cursor) {
                operands.push(expr);
            }
            CXChildVisit_Continue
        }
        CXCursor_UnexposedExpr | CXCursor_ParenExpr => {
            // Unexposed expressions might be sizeof or wrap other expressions
            if let Some(expr) = extract_sizeof(cursor) {
                operands.push(expr);
                CXChildVisit_Continue
            } else {
                CXChildVisit_Recurse
            }
        }
        136 => {
            // CXCursor_UnaryExpr - includes sizeof, alignof, etc.
            if let Some(expr) = extract_sizeof(cursor) {
                operands.push(expr);
                CXChildVisit_Continue
            } else {
                CXChildVisit_Recurse
            }
        }
        CXCursor_CallExpr => {
            // Function call expression (e.g., malloc(size))
            if let Some(expr) = extract_function_call(cursor) {
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
    // For compound expressions like "a > 0 && b > 0", we need to find the LAST
    // operator (the one with lowest precedence) which represents THIS binary operation.
    // We scan from right to left to find operators with lowest precedence first.
    // Precedence (lowest to highest): || > && > == != > < > <= >= > + - > * / %

    let mut candidates: Vec<(usize, BinaryOperator)> = Vec::new();
    let mut found_first_operand = false;

    for i in 0..num_tokens {
        unsafe {
            let token = *tokens.add(i as usize);
            let token_kind = clang_getTokenKind(token);

            // Track when we've seen the first operand (identifier or literal)
            if token_kind == CXToken_Identifier || token_kind == CXToken_Literal {
                found_first_operand = true;
            }

            // Collect all operator candidates after the first operand
            if token_kind == CXToken_Punctuation && found_first_operand {
                let token_cxstring = clang_getTokenSpelling(tu, token);
                let c_str = CStr::from_ptr(clang_getCString(token_cxstring));
                if let Ok(token_str) = c_str.to_str() {
                    let op = match token_str {
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
                        "&&" => Some(BinaryOperator::LogicalAnd),
                        "||" => Some(BinaryOperator::LogicalOr),
                        _ => None,
                    };
                    if let Some(op) = op {
                        candidates.push((i as usize, op));
                    }
                }
                clang_disposeString(token_cxstring);
            }
        }
    }

    // Select the operator with lowest precedence (appears last in our search)
    // This handles cases like "a > 0 && b > 0" where && should be selected over >
    if !candidates.is_empty() {
        // Priority: || > && > comparisons > arithmetic
        // Find the first || operator
        for (_, op) in &candidates {
            if matches!(op, BinaryOperator::LogicalOr) {
                operator = Some(*op);
                break;
            }
        }
        // If no ||, find first &&
        if operator.is_none() {
            for (_, op) in &candidates {
                if matches!(op, BinaryOperator::LogicalAnd) {
                    operator = Some(*op);
                    break;
                }
            }
        }
        // If no logical operators, find operator with lowest precedence
        // Precedence (lowest to highest): comparisons (==, !=, <, >, <=, >=) > arithmetic (+, -) > multiplicative (*, /, %)
        if operator.is_none() {
            // Find first comparison operator (==, !=, <, >, <=, >=)
            for (_, op) in &candidates {
                if matches!(
                    op,
                    BinaryOperator::Equal
                        | BinaryOperator::NotEqual
                        | BinaryOperator::LessThan
                        | BinaryOperator::GreaterThan
                        | BinaryOperator::LessEqual
                        | BinaryOperator::GreaterEqual
                ) {
                    operator = Some(*op);
                    break;
                }
            }
        }
        // If no comparisons, find first additive operator (+, -)
        if operator.is_none() {
            for (_, op) in &candidates {
                if matches!(op, BinaryOperator::Add | BinaryOperator::Subtract) {
                    operator = Some(*op);
                    break;
                }
            }
        }
        // If no additive, take first multiplicative operator (*, /, %)
        if operator.is_none() {
            operator = Some(candidates[0].1);
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
    // We use a struct to track if we've seen the function reference yet
    #[repr(C)]
    struct ArgData {
        arguments: Vec<Expression>,
        skip_first_declref: bool,
    }

    let mut arg_data = ArgData {
        arguments: Vec::new(),
        skip_first_declref: true, // Skip the first DeclRefExpr (function name)
    };
    let args_ptr = &mut arg_data as *mut ArgData;

    unsafe {
        clang_visitChildren(cursor, visit_call_argument, args_ptr as CXClientData);
    }

    Some(Expression::FunctionCall {
        function,
        arguments: arg_data.arguments,
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
    #[repr(C)]
    struct ArgData {
        arguments: Vec<Expression>,
        skip_first_declref: bool,
    }

    // SAFETY: Converting client data back to ArgData pointer
    let arg_data = unsafe { &mut *(client_data as *mut ArgData) };

    // SAFETY: Getting cursor kind
    let kind = unsafe { clang_getCursorKind(cursor) };

    match kind {
        CXCursor_IntegerLiteral => {
            if let Some(expr) = extract_int_literal(cursor) {
                arg_data.arguments.push(expr);
            }
            CXChildVisit_Continue
        }
        CXCursor_StringLiteral => {
            if let Some(expr) = extract_string_literal(cursor) {
                arg_data.arguments.push(expr);
            }
            CXChildVisit_Continue
        }
        CXCursor_DeclRefExpr => {
            // Variable reference argument
            // The first DeclRefExpr is the function being called, skip it
            if arg_data.skip_first_declref {
                arg_data.skip_first_declref = false;
                CXChildVisit_Continue
            } else {
                if let Some(expr) = extract_variable_ref(cursor) {
                    arg_data.arguments.push(expr);
                }
                CXChildVisit_Continue
            }
        }
        CXCursor_BinaryOperator => {
            // Binary operation in argument (e.g., x + 1, y * 2)
            if let Some(expr) = extract_binary_op(cursor) {
                arg_data.arguments.push(expr);
            }
            CXChildVisit_Continue
        }
        CXCursor_CallExpr => {
            // Nested function call (e.g., add(add(x, 5), add(10, 20)))
            if let Some(expr) = extract_function_call(cursor) {
                arg_data.arguments.push(expr);
            }
            CXChildVisit_Continue
        }
        CXCursor_UnaryOperator => {
            // Unary operation in argument (e.g., -x, !flag)
            if let Some(expr) = extract_unary_op(cursor) {
                arg_data.arguments.push(expr);
            }
            CXChildVisit_Continue
        }
        CXCursor_ArraySubscriptExpr => {
            // Array indexing in argument (e.g., arr[i])
            if let Some(expr) = extract_array_index(cursor) {
                arg_data.arguments.push(expr);
            }
            CXChildVisit_Continue
        }
        CXCursor_MemberRefExpr => {
            // Field access in argument (e.g., ptr->field or obj.field)
            if let Some(expr) = extract_field_access(cursor) {
                arg_data.arguments.push(expr);
            }
            CXChildVisit_Continue
        }
        CXCursor_UnexposedExpr | CXCursor_ParenExpr => {
            // Unexposed expressions might wrap actual expressions or be sizeof, try to extract
            if let Some(expr) = extract_sizeof(cursor) {
                arg_data.arguments.push(expr);
                CXChildVisit_Continue
            } else {
                CXChildVisit_Recurse
            }
        }
        136 => {
            // CXCursor_UnaryExpr - includes sizeof, alignof, etc.
            if let Some(expr) = extract_sizeof(cursor) {
                arg_data.arguments.push(expr);
                CXChildVisit_Continue
            } else {
                CXChildVisit_Recurse
            }
        }
        _ => CXChildVisit_Continue, // Skip other unknown children
    }
}

/// Extract a unary operator expression.
fn extract_unary_op(cursor: CXCursor) -> Option<Expression> {
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

    let mut operator: Option<UnaryOperator> = None;
    let mut is_dereference = false;
    let mut is_increment = false;
    let mut is_decrement = false;
    let mut operator_position = 0;

    // Look through tokens to find the unary operator
    for i in 0..num_tokens {
        unsafe {
            let token = *tokens.add(i as usize);
            let token_kind = clang_getTokenKind(token);

            if token_kind == CXToken_Punctuation {
                let token_cxstring = clang_getTokenSpelling(tu, token);
                let c_str = CStr::from_ptr(clang_getCString(token_cxstring));
                if let Ok(token_str) = c_str.to_str() {
                    match token_str {
                        "*" => {
                            is_dereference = true;
                            clang_disposeString(token_cxstring);
                            break;
                        }
                        "-" => {
                            operator = Some(UnaryOperator::Minus);
                            clang_disposeString(token_cxstring);
                            break;
                        }
                        "!" => {
                            operator = Some(UnaryOperator::LogicalNot);
                            clang_disposeString(token_cxstring);
                            break;
                        }
                        "~" => {
                            operator = Some(UnaryOperator::BitwiseNot);
                            clang_disposeString(token_cxstring);
                            break;
                        }
                        "&" => {
                            operator = Some(UnaryOperator::AddressOf);
                            clang_disposeString(token_cxstring);
                            break;
                        }
                        "++" => {
                            is_increment = true;
                            operator_position = i;
                            clang_disposeString(token_cxstring);
                            break;
                        }
                        "--" => {
                            is_decrement = true;
                            operator_position = i;
                            clang_disposeString(token_cxstring);
                            break;
                        }
                        _ => {}
                    }
                }
                clang_disposeString(token_cxstring);
            }
        }
    }

    unsafe {
        clang_disposeTokens(tu, tokens, num_tokens);
    }

    // Extract the operand
    let mut operand: Option<Expression> = None;
    let operand_ptr = &mut operand as *mut Option<Expression>;

    unsafe {
        clang_visitChildren(cursor, visit_expression, operand_ptr as CXClientData);
    }

    let operand_expr = operand?;

    // Handle dereference separately (maintains backward compatibility)
    if is_dereference {
        return Some(Expression::Dereference(Box::new(operand_expr)));
    }

    // Handle increment/decrement operators
    if is_increment {
        // Check if pre or post increment
        let is_pre = operator_position == 0;
        if is_pre {
            return Some(Expression::PreIncrement {
                operand: Box::new(operand_expr),
            });
        } else {
            return Some(Expression::PostIncrement {
                operand: Box::new(operand_expr),
            });
        }
    }

    if is_decrement {
        // Check if pre or post decrement
        let is_pre = operator_position == 0;
        if is_pre {
            return Some(Expression::PreDecrement {
                operand: Box::new(operand_expr),
            });
        } else {
            return Some(Expression::PostDecrement {
                operand: Box::new(operand_expr),
            });
        }
    }

    // Handle other unary operators
    if let Some(op) = operator {
        return Some(Expression::UnaryOp {
            op,
            operand: Box::new(operand_expr),
        });
    }

    None
}

/// Extract an array indexing expression.
fn extract_array_index(cursor: CXCursor) -> Option<Expression> {
    // Extract array and index expressions by visiting children
    let mut operands: Vec<Expression> = Vec::new();
    let operands_ptr = &mut operands as *mut Vec<Expression>;

    unsafe {
        clang_visitChildren(cursor, visit_binary_operand, operands_ptr as CXClientData);
    }

    // Array subscript should have exactly 2 operands: array and index
    if operands.len() != 2 {
        return None;
    }

    Some(Expression::ArrayIndex {
        array: Box::new(operands[0].clone()),
        index: Box::new(operands[1].clone()),
    })
}

/// Extract a field access expression (obj.field or ptr->field).
fn extract_field_access(cursor: CXCursor) -> Option<Expression> {
    // Get the field name
    let field_name_cxstring = unsafe { clang_getCursorSpelling(cursor) };
    let field = unsafe {
        let c_str = CStr::from_ptr(clang_getCString(field_name_cxstring));
        let name = c_str.to_string_lossy().into_owned();
        clang_disposeString(field_name_cxstring);
        name
    };

    // Determine if this is -> or . by tokenizing
    let tu = unsafe { clang_Cursor_getTranslationUnit(cursor) };
    if tu.is_null() {
        return None;
    }

    let extent = unsafe { clang_getCursorExtent(cursor) };
    let mut tokens = ptr::null_mut();
    let mut num_tokens = 0;

    unsafe {
        clang_tokenize(tu, extent, &mut tokens, &mut num_tokens);
    }

    let mut is_arrow = false;

    // Look through tokens to find the LAST '->' or '.' operator
    // (the rightmost operator is the one for this specific MemberRefExpr)
    // For nested access like r->bottom_right.x, the extent includes all tokens,
    // so we need the last operator, not the first
    for i in 0..num_tokens {
        unsafe {
            let token = *tokens.add(i as usize);
            let token_kind = clang_getTokenKind(token);

            if token_kind == CXToken_Punctuation {
                let token_cxstring = clang_getTokenSpelling(tu, token);
                let c_str = CStr::from_ptr(clang_getCString(token_cxstring));
                if let Ok(token_str) = c_str.to_str() {
                    if token_str == "->" {
                        is_arrow = true;
                        // Don't break - keep looking for later operators
                    } else if token_str == "." {
                        is_arrow = false;
                        // Don't break - keep looking for later operators
                    }
                }
                clang_disposeString(token_cxstring);
            }
        }
    }

    unsafe {
        clang_disposeTokens(tu, tokens, num_tokens);
    }

    // Extract the object/pointer expression by visiting children
    let mut object_expr: Option<Expression> = None;
    let expr_ptr = &mut object_expr as *mut Option<Expression>;

    unsafe {
        clang_visitChildren(cursor, visit_expression, expr_ptr as CXClientData);
    }

    let object = object_expr?;

    if is_arrow {
        Some(Expression::PointerFieldAccess {
            pointer: Box::new(object),
            field,
        })
    } else {
        Some(Expression::FieldAccess {
            object: Box::new(object),
            field,
        })
    }
}

/// Extract a sizeof expression.
fn extract_sizeof(cursor: CXCursor) -> Option<Expression> {
    // Get the translation unit
    let tu = unsafe { clang_Cursor_getTranslationUnit(cursor) };
    if tu.is_null() {
        return None;
    }

    // Get the extent (source range) of the cursor
    let extent = unsafe { clang_getCursorExtent(cursor) };

    // Tokenize to find "sizeof" keyword
    let mut tokens = ptr::null_mut();
    let mut num_tokens = 0;

    unsafe {
        clang_tokenize(tu, extent, &mut tokens, &mut num_tokens);
    }

    let mut is_sizeof = false;
    let mut type_name = String::new();

    // Look through tokens to find "sizeof" keyword and extract type name
    for i in 0..num_tokens {
        unsafe {
            let token = *tokens.add(i as usize);
            let token_kind = clang_getTokenKind(token);
            let token_cxstring = clang_getTokenSpelling(tu, token);
            let c_str = CStr::from_ptr(clang_getCString(token_cxstring));

            if let Ok(token_str) = c_str.to_str() {
                if token_str == "sizeof" {
                    is_sizeof = true;
                } else if is_sizeof
                    && (token_kind == CXToken_Identifier || token_kind == CXToken_Keyword)
                    && token_str != "("
                    && token_str != ")"
                {
                    // This is part of the type name (e.g., "int", "Data", "struct")
                    if !type_name.is_empty() {
                        type_name.push(' ');
                    }
                    type_name.push_str(token_str);
                }
            }

            clang_disposeString(token_cxstring);
        }
    }

    unsafe {
        clang_disposeTokens(tu, tokens, num_tokens);
    }

    if is_sizeof && !type_name.is_empty() {
        Some(Expression::Sizeof { type_name })
    } else {
        None
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

            // Check if the pointee is a function - this is a function pointer
            if pointee.kind == CXType_FunctionProto || pointee.kind == CXType_FunctionNoProto {
                // This is a function pointer type
                // Extract return type
                let return_cx_type = unsafe { clang_getResultType(pointee) };
                let return_type = convert_type(return_cx_type)?;

                // Extract parameter types
                let num_args = unsafe { clang_getNumArgTypes(pointee) };
                let mut param_types = Vec::new();

                for i in 0..num_args {
                    let arg_type = unsafe { clang_getArgType(pointee, i as u32) };
                    if let Some(param_type) = convert_type(arg_type) {
                        param_types.push(param_type);
                    }
                }

                return Some(Type::FunctionPointer {
                    param_types,
                    return_type: Box::new(return_type),
                });
            }

            // Regular pointer (not function pointer)
            convert_type(pointee).map(|t| Type::Pointer(Box::new(t)))
        }
        CXType_FunctionProto | CXType_FunctionNoProto => {
            // Function type (not a pointer to function, but the function type itself)
            // This can occur in typedefs like: typedef int Func(int);
            // Extract return type
            let return_cx_type = unsafe { clang_getResultType(cx_type) };
            let return_type = convert_type(return_cx_type)?;

            // Extract parameter types
            let num_args = unsafe { clang_getNumArgTypes(cx_type) };
            let mut param_types = Vec::new();

            for i in 0..num_args {
                let arg_type = unsafe { clang_getArgType(cx_type, i as u32) };
                if let Some(param_type) = convert_type(arg_type) {
                    param_types.push(param_type);
                }
            }

            Some(Type::FunctionPointer {
                param_types,
                return_type: Box::new(return_type),
            })
        }
        CXType_Record => {
            // SAFETY: Getting type declaration to extract struct name
            let decl = unsafe { clang_getTypeDeclaration(cx_type) };
            let name_cxstring = unsafe { clang_getCursorSpelling(decl) };
            let name = unsafe {
                let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
                let struct_name = c_str.to_string_lossy().into_owned();
                clang_disposeString(name_cxstring);
                struct_name
            };
            Some(Type::Struct(name))
        }
        CXType_Elaborated => {
            // Elaborated types wrap other types (e.g., "struct Point" wraps the Record type)
            // Get the canonical type to unwrap it
            let canonical = unsafe { clang_getCanonicalType(cx_type) };
            convert_type(canonical)
        }
        CXType_Typedef => {
            // Typedef types wrap the actual underlying type
            // Get the canonical type to unwrap it
            let canonical = unsafe { clang_getCanonicalType(cx_type) };
            convert_type(canonical)
        }
        _ => None,
    }
}

/// Represents a single case in a switch statement.
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    /// Case value expression (None for default case)
    pub value: Option<Expression>,
    /// Statements to execute for this case
    pub body: Vec<Statement>,
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
    /// If statement: `if (cond) { ... } else { ... }`
    If {
        /// Condition expression
        condition: Expression,
        /// Then block
        then_block: Vec<Statement>,
        /// Optional else block
        else_block: Option<Vec<Statement>>,
    },
    /// For loop: `for (init; cond; inc) { ... }`
    For {
        /// Optional init statement
        init: Option<Box<Statement>>,
        /// Optional condition expression
        condition: Option<Expression>,
        /// Optional increment statement
        increment: Option<Box<Statement>>,
        /// Loop body
        body: Vec<Statement>,
    },
    /// While loop: `while (cond) { ... }`
    While {
        /// Condition expression
        condition: Expression,
        /// Loop body
        body: Vec<Statement>,
    },
    /// Pointer dereference assignment: `*ptr = value;`
    DerefAssignment {
        /// Target expression to dereference
        target: Expression,
        /// Value expression to assign
        value: Expression,
    },
    /// Array index assignment: `arr[i] = value;`
    ArrayIndexAssignment {
        /// Array expression
        array: Box<Expression>,
        /// Index expression
        index: Box<Expression>,
        /// Value expression to assign
        value: Expression,
    },
    /// Field assignment: `ptr->field = value;` or `obj.field = value;`
    FieldAssignment {
        /// Object/pointer expression
        object: Expression,
        /// Field name
        field: String,
        /// Value expression to assign
        value: Expression,
    },
    /// Break statement: `break;`
    Break,
    /// Continue statement: `continue;`
    Continue,
    /// Switch statement: `switch (expr) { case 1: ...; default: ...; }`
    Switch {
        /// Condition expression to switch on
        condition: Expression,
        /// List of case statements
        cases: Vec<SwitchCase>,
        /// Optional default case body
        default_case: Option<Vec<Statement>>,
    },
    /// Post-increment statement: `ptr++;`
    PostIncrement {
        /// Target variable name
        target: String,
    },
    /// Pre-increment statement: `++ptr;`
    PreIncrement {
        /// Target variable name
        target: String,
    },
    /// Post-decrement statement: `ptr--;`
    PostDecrement {
        /// Target variable name
        target: String,
    },
    /// Pre-decrement statement: `--ptr;`
    PreDecrement {
        /// Target variable name
        target: String,
    },
    /// Compound assignment: `ptr += offset;`, `x *= 2;`, etc.
    CompoundAssignment {
        /// Target variable name
        target: String,
        /// Binary operator to apply
        op: BinaryOperator,
        /// Value expression
        value: Expression,
    },
    /// Function call statement: `strlen(s);`, `strcpy(dst, src);`
    FunctionCall {
        /// Function name
        function: String,
        /// Arguments
        arguments: Vec<Expression>,
    },
}

impl Statement {
    /// Check if this statement is a string function call.
    pub fn is_string_function_call(&self) -> bool {
        match self {
            Statement::FunctionCall { function, .. } => {
                matches!(function.as_str(), "strlen" | "strcmp" | "strcpy" | "strdup")
            }
            _ => false,
        }
    }

    /// Check if this statement is a function call.
    pub fn is_function_call(&self) -> bool {
        matches!(self, Statement::FunctionCall { .. })
    }

    /// Convert this statement to a function call expression if it is one.
    ///
    /// # Implementation Status
    ///
    /// Stub implementation - always returns `None`.
    /// The `Statement::FunctionCall` variant doesn't store the call as an `Expression`,
    /// so conversion would require reconstructing an `Expression::FunctionCall` from
    /// the statement's fields.
    pub fn as_function_call(&self) -> Option<&Expression> {
        None
    }
}

/// Unary operators for C expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    /// Unary minus (-x)
    Minus,
    /// Logical NOT (!x)
    LogicalNot,
    /// Bitwise NOT (~x)
    BitwiseNot,
    /// Address-of (&x)
    AddressOf,
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
    /// Logical AND (&&)
    LogicalAnd,
    /// Logical OR (||)
    LogicalOr,
}

/// Represents a C expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Integer literal: `42`
    IntLiteral(i32),
    /// String literal: `"hello"`
    StringLiteral(String),
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
    /// Pointer dereference: `*ptr`
    Dereference(Box<Expression>),
    /// Unary operation: `-x`, `!x`
    UnaryOp {
        /// Operator
        op: UnaryOperator,
        /// Operand
        operand: Box<Expression>,
    },
    /// Array indexing: `arr[i]`
    ArrayIndex {
        /// Array expression
        array: Box<Expression>,
        /// Index expression
        index: Box<Expression>,
    },
    /// Struct field access: `obj.field`
    FieldAccess {
        /// Object expression
        object: Box<Expression>,
        /// Field name
        field: String,
    },
    /// Pointer field access: `ptr->field`
    PointerFieldAccess {
        /// Pointer expression
        pointer: Box<Expression>,
        /// Field name
        field: String,
    },
    /// Post-increment expression: `ptr++`
    PostIncrement {
        /// Operand expression
        operand: Box<Expression>,
    },
    /// Pre-increment expression: `++ptr`
    PreIncrement {
        /// Operand expression
        operand: Box<Expression>,
    },
    /// Post-decrement expression: `ptr--`
    PostDecrement {
        /// Operand expression
        operand: Box<Expression>,
    },
    /// Pre-decrement expression: `--ptr`
    PreDecrement {
        /// Operand expression
        operand: Box<Expression>,
    },
    /// Sizeof expression: `sizeof(int)` or `sizeof(struct Data)`
    Sizeof {
        /// Type name as a string (e.g., "int", "struct Data")
        type_name: String,
    },
}

impl Expression {
    /// Check if this expression is a string function call (strlen, strcmp, strcpy, strdup).
    pub fn is_string_function_call(&self) -> bool {
        match self {
            Expression::FunctionCall { function, .. } => {
                matches!(function.as_str(), "strlen" | "strcmp" | "strcpy" | "strdup")
            }
            _ => false,
        }
    }

    /// Get the string function name if this is a string function call.
    pub fn string_function_name(&self) -> Option<&str> {
        match self {
            Expression::FunctionCall { function, .. } if self.is_string_function_call() => {
                Some(function.as_str())
            }
            _ => None,
        }
    }

    /// Check if this expression has a string literal argument.
    pub fn has_string_literal_argument(&self) -> bool {
        match self {
            Expression::FunctionCall { arguments, .. } => arguments
                .iter()
                .any(|arg| matches!(arg, Expression::StringLiteral(_))),
            _ => false,
        }
    }
}

/// Represents a C typedef declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct Typedef {
    /// Typedef name (the alias)
    pub name: String,
    /// Underlying type being aliased
    pub underlying_type: Type,
}

impl Typedef {
    /// Create a new typedef.
    pub fn new(name: String, underlying_type: Type) -> Self {
        Self {
            name,
            underlying_type,
        }
    }

    /// Get the typedef name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the underlying type as a string representation.
    pub fn underlying_type(&self) -> &str {
        // Return a string representation of the type
        match &self.underlying_type {
            Type::Void => "void",
            Type::Int => "int",
            Type::Float => "float",
            Type::Double => "double",
            Type::Char => "char",
            Type::Pointer(inner) => match **inner {
                Type::Char => "char*",
                Type::Int => "int*",
                Type::Float => "float*",
                Type::Double => "double*",
                Type::Void => "void*",
                _ => "pointer",
            },
            Type::Struct(name) => name,
            Type::FunctionPointer { .. } => "function pointer",
        }
    }

    /// Check if this typedef is a pointer type.
    pub fn is_pointer(&self) -> bool {
        matches!(self.underlying_type, Type::Pointer(_))
    }

    /// Check if this typedef is a struct type.
    pub fn is_struct(&self) -> bool {
        matches!(self.underlying_type, Type::Struct(_))
    }

    /// Check if this typedef is a function pointer type.
    pub fn is_function_pointer(&self) -> bool {
        matches!(self.underlying_type, Type::FunctionPointer { .. })
    }

    /// Check if this typedef is an array type.
    pub fn is_array(&self) -> bool {
        // Arrays are not yet in the Type enum, so return false for now
        false
    }
}

/// Represents a struct field.
#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    /// Field name
    pub name: String,
    /// Field type
    pub field_type: Type,
}

impl StructField {
    /// Create a new struct field.
    pub fn new(name: String, field_type: Type) -> Self {
        Self { name, field_type }
    }

    /// Get the field name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Check if this field is a function pointer.
    pub fn is_function_pointer(&self) -> bool {
        matches!(self.field_type, Type::FunctionPointer { .. })
    }
}

/// Represents a struct definition.
#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    /// Struct name
    pub name: String,
    /// Struct fields
    pub fields: Vec<StructField>,
}

impl Struct {
    /// Create a new struct.
    pub fn new(name: String, fields: Vec<StructField>) -> Self {
        Self { name, fields }
    }

    /// Get the struct name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the struct fields.
    pub fn fields(&self) -> &[StructField] {
        &self.fields
    }
}

/// Represents a variable declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    /// Variable name
    name: String,
    /// Variable type
    var_type: Type,
    /// Optional initializer expression
    initializer: Option<Expression>,
}

impl Variable {
    /// Create a new variable.
    pub fn new(name: String, var_type: Type) -> Self {
        Self {
            name,
            var_type,
            initializer: None,
        }
    }

    /// Create a new variable with an initializer.
    pub fn new_with_initializer(name: String, var_type: Type, initializer: Expression) -> Self {
        Self {
            name,
            var_type,
            initializer: Some(initializer),
        }
    }

    /// Get the variable name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the variable type.
    pub fn var_type(&self) -> &Type {
        &self.var_type
    }

    /// Check if this variable is a function pointer.
    pub fn is_function_pointer(&self) -> bool {
        matches!(self.var_type, Type::FunctionPointer { .. })
    }

    /// Get the number of parameters if this is a function pointer.
    pub fn function_pointer_param_count(&self) -> usize {
        match &self.var_type {
            Type::FunctionPointer { param_types, .. } => param_types.len(),
            _ => 0,
        }
    }

    /// Check if this function pointer has a void return type.
    pub fn function_pointer_has_void_return(&self) -> bool {
        match &self.var_type {
            Type::FunctionPointer { return_type, .. } => matches!(**return_type, Type::Void),
            _ => false,
        }
    }

    /// Check if this variable is a string literal (char* with literal initializer).
    ///
    /// Detects patterns like: `const char* msg = "Hello";`
    ///
    /// # Implementation
    ///
    /// Checks if:
    /// - Type is a pointer to char (`char*`)
    /// - Has an initializer that is a `StringLiteral` expression
    ///
    /// Note: Const qualifier detection not yet implemented - checks all char* pointers.
    pub fn is_string_literal(&self) -> bool {
        // Check if type is char*
        let is_char_ptr =
            matches!(self.var_type, Type::Pointer(ref inner) if **inner == Type::Char);

        // Check if initializer is a string literal
        if let Some(initializer) = &self.initializer {
            is_char_ptr && matches!(initializer, Expression::StringLiteral(_))
        } else {
            false
        }
    }

    /// Check if this variable is a string buffer (char* allocated with malloc).
    ///
    /// Detects patterns like: `char* buffer = malloc(100);`
    ///
    /// # Implementation
    ///
    /// Checks if:
    /// - Type is a pointer to char (`char*`)
    /// - Has an initializer that is a malloc/calloc function call
    pub fn is_string_buffer(&self) -> bool {
        // Check if type is char*
        let is_char_ptr =
            matches!(self.var_type, Type::Pointer(ref inner) if **inner == Type::Char);

        // Check if initializer is malloc/calloc call
        if let Some(Expression::FunctionCall { function, .. }) = &self.initializer {
            is_char_ptr && (function == "malloc" || function == "calloc")
        } else {
            false
        }
    }

    /// Get the initializer expression for this variable.
    ///
    /// Returns `Some(&Expression)` if the variable has an initializer, `None` otherwise.
    pub fn initializer(&self) -> Option<&Expression> {
        self.initializer.as_ref()
    }
}

/// Abstract Syntax Tree representing parsed C code.
#[derive(Debug, Clone, PartialEq)]
pub struct Ast {
    functions: Vec<Function>,
    typedefs: Vec<Typedef>,
    structs: Vec<Struct>,
    macros: Vec<MacroDefinition>,
    variables: Vec<Variable>,
}

/// Represents a C macro definition (#define).
///
/// C macros come in two forms:
/// - **Object-like**: Simple text replacement (e.g., `#define MAX 100`)
/// - **Function-like**: Parameterized text replacement (e.g., `#define SQR(x) ((x) * (x))`)
///
/// # Examples
///
/// ```no_run
/// use decy_parser::parser::{CParser, MacroDefinition};
///
/// // Parse a simple object-like macro
/// let parser = CParser::new()?;
/// let ast = parser.parse("#define MAX 100\nint main() { return 0; }")?;
/// assert_eq!(ast.macros().len(), 1);
/// assert_eq!(ast.macros()[0].name(), "MAX");
/// assert!(ast.macros()[0].is_object_like());
///
/// // Parse a function-like macro
/// let ast2 = parser.parse("#define SQR(x) ((x) * (x))\nint main() { return 0; }")?;
/// assert_eq!(ast2.macros()[0].name(), "SQR");
/// assert!(ast2.macros()[0].is_function_like());
/// assert_eq!(ast2.macros()[0].parameters(), &["x"]);
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Reference
///
/// K&R §4.11, ISO C99 §6.10.3
#[derive(Debug, Clone, PartialEq)]
pub struct MacroDefinition {
    /// Macro name
    pub name: String,
    /// Parameters (empty for object-like macros)
    pub parameters: Vec<String>,
    /// Macro body (unparsed, tokenized without spaces)
    pub body: String,
}

impl MacroDefinition {
    /// Create a new object-like macro.
    pub fn new_object_like(name: String, body: String) -> Self {
        Self {
            name,
            parameters: vec![],
            body,
        }
    }

    /// Create a new function-like macro.
    pub fn new_function_like(name: String, parameters: Vec<String>, body: String) -> Self {
        Self {
            name,
            parameters,
            body,
        }
    }

    /// Get the macro name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the macro parameters.
    pub fn parameters(&self) -> &[String] {
        &self.parameters
    }

    /// Get the macro body.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Check if this is a function-like macro.
    pub fn is_function_like(&self) -> bool {
        !self.parameters.is_empty()
    }

    /// Check if this is an object-like macro.
    pub fn is_object_like(&self) -> bool {
        self.parameters.is_empty()
    }
}

impl Ast {
    /// Create a new empty AST.
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            typedefs: Vec::new(),
            structs: Vec::new(),
            macros: Vec::new(),
            variables: Vec::new(),
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

    /// Get the typedefs in the AST.
    pub fn typedefs(&self) -> &[Typedef] {
        &self.typedefs
    }

    /// Add a typedef to the AST.
    pub fn add_typedef(&mut self, typedef: Typedef) {
        self.typedefs.push(typedef);
    }

    /// Get the structs in the AST.
    pub fn structs(&self) -> &[Struct] {
        &self.structs
    }

    /// Add a struct to the AST.
    pub fn add_struct(&mut self, struct_def: Struct) {
        self.structs.push(struct_def);
    }

    /// Get the macro definitions in the AST.
    pub fn macros(&self) -> &[MacroDefinition] {
        &self.macros
    }

    /// Add a macro definition to the AST.
    pub fn add_macro(&mut self, macro_def: MacroDefinition) {
        self.macros.push(macro_def);
    }

    /// Get the variables in the AST.
    pub fn variables(&self) -> &[Variable] {
        &self.variables
    }

    /// Add a variable to the AST.
    pub fn add_variable(&mut self, variable: Variable) {
        self.variables.push(variable);
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
    /// Struct type (e.g., struct Point)
    Struct(String),
    /// Function pointer type (e.g., int (*callback)(int))
    FunctionPointer {
        /// Parameter types
        param_types: Vec<Type>,
        /// Return type
        return_type: Box<Type>,
    },
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

    /// Check if this parameter is a function pointer.
    pub fn is_function_pointer(&self) -> bool {
        matches!(self.param_type, Type::FunctionPointer { .. })
    }

    /// Check if this parameter is a const char pointer (const char*).
    ///
    /// # Implementation Status
    ///
    /// Partial implementation - detects `char*` pointers but doesn't check const qualifier.
    /// Returns `true` for any `Pointer(Char)` type.
    /// Full implementation requires adding const tracking to the `Type` enum.
    pub fn is_const_char_pointer(&self) -> bool {
        matches!(self.param_type, Type::Pointer(ref inner) if matches!(**inner, Type::Char))
    }
}

#[cfg(test)]
#[path = "parser_tests.rs"]
mod parser_tests;

#[cfg(test)]
#[path = "pointer_arithmetic_tests.rs"]
mod pointer_arithmetic_tests;

#[cfg(test)]
#[path = "break_continue_tests.rs"]
mod break_continue_tests;
