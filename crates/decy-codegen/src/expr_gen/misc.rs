//! Miscellaneous expression generation: pointer field access, array index, sizeof,
//! memory allocation, cast, compound literals, increment/decrement, ternary,
//! and utility/format helper functions.

use super::{escape_rust_keyword, CodeGenerator, TypeContext};
use decy_hir::{BinaryOperator, HirExpression, HirType};

impl CodeGenerator {
    pub(crate) fn gen_expr_pointer_field_access(
        &self,
        pointer: &HirExpression,
        field: &str,
        ctx: &TypeContext,
    ) -> String {
        let escaped_field = escape_rust_keyword(field);
        match pointer {
            HirExpression::PointerFieldAccess { .. }
            | HirExpression::FieldAccess { .. } => {
                format!(
                    "{}.{}",
                    self.generate_expression_with_context(pointer, ctx),
                    escaped_field
                )
            }
            _ => {
                let ptr_code = self.generate_expression_with_context(pointer, ctx);
                if let HirExpression::Variable(var_name) = pointer {
                    if ctx.is_pointer(var_name) {
                        return Self::unsafe_block(
                            &format!("(*{}).{}", ptr_code, escaped_field),
                            "pointer is non-null and points to valid struct",
                        );
                    }
                }
                format!("(*{}).{}", ptr_code, escaped_field)
            }
        }
    }

    pub(crate) fn gen_expr_array_index(
        &self,
        array: &HirExpression,
        index: &HirExpression,
        ctx: &TypeContext,
    ) -> String {
        let is_global_array = if let HirExpression::Variable(var_name) = array {
            ctx.is_global(var_name)
        } else {
            false
        };

        let is_raw_pointer = if let HirExpression::Variable(var_name) = array {
            ctx.is_pointer(var_name)
        } else {
            matches!(ctx.infer_expression_type(array), Some(HirType::Pointer(_)))
        };

        let array_code = if is_global_array {
            if let HirExpression::Variable(var_name) = array {
                var_name.clone()
            } else {
                self.generate_expression_with_context(array, ctx)
            }
        } else {
            self.generate_expression_with_context(array, ctx)
        };
        let index_code = self.generate_expression_with_context(index, ctx);

        if is_raw_pointer {
            return Self::unsafe_block(
                &format!("*{}.add(({}) as usize)", array_code, index_code),
                "index is within bounds of allocated array",
            );
        }

        let index_expr = format!("{}[({}) as usize]", array_code, index_code);
        if is_global_array {
            format!("unsafe {{ {} }}", index_expr)
        } else {
            index_expr
        }
    }

