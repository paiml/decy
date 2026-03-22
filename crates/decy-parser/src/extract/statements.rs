//! Statement extraction from clang cursors.

#[allow(non_upper_case_globals)]

use crate::ast_types::*;
use clang_sys::*;
use std::ffi::CStr;
use std::ptr;

use super::types::convert_type;
use super::visit_statement;
use super::expressions::{
    extract_binary_op, extract_binary_operator, extract_int_literal, extract_float_literal,
    extract_char_literal, extract_variable_ref, extract_function_call, extract_unary_op,
    extract_field_access, extract_array_index, extract_statement,
    extract_expression_from_cursor, extract_single_statement,
    visit_expression, visit_binary_operand, try_extract_expression,
};

/// Extract a variable declaration statement.
pub(crate) fn extract_var_decl(cursor: CXCursor) -> Option<Statement> {
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
pub(crate) fn extract_return_stmt(cursor: CXCursor) -> Option<Statement> {
    // Extract return expression by visiting children
    let mut return_expr: Option<Expression> = None;
    let expr_ptr = &mut return_expr as *mut Option<Expression>;

    unsafe {
        clang_visitChildren(cursor, visit_expression, expr_ptr as CXClientData);
    }

    Some(Statement::Return(return_expr))
}

/// Extract an assignment statement.
pub(crate) fn extract_assignment_stmt(cursor: CXCursor) -> Option<Statement> {
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
pub(crate) fn extract_inc_dec_stmt(cursor: CXCursor) -> Option<Statement> {
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
pub(crate) fn extract_compound_assignment_stmt(cursor: CXCursor) -> Option<Statement> {
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
pub(crate) fn extract_if_stmt(cursor: CXCursor) -> Option<Statement> {
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
pub(crate) extern "C" fn visit_if_children(
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
pub(crate) fn extract_for_stmt(cursor: CXCursor) -> Option<Statement> {
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

/// Extract a while loop statement.
pub(crate) fn extract_while_stmt(cursor: CXCursor) -> Option<Statement> {
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
pub(crate) extern "C" fn visit_while_children(
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
pub(crate) fn extract_switch_stmt(cursor: CXCursor) -> Option<Statement> {
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
pub(crate) extern "C" fn visit_switch_children(
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
pub(crate) extern "C" fn visit_switch_body(
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
pub(crate) fn extract_case_stmt(cursor: CXCursor) -> Option<SwitchCase> {
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
pub(crate) extern "C" fn visit_case_children(
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
pub(crate) fn extract_default_stmt(cursor: CXCursor) -> Option<Vec<Statement>> {
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
pub(crate) extern "C" fn visit_default_children(
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

