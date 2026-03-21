//! Clang AST extraction functions.
//!
//! This module contains the functions that traverse clang cursors and extract
//! C language constructs (functions, statements, expressions, types) into
//! our AST representation.

// Allow non-upper-case globals from clang-sys FFI bindings (CXCursor_* constants)
#![allow(non_upper_case_globals)]

use crate::ast_types::*;
use crate::diagnostic::{Diagnostic, Severity};
use clang_sys::*;
use std::ffi::CStr;
use std::ptr;

pub(crate) fn extract_diagnostics(
    tu: CXTranslationUnit,
    source: &str,
    file_override: Option<&str>,
) -> Vec<Diagnostic> {
    let num_diagnostics = unsafe { clang_getNumDiagnostics(tu) };
    let mut diagnostics = Vec::new();

    for i in 0..num_diagnostics {
        let diag = unsafe { clang_getDiagnostic(tu, i) };
        let raw_severity = unsafe { clang_getDiagnosticSeverity(diag) };

        let severity = match raw_severity {
            CXDiagnostic_Note => Severity::Note,
            CXDiagnostic_Warning => Severity::Warning,
            CXDiagnostic_Error => Severity::Error,
            CXDiagnostic_Fatal => Severity::Fatal,
            _ => {
                unsafe { clang_disposeDiagnostic(diag) };
                continue; // Skip ignored diagnostics
            }
        };

        // Extract message text
        let diag_str = unsafe { clang_getDiagnosticSpelling(diag) };
        let c_str = unsafe { CStr::from_ptr(clang_getCString(diag_str)) };
        let message = c_str.to_str().unwrap_or("unknown error").to_string();
        unsafe { clang_disposeString(diag_str) };

        let mut d = Diagnostic::new(severity, message);

        // Extract source location (line, column, file)
        let loc = unsafe { clang_getDiagnosticLocation(diag) };
        let mut file: CXFile = ptr::null_mut();
        let mut line: u32 = 0;
        let mut column: u32 = 0;
        unsafe {
            clang_getFileLocation(loc, &mut file, &mut line, &mut column, ptr::null_mut());
        }

        if line > 0 {
            d.line = Some(line);
            d.column = Some(column);
        }

        // Use file override if provided, otherwise extract from clang
        if let Some(name) = file_override {
            d.file = Some(name.to_string());
        } else if !file.is_null() {
            let file_name = unsafe {
                let name_cx = clang_getFileName(file);
                let name_c = CStr::from_ptr(clang_getCString(name_cx));
                let name = name_c.to_string_lossy().into_owned();
                clang_disposeString(name_cx);
                name
            };
            d.file = Some(file_name);
        } else {
            d.file = Some("input.c".to_string());
        }

        // Extract category text
        let cat_idx = unsafe { clang_getDiagnosticCategory(diag) };
        if cat_idx != 0 {
            let cat_str = unsafe { clang_getDiagnosticCategoryText(diag) };
            let cat_c = unsafe { CStr::from_ptr(clang_getCString(cat_str)) };
            let category = cat_c.to_str().unwrap_or("").to_string();
            unsafe { clang_disposeString(cat_str) };
            if !category.is_empty() {
                d.category = Some(category);
            }
        }

        // Extract fix-it suggestions
        let num_fix_its = unsafe { clang_getDiagnosticNumFixIts(diag) };
        for fi in 0..num_fix_its {
            let mut range = unsafe { std::mem::zeroed::<CXSourceRange>() };
            let fix_str = unsafe { clang_getDiagnosticFixIt(diag, fi, &mut range) };
            let fix_c = unsafe { CStr::from_ptr(clang_getCString(fix_str)) };
            let fix_text = fix_c.to_str().unwrap_or("").to_string();
            unsafe { clang_disposeString(fix_str) };
            if !fix_text.is_empty() {
                d.fix_its.push(format!("insert '{}'", fix_text));
            }
        }

        // Build code snippet from source
        if let Some(line_num) = d.line {
            d.snippet = Diagnostic::build_snippet(source, line_num, d.column);
        }

        // Infer note and help from message patterns
        d.infer_note_and_help();

        unsafe { clang_disposeDiagnostic(diag) };
        diagnostics.push(d);
    }

    diagnostics
}

