//! Statement code generation methods for CodeGenerator.
//!
//! Contains all methods related to generating Rust code from HIR statements,
//! including declarations, assignments, control flow (if/while/for/switch),
//! and pointer/array/field assignments.

use super::{escape_rust_keyword, CodeGenerator, TypeContext};
use decy_hir::{BinaryOperator, HirExpression, HirStatement, HirType};

impl CodeGenerator {
    /// Generate code for a statement.
    pub fn generate_statement(&self, stmt: &HirStatement) -> String {
        self.generate_statement_for_function(stmt, None)
    }

    /// Generate code for a statement, with optional function context.
    ///
    /// When function_name is "main", special handling applies (DECY-AUDIT-001):
    /// - return N; becomes std::process::exit(N);
    pub(crate) fn generate_statement_for_function(
        &self,
        stmt: &HirStatement,
        function_name: Option<&str>,
    ) -> String {
        self.generate_statement_with_context(stmt, function_name, &mut TypeContext::new(), None)
    }

    /// Generate code for a statement with type context for pointer arithmetic and return type for null pointer detection.
    pub(crate) fn generate_statement_with_context(
        &self,
        stmt: &HirStatement,
        function_name: Option<&str>,
        ctx: &mut TypeContext,
        return_type: Option<&HirType>,
    ) -> String {
        match stmt {
            HirStatement::VariableDeclaration { name, var_type, initializer } => {
                self.generate_declaration_statement(name, var_type, initializer.as_ref(), ctx)
            }
            HirStatement::Return(expr_opt) => {
                self.generate_return_statement(expr_opt.as_ref(), function_name, ctx, return_type)
            }
            HirStatement::If { condition, then_block, else_block } => {
                self.generate_if_statement(
                    condition,
                    then_block,
                    else_block.as_deref(),
                    function_name,
                    ctx,
                    return_type,
                )
            }
            HirStatement::While { condition, body } => {
                self.generate_while_statement(
                    condition,
                    body,
                    function_name,
                    ctx,
                    return_type,
                )
            }
            HirStatement::Break => "break;".to_string(),
            HirStatement::Continue => "continue;".to_string(),
            HirStatement::Assignment { target, value } => {
                self.generate_assignment_statement(target, value, ctx)
            }
            HirStatement::For { init, condition, increment, body } => {
                self.generate_for_statement(
                    init,
                    condition.as_ref(),
                    increment,
                    body,
                    function_name,
                    ctx,
                    return_type,
                )
            }
            HirStatement::Switch { condition, cases, default_case } => {
                self.generate_switch_statement(
                    condition,
                    cases,
                    default_case.as_deref(),
                    function_name,
                    ctx,
                    return_type,
                )
            }
            HirStatement::DerefAssignment { target, value } => {
                self.generate_deref_assignment_statement(target, value, ctx)
            }
            HirStatement::ArrayIndexAssignment { array, index, value } => {
                self.generate_array_index_assignment_statement(array, index, value, ctx)
            }
            HirStatement::FieldAssignment { object, field, value } => {
                self.generate_field_assignment_statement(object, field, value, ctx)
            }
            HirStatement::Free { pointer } => {
                let pointer_name = match pointer {
                    HirExpression::Variable(name) => name.clone(),
                    _ => self.generate_expression_with_context(pointer, ctx),
                };
                format!("// Memory for '{}' deallocated automatically by RAII", pointer_name)
            }
            HirStatement::Expression(expr) => {
                format!("{};", self.generate_expression_with_context(expr, ctx))
            }
            HirStatement::InlineAsm { text, translatable } => {
                let mut result = String::new();
                result.push_str("// DECY: manual review required - inline assembly\n");
                if *translatable {
                    result.push_str(
                        "// DECY: this assembly may be translatable to Rust intrinsics\n",
                    );
                }
                result.push_str(&format!("// Original asm: {}", text.replace('\n', "\n// ")));
                result
            }
        }
    }

