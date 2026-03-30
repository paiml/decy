//! Expression extraction from clang cursors.

#[allow(non_upper_case_globals)]

use crate::ast_types::*;
use clang_sys::*;
use std::ffi::CStr;
use std::ptr;

use super::types::convert_type;
use super::statements::{
    extract_var_decl, extract_return_stmt, extract_if_stmt, extract_for_stmt,
    extract_while_stmt, extract_switch_stmt, extract_inc_dec_stmt,
    extract_assignment_stmt,
};

/// Helper function to extract an expression from a cursor.
/// Dispatches to the appropriate extract function based on cursor kind.
#[allow(non_upper_case_globals)]
pub(crate) fn try_extract_expression(cursor: CXCursor) -> Option<Expression> {
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
pub(crate) extern "C" fn visit_variable_initializer(
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

/// Helper function to extract a statement from a cursor based on its kind.
#[allow(non_upper_case_globals)]
pub(crate) fn extract_statement(cursor: CXCursor) -> Option<Statement> {
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

/// Extract expression from cursor for for-loop condition
pub(crate) fn extract_expression_from_cursor(cursor: CXCursor) -> Option<Expression> {
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
pub(crate) fn extract_single_statement(cursor: CXCursor) -> Option<Statement> {
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

/// Visitor callback for extracting expressions.
///
/// # Safety
///
/// This function is called by clang_visitChildren and must follow C calling conventions.
#[allow(non_upper_case_globals)]
pub(crate) extern "C" fn visit_expression(
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
        132 => {
            // CXCursor_CXXThisExpr - implicit 'this' pointer in C++ methods
            // DECY-214: Map to 'self' for Rust method access
            *expr_opt = Some(Expression::Variable("self".to_string()));
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
pub(crate) fn extract_int_literal(cursor: CXCursor) -> Option<Expression> {
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
pub(crate) fn extract_float_literal(cursor: CXCursor) -> Option<Expression> {
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
pub(crate) fn extract_string_literal(cursor: CXCursor) -> Option<Expression> {
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
pub(crate) fn extract_char_literal(cursor: CXCursor) -> Option<Expression> {
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
pub(crate) fn parse_char_literal(s: &str) -> i8 {
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
pub(crate) fn extract_variable_ref(cursor: CXCursor) -> Option<Expression> {
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
pub(crate) fn extract_binary_op(cursor: CXCursor) -> Option<Expression> {
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
pub(crate) extern "C" fn visit_binary_operand(
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
pub(crate) fn extract_binary_operator_from_children(
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

pub(crate) fn token_str_to_binary_operator(token_str: &str) -> Option<BinaryOperator> {
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

pub(crate) fn find_first_matching(
    candidates: &[(usize, BinaryOperator)],
    predicate: fn(&BinaryOperator) -> bool,
) -> Option<BinaryOperator> {
    candidates.iter().find(|(_, op)| predicate(op)).map(|(_, op)| *op)
}

pub(crate) fn select_lowest_precedence_operator(
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
pub(crate) fn extract_binary_operator(cursor: CXCursor) -> Option<BinaryOperator> {
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
pub(crate) fn extract_function_call(cursor: CXCursor) -> Option<Expression> {
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
pub(crate) extern "C" fn visit_call_argument(
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
pub(crate) fn extract_unary_op(cursor: CXCursor) -> Option<Expression> {
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
pub(crate) fn extract_array_index(cursor: CXCursor) -> Option<Expression> {
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
pub(crate) fn extract_field_access(cursor: CXCursor) -> Option<Expression> {
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

    // DECY-214: If no object expression found, this is implicit 'this' access
    // in a C++ method body (e.g., `return x;` where x is a member field).
    // Map to self.field for Rust.
    let object = object_expr.unwrap_or_else(|| Expression::Variable("self".to_string()));

    if is_arrow {
        Some(Expression::PointerFieldAccess { pointer: Box::new(object), field })
    } else {
        Some(Expression::FieldAccess { object: Box::new(object), field })
    }
}

/// Extract a sizeof expression.
/// DECY-119: Only match if sizeof is the FIRST token (not from other statements)
pub(crate) fn extract_sizeof(cursor: CXCursor) -> Option<Expression> {
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
pub(crate) fn extract_cast(cursor: CXCursor) -> Option<Expression> {
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
pub(crate) extern "C" fn visit_cast_inner(
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
pub(crate) fn extract_compound_literal(cursor: CXCursor) -> Option<Expression> {
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
pub(crate) fn extract_conditional_op(cursor: CXCursor) -> Option<Expression> {
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
pub(crate) extern "C" fn visit_conditional_operand(
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
pub(crate) fn extract_init_list(cursor: CXCursor) -> Option<Expression> {
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
pub(crate) extern "C" fn visit_init_list_children(
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
pub(crate) extern "C" fn visit_compound_literal_initializers(
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