    pub(crate) fn gen_expr_sizeof(&self, type_name: &str, ctx: &TypeContext) -> String {
        let trimmed = type_name.trim();

        let normalized = trimmed.strip_prefix("struct ").unwrap_or(trimmed);
        let parts: Vec<&str> = normalized.split_whitespace().collect();

        let is_struct_field_sizeof = parts.len() == 2 && ctx.structs.contains_key(parts[0]);

        let is_member_access = trimmed.contains(' ')
            && !trimmed.starts_with("struct ")
            && !trimmed.starts_with("unsigned ")
            && !trimmed.starts_with("signed ")
            && !trimmed.starts_with("long ")
            && !trimmed.starts_with("short ");

        if is_struct_field_sizeof {
            let struct_name = parts[0];
            let field_name = parts[1];
            let field_type = ctx.structs.get(struct_name).and_then(|fields| {
                fields.iter().find(|(name, _)| name == field_name).map(|(_, ty)| ty.clone())
            });
            if let Some(field_type) = field_type {
                let rust_type = Self::map_type(&field_type);
                format!("std::mem::size_of::<{}>() as i32", rust_type)
            } else {
                format!("std::mem::size_of::<{}>() as i32", field_name)
            }
        } else if is_member_access {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                let struct_name = parts[0];
                let field_name = parts[1];
                let field_type = ctx.structs.get(struct_name).and_then(|fields| {
                    fields
                        .iter()
                        .find(|(name, _)| name == field_name)
                        .map(|(_, ty)| ty.clone())
                });
                if let Some(field_type) = field_type {
                    let rust_type = Self::map_type(&field_type);
                    format!("std::mem::size_of::<{}>() as i32", rust_type)
                } else if ctx.get_type(struct_name).is_some() {
                    let field = parts[1..].join(".");
                    format!("std::mem::size_of_val(&(*{}).{}) as i32", struct_name, field)
                } else {
                    let rust_type = self.map_sizeof_type(type_name);
                    format!("std::mem::size_of::<{}>() as i32", rust_type)
                }
            } else {
                let rust_type = self.map_sizeof_type(type_name);
                format!("std::mem::size_of::<{}>() as i32", rust_type)
            }
        } else {
            if ctx.get_type(trimmed).is_some() {
                format!("std::mem::size_of_val(&{}) as i32", trimmed)
            } else {
                let rust_type = self.map_sizeof_type(type_name);
                format!("std::mem::size_of::<{}>() as i32", rust_type)
            }
        }
    }

    pub(crate) fn gen_expr_calloc(
        &self,
        count: &HirExpression,
        element_type: &HirType,
        ctx: &TypeContext,
    ) -> String {
        let count_code = self.generate_expression_with_context(count, ctx);

        let default_value = match element_type {
            HirType::Int => "0i32",
            HirType::UnsignedInt => "0u32",
            HirType::Float => "0.0f32",
            HirType::Double => "0.0f64",
            HirType::Char => "0u8",
            HirType::SignedChar => "0i8",
            _ => &Self::default_value_for_type(element_type),
        };

        format!("vec![{}; {}]", default_value, count_code)
    }

    pub(crate) fn gen_expr_malloc(&self, size: &HirExpression, ctx: &TypeContext) -> String {
        if let HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::Multiply,
            left,
            ..
        } = size
        {
            let capacity_code = self.generate_expression_with_context(left, ctx);
            format!("Vec::with_capacity({})", capacity_code)
        } else {
            "Box::new(0i32)".to_string()
        }
    }

    pub(crate) fn gen_expr_realloc(
        &self,
        pointer: &HirExpression,
        new_size: &HirExpression,
        ctx: &TypeContext,
    ) -> String {
        if matches!(*pointer, HirExpression::NullLiteral) {
            if let HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Multiply,
                left,
                ..
            } = new_size
            {
                let count_code = self.generate_expression_with_context(left, ctx);
                format!("vec![0i32; {}]", count_code)
            } else {
                "Vec::new()".to_string()
            }
        } else {
            self.generate_expression_with_context(pointer, ctx)
        }
    }

    pub(crate) fn gen_expr_string_method_call(
        &self,
        receiver: &HirExpression,
        method: &str,
        arguments: &[HirExpression],
        ctx: &TypeContext,
    ) -> String {
        let receiver_code = self.generate_expression_with_context(receiver, ctx);
        if arguments.is_empty() {
            if method == "len" {
                format!("{}.{}() as i32", receiver_code, method)
            } else {
                format!("{}.{}()", receiver_code, method)
            }
        } else {
            let args: Vec<String> = arguments
                .iter()
                .map(|arg| {
                    if method == "clone_into" {
                        format!("&mut {}", self.generate_expression_with_context(arg, ctx))
                    } else {
                        self.generate_expression_with_context(arg, ctx)
                    }
                })
                .collect();
            format!("{}.{}({})", receiver_code, method, args.join(", "))
        }
    }

    pub(crate) fn gen_expr_cast(
        &self,
        cast_target: &HirType,
        expr: &HirExpression,
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> String {
        if let Some(vec_type @ HirType::Vec(_)) = target_type {
            if Self::is_any_malloc_or_calloc(expr) {
                return self.generate_expression_with_target_type(
                    expr,
                    ctx,
                    Some(vec_type),
                );
            }
        }

        let expr_code = self.generate_expression_with_context(expr, ctx);
        let rust_type = Self::map_type(cast_target);

        let expr_str = if matches!(*expr, HirExpression::BinaryOp { .. }) {
            format!("({})", expr_code)
        } else {
            expr_code.clone()
        };

        let is_address_of = matches!(*expr, HirExpression::AddressOf(_))
            || matches!(
                expr,
                HirExpression::UnaryOp { op: decy_hir::UnaryOperator::AddressOf, .. }
            );

        let is_integer_target =
            matches!(cast_target, HirType::Int | HirType::UnsignedInt | HirType::Char);

        if is_address_of && is_integer_target {
            format!("{} as *const _ as isize as {}", expr_str, rust_type)
        } else {
            format!("{} as {}", expr_str, rust_type)
        }
    }

    pub(crate) fn gen_expr_compound_literal(
        &self,
        literal_type: &HirType,
        initializers: &[HirExpression],
        ctx: &TypeContext,
    ) -> String {
        match literal_type {
            HirType::Struct(name) => {
                if initializers.is_empty() {
                    format!("{} {{}}", name)
                } else {
                    let struct_fields = ctx.structs.get(name);
                    let num_struct_fields = struct_fields.map(|f| f.len()).unwrap_or(0);

                    let fields: Vec<String> = initializers
                        .iter()
                        .enumerate()
                        .map(|(i, init)| {
                            let init_code =
                                self.generate_expression_with_context(init, ctx);
                            let field_name = struct_fields
                                .and_then(|f| f.get(i))
                                .map(|(name, _)| name.as_str())
                                .unwrap_or_else(|| {
                                    Box::leak(format!("field{}", i).into_boxed_str())
                                });
                            format!("{}: {}", field_name, init_code)
                        })
                        .collect();

                    if initializers.len() < num_struct_fields {
                        format!(
                            "{} {{ {}, ..Default::default() }}",
                            name,
                            fields.join(", ")
                        )
                    } else {
                        format!("{} {{ {} }}", name, fields.join(", "))
                    }
                }
            }
            HirType::Array { element_type, size } => {
                if initializers.is_empty() {
                    if let Some(n) = size {
                        format!("[{}; {}]", Self::default_value_for_type(element_type), n)
                    } else {
                        "[]".to_string()
                    }
                } else {
                    let elements: Vec<String> = initializers
                        .iter()
                        .map(|init| self.generate_expression_with_context(init, ctx))
                        .collect();

                    if let Some(n) = size {
                        if elements.len() == 1 {
                            format!("[{}; {}]", elements[0], *n)
                        } else if elements.len() < *n {
                            let mut padded = elements.clone();
                            let default = Self::default_value_for_type(element_type);
                            while padded.len() < *n {
                                padded.push(default.clone());
                            }
                            format!("[{}]", padded.join(", "))
                        } else {
                            format!("[{}]", elements.join(", "))
                        }
                    } else {
                        format!("[{}]", elements.join(", "))
                    }
                }
            }
            _ => {
                format!("/* Compound literal of type {} */", Self::map_type(literal_type))
            }
        }
    }

    pub(crate) fn gen_expr_post_increment(&self, operand: &HirExpression, ctx: &TypeContext) -> String {
        if let HirExpression::Variable(var_name) = operand {
            if let Some(var_type) = ctx.get_type(var_name) {
                if matches!(var_type, HirType::StringReference | HirType::StringLiteral) {
                    let operand_code = self.generate_expression_with_context(operand, ctx);
                    return format!(
                        "{{ let __tmp = {var}.as_bytes()[0] as u32; {var} = &{var}[1..]; __tmp }}",
                        var = operand_code
                    );
                }
            }
        }

        if let HirExpression::Dereference(inner) = operand {
            if let HirExpression::Variable(var_name) = &**inner {
                if ctx.is_pointer(var_name) {
                    return format!(
                        "{{ let __tmp = unsafe {{ *{} }}; unsafe {{ *{} += 1 }}; __tmp }}",
                        var_name, var_name
                    );
                }
            }
        }

        let operand_code = self.generate_expression_with_context(operand, ctx);

        let operand_type = ctx.infer_expression_type(operand);
        if matches!(operand_type, Some(HirType::Pointer(_))) {
            format!(
                "{{ let __tmp = {operand}; {operand} = {operand}.wrapping_add(1); __tmp }}",
                operand = operand_code
            )
        } else {
            format!(
                "{{ let __tmp = {operand}; {operand} += 1; __tmp }}",
                operand = operand_code
            )
        }
    }

    pub(crate) fn gen_expr_pre_increment(&self, operand: &HirExpression, ctx: &TypeContext) -> String {
        if let HirExpression::Dereference(inner) = operand {
            if let HirExpression::Variable(var_name) = &**inner {
                if ctx.is_pointer(var_name) {
                    return format!(
                        "{{ unsafe {{ *{} += 1 }}; unsafe {{ *{} }} }}",
                        var_name, var_name
                    );
                }
            }
        }

        let operand_code = self.generate_expression_with_context(operand, ctx);
        let operand_type = ctx.infer_expression_type(operand);
        if matches!(operand_type, Some(HirType::Pointer(_))) {
            format!(
                "{{ {operand} = {operand}.wrapping_add(1); {operand} }}",
                operand = operand_code
            )
        } else {
            format!("{{ {operand} += 1; {operand} }}", operand = operand_code)
        }
    }

    pub(crate) fn gen_expr_post_decrement(&self, operand: &HirExpression, ctx: &TypeContext) -> String {
        if let HirExpression::Dereference(inner) = operand {
            if let HirExpression::Variable(var_name) = &**inner {
                if ctx.is_pointer(var_name) {
                    return format!(
                        "{{ let __tmp = unsafe {{ *{} }}; unsafe {{ *{} -= 1 }}; __tmp }}",
                        var_name, var_name
                    );
                }
            }
        }

        let operand_code = self.generate_expression_with_context(operand, ctx);
        let operand_type = ctx.infer_expression_type(operand);
        if matches!(operand_type, Some(HirType::Pointer(_))) {
            format!(
                "{{ let __tmp = {operand}; {operand} = {operand}.wrapping_sub(1); __tmp }}",
                operand = operand_code
            )
        } else {
            format!(
                "{{ let __tmp = {operand}; {operand} -= 1; __tmp }}",
                operand = operand_code
            )
        }
    }

    pub(crate) fn gen_expr_pre_decrement(&self, operand: &HirExpression, ctx: &TypeContext) -> String {
        if let HirExpression::Dereference(inner) = operand {
            if let HirExpression::Variable(var_name) = &**inner {
                if ctx.is_pointer(var_name) {
                    return format!(
                        "{{ unsafe {{ *{} -= 1 }}; unsafe {{ *{} }} }}",
                        var_name, var_name
                    );
                }
            }
        }

        let operand_code = self.generate_expression_with_context(operand, ctx);
        let operand_type = ctx.infer_expression_type(operand);
        if matches!(operand_type, Some(HirType::Pointer(_))) {
            format!(
                "{{ {operand} = {operand}.wrapping_sub(1); {operand} }}",
                operand = operand_code
            )
        } else {
            format!("{{ {operand} -= 1; {operand} }}", operand = operand_code)
        }
    }

    pub(crate) fn gen_expr_ternary(
        &self,
        condition: &HirExpression,
        then_expr: &HirExpression,
        else_expr: &HirExpression,
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> String {
        let cond_code = self.generate_expression_with_context(condition, ctx);
        let then_code =
            self.generate_expression_with_target_type(then_expr, ctx, target_type);
        let else_code =
            self.generate_expression_with_target_type(else_expr, ctx, target_type);

        let cond_bool = if Self::is_boolean_expression(condition) {
            cond_code
        } else {
            format!("{} != 0", cond_code)
        };

        format!("if {} {{ {} }} else {{ {} }}", cond_bool, then_code, else_code)
    }

    /// Convert unary operator to string.
    pub(crate) fn unary_operator_to_string(op: &decy_hir::UnaryOperator) -> &'static str {
        use decy_hir::UnaryOperator;
        match op {
            UnaryOperator::Minus => "-",
            UnaryOperator::LogicalNot => "!",
            // DECY-193: In Rust, bitwise NOT is ! (same as logical NOT for bool)
            UnaryOperator::BitwiseNot => "!",
            UnaryOperator::AddressOf => "&",
            // Post/Pre-increment/decrement are handled as block expressions
            // in generate_expression_with_context, so should never reach here
            UnaryOperator::PostIncrement
            | UnaryOperator::PostDecrement
            | UnaryOperator::PreIncrement
            | UnaryOperator::PreDecrement => {
                unreachable!("Increment/decrement operators should be handled as block expressions")
            }
        }
    }

    /// Check if an expression already produces a boolean result.
    /// Used to avoid redundant `!= 0` conversions for expressions that are already boolean.
    pub(crate) fn is_boolean_expression(expr: &HirExpression) -> bool {
        match expr {
            // Comparison operators always produce bool
            HirExpression::BinaryOp { op, .. } => matches!(
                op,
                BinaryOperator::Equal
                    | BinaryOperator::NotEqual
                    | BinaryOperator::LessThan
                    | BinaryOperator::GreaterThan
                    | BinaryOperator::LessEqual
                    | BinaryOperator::GreaterEqual
                    | BinaryOperator::LogicalAnd
                    | BinaryOperator::LogicalOr
            ),
            // Logical NOT produces bool
            HirExpression::UnaryOp { op: decy_hir::UnaryOperator::LogicalNot, .. } => true,
            // Other expressions are assumed to be non-boolean (integers, etc.)
            _ => false,
        }
    }

    /// DECY-138: Check if expression is a dereference of a string variable.
    /// Returns the variable name if it's a string dereference, None otherwise.
    /// Used for string iteration pattern: while (*str) → while !str.is_empty()
    pub(crate) fn get_string_deref_var(expr: &HirExpression, ctx: &TypeContext) -> Option<String> {
        match expr {
            // Direct dereference: *str
            HirExpression::Dereference(inner) => {
                if let HirExpression::Variable(var_name) = &**inner {
                    if let Some(var_type) = ctx.get_type(var_name) {
                        if matches!(var_type, HirType::StringReference | HirType::StringLiteral) {
                            return Some(var_name.clone());
                        }
                    }
                }
                None
            }
            // Comparison with 0: *str != 0 or *str == 0
            HirExpression::BinaryOp { op, left, right } => {
                if matches!(op, BinaryOperator::NotEqual | BinaryOperator::Equal) {
                    // Check: *str != 0 or *str == 0
                    if let HirExpression::IntLiteral(0) = &**right {
                        return Self::get_string_deref_var(left, ctx);
                    }
                    if let HirExpression::IntLiteral(0) = &**left {
                        return Self::get_string_deref_var(right, ctx);
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Convert binary operator to string.
    pub(crate) fn binary_operator_to_string(op: &BinaryOperator) -> &'static str {
        match op {
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Modulo => "%",
            BinaryOperator::Equal => "==",
            BinaryOperator::NotEqual => "!=",
            BinaryOperator::LessThan => "<",
            BinaryOperator::GreaterThan => ">",
            BinaryOperator::LessEqual => "<=",
            BinaryOperator::GreaterEqual => ">=",
            BinaryOperator::LogicalAnd => "&&",
            BinaryOperator::LogicalOr => "||",
            // DECY-137: Bitwise and shift operators
            BinaryOperator::LeftShift => "<<",
            BinaryOperator::RightShift => ">>",
            BinaryOperator::BitwiseAnd => "&",
            BinaryOperator::BitwiseOr => "|",
            BinaryOperator::BitwiseXor => "^",
            // DECY-195: Assignment operator (for embedded assignments)
            BinaryOperator::Assign => "=",
            // DECY-224: Comma operator (rarely used in expressions in Rust)
            BinaryOperator::Comma => ",",
        }
    }

    /// DECY-231: Check if an expression is a malloc/calloc call, including when wrapped in casts.
    /// Handles patterns like:
    /// - malloc(size)
    /// - calloc(count, size)
    /// - (node *)malloc(sizeof(struct node))
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn is_malloc_expression(expr: &HirExpression) -> bool {
        match expr {
            HirExpression::Malloc { .. } => true,
            HirExpression::Calloc { .. } => true,
            HirExpression::FunctionCall { function, .. } => {
                function == "malloc" || function == "calloc"
            }
            // DECY-231: Check for casts wrapping malloc
            HirExpression::Cast { expr, .. } => Self::is_malloc_expression(expr),
            _ => false,
        }
    }

    /// DECY-231: Check if a malloc expression has an array pattern argument (n * sizeof(T)).
    /// This distinguishes single-element allocations (use Box) from array allocations (use Vec).
    pub(crate) fn is_malloc_array_pattern(expr: &HirExpression) -> bool {
        match expr {
            HirExpression::FunctionCall { function, arguments }
                if function == "malloc" || function == "calloc" =>
            {
                arguments
                    .first()
                    .map(|arg| {
                        matches!(
                            arg,
                            HirExpression::BinaryOp { op: decy_hir::BinaryOperator::Multiply, .. }
                        )
                    })
                    .unwrap_or(false)
            }
            HirExpression::Malloc { size } => {
                matches!(
                    size.as_ref(),
                    HirExpression::BinaryOp { op: decy_hir::BinaryOperator::Multiply, .. }
                )
            }
            // DECY-231: Unwrap Cast expressions to check the inner malloc
            HirExpression::Cast { expr, .. } => Self::is_malloc_array_pattern(expr),
            _ => false,
        }
    }

    /// Get default value for a type (for uninitialized variables).
    pub(crate) fn default_value_for_type(hir_type: &HirType) -> String {
        match hir_type {
            HirType::Void => "()".to_string(),
            HirType::Bool => "false".to_string(),
            HirType::Int => "0i32".to_string(),
            HirType::UnsignedInt => "0u32".to_string(), // DECY-158
            HirType::Float => "0.0f32".to_string(),
            HirType::Double => "0.0f64".to_string(),
            HirType::Char => "0u8".to_string(),
            HirType::SignedChar => "0i8".to_string(), // DECY-250
            HirType::Pointer(_) => "std::ptr::null_mut()".to_string(),
            HirType::Box(inner) => {
                // Box types should not use default values, they should be initialized with Box::new
                // This is just a fallback
                format!("Box::new({})", Self::default_value_for_type(inner))
            }
            HirType::Vec(_) => {
                // Vec types default to empty Vec
                "Vec::new()".to_string()
            }
            HirType::Option(_) => {
                // Option types default to None
                "None".to_string()
            }
            HirType::Reference { .. } => {
                // References cannot have default values - they must always be initialized
                // This should never be reached in valid code
                panic!("References must be initialized and cannot have default values")
            }
            HirType::Struct(name) => {
                // Use ::default() for struct initialization
                // For structs with large arrays (>32 elements), see DECY-123 special handling
                // in variable declaration codegen where struct definitions are available
                format!("{}::default()", name)
            }
            HirType::Enum(name) => {
                format!("{}::default()", name)
            }
            HirType::Array { element_type, size } => {
                if let Some(n) = size {
                    format!("[{}; {}]", Self::default_value_for_type(element_type), n)
                } else {
                    // Unsized arrays cannot have default values - this should be initialized from parameter
                    panic!("Unsized arrays must be initialized and cannot have default values")
                }
            }
            HirType::FunctionPointer { .. } => {
                // DECY-202: Function pointers default to None when wrapped in Option
                // For local variables, return None which will be unwrapped if needed
                "None".to_string()
            }
            HirType::StringLiteral => {
                // String literals default to empty string slice
                r#""""#.to_string()
            }
            HirType::OwnedString => {
                // Owned strings default to String::new()
                "String::new()".to_string()
            }
            HirType::StringReference => {
                // String references default to empty string slice
                r#""""#.to_string()
            }
            HirType::Union(_) => {
                // Unions will be transformed to enums
                // Default to the first variant's default value
                panic!("Union types must be initialized and cannot have default values")
            }
            // DECY-172: Type aliases use 0 as default (for size_t/ssize_t/ptrdiff_t)
            HirType::TypeAlias(name) => {
                // size_t/ssize_t/ptrdiff_t default to 0
                match name.as_str() {
                    "size_t" => "0usize".to_string(),
                    "ssize_t" | "ptrdiff_t" => "0isize".to_string(),
                    _ => "0".to_string(),
                }
            }
        }
    }

    /// Convert C printf format specifiers to Rust format specifiers.
    /// DECY-119: %d → {}, %s → {}, %f → {}, etc.
    pub(crate) fn convert_c_format_to_rust(c_fmt: &str) -> String {
        // If it's a quoted string literal, process the contents
        let trimmed = c_fmt.trim();
        if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
            let inner = &trimmed[1..trimmed.len() - 1];
            let converted = Self::convert_format_specifiers(inner);
            format!("\"{}\"", converted)
        } else {
            // Not a string literal, return as-is
            c_fmt.to_string()
        }
    }

    /// DECY-193: Convert C format specifiers to Rust format specifiers.
    /// Handles width, precision, and flags like %02X, %10.3f, etc.
    pub(crate) fn convert_format_specifiers(input: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '%' {
                if i + 1 < chars.len() && chars[i + 1] == '%' {
                    // %% -> %
                    result.push('%');
                    i += 2;
                    continue;
                }

                // Parse format specifier: %[flags][width][.precision][length]specifier
                let start = i;
                i += 1; // skip %

                let mut flags = String::new();
                let mut width = String::new();
                let mut precision = String::new();

                // Parse flags: -, +, space, #, 0
                while i < chars.len() && "-+ #0".contains(chars[i]) {
                    if chars[i] == '0' {
                        flags.push('0'); // zero-padding
                    }
                    // Skip other flags for now (- is left-align, + is sign, etc.)
                    i += 1;
                }

                // Parse width
                while i < chars.len() && chars[i].is_ascii_digit() {
                    width.push(chars[i]);
                    i += 1;
                }

                // Parse precision
                if i < chars.len() && chars[i] == '.' {
                    i += 1;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        precision.push(chars[i]);
                        i += 1;
                    }
                }

                // Skip length modifiers: h, hh, l, ll, L, z, j, t
                while i < chars.len() && "hlLzjt".contains(chars[i]) {
                    i += 1;
                }

                // Parse specifier
                if i < chars.len() {
                    let spec = chars[i];
                    i += 1;

                    // Build Rust format specifier
                    let rust_spec = match spec {
                        'd' | 'i' | 'u' => {
                            if !width.is_empty() || !flags.is_empty() {
                                format!("{{:{}{}}}", flags, width)
                            } else {
                                "{}".to_string()
                            }
                        }
                        'x' => {
                            if !width.is_empty() || !flags.is_empty() {
                                format!("{{:{}{}x}}", flags, width)
                            } else {
                                "{:x}".to_string()
                            }
                        }
                        'X' => {
                            if !width.is_empty() || !flags.is_empty() {
                                format!("{{:{}{}X}}", flags, width)
                            } else {
                                "{:X}".to_string()
                            }
                        }
                        'o' => {
                            if !width.is_empty() || !flags.is_empty() {
                                format!("{{:{}{}o}}", flags, width)
                            } else {
                                "{:o}".to_string()
                            }
                        }
                        // DECY-247: Binary format specifier (non-standard but common extension)
                        'b' => {
                            if !width.is_empty() || !flags.is_empty() {
                                format!("{{:{}{}b}}", flags, width)
                            } else {
                                "{:b}".to_string()
                            }
                        }
                        'f' | 'F' => {
                            if !precision.is_empty() {
                                if !width.is_empty() {
                                    format!("{{:{}{}.{}}}", flags, width, precision)
                                } else {
                                    format!("{{:.{}}}", precision)
                                }
                            } else if !width.is_empty() {
                                format!("{{:{}{}}}", flags, width)
                            } else {
                                "{}".to_string()
                            }
                        }
                        'e' => "{:e}".to_string(),
                        'E' => "{:E}".to_string(),
                        'g' | 'G' => "{}".to_string(),
                        's' => {
                            if !width.is_empty() {
                                format!("{{:{}}}", width)
                            } else {
                                "{}".to_string()
                            }
                        }
                        'c' => "{}".to_string(),
                        'p' => "{:p}".to_string(),
                        _ => {
                            // Unknown specifier, keep original
                            input[start..i].to_string()
                        }
                    };
                    result.push_str(&rust_spec);
                } else {
                    // Incomplete format specifier at end of string
                    result.push_str(&input[start..]);
                }
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }

        result
    }

    /// DECY-187: Find positions of %s format specifiers in a format string.
    /// Returns 0-indexed positions corresponding to printf arguments.
    pub(crate) fn find_string_format_positions(fmt: &str) -> Vec<usize> {
        let mut positions = Vec::new();
        let mut arg_index = 0;
        let trimmed = fmt.trim();

        // Extract inner content if quoted
        let inner = if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
            &trimmed[1..trimmed.len() - 1]
        } else {
            trimmed
        };

        let chars: Vec<char> = inner.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '%' && i + 1 < chars.len() {
                let next = chars[i + 1];
                // Skip %% (literal percent)
                if next == '%' {
                    i += 2;
                    continue;
                }
                // Check for format specifiers
                // Skip width/precision modifiers and length specifiers
                let mut j = i + 1;
                while j < chars.len()
                    && (chars[j].is_ascii_digit()
                        || chars[j] == '.'
                        || chars[j] == '-'
                        || chars[j] == '+'
                        || chars[j] == ' '
                        || chars[j] == '#'
                        || chars[j] == '*')
                {
                    j += 1;
                }
                // Skip length modifiers (l, ll, h, hh, z, etc.)
                while j < chars.len()
                    && (chars[j] == 'l'
                        || chars[j] == 'h'
                        || chars[j] == 'z'
                        || chars[j] == 'j'
                        || chars[j] == 't'
                        || chars[j] == 'L')
                {
                    j += 1;
                }
                // Now we should be at the conversion specifier
                if j < chars.len() {
                    let specifier = chars[j];
                    if specifier == 's' {
                        positions.push(arg_index);
                    }
                    // Count this as an argument position (for d, i, u, f, s, c, p, x, X, o, e, E, g, G, n)
                    if specifier == 'd'
                        || specifier == 'i'
                        || specifier == 'u'
                        || specifier == 'f'
                        || specifier == 's'
                        || specifier == 'c'
                        || specifier == 'p'
                        || specifier == 'x'
                        || specifier == 'X'
                        || specifier == 'o'
                        || specifier == 'e'
                        || specifier == 'E'
                        || specifier == 'g'
                        || specifier == 'G'
                        || specifier == 'n'
                        || specifier == 'a'
                        || specifier == 'A'
                    {
                        arg_index += 1;
                    }
                    i = j + 1;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
        positions
    }

    /// DECY-187: Wrap a char* argument with CStr conversion for safe printing.
    /// DECY-199: Use .as_ptr() for arrays which can't be cast directly to pointers.
    pub(crate) fn wrap_with_cstr(arg: &str) -> String {
        format!(
            "unsafe {{ std::ffi::CStr::from_ptr({}.as_ptr() as *const i8).to_str().unwrap_or(\"\") }}",
            arg
        )
    }

    /// DECY-221: Wrap a raw pointer (*mut u8) with CStr conversion for safe printing.
    /// Unlike wrap_with_cstr, this doesn't call .as_ptr() since the arg is already a pointer.
    pub(crate) fn wrap_raw_ptr_with_cstr(arg: &str) -> String {
        format!(
            "unsafe {{ std::ffi::CStr::from_ptr({} as *const i8).to_str().unwrap_or(\"\") }}",
            arg
        )
    }

    /// DECY-192: Check if expression is a ternary that returns string literals.
    /// Such expressions should not be wrapped with CStr since they return &str directly in Rust.
    pub(crate) fn is_string_ternary(expr: &HirExpression) -> bool {
        if let HirExpression::Ternary { then_expr, else_expr, .. } = expr {
            matches!(**then_expr, HirExpression::StringLiteral(_))
                && matches!(**else_expr, HirExpression::StringLiteral(_))
        } else {
            false
        }
    }
}