    /// Generate a variable declaration statement.
    fn resolve_declaration_type(
        name: &str,
        var_type: &HirType,
        initializer: Option<&HirExpression>,
        is_malloc_init: bool,
        ctx: &mut TypeContext,
    ) -> (HirType, String) {
        if is_malloc_init {
            if let HirType::Pointer(inner) = var_type {
                let is_struct_alloc = matches!(&**inner, HirType::Struct(_));
                let is_array_pattern = if let Some(init_expr) = initializer {
                    Self::is_malloc_array_pattern(init_expr)
                } else {
                    false
                };

                if is_struct_alloc && !is_array_pattern {
                    let box_type = HirType::Box(inner.clone());
                    ctx.add_variable(name.to_string(), box_type.clone());
                    return (box_type.clone(), Self::map_type(&box_type));
                } else {
                    let vec_type = HirType::Vec(inner.clone());
                    ctx.add_variable(name.to_string(), vec_type.clone());
                    return (vec_type.clone(), Self::map_type(&vec_type));
                }
            } else {
                ctx.add_variable(name.to_string(), var_type.clone());
                return (var_type.clone(), Self::map_type(var_type));
            }
        }

        let is_string_literal_init =
            matches!(initializer, Some(HirExpression::StringLiteral(_)));
        let is_char_pointer = matches!(
            var_type,
            HirType::Pointer(inner) if matches!(&**inner, HirType::Char)
        );
        let is_char_pointer_array = matches!(
            var_type,
            HirType::Array { element_type, .. }
            if matches!(&**element_type, HirType::Pointer(inner) if matches!(&**inner, HirType::Char))
        );
        let is_array_of_string_literals = matches!(
            initializer,
            Some(HirExpression::CompoundLiteral { initializers, .. })
            if initializers.iter().all(|e| matches!(e, HirExpression::StringLiteral(_)))
        );

        if is_char_pointer && is_string_literal_init {
            ctx.add_variable(name.to_string(), HirType::StringReference);
            (HirType::StringReference, "&str".to_string())
        } else if is_char_pointer_array && is_array_of_string_literals {
            let size =
                if let HirType::Array { size, .. } = var_type { *size } else { None };
            let array_type = HirType::Array {
                element_type: Box::new(HirType::StringReference),
                size,
            };
            ctx.add_variable(name.to_string(), array_type.clone());
            let type_str = if let Some(n) = size {
                format!("[&str; {}]", n)
            } else {
                "[&str]".to_string()
            };
            (array_type, type_str)
        } else {
            ctx.add_variable(name.to_string(), var_type.clone());
            (var_type.clone(), Self::map_type(var_type))
        }
    }

    fn generate_malloc_expr_init(
        &self,
        code: &mut String,
        var_type: &HirType,
        init_expr: &HirExpression,
        ctx: &TypeContext,
    ) {
        match var_type {
            HirType::Box(inner) => {
                code.push_str(&format!(
                    " = Box::new({});",
                    Self::default_value_for_type(inner)
                ));
            }
            HirType::Vec(_) => {
                if let HirExpression::Malloc { size } = init_expr {
                    if let HirExpression::BinaryOp {
                        op: decy_hir::BinaryOperator::Multiply,
                        left,
                        ..
                    } = size.as_ref()
                    {
                        let capacity_code =
                            self.generate_expression_with_context(left, ctx);
                        code.push_str(&format!(
                            " = Vec::with_capacity({});",
                            capacity_code
                        ));
                    } else {
                        code.push_str(" = Vec::new();");
                    }
                } else {
                    code.push_str(" = Vec::new();");
                }
            }
            _ => {
                code.push_str(" = Box::new(0i32);");
            }
        }
    }

    fn generate_malloc_funcall_init(
        &self,
        code: &mut String,
        var_type: &HirType,
        actual_type: &HirType,
        init_expr: &HirExpression,
        ctx: &mut TypeContext,
    ) {
        match actual_type {
            HirType::Box(inner) => {
                let use_default =
                    if let HirType::Struct(struct_name) = inner.as_ref() {
                        ctx.struct_has_default(struct_name)
                    } else {
                        false
                    };

                if use_default {
                    code.push_str(" = Box::default();");
                } else {
                    let inner_type = Self::map_type(inner);
                    code.push_str(&format!(
                        " = Box::new(/* SAFETY: {} is valid when zero-initialized */ unsafe {{ std::mem::zeroed::<{}>() }});",
                        inner_type, inner_type
                    ));
                }
            }
            HirType::Vec(_) => {
                code.push_str(&format!(
                    " = {};",
                    self.generate_expression_with_target_type(
                        init_expr,
                        ctx,
                        Some(actual_type)
                    )
                ));
            }
            _ => {
                code.push_str(&format!(
                    " = {};",
                    self.generate_expression_with_target_type(
                        init_expr,
                        ctx,
                        Some(var_type)
                    )
                ));
            }
        }
    }