/// Visitor callback for clang AST traversal.
///
/// # Safety
///
/// This function is called by clang_visitChildren and must follow C calling conventions.
pub(crate) extern "C" fn visit_function(
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
        // DECY-147: Handle anonymous struct typedefs
        let (typedef_opt, struct_opt) = extract_typedef(cursor);
        if let Some(typedef) = typedef_opt {
            ast.add_typedef(typedef);
        }
        if let Some(struct_def) = struct_opt {
            ast.add_struct(struct_def);
        }
    } else if kind == CXCursor_StructDecl {
        // Extract struct information
        if let Some(struct_def) = extract_struct(cursor) {
            ast.add_struct(struct_def);
        }
    } else if kind == CXCursor_VarDecl {
        // Extract variable declaration - only add if it's at file scope (global)
        // Check if parent is translation unit (file scope) vs function scope
        let semantic_parent = unsafe { clang_getCursorSemanticParent(cursor) };
        let parent_kind = unsafe { clang_getCursorKind(semantic_parent) };

        // Check if parent is file scope: either TranslationUnit or nullptr
        // Function declarations have parent kind = CXCursor_FunctionDecl (8)
        // File-scope variables typically have parent kind = CXCursor_TranslationUnit (300 in clang-sys)
        let is_file_scope = parent_kind != CXCursor_FunctionDecl;

        if is_file_scope {
            if let Some(variable) = extract_variable(cursor) {
                ast.add_variable(variable);
            }
        }
        // Local variables in functions are handled by extract_statement in function body parsing
    } else if kind == 5 {
        // CXCursor_EnumDecl = 5
        // DECY-240: Extract enum declaration
        if let Some(enum_def) = extract_enum(cursor) {
            ast.add_enum(enum_def);
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

    // Return Recurse to ensure we visit children of all nodes
    // This is needed in C++ mode to reach LinkageSpec and its children
    CXChildVisit_Recurse
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
            // DECY-135: Check if this is a pointer with const-qualified pointee
            let is_pointee_const = unsafe {
                if param_cx_type.kind == clang_sys::CXType_Pointer {
                    let pointee = clang_sys::clang_getPointeeType(param_cx_type);
                    clang_isConstQualifiedType(pointee) != 0
                } else {
                    false
                }
            };
            parameters.push(Parameter::new_with_const(param_name, param_type, is_pointee_const));
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
/// Returns (Option<Typedef>, Option<Struct>) - struct is Some when typedef is for anonymous struct.
fn extract_typedef(cursor: CXCursor) -> (Option<Typedef>, Option<Struct>) {
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

    // DECY-147: Check if underlying type is anonymous struct
    // Anonymous struct pattern: typedef struct { ... } Name;
    let canonical = unsafe { clang_getCanonicalType(cx_type) };
    if canonical.kind == CXType_Record {
        let decl = unsafe { clang_getTypeDeclaration(canonical) };
        let struct_name_cxstring = unsafe { clang_getCursorSpelling(decl) };
        let struct_name = unsafe {
            let c_str = CStr::from_ptr(clang_getCString(struct_name_cxstring));
            let sn = c_str.to_string_lossy().into_owned();
            clang_disposeString(struct_name_cxstring);
            sn
        };

        // If struct name is empty, this is an anonymous struct typedef
        if struct_name.is_empty() {
            // Extract struct fields from the declaration
            let mut fields = Vec::new();
            let fields_ptr = &mut fields as *mut Vec<StructField>;

            unsafe {
                clang_visitChildren(decl, visit_struct_fields, fields_ptr as CXClientData);
            }

            // Return struct with typedef name, no typedef needed
            return (None, Some(Struct::new(name, fields)));
        }
    }

    let underlying_type = convert_type(cx_type);
    match underlying_type {
        Some(ut) => (Some(Typedef::new(name, ut)), None),
        None => (None, None),
    }
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

/// DECY-240: Extract enum information from a clang cursor.
///
/// Extracts C enum declarations, including explicit values.
fn extract_enum(cursor: CXCursor) -> Option<Enum> {
    // SAFETY: Getting enum name
    let name_cxstring = unsafe { clang_getCursorSpelling(cursor) };
    let name = unsafe {
        let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
        let name = c_str.to_string_lossy().into_owned();
        clang_disposeString(name_cxstring);
        name
    };

    // Extract enum variants by visiting children
    let mut variants: Vec<EnumVariant> = Vec::new();
    let variants_ptr = &mut variants as *mut Vec<EnumVariant>;

    // Visitor callback for enum constants
    extern "C" fn visit_enum_constants(
        cursor: CXCursor,
        _parent: CXCursor,
        client_data: CXClientData,
    ) -> CXChildVisitResult {
        let variants = unsafe { &mut *(client_data as *mut Vec<EnumVariant>) };

        // SAFETY: Getting cursor kind
        let kind = unsafe { clang_getCursorKind(cursor) };

        // CXCursor_EnumConstantDecl = 7
        if kind == 7 {
            // Get variant name
            let name_cxstring = unsafe { clang_getCursorSpelling(cursor) };
            let variant_name = unsafe {
                let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
                let name = c_str.to_string_lossy().into_owned();
                clang_disposeString(name_cxstring);
                name
            };

            // Get variant value
            let value = unsafe { clang_getEnumConstantDeclValue(cursor) };

            variants.push(EnumVariant::new(variant_name, Some(value)));
        }

        CXChildVisit_Continue
    }

    unsafe {
        clang_visitChildren(cursor, visit_enum_constants, variants_ptr as CXClientData);
    }

    // Only return if there are variants (skip empty enums)
    if variants.is_empty() {
        return None;
    }

    Some(Enum::new(name, variants))
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

    // Extract storage class specifiers
    // CX_StorageClass values (from clang-sys):
    // CX_SC_Invalid = 0, CX_SC_None = 1, CX_SC_Extern = 2, CX_SC_Static = 3,
    // CX_SC_PrivateExtern = 4, CX_SC_OpenCLWorkGroupLocal = 5,
    // CX_SC_Auto = 6, CX_SC_Register = 7
    let storage_class = unsafe { clang_Cursor_getStorageClass(cursor) };
    let is_static = storage_class == 3; // CX_SC_Static
    let is_extern = storage_class == 2; // CX_SC_Extern

    // Check if type is const-qualified
    let is_const = unsafe { clang_isConstQualifiedType(cx_type) != 0 };

    // Extract initializer by visiting children
    let mut initializer: Option<Expression> = None;
    let initializer_ptr = &mut initializer as *mut Option<Expression>;

    unsafe {
        clang_visitChildren(cursor, visit_variable_initializer, initializer_ptr as CXClientData);
    }

    Some(Variable::new_with_storage_class(
        name,
        var_type,
        initializer,
        is_static,
        is_extern,
        is_const,
    ))
}

/// Helper function to extract an expression from a cursor.
/// Dispatches to the appropriate extract function based on cursor kind.
#[allow(non_upper_case_globals)]
fn try_extract_expression(cursor: CXCursor) -> Option<Expression> {
    let kind = unsafe { clang_getCursorKind(cursor) };

    match kind {
        CXCursor_IntegerLiteral => extract_int_literal(cursor),
        107 => extract_float_literal(cursor), // CXCursor_FloatingLiteral
        CXCursor_StringLiteral => extract_string_literal(cursor),
        110 => extract_char_literal(cursor), // CXCursor_CharacterLiteral
        CXCursor_DeclRefExpr => extract_variable_ref(cursor),
        CXCursor_BinaryOperator => extract_binary_op(cursor),
        CXCursor_CallExpr => extract_function_call(cursor),
        CXCursor_UnaryOperator => extract_unary_op(cursor),
        CXCursor_ArraySubscriptExpr => extract_array_index(cursor),
        CXCursor_MemberRefExpr => extract_field_access(cursor),
        116 => extract_conditional_op(cursor), // CXCursor_ConditionalOperator (ternary)
        117 => extract_cast(cursor),           // CXCursor_CStyleCastExpr
        118 => extract_compound_literal(cursor), // CXCursor_CompoundLiteralExpr
        111 => {
            // CXCursor_ParenExpr - parenthesized expression like (a > b)
            // Recurse into children to extract the inner expression
            let mut result: Option<Expression> = None;
            let result_ptr = &mut result as *mut Option<Expression>;
            unsafe {
                clang_visitChildren(cursor, visit_variable_initializer, result_ptr as CXClientData);
            }
            result
        }
        CXCursor_UnexposedExpr => {
            // UnexposedExpr is a wrapper - recurse into children
            let mut result: Option<Expression> = None;
            let result_ptr = &mut result as *mut Option<Expression>;
            unsafe {
                clang_visitChildren(cursor, visit_variable_initializer, result_ptr as CXClientData);
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
        CXCursor_CallExpr => {
            // Function call as statement (DECY-066)
            // e.g., printf("Hello"); or free(ptr);
            if let Some(stmt) = extract_statement(cursor) {
                statements.push(stmt);
            }
            CXChildVisit_Continue
        }
        _ => CXChildVisit_Recurse, // Recurse into unknown nodes to find statements
    }
}

/// Extract a variable declaration statement.
fn extract_var_decl(cursor: CXCursor) -> Option<Statement> {
    // DECY-223: Check storage class - skip extern declarations without initializers
    // These are references to globals, not new local variables
    // CX_SC_Extern = 2
    let storage_class = unsafe { clang_Cursor_getStorageClass(cursor) };
    let is_extern = storage_class == 2;

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

    // DECY-223: Early check for extern without initializer - check before visiting children
    // extern int max; → skip (reference to global)
    if is_extern {
        // We need to check if there's an initializer - visit children first
        let mut has_real_initializer = false;
        extern "C" fn check_initializer(
            cursor: CXCursor,
            _parent: CXCursor,
            client_data: CXClientData,
        ) -> CXChildVisitResult {
            let has_init = unsafe { &mut *(client_data as *mut bool) };
            let kind = unsafe { clang_getCursorKind(cursor) };
            // Check for expression kinds that indicate a real initializer
            if kind == CXCursor_IntegerLiteral
                || kind == 107 // CXCursor_FloatingLiteral
                || kind == CXCursor_StringLiteral
                || kind == CXCursor_CallExpr
                || kind == CXCursor_BinaryOperator
                || kind == CXCursor_UnaryOperator
            {
                *has_init = true;
                return CXChildVisit_Break;
            }
            CXChildVisit_Continue
        }
        let init_ptr = &mut has_real_initializer as *mut bool;
        unsafe {
            clang_visitChildren(cursor, check_initializer, init_ptr as CXClientData);
        }
        // If extern without initializer, skip it
        if !has_real_initializer {
            return None;
        }
    }

    // Extract initializer by visiting children
    let mut initializer: Option<Expression> = None;
    let init_ptr = &mut initializer as *mut Option<Expression>;

    unsafe {
        clang_visitChildren(cursor, visit_expression, init_ptr as CXClientData);
    }

    // DECY-112 FIX: For array types, the visit_expression callback may incorrectly
    // capture the array size as an initializer. For example, `int nums[5];` has
    // the integer literal 5 as a child node (the array size), which gets captured.
    //
    // Fix: If the variable is an array type and the initializer is an integer literal
    // that matches the array size, clear the initializer (it's the size, not an init).
    let initializer = match (&var_type, &initializer) {
        (Type::Array { size: Some(array_size), .. }, Some(Expression::IntLiteral(init_val)))
            if i64::from(*init_val) == *array_size =>
        {
            // The "initializer" is actually the array size expression, not a real initializer
            None
        }
        _ => initializer,
    };

    Some(Statement::VariableDeclaration { name, var_type, initializer })
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

        return Some(Statement::FieldAssignment { object, field, value: operands[1].clone() });
    }

    // Left side must be a variable reference for regular assignment
    let target = match &operands[0] {
        Expression::Variable(name) => name.clone(),
        _ => return None, // Can't assign to non-variables (yet)
    };

    Some(Statement::Assignment { target, value: operands[1].clone() })
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

    // DECY-166: First check if this is a member expression increment (e.g., sb->length++)
    // We need to detect this case and create a FieldAssignment instead of PostIncrement
    let mut member_expr: Option<Expression> = None;
    let mut simple_var: Option<String> = None;

    // Visit children to find MemberRefExpr, ArraySubscriptExpr, or DeclRefExpr
    extern "C" fn visit_for_inc_target(
        cursor: CXCursor,
        _parent: CXCursor,
        client_data: CXClientData,
    ) -> CXChildVisitResult {
        let data = unsafe { &mut *(client_data as *mut (Option<Expression>, Option<String>)) };
        let kind = unsafe { clang_getCursorKind(cursor) };

        // DECY-219: Check for array subscript expression first (arr[i]++, ndigit[c-'0']++)
        // Must be checked before recursing, otherwise we only find the DeclRefExpr
        if kind == CXCursor_ArraySubscriptExpr {
            if let Some(expr) = extract_array_index(cursor) {
                data.0 = Some(expr);
                return CXChildVisit_Break;
            }
        }

        // Check for member expression (sb->length, obj.field)
        if kind == CXCursor_MemberRefExpr {
            if let Some(expr) = extract_field_access(cursor) {
                data.0 = Some(expr);
                return CXChildVisit_Break;
            }
        }

        // Fall back to simple variable reference
        if kind == CXCursor_DeclRefExpr {
            let name_cxstring = unsafe { clang_getCursorSpelling(cursor) };
            let name = unsafe {
                let c_str = CStr::from_ptr(clang_getCString(name_cxstring));
                let var_name = c_str.to_string_lossy().into_owned();
                clang_disposeString(name_cxstring);
                var_name
            };
            data.1 = Some(name);
            CXChildVisit_Break
        } else {
            CXChildVisit_Recurse
        }
    }

    let mut target_data = (member_expr, simple_var);
    let target_ptr = &mut target_data as *mut (Option<Expression>, Option<String>);
    unsafe {
        clang_visitChildren(cursor, visit_for_inc_target, target_ptr as CXClientData);
    }
    member_expr = target_data.0;
    simple_var = target_data.1;

    let operator = operator?;
    let op_str = operator.as_str();

    // DECY-166: If we found a member expression, create a FieldAssignment
    // sb->length++ becomes FieldAssignment { object: sb, field: "length", value: sb->length + 1 }
    if let Some(expr) = member_expr {
        // Determine the delta (+1 or -1) based on operator
        let delta = match op_str {
            "++" => 1,
            "--" => -1,
            _ => return None,
        };

        // Extract object and field from the expression
        match expr {
            Expression::PointerFieldAccess { pointer, field } => {
                // Create the increment/decrement value expression
                let value = if delta > 0 {
                    Expression::BinaryOp {
                        left: Box::new(Expression::PointerFieldAccess {
                            pointer: pointer.clone(),
                            field: field.clone(),
                        }),
                        op: BinaryOperator::Add,
                        right: Box::new(Expression::IntLiteral(1)),
                    }
                } else {
                    Expression::BinaryOp {
                        left: Box::new(Expression::PointerFieldAccess {
                            pointer: pointer.clone(),
                            field: field.clone(),
                        }),
                        op: BinaryOperator::Subtract,
                        right: Box::new(Expression::IntLiteral(1)),
                    }
                };

                return Some(Statement::FieldAssignment { object: *pointer, field, value });
            }
            Expression::FieldAccess { object, field } => {
                // Create the increment/decrement value expression
                let value = if delta > 0 {
                    Expression::BinaryOp {
                        left: Box::new(Expression::FieldAccess {
                            object: object.clone(),
                            field: field.clone(),
                        }),
                        op: BinaryOperator::Add,
                        right: Box::new(Expression::IntLiteral(1)),
                    }
                } else {
                    Expression::BinaryOp {
                        left: Box::new(Expression::FieldAccess {
                            object: object.clone(),
                            field: field.clone(),
                        }),
                        op: BinaryOperator::Subtract,
                        right: Box::new(Expression::IntLiteral(1)),
                    }
                };

                return Some(Statement::FieldAssignment { object: *object, field, value });
            }
            // DECY-219: Array subscript increment/decrement: arr[i]++ → arr[i] = arr[i] + 1
            Expression::ArrayIndex { array, index } => {
                // Create the increment/decrement value expression
                let value = if delta > 0 {
                    Expression::BinaryOp {
                        left: Box::new(Expression::ArrayIndex {
                            array: array.clone(),
                            index: index.clone(),
                        }),
                        op: BinaryOperator::Add,
                        right: Box::new(Expression::IntLiteral(1)),
                    }
                } else {
                    Expression::BinaryOp {
                        left: Box::new(Expression::ArrayIndex {
                            array: array.clone(),
                            index: index.clone(),
                        }),
                        op: BinaryOperator::Subtract,
                        right: Box::new(Expression::IntLiteral(1)),
                    }
                };

                return Some(Statement::ArrayIndexAssignment { array, index, value });
            }
            _ => {} // Fall through to simple variable handling
        }
    }

    // Simple variable increment/decrement
    let target = simple_var?;

    match op_str {
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

    // DECY-185: Check for complex targets first (Dereference, PointerFieldAccess, FieldAccess)
    // These need DerefCompoundAssignment since target is an Expression, not just a String

    // Check if left side is a dereference (e.g., *ptr *= 2)
    if let Expression::Dereference(inner) = &operands[0] {
        return Some(Statement::DerefCompoundAssignment {
            target: (**inner).clone(), // The thing being dereferenced (e.g., 'ptr')
            op,
            value: operands[1].clone(),
        });
    }

    // Check if left side is a pointer field access (e.g., sb->capacity *= 2)
    if let Expression::PointerFieldAccess { .. } = &operands[0] {
        return Some(Statement::DerefCompoundAssignment {
            target: operands[0].clone(), // The full PointerFieldAccess expression
            op,
            value: operands[1].clone(),
        });
    }

    // Check if left side is a struct field access (e.g., obj.field *= 2)
    if let Expression::FieldAccess { .. } = &operands[0] {
        return Some(Statement::DerefCompoundAssignment {
            target: operands[0].clone(), // The full FieldAccess expression
            op,
            value: operands[1].clone(),
        });
    }

    // Check if left side is an array index (e.g., arr[i] *= 2)
    if let Expression::ArrayIndex { .. } = &operands[0] {
        return Some(Statement::DerefCompoundAssignment {
            target: operands[0].clone(), // The full ArrayIndex expression
            op,
            value: operands[1].clone(),
        });
    }

    // Simple variable target (existing behavior)
    let target = match &operands[0] {
        Expression::Variable(name) => name.clone(),
        _ => return None, // Unknown target type
    };

    Some(Statement::CompoundAssignment { target, op, value: operands[1].clone() })
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

    let mut if_data =
        IfData { condition: None, then_block: Vec::new(), else_block: None, child_index: 0 };

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
                107 => extract_float_literal(cursor), // CXCursor_FloatingLiteral
                110 => extract_char_literal(cursor),  // CXCursor_CharacterLiteral
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
            // DECY-216: Handle both compound statements (with braces) and single statements
            if kind == CXCursor_CompoundStmt {
                let body_ptr = &mut if_data.then_block as *mut Vec<Statement>;
                unsafe {
                    clang_visitChildren(cursor, visit_statement, body_ptr as CXClientData);
                }
            } else {
                // Single statement without braces: if (cond) return 1;
                if let Some(stmt) = extract_single_statement(cursor) {
                    if_data.then_block.push(stmt);
                }
            }
            if_data.child_index += 1;
            CXChildVisit_Continue
        }
        2 => {
            // Third child (optional): else block
            // DECY-216: Handle compound, if-else chain, and single statement
            if kind == CXCursor_CompoundStmt {
                let mut else_stmts = Vec::new();
                let body_ptr = &mut else_stmts as *mut Vec<Statement>;
                unsafe {
                    clang_visitChildren(cursor, visit_statement, body_ptr as CXClientData);
                }
                if_data.else_block = Some(else_stmts);
            } else if kind == CXCursor_IfStmt {
                // else if chain
                let mut else_stmts = Vec::new();
                let body_ptr = &mut else_stmts as *mut Vec<Statement>;
                unsafe {
                    clang_visitChildren(cursor, visit_statement, body_ptr as CXClientData);
                }
                if_data.else_block = Some(else_stmts);
            } else {
                // Single statement in else: else return 0;
                if let Some(stmt) = extract_single_statement(cursor) {
                    if_data.else_block = Some(vec![stmt]);
                }
            }
            if_data.child_index += 1;
            CXChildVisit_Continue
        }
        _ => CXChildVisit_Continue,
    }
}

/// Extract a for loop statement.
fn extract_for_stmt(cursor: CXCursor) -> Option<Statement> {
    // DECY-200: Two-pass approach to handle for loops with empty parts
    // Clang skips empty parts entirely, so we can't rely on fixed indices
    //
    // Pass 1: Collect all children with their cursor kinds
    // Pass 2: Identify what each child represents based on type and position

    #[repr(C)]
    struct ForChildInfo {
        cursor: CXCursor,
        kind: i32,
    }

    #[repr(C)]
    struct ForCollector {
        children: Vec<ForChildInfo>,
    }

    // First pass: collect all children
    extern "C" fn collect_for_children(
        cursor: CXCursor,
        _parent: CXCursor,
        client_data: CXClientData,
    ) -> CXChildVisitResult {
        let collector = unsafe { &mut *(client_data as *mut ForCollector) };
        let kind = unsafe { clang_getCursorKind(cursor) };
        collector.children.push(ForChildInfo { cursor, kind });
        CXChildVisit_Continue
    }

    let mut collector = ForCollector { children: Vec::new() };

    unsafe {
        clang_visitChildren(cursor, collect_for_children, &mut collector as *mut _ as CXClientData);
    }

    // Second pass: identify what each child is
    // DECY-224: Use Vec to support multiple init/increment declarations
    let mut init: Vec<Statement> = Vec::new();
    let mut condition: Option<Expression> = None;
    let mut increment: Vec<Statement> = Vec::new();
    let mut body: Vec<Statement> = Vec::new();

    let num_children = collector.children.len();

    // Body is always the LAST child
    // The children before body are init/condition/increment in that order,
    // but clang omits empty ones

    // Helper to check if a BinaryOperator is an assignment
    fn is_assignment_op(cursor: CXCursor) -> bool {
        if let Some(op) = extract_binary_operator(cursor) {
            matches!(op, BinaryOperator::Assign)
        } else {
            false
        }
    }

    // Helper to check if a BinaryOperator is a comparison/logical (condition)
    fn is_condition_op(cursor: CXCursor) -> bool {
        if let Some(op) = extract_binary_operator(cursor) {
            matches!(
                op,
                BinaryOperator::Equal
                    | BinaryOperator::NotEqual
                    | BinaryOperator::LessThan
                    | BinaryOperator::GreaterThan
                    | BinaryOperator::LessEqual
                    | BinaryOperator::GreaterEqual
                    | BinaryOperator::LogicalAnd
                    | BinaryOperator::LogicalOr
            )
        } else {
            false
        }
    }

    // DECY-224: Helper to extract increment statements (handles comma operator)
    fn extract_increment_stmts(cursor: CXCursor) -> Vec<Statement> {
        let kind = unsafe { clang_getCursorKind(cursor) };
        let mut stmts = Vec::new();

        // Check for comma operator (BinaryOperator with comma)
        if kind == CXCursor_BinaryOperator {
            // Check if it's a comma operator by looking at the operator
            if let Some(BinaryOperator::Comma) = extract_binary_operator(cursor) {
                // Comma operator - recursively extract from both sides
                let mut children: Vec<CXCursor> = Vec::new();
                let children_ptr = &mut children as *mut Vec<CXCursor>;

                extern "C" fn collect_children(
                    cursor: CXCursor,
                    _parent: CXCursor,
                    client_data: CXClientData,
                ) -> CXChildVisitResult {
                    let children = unsafe { &mut *(client_data as *mut Vec<CXCursor>) };
                    children.push(cursor);
                    CXChildVisit_Continue
                }

                unsafe {
                    clang_visitChildren(cursor, collect_children, children_ptr as CXClientData);
                }

                for child in children {
                    stmts.extend(extract_increment_stmts(child));
                }
                return stmts;
            }
            // Assignment increment
            if let Some(stmt) = extract_assignment_stmt(cursor) {
                stmts.push(stmt);
            }
        } else if kind == CXCursor_UnaryOperator {
            if let Some(stmt) = extract_inc_dec_stmt(cursor) {
                stmts.push(stmt);
            }
        }
        stmts
    }

    if num_children == 0 {
        return Some(Statement::For { init, condition, increment, body });
    }

    // Process children based on count and types
    // The LAST child is always the body
    let body_idx = num_children - 1;
    let body_child = &collector.children[body_idx];

    // Extract body
    if body_child.kind == CXCursor_CompoundStmt {
        let body_ptr = &mut body as *mut Vec<Statement>;
        unsafe {
            clang_visitChildren(body_child.cursor, visit_statement, body_ptr as CXClientData);
        }
    } else {
        // Single statement body - extract it
        if let Some(stmt) = extract_single_statement(body_child.cursor) {
            body.push(stmt);
        }
    }

    // Process children before body
    let pre_body = &collector.children[..body_idx];

    match pre_body.len() {
        0 => {
            // for (;;) - infinite loop with no init/condition/increment
        }
        1 => {
            // One child before body - could be init, condition, or increment
            // Use heuristics to determine which
            let child = &pre_body[0];
            if child.kind == CXCursor_DeclStmt {
                // DeclStmt - always init
                let mut init_stmts = Vec::new();
                let ptr = &mut init_stmts as *mut Vec<Statement>;
                unsafe {
                    clang_visitChildren(child.cursor, visit_statement, ptr as CXClientData);
                }
            } else if child.kind == CXCursor_BinaryOperator {
                if is_assignment_op(child.cursor) {
                    // Assignment - treat as init
                    if let Some(stmt) = extract_assignment_stmt(child.cursor) {
                        init.push(stmt);
                    }
                } else if is_condition_op(child.cursor) {
                    // Comparison - treat as condition
                    condition = extract_binary_op(child.cursor);
                } else {
                    // Ambiguous - default to condition
                    condition = extract_binary_op(child.cursor);
                }
            } else if child.kind == CXCursor_UnaryOperator {
                increment = extract_increment_stmts(child.cursor);
            } else {
                // Treat as condition by default
                condition = extract_expression_from_cursor(child.cursor);
            }
        }
        2 => {
            // Two children before body
            // Most common case: condition and increment (init is empty)
            let child0 = &pre_body[0];
            let child1 = &pre_body[1];

            // Check if first child is init (DeclStmt or assignment)
            let first_is_init = child0.kind == CXCursor_DeclStmt
                || (child0.kind == CXCursor_BinaryOperator && is_assignment_op(child0.cursor));

            if first_is_init {
                // child0 = init, child1 = condition (skip increment)
                if child0.kind == CXCursor_DeclStmt {
                    // DECY-224: Collect ALL declarations
                    let ptr = &mut init as *mut Vec<Statement>;
                    unsafe {
                        clang_visitChildren(child0.cursor, visit_statement, ptr as CXClientData);
                    }
                } else if let Some(stmt) = extract_assignment_stmt(child0.cursor) {
                    init.push(stmt);
                }
                condition = extract_expression_from_cursor(child1.cursor);
            } else {
                // child0 = condition, child1 = increment (no init)
                condition = extract_expression_from_cursor(child0.cursor);
                increment = extract_increment_stmts(child1.cursor);
            }
        }
        3 => {
            // Three children before body - init, condition, increment all present
            let child0 = &pre_body[0];
            let child1 = &pre_body[1];
            let child2 = &pre_body[2];

            // Init - DECY-224: Collect ALL declarations
            if child0.kind == CXCursor_DeclStmt {
                let ptr = &mut init as *mut Vec<Statement>;
                unsafe {
                    clang_visitChildren(child0.cursor, visit_statement, ptr as CXClientData);
                }
            } else if child0.kind == CXCursor_BinaryOperator {
                if let Some(stmt) = extract_assignment_stmt(child0.cursor) {
                    init.push(stmt);
                }
            }

            // Condition
            condition = extract_expression_from_cursor(child1.cursor);

            // Increment - DECY-224: Handle comma operators
            increment = extract_increment_stmts(child2.cursor);
        }
        _ => {
            // More than 3 children before body - unexpected, handle gracefully
        }
    }

    Some(Statement::For { init, condition, increment, body })
}

/// Extract expression from cursor for for-loop condition
fn extract_expression_from_cursor(cursor: CXCursor) -> Option<Expression> {
    let kind = unsafe { clang_getCursorKind(cursor) };
    match kind {
        CXCursor_BinaryOperator => extract_binary_op(cursor),
        CXCursor_IntegerLiteral => extract_int_literal(cursor),
        107 => extract_float_literal(cursor), // CXCursor_FloatingLiteral
        110 => extract_char_literal(cursor),  // CXCursor_CharacterLiteral
        CXCursor_DeclRefExpr => extract_variable_ref(cursor),
        CXCursor_CallExpr => extract_function_call(cursor),
        CXCursor_UnaryOperator => extract_unary_op(cursor),
        _ => {
            let mut expr: Option<Expression> = None;
            let expr_ptr = &mut expr as *mut Option<Expression>;
            unsafe {
                clang_visitChildren(cursor, visit_expression, expr_ptr as CXClientData);
            }
            expr
        }
    }
}

/// Extract a single statement from a cursor (for non-compound for bodies)
fn extract_single_statement(cursor: CXCursor) -> Option<Statement> {
    let kind = unsafe { clang_getCursorKind(cursor) };
    match kind {
        CXCursor_IfStmt => extract_if_stmt(cursor),
        CXCursor_ForStmt => extract_for_stmt(cursor),
        CXCursor_WhileStmt => extract_while_stmt(cursor),
        CXCursor_ReturnStmt => extract_return_stmt(cursor),
        CXCursor_SwitchStmt => extract_switch_stmt(cursor),
        CXCursor_UnaryOperator => extract_inc_dec_stmt(cursor),
        CXCursor_BinaryOperator => extract_assignment_stmt(cursor),
        CXCursor_CallExpr => {
            if let Some(Expression::FunctionCall { function, arguments }) =
                extract_function_call(cursor)
            {
                Some(Statement::FunctionCall { function, arguments })
            } else {
                None
            }
        }
        CXCursor_BreakStmt => Some(Statement::Break),
        CXCursor_ContinueStmt => Some(Statement::Continue),
        CXCursor_DoStmt | CXCursor_NullStmt => None, // Not supported yet
        _ => None,
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

    let mut while_data = WhileData { condition: None, body: Vec::new(), child_index: 0 };

    let data_ptr = &mut while_data as *mut WhileData;

    unsafe {
        clang_visitChildren(cursor, visit_while_children, data_ptr as CXClientData);
    }

    Some(Statement::While { condition: while_data.condition?, body: while_data.body })
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
                107 => extract_float_literal(cursor), // CXCursor_FloatingLiteral
                110 => extract_char_literal(cursor),  // CXCursor_CharacterLiteral
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

    let mut switch_data =
        SwitchData { condition: None, cases: Vec::new(), default_case: None, child_index: 0 };

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

    let mut case_data = CaseData { value: None, body: Vec::new(), child_index: 0 };

    let data_ptr = &mut case_data as *mut CaseData;

    unsafe {
        clang_visitChildren(cursor, visit_case_children, data_ptr as CXClientData);
    }

    Some(SwitchCase { value: case_data.value, body: case_data.body })
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
            if let Some(Expression::FunctionCall { function, arguments }) =
                extract_function_call(cursor)
            {
                return Some(Statement::FunctionCall { function, arguments });
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
        107 => {
            // Floating-point literal (CXCursor_FloatingLiteral)
            if let Some(expr) = extract_float_literal(cursor) {
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
        110 => {
            // Character literal (CXCursor_CharacterLiteral)
            if let Some(expr) = extract_char_literal(cursor) {
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
        116 => {
            // CXCursor_ConditionalOperator (ternary)
            // DECY-192: Ternary expressions like (a > b) ? a : b
            if let Some(expr) = extract_conditional_op(cursor) {
                *expr_opt = Some(expr);
            }
            CXChildVisit_Continue
        }
        117 => {
            // CXCursor_CStyleCastExpr - cast expression like (int)x or (long)&ptr
            // DECY-208: Extract cast expressions to preserve type conversions
            if let Some(expr) = extract_cast(cursor) {
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
        119 => {
            // CXCursor_InitListExpr - initializer list for struct/array
            // DECY-133: Handle designated initializers like {.x = 10, .y = 20}
            if let Some(expr) = extract_init_list(cursor) {
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
    } else {
        // DECY-195: Fallback for system headers where tokenization fails
        // Use clang_Cursor_Evaluate to get the constant value
        unsafe {
            let eval_result = clang_Cursor_Evaluate(cursor);
            if !eval_result.is_null() {
                value = clang_EvalResult_getAsInt(eval_result);
                clang_EvalResult_dispose(eval_result);
            }
        }
    }

    Some(Expression::IntLiteral(value))
}

/// DECY-207: Extract a floating-point literal expression.
fn extract_float_literal(cursor: CXCursor) -> Option<Expression> {
    // SAFETY: Get the extent (source range) of the cursor
    let extent = unsafe { clang_getCursorExtent(cursor) };

    // SAFETY: Get the translation unit from the cursor
    let tu = unsafe { clang_Cursor_getTranslationUnit(cursor) };

    if tu.is_null() {
        return Some(Expression::FloatLiteral("0.0".to_string()));
    }

    // SAFETY: Tokenize the extent
    let mut tokens = ptr::null_mut();
    let mut num_tokens = 0;

    unsafe {
        clang_tokenize(tu, extent, &mut tokens, &mut num_tokens);
    }

    let mut value = "0.0".to_string();

    if num_tokens > 0 {
        // SAFETY: Get the spelling of the first token
        unsafe {
            let token_cxstring = clang_getTokenSpelling(tu, *tokens);
            let c_str = CStr::from_ptr(clang_getCString(token_cxstring));
            if let Ok(token_str) = c_str.to_str() {
                // Keep the string as-is (preserves precision)
                value = token_str.to_string();
            }
            clang_disposeString(token_cxstring);

            // SAFETY: Dispose tokens
            clang_disposeTokens(tu, tokens, num_tokens);
        }
    } else {
        // Fallback using evaluate
        unsafe {
            let eval_result = clang_Cursor_Evaluate(cursor);
            if !eval_result.is_null() {
                let float_val = clang_EvalResult_getAsDouble(eval_result);
                value = format!("{}", float_val);
                clang_EvalResult_dispose(eval_result);
            }
        }
    }

    Some(Expression::FloatLiteral(value))
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

/// Extract a character literal expression.
/// Handles plain characters ('a'), escape sequences ('\0', '\n', '\t', etc.)
fn extract_char_literal(cursor: CXCursor) -> Option<Expression> {
    // SAFETY: Get the extent (source range) of the cursor
    let extent = unsafe { clang_getCursorExtent(cursor) };

    // SAFETY: Get the translation unit from the cursor
    let tu = unsafe { clang_Cursor_getTranslationUnit(cursor) };

    if tu.is_null() {
        return Some(Expression::CharLiteral(0));
    }

    // SAFETY: Tokenize the extent
    let mut tokens = ptr::null_mut();
    let mut num_tokens = 0;

    unsafe {
        clang_tokenize(tu, extent, &mut tokens, &mut num_tokens);
    }

    let mut value: i8 = 0;

    if num_tokens > 0 {
        // SAFETY: Get the spelling of the first token
        unsafe {
            let token_cxstring = clang_getTokenSpelling(tu, *tokens);
            let c_str = CStr::from_ptr(clang_getCString(token_cxstring));
            if let Ok(token_str) = c_str.to_str() {
                // Remove surrounding quotes from character literal
                let inner = token_str.trim_matches('\'');
                value = parse_char_literal(inner);
            }
            clang_disposeString(token_cxstring);

            // SAFETY: Dispose tokens
            clang_disposeTokens(tu, tokens, num_tokens);
        }
    }

    Some(Expression::CharLiteral(value))
}

/// Parse a character literal string (without quotes) into its i8 value.
/// Handles escape sequences like \0, \n, \t, \r, \\, \', \"
#[allow(clippy::disallowed_methods)] // .unwrap() after !is_empty() check
fn parse_char_literal(s: &str) -> i8 {
    if s.is_empty() {
        return 0;
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap();

    if first == '\\' {
        // Escape sequence
        match chars.next() {
            Some('0') => 0, // null character
            Some('n') => b'\n' as i8,
            Some('t') => b'\t' as i8,
            Some('r') => b'\r' as i8,
            Some('\\') => b'\\' as i8,
            Some('\'') => b'\'' as i8,
            Some('"') => b'"' as i8,
            Some('a') => 7,  // bell
            Some('b') => 8,  // backspace
            Some('f') => 12, // form feed
            Some('v') => 11, // vertical tab
            Some('x') => {
                // Hex escape: \xNN
                let hex: String = chars.take(2).collect();
                i8::from_str_radix(&hex, 16).unwrap_or(0)
            }
            Some(c) if c.is_ascii_digit() => {
                // Octal escape: \NNN
                let mut octal = String::new();
                octal.push(c);
                for _ in 0..2 {
                    if let Some(d) = chars.next() {
                        if d.is_ascii_digit() && d < '8' {
                            octal.push(d);
                        } else {
                            break;
                        }
                    }
                }
                i8::from_str_radix(&octal, 8).unwrap_or(0)
            }
            _ => first as i8,
        }
    } else {
        // Plain character
        first as i8
    }
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
        107 => {
            // Floating-point literal (CXCursor_FloatingLiteral)
            if let Some(expr) = extract_float_literal(cursor) {
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
        110 => {
            // Character literal (CXCursor_CharacterLiteral)
            if let Some(expr) = extract_char_literal(cursor) {
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
        116 => {
            // CXCursor_ConditionalOperator (ternary) - DECY-192
            if let Some(expr) = extract_conditional_op(cursor) {
                operands.push(expr);
            }
            CXChildVisit_Continue
        }
        _ => CXChildVisit_Recurse,
    }
}

/// DECY-234: Extract binary operator by looking between child cursor locations.
/// This is more reliable than tokenizing the full extent for macro-expanded expressions.
#[allow(non_upper_case_globals)]
fn extract_binary_operator_from_children(
    cursor: CXCursor,
    tu: CXTranslationUnit,
) -> Option<BinaryOperator> {
    // Collect the two child cursors
    let mut children: Vec<CXCursor> = Vec::new();
    let children_ptr = &mut children as *mut Vec<CXCursor>;

    extern "C" fn collect_children(
        cursor: CXCursor,
        _parent: CXCursor,
        client_data: CXClientData,
    ) -> CXChildVisitResult {
        let children = unsafe { &mut *(client_data as *mut Vec<CXCursor>) };
        children.push(cursor);
        CXChildVisit_Continue
    }

    unsafe {
        clang_visitChildren(cursor, collect_children, children_ptr as CXClientData);
    }

    // Need exactly 2 children for a binary operator
    if children.len() != 2 {
        return None;
    }

    // Get the end location of first child and start location of second child
    let first_extent = unsafe { clang_getCursorExtent(children[0]) };
    let second_extent = unsafe { clang_getCursorExtent(children[1]) };

    let first_end = unsafe { clang_getRangeEnd(first_extent) };
    let second_start = unsafe { clang_getRangeStart(second_extent) };

    // Create a source range between the two children
    let operator_range = unsafe { clang_getRange(first_end, second_start) };

    // Tokenize this specific range to find the operator
    let mut tokens = ptr::null_mut();
    let mut num_tokens = 0;

    unsafe {
        clang_tokenize(tu, operator_range, &mut tokens, &mut num_tokens);
    }

    if tokens.is_null() || num_tokens == 0 {
        return None;
    }

    // Look for an operator token in this range
    let mut result = None;
    for i in 0..num_tokens {
        unsafe {
            let token = *tokens.add(i as usize);
            let token_kind = clang_getTokenKind(token);

            if token_kind == CXToken_Punctuation {
                let token_cxstring = clang_getTokenSpelling(tu, token);
                let c_str = CStr::from_ptr(clang_getCString(token_cxstring));
                if let Ok(token_str) = c_str.to_str() {
                    result = match token_str {
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
                        "<<" => Some(BinaryOperator::LeftShift),
                        ">>" => Some(BinaryOperator::RightShift),
                        "&" => Some(BinaryOperator::BitwiseAnd),
                        "|" => Some(BinaryOperator::BitwiseOr),
                        "^" => Some(BinaryOperator::BitwiseXor),
                        "=" => Some(BinaryOperator::Assign),
                        "," => Some(BinaryOperator::Comma),
                        _ => None,
                    };
                    if result.is_some() {
                        clang_disposeString(token_cxstring);
                        break;
                    }
                }
                clang_disposeString(token_cxstring);
            }
        }
    }

    // Dispose tokens
    unsafe {
        clang_disposeTokens(tu, tokens, num_tokens);
    }

    result
}

fn token_str_to_binary_operator(token_str: &str) -> Option<BinaryOperator> {
    match token_str {
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
        "<<" => Some(BinaryOperator::LeftShift),
        ">>" => Some(BinaryOperator::RightShift),
        "&" => Some(BinaryOperator::BitwiseAnd),
        "|" => Some(BinaryOperator::BitwiseOr),
        "^" => Some(BinaryOperator::BitwiseXor),
        "=" => Some(BinaryOperator::Assign),
        "," => Some(BinaryOperator::Comma),
        _ => None,
    }
}

fn find_first_matching(
    candidates: &[(usize, BinaryOperator)],
    predicate: fn(&BinaryOperator) -> bool,
) -> Option<BinaryOperator> {
    candidates.iter().find(|(_, op)| predicate(op)).map(|(_, op)| *op)
}

fn select_lowest_precedence_operator(
    mut candidates: Vec<(usize, BinaryOperator)>,
) -> Option<BinaryOperator> {
    if candidates.is_empty() {
        return None;
    }

    let has_arithmetic = candidates.iter().any(|(_, op)| {
        matches!(
            op,
            BinaryOperator::Add
                | BinaryOperator::Subtract
                | BinaryOperator::Multiply
                | BinaryOperator::Divide
                | BinaryOperator::Modulo
        )
    });
    let has_comparison = candidates.iter().any(|(_, op)| {
        matches!(
            op,
            BinaryOperator::LessThan
                | BinaryOperator::GreaterThan
                | BinaryOperator::LessEqual
                | BinaryOperator::GreaterEqual
                | BinaryOperator::Equal
                | BinaryOperator::NotEqual
        )
    });

    if has_arithmetic || has_comparison {
        candidates.retain(|(_, op)| !matches!(op, BinaryOperator::Comma));
    }

    // C precedence (low to high): , > = > || > && > | > ^ > & > == != > < > <= >= > << >> > + - > * / %
    let precedence_checks: Vec<fn(&BinaryOperator) -> bool> = vec![
        |op| matches!(op, BinaryOperator::Assign),
        |op| matches!(op, BinaryOperator::LogicalOr),
        |op| matches!(op, BinaryOperator::LogicalAnd),
        |op| matches!(op, BinaryOperator::BitwiseOr),
        |op| matches!(op, BinaryOperator::BitwiseXor),
        |op| matches!(op, BinaryOperator::BitwiseAnd),
        |op| matches!(op, BinaryOperator::Equal | BinaryOperator::NotEqual),
        |op| {
            matches!(
                op,
                BinaryOperator::LessThan
                    | BinaryOperator::GreaterThan
                    | BinaryOperator::LessEqual
                    | BinaryOperator::GreaterEqual
            )
        },
        |op| matches!(op, BinaryOperator::LeftShift | BinaryOperator::RightShift),
        |op| matches!(op, BinaryOperator::Add | BinaryOperator::Subtract),
    ];

    for check in &precedence_checks {
        if let Some(op) = find_first_matching(&candidates, *check) {
            return Some(op);
        }
    }

    Some(candidates[0].1)
}

/// Extract the binary operator from a cursor by tokenizing.
#[allow(non_upper_case_globals)]
fn extract_binary_operator(cursor: CXCursor) -> Option<BinaryOperator> {
    // Get the translation unit
    let tu = unsafe { clang_Cursor_getTranslationUnit(cursor) };
    if tu.is_null() {
        return None;
    }

    // DECY-234: First, try to get operator by looking between child cursors
    // This handles macro-expanded expressions better than tokenizing the full extent
    if let Some(op) = extract_binary_operator_from_children(cursor, tu) {
        return Some(op);
    }

    // Fallback: Get the extent (source range) of the cursor
    let extent = unsafe { clang_getCursorExtent(cursor) };

    // Tokenize to find the operator
    let mut tokens = ptr::null_mut();
    let mut num_tokens = 0;

    unsafe {
        clang_tokenize(tu, extent, &mut tokens, &mut num_tokens);
    }

    // DECY-234: Get the file of the cursor's expansion location to filter macro tokens
    // For macro-expanded code, the extent spans multiple locations including macro definitions
    // We only want operators from the expansion site, not from macro definitions
    let cursor_loc = unsafe { clang_getCursorLocation(cursor) };
    let mut cursor_file: CXFile = ptr::null_mut();
    let mut _line = 0u32;
    let mut _col = 0u32;
    let mut _offset = 0u32;
    unsafe {
        clang_getExpansionLocation(
            cursor_loc,
            &mut cursor_file,
            &mut _line,
            &mut _col,
            &mut _offset,
        );
    }

    // Look through tokens to find the operator
    // For compound expressions like "a > 0 && b > 0", we need to find the LAST
    // operator (the one with lowest precedence) which represents THIS binary operation.
    // We scan from right to left to find operators with lowest precedence first.
    // Precedence (lowest to highest): || > && > == != > < > <= >= > + - > * / %

    let mut candidates: Vec<(usize, BinaryOperator)> = Vec::new();
    let mut found_first_operand = false;
    let mut paren_depth: i32 = 0; // Track parenthesis nesting depth

    // DECY-234: Get extent line bounds to filter tokens from macro definitions
    let extent_start = unsafe { clang_getRangeStart(extent) };
    let extent_end = unsafe { clang_getRangeEnd(extent) };
    let mut start_line = 0u32;
    let mut end_line = 0u32;
    unsafe {
        clang_getExpansionLocation(
            extent_start,
            ptr::null_mut(),
            &mut start_line,
            ptr::null_mut(),
            ptr::null_mut(),
        );
        clang_getExpansionLocation(
            extent_end,
            ptr::null_mut(),
            &mut end_line,
            ptr::null_mut(),
            ptr::null_mut(),
        );
    }

    for i in 0..num_tokens {
        unsafe {
            let token = *tokens.add(i as usize);
            let token_kind = clang_getTokenKind(token);

            // DECY-234: Skip tokens that are from macro definitions (different file)
            let token_loc = clang_getTokenLocation(tu, token);
            let mut token_file: CXFile = ptr::null_mut();
            let mut token_line = 0u32;
            clang_getExpansionLocation(
                token_loc,
                &mut token_file,
                &mut token_line,
                &mut _col,
                &mut _offset,
            );

            // Skip tokens from different files (macro definition files)
            if !cursor_file.is_null() && !token_file.is_null() && token_file != cursor_file {
                continue;
            }

            // DECY-234: Skip tokens outside the extent's line range
            // When macros are involved, clang_tokenize returns tokens from the macro
            // definition site (e.g., line 34) even though extent is single-line (e.g., 38-38).
            // Filter out these spurious tokens.
            if start_line > 0
                && end_line > 0
                && token_line > 0
                && (token_line < start_line || token_line > end_line)
            {
                continue;
            }

            // Track when we've seen the first operand (identifier or literal)
            if token_kind == CXToken_Identifier || token_kind == CXToken_Literal {
                found_first_operand = true;
            }

            // Track parenthesis depth to avoid operators inside function calls
            if token_kind == CXToken_Punctuation {
                let token_cxstring = clang_getTokenSpelling(tu, token);
                let c_str = CStr::from_ptr(clang_getCString(token_cxstring));
                if let Ok(token_str) = c_str.to_str() {
                    match token_str {
                        "(" => paren_depth += 1,
                        ")" => paren_depth = paren_depth.saturating_sub(1),
                        _ => {}
                    }

                    // Only collect operator candidates at depth 0 (outside parentheses)
                    // This fixes DECY-116: n * func(n - 1) was picking up the - inside parens
                    if found_first_operand && paren_depth == 0 {
                        if let Some(op) = token_str_to_binary_operator(token_str) {
                            candidates.push((i as usize, op));
                        }
                    }
                }
                clang_disposeString(token_cxstring);
            }
        }
    }

    let operator = select_lowest_precedence_operator(candidates);

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

    Some(Expression::FunctionCall { function, arguments: arg_data.arguments })
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
        107 => {
            // Floating-point literal (CXCursor_FloatingLiteral)
            if let Some(expr) = extract_float_literal(cursor) {
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
        110 => {
            // Character literal (CXCursor_CharacterLiteral)
            if let Some(expr) = extract_char_literal(cursor) {
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
        116 => {
            // CXCursor_ConditionalOperator (ternary) - DECY-192
            if let Some(expr) = extract_conditional_op(cursor) {
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
        117 => {
            // CXCursor_CStyleCastExpr - DECY-243: Cast expression in argument (e.g., (int)x, (unsigned char)cp[i])
            if let Some(expr) = extract_cast(cursor) {
                arg_data.arguments.push(expr);
            }
            CXChildVisit_Continue
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

    // DECY-255: For expressions like (*p)++, we need to find ++ as the operator
    // The key insight is: ++ or -- at position > 0 is post-increment/decrement
    // while * at position 0 is dereference
    // So we scan all tokens and pick the right operator based on position
    let mut found_star_at_zero = false;
    let mut found_open_paren_at_zero = false;
    let mut found_increment: Option<u32> = None;
    let mut found_decrement: Option<u32> = None;

    for i in 0..num_tokens {
        unsafe {
            let token = *tokens.add(i as usize);
            let token_kind = clang_getTokenKind(token);

            if token_kind == CXToken_Punctuation {
                let token_cxstring = clang_getTokenSpelling(tu, token);
                let c_str = CStr::from_ptr(clang_getCString(token_cxstring));
                if let Ok(token_str) = c_str.to_str() {
                    match token_str {
                        "*" if i == 0 => found_star_at_zero = true,
                        "(" if i == 0 => found_open_paren_at_zero = true,
                        "++" => {
                            found_increment = Some(i);
                        }
                        "--" => {
                            found_decrement = Some(i);
                        }
                        "-" if i == 0 && operator.is_none() => {
                            operator = Some(UnaryOperator::Minus);
                        }
                        "!" if i == 0 && operator.is_none() => {
                            operator = Some(UnaryOperator::LogicalNot);
                        }
                        "~" if i == 0 && operator.is_none() => {
                            operator = Some(UnaryOperator::BitwiseNot);
                        }
                        "&" if i == 0 && operator.is_none() => {
                            operator = Some(UnaryOperator::AddressOf);
                        }
                        _ => {}
                    }
                }
                clang_disposeString(token_cxstring);
            }
        }
    }

    // DECY-255: For (*p)++, we have ( at position 0, * at position 1, and ++ at the end
    // The ++ is the operator for THIS cursor, * is part of the operand
    // For *p++ (without parens), the * is the operator and ++ is handled by a nested cursor
    // CRITICAL: If we have * at position 0 (dereference), the ++ belongs to a nested expression
    // CRITICAL: If we found a unary operator (-, !, ~, &) at position 0, do NOT override
    // with ++/-- found at later positions - those belong to different expressions
    let has_unary_op_at_zero = operator.is_some();

    if let Some(pos) = found_increment {
        // Only use ++ as operator if:
        // 1. It's at position 0 (pre-increment), OR
        // 2. We have ( at position 0 (parenthesized expression like (*p)++), OR
        // 3. No other unary operator at position 0 and no * at position 0
        if pos == 0 {
            is_increment = true;
            operator_position = pos;
        } else if found_open_paren_at_zero {
            // (*p)++ case - the ++ applies to the whole parenthesized expression
            is_increment = true;
            operator_position = pos;
        } else if !has_unary_op_at_zero && !found_star_at_zero {
            // No dereference and no other unary op at position 0
            is_increment = true;
            operator_position = pos;
        }
    }
    if let Some(pos) = found_decrement {
        // Decrement takes precedence if at position 0, after open paren, or no other unary op
        if pos == 0 || found_open_paren_at_zero || (!has_unary_op_at_zero && !found_star_at_zero) {
            is_decrement = true;
            operator_position = pos;
        }
    }

    // If no increment/decrement selected, use dereference if found at position 0
    if !is_increment && !is_decrement && found_star_at_zero {
        is_dereference = true;
    }
    // operator is already set if we found -, !, ~, or & at position 0

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
            return Some(Expression::PreIncrement { operand: Box::new(operand_expr) });
        } else {
            return Some(Expression::PostIncrement { operand: Box::new(operand_expr) });
        }
    }

    if is_decrement {
        // Check if pre or post decrement
        let is_pre = operator_position == 0;
        if is_pre {
            return Some(Expression::PreDecrement { operand: Box::new(operand_expr) });
        } else {
            return Some(Expression::PostDecrement { operand: Box::new(operand_expr) });
        }
    }

    // Handle other unary operators
    if let Some(op) = operator {
        return Some(Expression::UnaryOp { op, operand: Box::new(operand_expr) });
    }

    // DECY-195: Fallback for system headers where tokenization fails
    // If we have a UnaryOperator cursor with an operand but couldn't identify the operator,
    // try to infer it from context. For macro expansions like EOF=(-1),
    // the unary minus might not be tokenizable.
    // Check if the operand is an integer literal - if so, it might be a negation
    // For now, return the operand wrapped as unary minus if it's an integer
    // This handles the common case of EOF = (-1) from stdio.h
    if let Expression::IntLiteral(_) = &operand_expr {
        // If we found an integer inside a UnaryOperator, assume it's negation
        return Some(Expression::UnaryOp {
            op: UnaryOperator::Minus,
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
        Some(Expression::PointerFieldAccess { pointer: Box::new(object), field })
    } else {
        Some(Expression::FieldAccess { object: Box::new(object), field })
    }
}

/// Extract a sizeof expression.
/// DECY-119: Only match if sizeof is the FIRST token (not from other statements)
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

    // DECY-119: sizeof must be the FIRST token, otherwise this cursor
    // is not a sizeof expression (it might just contain one elsewhere)
    if num_tokens == 0 {
        unsafe {
            clang_disposeTokens(tu, tokens, num_tokens);
        }
        return None;
    }

    let first_token_is_sizeof = unsafe {
        let token = *tokens.add(0);
        let token_cxstring = clang_getTokenSpelling(tu, token);
        let c_str = CStr::from_ptr(clang_getCString(token_cxstring));
        let is_sizeof = c_str.to_str().map(|s| s == "sizeof").unwrap_or(false);
        clang_disposeString(token_cxstring);
        is_sizeof
    };

    if !first_token_is_sizeof {
        unsafe {
            clang_disposeTokens(tu, tokens, num_tokens);
        }
        return None;
    }

    let mut type_name = String::new();
    let mut paren_depth = 0;
    let mut in_sizeof_parens = false;

    // Look through tokens to extract type name (skip first token which is "sizeof")
    // DECY-119: Track paren depth to stop at closing paren
    for i in 1..num_tokens {
        unsafe {
            let token = *tokens.add(i as usize);
            let token_kind = clang_getTokenKind(token);
            let token_cxstring = clang_getTokenSpelling(tu, token);
            let c_str = CStr::from_ptr(clang_getCString(token_cxstring));

            if let Ok(token_str) = c_str.to_str() {
                if token_str == "(" {
                    paren_depth += 1;
                    in_sizeof_parens = true;
                } else if token_str == ")" {
                    paren_depth -= 1;
                    // DECY-119: Stop when we close the sizeof parenthesis
                    if paren_depth == 0 && in_sizeof_parens {
                        clang_disposeString(token_cxstring);
                        break;
                    }
                } else if in_sizeof_parens
                    && (token_kind == CXToken_Identifier || token_kind == CXToken_Keyword)
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

    // We already verified first token is sizeof, just check we got a type name
    if !type_name.is_empty() {
        Some(Expression::Sizeof { type_name })
    } else {
        None
    }
}

/// Convert clang type to our Type enum.
#[allow(non_upper_case_globals)]
/// Extract a cast expression from a clang cursor.
///
/// Parses C-style cast expressions like `(int)x` or `(void*)ptr`.
/// Extracts the target type and the expression being cast.
fn extract_cast(cursor: CXCursor) -> Option<Expression> {
    // SAFETY: Getting the type this expression evaluates to (the cast result type)
    let target_cx_type = unsafe { clang_getCursorType(cursor) };
    let target_type = convert_type(target_cx_type)?;

    // Extract the inner expression by visiting children
    let mut inner_expr: Option<Expression> = None;
    let inner_ptr = &mut inner_expr as *mut Option<Expression>;

    unsafe {
        clang_visitChildren(cursor, visit_cast_inner, inner_ptr as CXClientData);
    }

    inner_expr.map(|expr| Expression::Cast { target_type, expr: Box::new(expr) })
}

/// Visitor callback to extract the inner expression of a cast.
#[allow(non_upper_case_globals)]
extern "C" fn visit_cast_inner(
    cursor: CXCursor,
    _parent: CXCursor,
    client_data: CXClientData,
) -> CXChildVisitResult {
    let inner_expr = unsafe { &mut *(client_data as *mut Option<Expression>) };
    let kind = unsafe { clang_getCursorKind(cursor) };

    // Try to extract any expression
    if let Some(expr) = try_extract_expression(cursor) {
        *inner_expr = Some(expr);
        return CXChildVisit_Break; // Found the inner expression, stop visiting
    }

    // For some expression types, we need to recurse
    match kind {
        CXCursor_UnexposedExpr | CXCursor_ParenExpr => CXChildVisit_Recurse,
        _ => CXChildVisit_Continue,
    }
}

/// Extract a compound literal expression from a clang cursor.
///
/// Parses C99 compound literals like `(struct Point){10, 20}` or `(int[]){1, 2, 3}`.
/// Extracts the type and initializer expressions.
fn extract_compound_literal(cursor: CXCursor) -> Option<Expression> {
    // SAFETY: Getting the type of the compound literal
    let literal_cx_type = unsafe { clang_getCursorType(cursor) };
    let literal_type = convert_type(literal_cx_type)?;

    // Extract initializer expressions by visiting children
    let mut initializers: Vec<Expression> = Vec::new();
    let initializers_ptr = &mut initializers as *mut Vec<Expression>;

    unsafe {
        clang_visitChildren(
            cursor,
            visit_compound_literal_initializers,
            initializers_ptr as CXClientData,
        );
    }

    Some(Expression::CompoundLiteral { literal_type, initializers })
}

/// DECY-192: Extract a ternary/conditional expression.
///
/// Parses C conditional expressions like `cond ? then_val : else_val`.
/// The ternary operator has 3 children: condition, then expression, else expression.
fn extract_conditional_op(cursor: CXCursor) -> Option<Expression> {
    // Extract all three operands by visiting children
    let mut operands: Vec<Expression> = Vec::new();
    let operands_ptr = &mut operands as *mut Vec<Expression>;

    unsafe {
        clang_visitChildren(cursor, visit_conditional_operand, operands_ptr as CXClientData);
    }

    // Ternary operators should have exactly 3 operands: condition, then, else
    // However, sometimes clang may emit extra implicit expressions
    if operands.len() >= 3 {
        Some(Expression::Ternary {
            condition: Box::new(operands[0].clone()),
            then_expr: Box::new(operands[1].clone()),
            else_expr: Box::new(operands[2].clone()),
        })
    } else if operands.len() == 2 {
        // GNU extension: `x ?: y` is equivalent to `x ? x : y`
        // Clang may represent this with only 2 children
        Some(Expression::Ternary {
            condition: Box::new(operands[0].clone()),
            then_expr: Box::new(operands[0].clone()),
            else_expr: Box::new(operands[1].clone()),
        })
    } else {
        None
    }
}

/// Visitor callback for conditional operator (ternary) operands.
/// DECY-192: Collects condition, then_expr, and else_expr.
#[allow(non_upper_case_globals)]
extern "C" fn visit_conditional_operand(
    cursor: CXCursor,
    _parent: CXCursor,
    client_data: CXClientData,
) -> CXChildVisitResult {
    let operands = unsafe { &mut *(client_data as *mut Vec<Expression>) };

    // Try to extract expression using the general expression extractor
    if let Some(expr) = try_extract_expression(cursor) {
        operands.push(expr);
    }

    CXChildVisit_Continue
}

/// DECY-133: Extract an initializer list expression for struct/array initialization.
///
/// Handles C99 designated initializers like `{.x = 10, .y = 20}` or `{[2] = 100}`.
/// Clang resolves designated initializers to positional order and inserts ImplicitValueInitExpr
/// for uninitialized fields.
fn extract_init_list(cursor: CXCursor) -> Option<Expression> {
    // SAFETY: Getting the type of the initializer list
    let literal_cx_type = unsafe { clang_getCursorType(cursor) };
    let literal_type = convert_type(literal_cx_type)?;

    // Extract initializer expressions by visiting children
    let mut initializers: Vec<Expression> = Vec::new();
    let initializers_ptr = &mut initializers as *mut Vec<Expression>;

    unsafe {
        clang_visitChildren(cursor, visit_init_list_children, initializers_ptr as CXClientData);
    }

    Some(Expression::CompoundLiteral { literal_type, initializers })
}

/// Visitor callback to extract initializers from an InitListExpr.
/// DECY-133: Handles both regular and designated initializers.
#[allow(non_upper_case_globals)]
extern "C" fn visit_init_list_children(
    cursor: CXCursor,
    _parent: CXCursor,
    client_data: CXClientData,
) -> CXChildVisitResult {
    let initializers = unsafe { &mut *(client_data as *mut Vec<Expression>) };
    let kind = unsafe { clang_getCursorKind(cursor) };

    // Handle ImplicitValueInitExpr (115) - default value for uninitialized fields
    // This appears when designated initializers skip some fields
    if kind == 115 {
        // Get the type to determine the default value
        let cx_type = unsafe { clang_getCursorType(cursor) };
        if let Some(var_type) = convert_type(cx_type) {
            // Generate appropriate default based on type
            let default_expr = match var_type {
                Type::Int => Expression::IntLiteral(0),
                Type::Float | Type::Double => Expression::IntLiteral(0), // Will be cast
                Type::Char => Expression::IntLiteral(0),
                _ => Expression::IntLiteral(0), // Fallback
            };
            initializers.push(default_expr);
        }
        return CXChildVisit_Continue;
    }

    // DECY-133b: Handle designated initializers
    // Array: [idx] = value  → UnexposedExpr with children [IntLiteral(idx), value]
    // Struct: .field = value → UnexposedExpr with children [MemberRef, value]
    if kind == CXCursor_UnexposedExpr {
        // Collect cursor kinds and expressions for all children
        #[repr(C)]
        struct ChildInfo {
            kinds: Vec<u32>,
            exprs: Vec<Expression>,
        }

        let mut info = ChildInfo { kinds: Vec::new(), exprs: Vec::new() };
        let info_ptr = &mut info as *mut ChildInfo;

        #[allow(clippy::disallowed_methods)] // .unwrap() after !is_empty() check
        extern "C" fn collect_child_info(
            cursor: CXCursor,
            _parent: CXCursor,
            client_data: CXClientData,
        ) -> CXChildVisitResult {
            let info = unsafe { &mut *(client_data as *mut ChildInfo) };
            let kind = unsafe { clang_getCursorKind(cursor) };
            info.kinds.push(kind as u32);

            // Try to extract expression, including InitListExpr
            if kind == 119 {
                // InitListExpr - extract as CompoundLiteral
                if let Some(expr) = extract_init_list(cursor) {
                    info.exprs.push(expr);
                }
            } else if let Some(expr) = try_extract_expression(cursor) {
                info.exprs.push(expr);
            }
            CXChildVisit_Continue
        }

        unsafe {
            clang_visitChildren(cursor, collect_child_info, info_ptr as CXClientData);
        }

        // Array designated init: [idx] = value → 2 children, first is IntLiteral
        if info.exprs.len() == 2 && matches!(&info.exprs[0], Expression::IntLiteral(_)) {
            initializers.push(info.exprs[1].clone());
            return CXChildVisit_Continue;
        }

        // Struct field designated init: .field = value → first kind is MemberRef (47)
        // Second child is the value (could be InitListExpr or other expression)
        if info.kinds.len() == 2 && info.kinds[0] == 47 && !info.exprs.is_empty() {
            // Take the last expression (the value)
            #[allow(clippy::disallowed_methods)]
            initializers.push(info.exprs.last().unwrap().clone());
            return CXChildVisit_Continue;
        }

        // Not a designated initializer - fall through to recursion
        return CXChildVisit_Recurse;
    }

    // Try to extract any expression as an initializer
    if let Some(expr) = try_extract_expression(cursor) {
        initializers.push(expr);
        return CXChildVisit_Continue;
    }

    // For some expression types, recurse
    match kind {
        CXCursor_ParenExpr => CXChildVisit_Recurse,
        _ => CXChildVisit_Continue,
    }
}

/// Visitor callback to extract initializers from a compound literal.
#[allow(non_upper_case_globals)]
extern "C" fn visit_compound_literal_initializers(
    cursor: CXCursor,
    _parent: CXCursor,
    client_data: CXClientData,
) -> CXChildVisitResult {
    let initializers = unsafe { &mut *(client_data as *mut Vec<Expression>) };
    let kind = unsafe { clang_getCursorKind(cursor) };

    // The compound literal typically has an InitListExpr child
    // CXCursor_InitListExpr = 119
    if kind == 119 {
        // This is the initializer list - visit its children to get individual initializers
        return CXChildVisit_Recurse;
    }

    // Try to extract any expression as an initializer
    if let Some(expr) = try_extract_expression(cursor) {
        initializers.push(expr);
        return CXChildVisit_Continue;
    }

    // For some expression types, recurse
    match kind {
        CXCursor_UnexposedExpr | CXCursor_ParenExpr => CXChildVisit_Recurse,
        _ => CXChildVisit_Continue,
    }
}

#[allow(non_upper_case_globals)]
fn convert_type(cx_type: CXType) -> Option<Type> {
    // SAFETY: Getting type kind
    match cx_type.kind {
        CXType_Void => Some(Type::Void),
        3 => Some(Type::Bool), // CXType_Bool = 3 — C99 _Bool
        CXType_Int => Some(Type::Int),
        CXType_UInt => Some(Type::UnsignedInt), // DECY-158: unsigned int → u32
        CXType_UChar => Some(Type::Char),       // unsigned char → u8 (DECY-057 fix)
        CXType_UShort => Some(Type::UnsignedInt), // unsigned short → u32 (safe approximation)
        CXType_ULong => Some(Type::UnsignedInt), // unsigned long → u32 (safe approximation)
        CXType_Short => Some(Type::Int),        // short → i32
        CXType_Long => Some(Type::Int),         // long → i32
        CXType_LongLong => Some(Type::Int),     // long long → i32 (simplified)
        CXType_ULongLong => Some(Type::UnsignedInt), // DECY-158: unsigned long long → u32
        CXType_Float => Some(Type::Float),
        CXType_Double => Some(Type::Double),
        23 => Some(Type::Double), // CXType_LongDouble → f64 (Rust has no long double)
        CXType_Char_S | CXType_Char_U => Some(Type::Char),
        14 => Some(Type::SignedChar), // CXType_SChar - explicitly signed char → i8 (DECY-250)
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

            Some(Type::FunctionPointer { param_types, return_type: Box::new(return_type) })
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
            // DECY-172: Get typedef name first to check for known type aliases
            let typedef_decl = unsafe { clang_getTypeDeclaration(cx_type) };
            let typedef_name_cxstring = unsafe { clang_getCursorSpelling(typedef_decl) };
            let typedef_name = unsafe {
                let c_str = CStr::from_ptr(clang_getCString(typedef_name_cxstring));
                let tn = c_str.to_string_lossy().into_owned();
                clang_disposeString(typedef_name_cxstring);
                tn
            };

            // DECY-172: Preserve size_t, ssize_t, ptrdiff_t as TypeAlias
            // These need to map to usize/isize in Rust for compatibility with .len() etc.
            match typedef_name.as_str() {
                "size_t" | "ssize_t" | "ptrdiff_t" => {
                    return Some(Type::TypeAlias(typedef_name));
                }
                _ => {}
            }

            // DECY-147: For typedefs to anonymous structs, use typedef name as struct name
            // Example: typedef struct { int x; } Point; → Type::Struct("Point")
            let canonical = unsafe { clang_getCanonicalType(cx_type) };

            // Check if this is a typedef to an anonymous struct
            if canonical.kind == CXType_Record {
                let decl = unsafe { clang_getTypeDeclaration(canonical) };
                let struct_name_cxstring = unsafe { clang_getCursorSpelling(decl) };
                let struct_name = unsafe {
                    let c_str = CStr::from_ptr(clang_getCString(struct_name_cxstring));
                    let sn = c_str.to_string_lossy().into_owned();
                    clang_disposeString(struct_name_cxstring);
                    sn
                };

                // If struct is anonymous, use the typedef name instead
                if struct_name.is_empty() {
                    return Some(Type::Struct(typedef_name));
                }
            }

            // Default: recursively convert the canonical type
            convert_type(canonical)
        }
        CXType_ConstantArray => {
            // Array type - extract element type and size
            let element_cx_type = unsafe { clang_getArrayElementType(cx_type) };
            let element_type = convert_type(element_cx_type)?;

            // Get array size
            let array_size = unsafe { clang_getArraySize(cx_type) };
            let size = if array_size >= 0 { Some(array_size) } else { None };

            Some(Type::Array { element_type: Box::new(element_type), size })
        }
        114 => {
            // CXType_IncompleteArray - flexible array member (C99 §6.7.2.1)
            // DECY-136: char data[] → Vec<u8>
            // Flexible array members have no size specified
            let element_cx_type = unsafe { clang_getArrayElementType(cx_type) };
            let element_type = convert_type(element_cx_type)?;

            // Generate as Array with size None (will be transformed to Vec in codegen)
            Some(Type::Array { element_type: Box::new(element_type), size: None })
        }
        106 => {
            // CXType_Enum - C enums are integers
            // DECY-240: Map enum types to i32 for Rust compatibility
            Some(Type::Int)
        }
        _ => None,
    }
}
