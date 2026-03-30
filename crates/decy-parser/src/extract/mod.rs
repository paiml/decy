//! Clang AST extraction functions.
//!
//! This module contains the functions that traverse clang cursors and extract
//! C language constructs (functions, statements, expressions, types) into
//! our AST representation.

#[allow(non_upper_case_globals)]
mod types;
#[allow(non_upper_case_globals)]
mod statements;
#[allow(non_upper_case_globals)]
mod expressions;

pub(crate) use types::{
    extract_function, extract_typedef, extract_struct, extract_enum,
    extract_variable, extract_macro, extract_class, extract_namespace,
};

use crate::ast_types::*;
use crate::diagnostic::{Diagnostic, Severity};
use clang_sys::*;
use std::ffi::CStr;
use std::ptr;

use self::expressions::extract_statement;
use self::statements::{
    extract_var_decl, extract_return_stmt, extract_assignment_stmt,
    extract_inc_dec_stmt, extract_compound_assignment_stmt,
    extract_if_stmt, extract_for_stmt, extract_while_stmt,
    extract_switch_stmt,
};

#[allow(non_upper_case_globals)]
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
    } else if kind == 4 {
        // CXCursor_ClassDecl = 4 (DECY-200: C++ class extraction)
        // Don't recurse — extract_class handles its own children
        if let Some(class) = extract_class(cursor) {
            ast.add_class(class);
        }
        return CXChildVisit_Continue;
    } else if kind == 22 {
        // CXCursor_Namespace = 22 (DECY-201: C++ namespace extraction)
        // Don't recurse — extract_namespace handles its own children
        if let Some(ns) = extract_namespace(cursor) {
            ast.add_namespace(ns);
        }
        return CXChildVisit_Continue;
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

/// Visitor callback for extracting statements from function body.
///
/// # Safety
///
/// This function is called by clang_visitChildren and must follow C calling conventions.
#[allow(non_upper_case_globals)]
pub(crate) extern "C" fn visit_statement(
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