    fn generate_regular_init(
        &self,
        code: &mut String,
        var_type: &HirType,
        actual_type: &HirType,
        init_expr: &HirExpression,
        ctx: &mut TypeContext,
    ) {
        let is_char_array = matches!(
            var_type,
            HirType::Array { element_type, .. }
            if matches!(&**element_type, HirType::Char)
        );

        if is_char_array {
            if let HirExpression::StringLiteral(s) = init_expr {
                let escaped: String = s
                    .chars()
                    .map(|c| match c {
                        '"' => "\\\"".to_string(),
                        c => c.to_string(),
                    })
                    .collect();
                code.push_str(&format!(" = *b\"{}\\0\";", escaped));
            } else {
                code.push_str(&format!(
                    " = {};",
                    self.generate_expression_with_target_type(
                        init_expr,
                        ctx,
                        Some(var_type)
                    )
                ));
            }
        } else {
            code.push_str(&format!(
                " = {};",
                self.generate_expression_with_target_type(
                    init_expr,
                    ctx,
                    Some(actual_type)
                )
            ));
        }
    }

    fn generate_declaration_statement(
        &self,
        name: &str,
        var_type: &HirType,
        initializer: Option<&HirExpression>,
        ctx: &mut TypeContext,
    ) -> String {
        let escaped_name = escape_rust_keyword(name);
        let escaped_name = if ctx.is_global(&escaped_name) {
            let renamed = format!("{}_local", escaped_name);
            ctx.add_renamed_local(escaped_name.clone(), renamed.clone());
            renamed
        } else {
            escaped_name
        };
        if let HirType::Array { element_type, size: None } = var_type {
            if let Some(size_expr) = initializer {
                let size_code = self.generate_expression_with_context(size_expr, ctx);
                let default_value = match element_type.as_ref() {
                    HirType::Int => "0i32",
                    HirType::UnsignedInt => "0u32",
                    HirType::Float => "0.0f32",
                    HirType::Double => "0.0f64",
                    HirType::Char => "0u8",
                    HirType::SignedChar => "0i8",
                    _ => &Self::default_value_for_type(element_type),
                };

                ctx.add_variable(
                    name.to_string(),
                    HirType::Vec(Box::new(element_type.as_ref().clone())),
                );

                return format!(
                    "let mut {} = vec![{}; {}];",
                    escaped_name, default_value, size_code
                );
            }
        }

        let is_malloc_init = if let Some(init_expr) = initializer {
            Self::is_any_malloc_or_calloc(init_expr)
        } else {
            false
        };

        let (actual_type, type_str) =
            Self::resolve_declaration_type(name, var_type, initializer, is_malloc_init, ctx);

        let mutability = "mut ";
        let mut code = format!("let {}{}: {}", mutability, escaped_name, type_str);
        if let Some(init_expr) = initializer {
            if matches!(init_expr, HirExpression::Malloc { .. }) {
                self.generate_malloc_expr_init(&mut code, var_type, init_expr, ctx);
            } else if is_malloc_init {
                self.generate_malloc_funcall_init(
                    &mut code, var_type, &actual_type, init_expr, ctx,
                );
            } else {
                self.generate_regular_init(
                    &mut code, var_type, &actual_type, init_expr, ctx,
                );
            }
        } else {
            code.push_str(&format!(" = {};", Self::default_value_for_type(var_type)));
        }
        code
    }

    /// Generate a return statement.
    fn generate_return_statement(
        &self,
        expr_opt: Option<&HirExpression>,
        function_name: Option<&str>,
        ctx: &mut TypeContext,
        return_type: Option<&HirType>,
    ) -> String {
        // Special handling for main function (DECY-AUDIT-001)
        // return N; in main becomes std::process::exit(N);
        if function_name == Some("main") {
            if let Some(expr) = expr_opt {
                let expr_code = self.generate_expression_with_context(expr, ctx);
                // DECY-126: Check if expression type needs cast to i32
                // exit() expects i32, but char/u8 expressions need casting
                let expr_type = ctx.infer_expression_type(expr);
                let needs_cast = matches!(expr_type, Some(HirType::Char));
                if needs_cast {
                    format!("std::process::exit({} as i32);", expr_code)
                } else {
                    format!("std::process::exit({});", expr_code)
                }
            } else {
                "std::process::exit(0);".to_string()
            }
        } else if let Some(expr) = expr_opt {
            // Pass return type as target type hint for null pointer detection
            format!(
                "return {};",
                self.generate_expression_with_target_type(expr, ctx, return_type)
            )
        } else {
            "return;".to_string()
        }
    }

    /// Generate an if statement.
    fn generate_if_statement(
        &self,
        condition: &HirExpression,
        then_block: &[HirStatement],
        else_block: Option<&[HirStatement]>,
        function_name: Option<&str>,
        ctx: &mut TypeContext,
        return_type: Option<&HirType>,
    ) -> String {
        let mut code = String::new();

        // Generate if condition
        // DECY-131: If condition is not already boolean, wrap appropriately
        let cond_code = self.generate_expression_with_context(condition, ctx);
        let cond_str = if Self::is_boolean_expression(condition) {
            cond_code
        } else {
            // DECY-238: Check if condition is a pointer type - use !ptr.is_null()
            if let Some(cond_type) = ctx.infer_expression_type(condition) {
                if matches!(cond_type, HirType::Pointer(_)) {
                    format!("!{}.is_null()", cond_code)
                } else {
                    format!("({}) != 0", cond_code)
                }
            } else {
                format!("({}) != 0", cond_code)
            }
        };
        code.push_str(&format!("if {} {{\n", cond_str));

        // Generate then block
        for stmt in then_block {
            code.push_str("    ");
            code.push_str(&self.generate_statement_with_context(
                stmt,
                function_name,
                ctx,
                return_type,
            ));
            code.push('\n');
        }

        // Generate else block if present
        if let Some(else_stmts) = else_block {
            code.push_str("} else {\n");
            for stmt in else_stmts {
                code.push_str("    ");
                code.push_str(&self.generate_statement_with_context(
                    stmt,
                    function_name,
                    ctx,
                    return_type,
                ));
                code.push('\n');
            }
        }

        code.push('}');
        code
    }

    /// Generate a while statement.
    fn generate_while_statement(
        &self,
        condition: &HirExpression,
        body: &[HirStatement],
        function_name: Option<&str>,
        ctx: &mut TypeContext,
        return_type: Option<&HirType>,
    ) -> String {
        let mut code = String::new();

        // Generate while condition
        // DECY-138: Check for string iteration pattern: while (*str) → while !str.is_empty()
        let cond_str = if let Some(str_var) = Self::get_string_deref_var(condition, ctx) {
            format!("!{}.is_empty()", str_var)
        } else {
            // DECY-123: If condition is not already boolean, wrap appropriately
            let cond_code = self.generate_expression_with_context(condition, ctx);
            if Self::is_boolean_expression(condition) {
                cond_code
            } else {
                // DECY-238: Check if condition is a pointer type - use !ptr.is_null()
                if let Some(cond_type) = ctx.infer_expression_type(condition) {
                    if matches!(cond_type, HirType::Pointer(_)) {
                        format!("!{}.is_null()", cond_code)
                    } else {
                        format!("({}) != 0", cond_code)
                    }
                } else {
                    format!("({}) != 0", cond_code)
                }
            }
        };
        code.push_str(&format!("while {} {{\n", cond_str));

        // Generate loop body
        for stmt in body {
            code.push_str("    ");
            code.push_str(&self.generate_statement_with_context(
                stmt,
                function_name,
                ctx,
                return_type,
            ));
            code.push('\n');
        }

        code.push('}');
        code
    }

    /// Generate an assignment statement (including realloc handling).
    fn generate_assignment_statement(
        &self,
        target: &str,
        value: &HirExpression,
        ctx: &mut TypeContext,
    ) -> String {
        // Special handling for realloc() → Vec::resize/truncate/clear
        if let HirExpression::Realloc { pointer, new_size } = value {
            // target is a String (variable name) in Assignment statements
            let target_var = target.to_string();

            // Check if target is a Vec type to get element type
            let element_type = if let Some(HirType::Vec(inner)) = ctx.get_type(&target_var)
            {
                inner.as_ref().clone()
            } else {
                // Fallback: assume i32
                HirType::Int
            };

            // Check special cases:
            // 1. realloc(ptr, 0) → clear or RAII comment
            if let HirExpression::IntLiteral(0) = **new_size {
                return format!("{}.clear(); // Free equivalent: clear vector", target_var);
            }

            // 2. realloc(NULL, size) → should not appear in assignment (would be in initializer)
            //    but handle it gracefully if it does
            if matches!(**pointer, HirExpression::NullLiteral) {
                // This is essentially malloc - but in assignment context, we'll treat it as resize from 0
                if let HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Multiply,
                    left,
                    ..
                } = new_size.as_ref()
                {
                    let count_code = self.generate_expression_with_context(left, ctx);
                    let default_value = Self::default_value_for_type(&element_type);
                    return format!(
                        "{}.resize({}, {})",
                        target_var, count_code, default_value
                    );
                }
            }

            // 3. realloc(ptr, new_size) → vec.resize(new_count, default)
            // Extract count from new_size (typically n * sizeof(T))
            if let HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Multiply,
                left,
                ..
            } = new_size.as_ref()
            {
                let count_code = self.generate_expression_with_context(left, ctx);
                let default_value = Self::default_value_for_type(&element_type);
                format!("{}.resize({}, {});", target_var, count_code, default_value)
            } else {
                // Fallback: if new_size is not n * sizeof(T), generate direct resize
                // This handles edge cases where size isn't n * sizeof(T)
                let size_expr = self.generate_expression_with_context(new_size, ctx);
                let default_value = Self::default_value_for_type(&element_type);
                format!("{}.resize({} as usize, {});", target_var, size_expr, default_value)
            }
        } else {
            // DECY-134: Check for string iteration param pointer arithmetic
            // ptr = ptr + 1 → ptr_idx += 1
            if let Some(idx_var) = ctx.get_string_iter_index(target) {
                // Check if this is ptr = ptr + N or ptr = ptr - N
                if let HirExpression::BinaryOp { op, left, right } = value {
                    if let HirExpression::Variable(var_name) = &**left {
                        if var_name == target {
                            let right_code =
                                self.generate_expression_with_context(right, ctx);
                            return match op {
                                BinaryOperator::Add => {
                                    format!("{} += {} as usize;", idx_var, right_code)
                                }
                                BinaryOperator::Subtract => {
                                    format!("{} -= {} as usize;", idx_var, right_code)
                                }
                                _ => format!(
                                    "{} = {};",
                                    target,
                                    self.generate_expression_with_context(value, ctx)
                                ),
                            };
                        }
                    }
                }
            }
            // Regular assignment (not realloc)
            let target_type = ctx.get_type(target);
            let value_code =
                self.generate_expression_with_target_type(value, ctx, target_type);

            // DECY-261: Helper to strip nested unsafe blocks to avoid redundancy
            fn strip_nested_unsafe(code: &str) -> String {
                // Iteratively strip all unsafe { } wrappers from the code
                let mut result = code.to_string();
                while result.contains("unsafe { ") {
                    result = result.replace("unsafe { ", "").replace(" }", "");
                    // Handle case where there might be multiple closings
                    // Simple approach: strip pattern "unsafe { X }" → "X"
                }
                // More precise: use regex-like matching for simple cases
                result = code.replace("unsafe { ", "").replacen(
                    " }",
                    "",
                    code.matches("unsafe { ").count(),
                );
                result
            }

            // DECY-241: Handle errno assignment specially
            if target == "errno" {
                let clean_value = strip_nested_unsafe(&value_code);
                return format!("unsafe {{ ERRNO = {}; }}", clean_value);
            }
            // DECY-220: Wrap global variable assignment in unsafe block
            // DECY-261: Strip nested unsafe from value_code to avoid redundancy
            if ctx.is_global(target) {
                let clean_value = strip_nested_unsafe(&value_code);
                format!("unsafe {{ {} = {}; }}", target, clean_value)
            } else {
                format!("{} = {};", target, value_code)
            }
        }
    }

    /// Generate a for statement.
    fn generate_for_statement(
        &self,
        init: &[HirStatement],
        condition: Option<&HirExpression>,
        increment: &[HirStatement],
        body: &[HirStatement],
        function_name: Option<&str>,
        ctx: &mut TypeContext,
        return_type: Option<&HirType>,
    ) -> String {
        let mut code = String::new();

        // DECY-224: Generate ALL init statements before loop
        for init_stmt in init {
            code.push_str(&self.generate_statement_with_context(
                init_stmt,
                function_name,
                ctx,
                return_type,
            ));
            code.push('\n');
        }

        // Generate loop: `loop {}` for None (for(;;)), `while cond {}` for Some
        if let Some(cond) = condition {
            code.push_str(&format!(
                "while {} {{\n",
                self.generate_expression_with_context(cond, ctx)
            ));
        } else {
            code.push_str("loop {\n");
        }

        // Generate loop body
        for stmt in body {
            code.push_str("    ");
            code.push_str(&self.generate_statement_with_context(
                stmt,
                function_name,
                ctx,
                return_type,
            ));
            code.push('\n');
        }

        // DECY-224: Generate ALL increment statements at end of body
        for inc_stmt in increment {
            code.push_str("    ");
            code.push_str(&self.generate_statement_with_context(
                inc_stmt,
                function_name,
                ctx,
                return_type,
            ));
            code.push('\n');
        }

        code.push('}');
        code
    }

    /// Generate a switch statement as a Rust match expression.
    fn generate_switch_statement(
        &self,
        condition: &HirExpression,
        cases: &[decy_hir::SwitchCase],
        default_case: Option<&[HirStatement]>,
        function_name: Option<&str>,
        ctx: &mut TypeContext,
        return_type: Option<&HirType>,
    ) -> String {
        let mut code = String::new();

        // Generate match expression
        code.push_str(&format!(
            "match {} {{\n",
            self.generate_expression_with_context(condition, ctx)
        ));

        // DECY-209: Infer switch condition type for case pattern matching
        let condition_type = ctx.infer_expression_type(condition);
        let condition_is_int = matches!(condition_type, Some(HirType::Int));

        // Generate each case
        for case in cases {
            if let Some(value_expr) = &case.value {
                // Generate case pattern
                // DECY-209/DECY-219: If condition is Int and case is CharLiteral,
                // generate the numeric byte value directly as the pattern.
                // Rust match patterns don't allow casts like `b'0' as i32`,
                // so we must use the numeric value (e.g., 48 for '0')
                let case_pattern = if condition_is_int {
                    if let HirExpression::CharLiteral(ch) = value_expr {
                        // Direct numeric value for the character
                        format!("{}", (*ch) as i32)
                    } else {
                        self.generate_expression_with_context(value_expr, ctx)
                    }
                } else {
                    self.generate_expression_with_context(value_expr, ctx)
                };
                code.push_str(&format!("    {} => {{\n", case_pattern));

                // Generate case body (filter out Break statements)
                for stmt in &case.body {
                    if !matches!(stmt, HirStatement::Break) {
                        code.push_str("        ");
                        code.push_str(&self.generate_statement_with_context(
                            stmt,
                            function_name,
                            ctx,
                            return_type,
                        ));
                        code.push('\n');
                    }
                }

                code.push_str("    },\n");
            }
        }

        // Generate default case (or empty default if not present)
        code.push_str("    _ => {\n");
        if let Some(default_stmts) = default_case {
            for stmt in default_stmts {
                if !matches!(stmt, HirStatement::Break) {
                    code.push_str("        ");
                    code.push_str(&self.generate_statement_with_context(
                        stmt,
                        function_name,
                        ctx,
                        return_type,
                    ));
                    code.push('\n');
                }
            }
        }
        code.push_str("    },\n");

        code.push('}');
        code
    }

    /// Generate a dereference assignment statement.
    fn generate_deref_assignment_statement(
        &self,
        target: &HirExpression,
        value: &HirExpression,
        ctx: &mut TypeContext,
    ) -> String {
        // DECY-185: Handle struct field access targets directly (no dereference needed)
        // sb->capacity = value should generate (*sb).capacity = value, not *(*sb).capacity = value
        // DECY-254: ArrayIndex also doesn't need extra dereference
        // arr[i] *= 2 should generate arr[(i) as usize] = arr[(i) as usize] * 2
        if matches!(
            target,
            HirExpression::PointerFieldAccess { .. }
                | HirExpression::FieldAccess { .. }
                | HirExpression::ArrayIndex { .. }
        ) {
            let target_code = self.generate_expression_with_context(target, ctx);
            let value_code = self.generate_expression_with_context(value, ctx);
            return format!("{} = {};", target_code, value_code);
        }

        // DECY-134: Check for string iteration param - use slice indexing
        if let HirExpression::Variable(var_name) = target {
            if let Some(idx_var) = ctx.get_string_iter_index(var_name) {
                // Transform *ptr = value to slice[idx] = value - no unsafe needed!
                let value_code = self.generate_expression_with_context(value, ctx);
                return format!("{}[{}] = {};", var_name, idx_var, value_code);
            }
        }

        // Infer the type of *target for null pointer detection
        let target_type = ctx
            .infer_expression_type(&HirExpression::Dereference(Box::new(target.clone())));
        let target_code = self.generate_expression_with_context(target, ctx);
        let value_code =
            self.generate_expression_with_target_type(value, ctx, target_type.as_ref());

        // Helper to strip nested unsafe blocks - returns owned String to avoid lifetime issues
        fn strip_unsafe(code: &str) -> String {
            if code.starts_with("unsafe { ") && code.ends_with(" }") {
                code.strip_prefix("unsafe { ")
                    .and_then(|s| s.strip_suffix(" }"))
                    .unwrap_or(code)
                    .to_string()
            } else {
                code.to_string()
            }
        }

        // DECY-124: Check if target is a raw pointer - if so, wrap in unsafe
        if let HirExpression::Variable(var_name) = target {
            if ctx.is_pointer(var_name) {
                // DECY-127: Strip nested unsafe from value_code to avoid warnings
                let clean_value = strip_unsafe(&value_code);
                // DECY-143: Add SAFETY comment
                return Self::unsafe_stmt(
                    &format!("*{} = {}", target_code, clean_value),
                    "pointer is valid, aligned, and not aliased during write",
                );
            }
        }

        // DECY-128: Check if target is Dereference(Variable) where variable holds a raw pointer
        // e.g., **ptr = val where ptr is &mut *mut T
        // *ptr yields *mut T (raw pointer), so **ptr needs unsafe
        if let HirExpression::Dereference(inner) = target {
            if let HirExpression::Variable(var_name) = &**inner {
                // Check if dereferencing yields a raw pointer
                // This happens when var_type is Reference to Pointer or Pointer to Pointer
                if let Some(var_type) = ctx.get_type(var_name) {
                    let yields_raw_ptr = match var_type {
                        HirType::Reference { inner: ref_inner, .. } => {
                            matches!(&**ref_inner, HirType::Pointer(_))
                        }
                        HirType::Pointer(ptr_inner) => {
                            matches!(&**ptr_inner, HirType::Pointer(_))
                        }
                        _ => false,
                    };
                    if yields_raw_ptr {
                        let clean_value = strip_unsafe(&value_code);
                        // DECY-143: Add SAFETY comment
                        return Self::unsafe_stmt(
                            &format!("*{} = {}", target_code, clean_value),
                            "double pointer dereference - inner pointer is valid and writable",
                        );
                    }
                }
            }
        }

        format!("*{} = {};", target_code, value_code)
    }

    /// Generate an array index assignment statement.
    #[allow(clippy::borrowed_box)]
    fn generate_array_index_assignment_statement(
        &self,
        array: &Box<HirExpression>,
        index: &Box<HirExpression>,
        value: &HirExpression,
        ctx: &mut TypeContext,
    ) -> String {
        // Infer the type of array[index] for null pointer detection
        let target_expr =
            HirExpression::ArrayIndex { array: array.clone(), index: index.clone() };
        let target_type = ctx.infer_expression_type(&target_expr);

        // DECY-165: Check if array is a raw pointer - if so, use unsafe pointer arithmetic
        let is_raw_pointer = if let HirExpression::Variable(var_name) = &**array {
            ctx.is_pointer(var_name)
        } else {
            // Use type inference for complex expressions like sb->data
            matches!(ctx.infer_expression_type(array), Some(HirType::Pointer(_)))
        };

        // DECY-223: Check for global array BEFORE generating code
        let is_global_array = if let HirExpression::Variable(var_name) = &**array {
            ctx.is_global(var_name)
        } else {
            false
        };

        // Generate array code - get raw name for globals to avoid double unsafe
        let array_code = if is_global_array {
            if let HirExpression::Variable(var_name) = &**array {
                var_name.clone()
            } else {
                self.generate_expression_with_context(array, ctx)
            }
        } else {
            self.generate_expression_with_context(array, ctx)
        };
        let index_code = self.generate_expression_with_context(index, ctx);
        let mut value_code =
            self.generate_expression_with_target_type(value, ctx, target_type.as_ref());

        // DECY-210: Handle int-to-char coercion for array element assignment
        // In C, s[i] = (n % 10) + '0' works because char is widened to int then truncated back
        // In Rust, we need explicit cast when assigning int to u8 element
        if matches!(target_type, Some(HirType::Char)) {
            let value_type = ctx.infer_expression_type(value);
            if matches!(value_type, Some(HirType::Int)) {
                value_code = format!("({}) as u8", value_code);
            }
        }

        if is_raw_pointer {
            // Raw pointer indexing: arr[i] = v becomes unsafe { *arr.add(i as usize) = v }
            // DECY-143: Add SAFETY comment
            Self::unsafe_stmt(
                &format!("*{}.add(({}) as usize) = {}", array_code, index_code, value_code),
                "index is within bounds of allocated array",
            )
        } else {
            // DECY-072: Cast index to usize for slice indexing
            // DECY-150: Wrap index in parens to handle operator precedence
            // DECY-223: Wrap global array assignment in unsafe block
            if is_global_array {
                format!(
                    "unsafe {{ {}[({}) as usize] = {}; }}",
                    array_code, index_code, value_code
                )
            } else {
                format!("{}[({}) as usize] = {};", array_code, index_code, value_code)
            }
        }
    }

    /// Generate a field assignment statement.
    fn generate_field_assignment_statement(
        &self,
        object: &HirExpression,
        field: &str,
        value: &HirExpression,
        ctx: &mut TypeContext,
    ) -> String {
        // DECY-227: Escape reserved keywords in field names
        let escaped_field = escape_rust_keyword(field);
        // Look up field type for null pointer detection
        let field_type = ctx.get_field_type(object, field);
        let obj_code = self.generate_expression_with_context(object, ctx);
        let value_code =
            self.generate_expression_with_target_type(value, ctx, field_type.as_ref());

        // DECY-119: Check if object is a raw pointer - need unsafe deref
        let obj_type = if let HirExpression::Variable(name) = object {
            ctx.get_type(name)
        } else {
            None
        };

        if matches!(obj_type, Some(HirType::Pointer(_))) {
            // Raw pointer field assignment needs unsafe block
            // DECY-143: Add SAFETY comment
            Self::unsafe_stmt(
                &format!("(*{}).{} = {}", obj_code, escaped_field, value_code),
                "pointer is non-null and points to valid struct with exclusive access",
            )
        } else {
            // DECY-261: Check if object is a global struct - use single unsafe block
            if let HirExpression::Variable(name) = object {
                if ctx.is_global(name) {
                    // Strip nested unsafe from value_code
                    fn strip_nested_unsafe(code: &str) -> String {
                        code.replace("unsafe { ", "").replacen(
                            " }",
                            "",
                            code.matches("unsafe { ").count(),
                        )
                    }
                    let clean_value = strip_nested_unsafe(&value_code);
                    return format!(
                        "unsafe {{ {}.{} = {}; }}",
                        name, escaped_field, clean_value
                    );
                }
            }
            // Regular struct field assignment
            format!("{}.{} = {};", obj_code, escaped_field, value_code)
        }
    }
}
