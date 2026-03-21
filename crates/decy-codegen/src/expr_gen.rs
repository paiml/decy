//! Expression code generation methods for CodeGenerator.
//!
//! Contains all methods related to generating Rust code from HIR expressions,
//! including binary operations, unary operations, function calls, literals,
//! format string handling, and type coercion.

use super::{escape_rust_keyword, CodeGenerator, TypeContext};
use decy_hir::{BinaryOperator, HirExpression, HirType};

impl CodeGenerator {
    /// Generate code for an expression.
    #[allow(clippy::only_used_in_recursion)]
    pub fn generate_expression(&self, expr: &HirExpression) -> String {
        self.generate_expression_with_context(expr, &TypeContext::new())
    }

    /// Generate code for an expression with type context for pointer arithmetic.
    #[allow(clippy::only_used_in_recursion)]
    pub(crate) fn generate_expression_with_context(&self, expr: &HirExpression, ctx: &TypeContext) -> String {
        self.generate_expression_with_target_type(expr, ctx, None)
    }

    /// Generate code for an expression with optional target type hint for null pointer detection.
    /// If target_type is Some(HirType::Pointer(_)) and expr is IntLiteral(0), generates std::ptr::null_mut().
    #[allow(clippy::only_used_in_recursion)]
    pub(crate) fn generate_expression_with_target_type(
        &self,
        expr: &HirExpression,
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> String {
        match expr {
            HirExpression::IntLiteral(val) => self.gen_expr_int_literal(*val, target_type),
            HirExpression::FloatLiteral(val) => self.gen_expr_float_literal(val, target_type),
            HirExpression::AddressOf(inner) => {
                self.gen_expr_address_of(inner, ctx, target_type)
            }
            HirExpression::UnaryOp { op: decy_hir::UnaryOperator::AddressOf, operand } => {
                self.gen_expr_unary_address_of(operand, ctx, target_type)
            }
            HirExpression::UnaryOp { op: decy_hir::UnaryOperator::LogicalNot, operand } => {
                self.gen_expr_unary_logical_not(operand, ctx, target_type)
            }
            HirExpression::StringLiteral(s) => {
                self.gen_expr_string_literal(s, target_type)
            }
            HirExpression::CharLiteral(c) => Self::gen_expr_char_literal(*c),
            HirExpression::Variable(name) => {
                self.gen_expr_variable(name, ctx, target_type)
            }
            HirExpression::BinaryOp { op, left, right } => {
                self.gen_expr_binary_op(op, left, right, ctx, target_type)
            }
            HirExpression::Dereference(inner) => {
                self.gen_expr_dereference(inner, ctx)
            }
            HirExpression::UnaryOp { op, operand } => {
                self.gen_expr_unary_op(op, operand, ctx)
            }
            HirExpression::FunctionCall { function, arguments } => {
                self.gen_expr_function_call(function, arguments, ctx, target_type)
            }
            HirExpression::FieldAccess { object, field } => {
                format!(
                    "{}.{}",
                    self.generate_expression_with_context(object, ctx),
                    escape_rust_keyword(field)
                )
            }
            HirExpression::PointerFieldAccess { pointer, field } => {
                self.gen_expr_pointer_field_access(pointer, field, ctx)
            }
            HirExpression::ArrayIndex { array, index } => {
                self.gen_expr_array_index(array, index, ctx)
            }
            HirExpression::SliceIndex { slice, index, .. } => {
                let slice_code = self.generate_expression_with_context(slice, ctx);
                let index_code = self.generate_expression_with_context(index, ctx);
                format!("{}[({}) as usize]", slice_code, index_code)
            }
            HirExpression::Sizeof { type_name } => {
                self.gen_expr_sizeof(type_name, ctx)
            }
            HirExpression::NullLiteral => "None".to_string(),
            HirExpression::IsNotNull(inner) => {
                let inner_code = self.generate_expression_with_context(inner, ctx);
                format!("if let Some(_) = {}", inner_code)
            }
            HirExpression::Calloc { count, element_type } => {
                self.gen_expr_calloc(count, element_type, ctx)
            }
            HirExpression::Malloc { size } => self.gen_expr_malloc(size, ctx),
            HirExpression::Realloc { pointer, new_size } => {
                self.gen_expr_realloc(pointer, new_size, ctx)
            }
            HirExpression::StringMethodCall { receiver, method, arguments } => {
                self.gen_expr_string_method_call(receiver, method, arguments, ctx)
            }
            HirExpression::Cast { target_type: cast_target, expr } => {
                self.gen_expr_cast(cast_target, expr, ctx, target_type)
            }
            HirExpression::CompoundLiteral { literal_type, initializers } => {
                self.gen_expr_compound_literal(literal_type, initializers, ctx)
            }
            HirExpression::PostIncrement { operand } => {
                self.gen_expr_post_increment(operand, ctx)
            }
            HirExpression::PreIncrement { operand } => {
                self.gen_expr_pre_increment(operand, ctx)
            }
            HirExpression::PostDecrement { operand } => {
                self.gen_expr_post_decrement(operand, ctx)
            }
            HirExpression::PreDecrement { operand } => {
                self.gen_expr_pre_decrement(operand, ctx)
            }
            HirExpression::Ternary { condition, then_expr, else_expr } => {
                self.gen_expr_ternary(condition, then_expr, else_expr, ctx, target_type)
            }
        }
    }

    fn gen_expr_int_literal(&self, val: i32, target_type: Option<&HirType>) -> String {
        if val == 0 {
            if let Some(HirType::Option(_)) = target_type {
                return "None".to_string();
            }
            if let Some(HirType::Pointer(_)) = target_type {
                return "std::ptr::null_mut()".to_string();
            }
        }
        val.to_string()
    }

    fn gen_expr_float_literal(&self, val: &str, target_type: Option<&HirType>) -> String {
        let val_stripped = val.trim_end_matches(['f', 'F', 'l', 'L']);
        match target_type {
            Some(HirType::Float) => format!("{}f32", val_stripped),
            Some(HirType::Double) => format!("{}f64", val_stripped),
            _ => {
                if val_stripped.contains('.')
                    || val_stripped.contains('e')
                    || val_stripped.contains('E')
                {
                    format!("{}f64", val_stripped)
                } else {
                    format!("{}.0f64", val_stripped)
                }
            }
        }
    }

    fn gen_expr_address_of(
        &self,
        inner: &HirExpression,
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> String {
        if let Some(HirType::Pointer(ptr_inner)) = target_type {
            let inner_code = self.generate_expression_with_context(inner, ctx);
            let ptr_type = Self::map_type(&HirType::Pointer(ptr_inner.clone()));
            return format!("&mut {} as {}", inner_code, ptr_type);
        }
        let inner_code = self.generate_expression_with_context(inner, ctx);
        if matches!(*inner, HirExpression::Dereference(_)) {
            format!("&({})", inner_code)
        } else {
            format!("&{}", inner_code)
        }
    }

    fn gen_expr_unary_address_of(
        &self,
        operand: &HirExpression,
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> String {
        if let Some(HirType::Pointer(ptr_inner)) = target_type {
            let inner_code = self.generate_expression_with_context(operand, ctx);
            let ptr_type = Self::map_type(&HirType::Pointer(ptr_inner.clone()));
            return format!("&mut {} as {}", inner_code, ptr_type);
        }
        let inner_code = self.generate_expression_with_context(operand, ctx);
        format!("&{}", inner_code)
    }

    fn gen_expr_unary_logical_not(
        &self,
        operand: &HirExpression,
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> String {
        let inner_code = self.generate_expression_with_context(operand, ctx);
        let inner_parens = if matches!(*operand, HirExpression::BinaryOp { .. }) {
            format!("({})", inner_code)
        } else {
            inner_code.clone()
        };
        if let Some(HirType::Int) = target_type {
            if Self::is_boolean_expression(operand) {
                return format!("(!{}) as i32", inner_parens);
            } else {
                return format!("({} == 0) as i32", inner_code);
            }
        }
        if Self::is_boolean_expression(operand) {
            format!("!{}", inner_parens)
        } else {
            format!("({} == 0)", inner_code)
        }
    }

    fn gen_expr_string_literal(&self, s: &str, target_type: Option<&HirType>) -> String {
        if let Some(HirType::Pointer(inner)) = target_type {
            if matches!(inner.as_ref(), HirType::Char) {
                let escaped: String = s
                    .chars()
                    .map(|c| match c {
                        '"' => "\\\"".to_string(),
                        '\\' => "\\\\".to_string(),
                        c => c.to_string(),
                    })
                    .collect();
                return format!("b\"{}\\0\".as_ptr() as *mut u8", escaped);
            }
        }
        format!("\"{}\"", s)
    }

    fn gen_expr_char_literal(c: i8) -> String {
        let val = c as u8;
        if val == 0 {
            "0u8".to_string()
        } else if val.is_ascii_graphic() || val == b' ' {
            format!("b'{}'", val as char)
        } else {
            format!("{}u8", val)
        }
    }

    fn gen_expr_variable_pointer_target(
        escaped_name: &str,
        name: &str,
        ptr_inner: &Box<HirType>,
        ctx: &TypeContext,
    ) -> Option<String> {
        let var_type = ctx.get_type(name)?;
        if matches!(var_type, HirType::Box(_)) {
            return Some(format!("Box::into_raw({})", escaped_name));
        }
        match var_type {
            HirType::Reference { inner, mutable } => {
                let element_type_match = match inner.as_ref() {
                    HirType::Array { element_type, .. } => {
                        Some((element_type.as_ref(), *mutable))
                    }
                    HirType::Vec(elem_type) => Some((elem_type.as_ref(), *mutable)),
                    _ => None,
                };

                if let Some((elem_type, is_mutable)) = element_type_match {
                    if elem_type == ptr_inner.as_ref() {
                        if is_mutable {
                            return Some(format!("{}.as_mut_ptr()", escaped_name));
                        } else {
                            let ptr_type = Self::map_type(&HirType::Pointer(
                                ptr_inner.clone(),
                            ));
                            return Some(format!(
                                "{}.as_ptr() as {}",
                                escaped_name, ptr_type
                            ));
                        }
                    }
                } else if inner.as_ref() == ptr_inner.as_ref() {
                    if *mutable {
                        return Some(format!("{} as *mut _", escaped_name));
                    } else {
                        return Some(format!("{} as *const _ as *mut _", escaped_name));
                    }
                }
            }
            HirType::Vec(elem_type) => {
                if elem_type.as_ref() == ptr_inner.as_ref() {
                    return Some(format!("{}.as_mut_ptr()", escaped_name));
                }
            }
            HirType::Array { element_type, .. } => {
                if element_type.as_ref() == ptr_inner.as_ref() {
                    return Some(format!("{}.as_mut_ptr()", escaped_name));
                }
                if matches!(ptr_inner.as_ref(), HirType::Void) {
                    return Some(format!("{}.as_mut_ptr() as *mut ()", escaped_name));
                }
            }
            HirType::Pointer(_var_inner) => {
                return Some(escaped_name.to_string());
            }
            _ => {}
        }
        None
    }

    fn gen_expr_variable_numeric_coercion(
        escaped_name: &str,
        name: &str,
        target: &HirType,
        ctx: &TypeContext,
    ) -> Option<String> {
        let var_type = ctx.get_type(name)?;
        let cast_suffix = if matches!(var_type, HirType::Int | HirType::UnsignedInt) {
            match target {
                HirType::Float => Some("f32"),
                HirType::Double => Some("f64"),
                _ => None,
            }
        } else if matches!(var_type, HirType::Float | HirType::Double) {
            match target {
                HirType::Int => Some("i32"),
                HirType::UnsignedInt => Some("u32"),
                _ => None,
            }
        } else if matches!(var_type, HirType::Char) && matches!(target, HirType::Int) {
            Some("i32")
        } else {
            None
        };

        cast_suffix.map(|suffix| {
            let code = format!("{} as {}", escaped_name, suffix);
            if ctx.is_global(name) {
                format!("unsafe {{ {} }}", code)
            } else {
                code
            }
        })
    }

    fn gen_expr_variable(
        &self,
        name: &str,
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> String {
        match name {
            "stderr" => return "std::io::stderr()".to_string(),
            "stdin" => return "std::io::stdin()".to_string(),
            "stdout" => return "std::io::stdout()".to_string(),
            "errno" => return "unsafe { ERRNO }".to_string(),
            "ERANGE" => return "34i32".to_string(),
            "EINVAL" => return "22i32".to_string(),
            "ENOENT" => return "2i32".to_string(),
            "EACCES" => return "13i32".to_string(),
            _ => {}
        }
        let escaped_name = escape_rust_keyword(name);
        let escaped_name =
            ctx.get_renamed_local(&escaped_name).cloned().unwrap_or(escaped_name);
        if let Some(HirType::Vec(_)) = target_type {
            return escaped_name;
        }
        if let Some(HirType::Pointer(ptr_inner)) = target_type {
            if let Some(result) =
                Self::gen_expr_variable_pointer_target(&escaped_name, name, ptr_inner, ctx)
            {
                return result;
            }
        }

        if let Some(HirType::Char) = target_type {
            if let Some(var_type) = ctx.get_type(name) {
                if matches!(var_type, HirType::Int) {
                    return format!("{} as u8", escaped_name);
                }
            }
        }

        if let Some(target) = target_type {
            if let Some(result) =
                Self::gen_expr_variable_numeric_coercion(&escaped_name, name, target, ctx)
            {
                return result;
            }
        }

        if ctx.is_global(name) {
            format!("unsafe {{ {} }}", escaped_name)
        } else {
            escaped_name
        }
    }

    fn gen_expr_binary_op(
        &self,
        op: &BinaryOperator,
        left: &HirExpression,
        right: &HirExpression,
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> String {
        if matches!(op, BinaryOperator::Assign) {
            return self.gen_expr_binary_assign(left, right, ctx);
        }

        if matches!(op, BinaryOperator::Equal | BinaryOperator::NotEqual) {
            if let Some(result) = self.gen_expr_binary_equality_special(op, left, right, ctx) {
                return result;
            }
        }

        let is_comparison = matches!(
            op,
            BinaryOperator::Equal
                | BinaryOperator::NotEqual
                | BinaryOperator::LessThan
                | BinaryOperator::GreaterThan
                | BinaryOperator::LessEqual
                | BinaryOperator::GreaterEqual
        );

        if is_comparison {
            if let Some(result) = self.gen_expr_binary_char_int_comparison(op, left, right, ctx) {
                return result;
            }
        }

        if matches!(op, BinaryOperator::Add | BinaryOperator::Subtract) {
            if let Some(result) = self.gen_expr_binary_char_arithmetic(op, left, right, ctx) {
                return result;
            }
        }

        if matches!(op, BinaryOperator::Comma) {
            let left_code = self.generate_expression_with_context(left, ctx);
            let right_code = self.generate_expression_with_context(right, ctx);
            return format!("{{ {}; {} }}", left_code, right_code);
        }

        let left_code = self.generate_expression_with_context(left, ctx);
        let right_code = self.generate_expression_with_context(right, ctx);
        let op_str = Self::binary_operator_to_string(op);

        let left_str = if matches!(*left, HirExpression::BinaryOp { .. }) {
            format!("({})", left_code)
        } else {
            left_code.clone()
        };

        let right_str = if matches!(*right, HirExpression::BinaryOp { .. }) {
            format!("({})", right_code)
        } else {
            right_code.clone()
        };

        if matches!(op, BinaryOperator::Add | BinaryOperator::Subtract) {
            if let Some(result) =
                self.gen_expr_binary_pointer_arithmetic(op, left, right, &left_str, &right_str, ctx)
            {
                return result;
            }
        }

        if matches!(op, BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr) {
            return self.gen_expr_binary_logical(
                op, left, right, &left_str, &right_str, op_str, target_type,
            );
        }

        if matches!(
            op,
            BinaryOperator::Add
                | BinaryOperator::Subtract
                | BinaryOperator::Multiply
                | BinaryOperator::Divide
                | BinaryOperator::Modulo
        ) {
            if let Some(result) = self.gen_expr_binary_arithmetic_coercion(
                op, left, right, &left_str, &right_str, op_str, ctx, target_type,
            ) {
                return result;
            }
        }

        let returns_bool = matches!(
            op,
            BinaryOperator::GreaterThan
                | BinaryOperator::LessThan
                | BinaryOperator::GreaterEqual
                | BinaryOperator::LessEqual
                | BinaryOperator::Equal
                | BinaryOperator::NotEqual
                | BinaryOperator::LogicalAnd
                | BinaryOperator::LogicalOr
        );

        if is_comparison {
            if let Some(result) = self.gen_expr_binary_chained_comparison(
                left, right, &left_str, &right_str, op_str, ctx, target_type,
            ) {
                return result;
            }
            if let Some(result) = self.gen_expr_binary_signed_unsigned_comparison(
                left, right, &left_str, &right_str, op_str, ctx, target_type,
            ) {
                return result;
            }
        }

        if returns_bool {
            if let Some(HirType::Int) = target_type {
                return format!("({} {} {}) as i32", left_str, op_str, right_str);
            }
        }

        if matches!(
            op,
            BinaryOperator::Add
                | BinaryOperator::Subtract
                | BinaryOperator::Multiply
                | BinaryOperator::Divide
                | BinaryOperator::Modulo
        ) {
            if let Some(result) = self.gen_expr_binary_arithmetic_target_cast(
                left, right, &left_str, &right_str, op_str, ctx, target_type,
            ) {
                return result;
            }
        }

        if matches!(
            op,
            BinaryOperator::BitwiseAnd
                | BinaryOperator::BitwiseOr
                | BinaryOperator::BitwiseXor
        ) {
            if let Some(result) = self.gen_expr_binary_bitwise_bool(
                left, right, &left_str, &right_str, op_str, ctx,
            ) {
                return result;
            }
        }

        format!("{} {} {}", left_str, op_str, right_str)
    }

    fn gen_expr_binary_assign(
        &self,
        left: &HirExpression,
        right: &HirExpression,
        ctx: &TypeContext,
    ) -> String {
        let right_code = self.generate_expression_with_context(right, ctx);

        if let HirExpression::ArrayIndex { array, index } = left {
            if let HirExpression::Variable(var_name) = &**array {
                if ctx.is_global(var_name) {
                    let index_code = self.generate_expression_with_context(index, ctx);
                    return format!(
                        "{{ let __assign_tmp = {}; unsafe {{ {}[({}) as usize] = __assign_tmp }}; __assign_tmp }}",
                        right_code, var_name, index_code
                    );
                }
            }
        }

        let left_code = self.generate_expression_with_context(left, ctx);
        format!(
            "{{ let __assign_tmp = {}; {} = __assign_tmp; __assign_tmp }}",
            right_code, left_code
        )
    }

    fn gen_expr_binary_equality_special(
        &self,
        op: &BinaryOperator,
        left: &HirExpression,
        right: &HirExpression,
        ctx: &TypeContext,
    ) -> Option<String> {
        if let HirExpression::Variable(var_name) = left {
            if ctx.is_option(var_name) && matches!(*right, HirExpression::NullLiteral) {
                return Some(match op {
                    BinaryOperator::Equal => format!("{}.is_none()", var_name),
                    BinaryOperator::NotEqual => format!("{}.is_some()", var_name),
                    _ => unreachable!(),
                });
            }
        }
        if let HirExpression::Variable(var_name) = right {
            if ctx.is_option(var_name) && matches!(*left, HirExpression::NullLiteral) {
                return Some(match op {
                    BinaryOperator::Equal => format!("{}.is_none()", var_name),
                    BinaryOperator::NotEqual => format!("{}.is_some()", var_name),
                    _ => unreachable!(),
                });
            }
        }

        if let HirExpression::Variable(var_name) = left {
            if ctx.is_pointer(var_name) {
                if let HirExpression::IntLiteral(0) = right {
                    let op_str = Self::binary_operator_to_string(op);
                    return Some(format!("{} {} std::ptr::null_mut()", var_name, op_str));
                }
            }
        }
        if let HirExpression::Variable(var_name) = right {
            if ctx.is_pointer(var_name) {
                if let HirExpression::IntLiteral(0) = left {
                    let op_str = Self::binary_operator_to_string(op);
                    return Some(format!("std::ptr::null_mut() {} {}", op_str, var_name));
                }
            }
        }

        if let HirExpression::IntLiteral(0) = right {
            if let Some(left_type) = ctx.infer_expression_type(left) {
                if matches!(left_type, HirType::Pointer(_)) {
                    let left_code = self.generate_expression_with_context(left, ctx);
                    let op_str = Self::binary_operator_to_string(op);
                    return Some(format!("{} {} std::ptr::null_mut()", left_code, op_str));
                }
            }
        }
        if let HirExpression::IntLiteral(0) = left {
            if let Some(right_type) = ctx.infer_expression_type(right) {
                if matches!(right_type, HirType::Pointer(_)) {
                    let right_code = self.generate_expression_with_context(right, ctx);
                    let op_str = Self::binary_operator_to_string(op);
                    return Some(format!("std::ptr::null_mut() {} {}", op_str, right_code));
                }
            }
        }

        if let HirExpression::Variable(var_name) = left {
            if ctx.is_vec(var_name)
                && matches!(
                    *right,
                    HirExpression::IntLiteral(0) | HirExpression::NullLiteral
                )
            {
                return Some(match op {
                    BinaryOperator::Equal => "false /* Vec never null */".to_string(),
                    BinaryOperator::NotEqual => "true /* Vec never null */".to_string(),
                    _ => unreachable!(),
                });
            }
        }

        if let HirExpression::Variable(var_name) = left {
            if let Some(HirType::Box(_)) = ctx.get_type(var_name) {
                if matches!(
                    *right,
                    HirExpression::IntLiteral(0) | HirExpression::NullLiteral
                ) {
                    return Some(match op {
                        BinaryOperator::Equal => {
                            "false /* Box never null */".to_string()
                        }
                        BinaryOperator::NotEqual => {
                            "true /* Box never null */".to_string()
                        }
                        _ => unreachable!(),
                    });
                }
            }
        }

        if let HirExpression::FunctionCall { function, arguments } = left {
            if function == "strlen" && arguments.len() == 1 {
                if let HirExpression::IntLiteral(0) = right {
                    let arg_code =
                        self.generate_expression_with_context(&arguments[0], ctx);
                    return Some(match op {
                        BinaryOperator::Equal => format!("{}.is_empty()", arg_code),
                        BinaryOperator::NotEqual => format!("!{}.is_empty()", arg_code),
                        _ => unreachable!(),
                    });
                }
            }
        }
        if let HirExpression::FunctionCall { function, arguments } = right {
            if function == "strlen" && arguments.len() == 1 {
                if let HirExpression::IntLiteral(0) = left {
                    let arg_code =
                        self.generate_expression_with_context(&arguments[0], ctx);
                    return Some(match op {
                        BinaryOperator::Equal => format!("{}.is_empty()", arg_code),
                        BinaryOperator::NotEqual => format!("!{}.is_empty()", arg_code),
                        _ => unreachable!(),
                    });
                }
            }
        }

        None
    }

    fn gen_expr_binary_char_int_comparison(
        &self,
        op: &BinaryOperator,
        left: &HirExpression,
        right: &HirExpression,
        ctx: &TypeContext,
    ) -> Option<String> {
        if let HirExpression::Variable(var_name) = left {
            if let Some(HirType::Int) = ctx.get_type(var_name) {
                if let HirExpression::CharLiteral(c) = right {
                    let left_code = self.generate_expression_with_context(left, ctx);
                    let op_str = Self::binary_operator_to_string(op);
                    return Some(format!("({} {} {}i32)", left_code, op_str, *c as i32));
                }
            }
        }
        if let HirExpression::Variable(var_name) = right {
            if let Some(HirType::Int) = ctx.get_type(var_name) {
                if let HirExpression::CharLiteral(c) = left {
                    let right_code = self.generate_expression_with_context(right, ctx);
                    let op_str = Self::binary_operator_to_string(op);
                    return Some(format!("({}i32 {} {})", *c as i32, op_str, right_code));
                }
            }
        }
        None
    }

    fn gen_expr_binary_char_arithmetic(
        &self,
        op: &BinaryOperator,
        left: &HirExpression,
        right: &HirExpression,
        ctx: &TypeContext,
    ) -> Option<String> {
        if let HirExpression::CharLiteral(c) = right {
            let left_type = ctx.infer_expression_type(left);
            if matches!(left_type, Some(HirType::Int)) {
                let left_code = self.generate_expression_with_context(left, ctx);
                let op_str = Self::binary_operator_to_string(op);
                return Some(format!("({} {} {}i32)", left_code, op_str, *c as i32));
            }
        }
        if let HirExpression::CharLiteral(c) = left {
            let right_type = ctx.infer_expression_type(right);
            if matches!(right_type, Some(HirType::Int)) {
                let right_code = self.generate_expression_with_context(right, ctx);
                let op_str = Self::binary_operator_to_string(op);
                return Some(format!("({}i32 {} {})", *c as i32, op_str, right_code));
            }
        }
        None
    }

    fn gen_expr_binary_pointer_arithmetic(
        &self,
        op: &BinaryOperator,
        left: &HirExpression,
        right: &HirExpression,
        left_str: &str,
        right_str: &str,
        ctx: &TypeContext,
    ) -> Option<String> {
        if let HirExpression::Variable(var_name) = left {
            if ctx.is_pointer(var_name) {
                return Some(match op {
                    BinaryOperator::Add => {
                        format!("{}.wrapping_add({} as usize)", left_str, right_str)
                    }
                    BinaryOperator::Subtract => {
                        if let HirExpression::Variable(right_var) = right {
                            if ctx.is_pointer(right_var) {
                                Self::unsafe_block(
                                    &format!(
                                        "{}.offset_from({}) as i32",
                                        left_str, right_str
                                    ),
                                    "both pointers derive from same allocation",
                                )
                            } else {
                                format!(
                                    "{}.wrapping_sub({} as usize)",
                                    left_str, right_str
                                )
                            }
                        } else {
                            format!("{}.wrapping_sub({} as usize)", left_str, right_str)
                        }
                    }
                    _ => unreachable!(),
                });
            }
        }
        None
    }

    #[allow(clippy::too_many_arguments)]
    fn gen_expr_binary_logical(
        &self,
        _op: &BinaryOperator,
        left: &HirExpression,
        right: &HirExpression,
        left_str: &str,
        right_str: &str,
        op_str: &str,
        target_type: Option<&HirType>,
    ) -> String {
        let left_needs_bool = !Self::is_boolean_expression(left);
        let right_needs_bool = !Self::is_boolean_expression(right);

        let left_bool = if left_needs_bool {
            format!("({} != 0)", left_str)
        } else {
            left_str.to_string()
        };

        let right_bool = if right_needs_bool {
            format!("({} != 0)", right_str)
        } else {
            right_str.to_string()
        };

        if let Some(HirType::Int) = target_type {
            return format!("({} {} {}) as i32", left_bool, op_str, right_bool);
        }

        format!("{} {} {}", left_bool, op_str, right_bool)
    }

    #[allow(clippy::too_many_arguments)]
    fn gen_expr_binary_arithmetic_coercion(
        &self,
        _op: &BinaryOperator,
        left: &HirExpression,
        right: &HirExpression,
        left_str: &str,
        right_str: &str,
        op_str: &str,
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> Option<String> {
        if let Some(HirType::Int) = target_type {
            let left_type = ctx.infer_expression_type(left);
            let right_type = ctx.infer_expression_type(right);

            let left_is_char = matches!(left_type, Some(HirType::Char));
            let right_is_char = matches!(right_type, Some(HirType::Char));

            if left_is_char || right_is_char {
                let left_cast = if left_is_char {
                    format!("({} as i32)", left_str)
                } else {
                    left_str.to_string()
                };
                let right_cast = if right_is_char {
                    format!("({} as i32)", right_str)
                } else {
                    right_str.to_string()
                };
                return Some(format!("{} {} {}", left_cast, op_str, right_cast));
            }
        }

        let left_type = ctx.infer_expression_type(left);
        let right_type = ctx.infer_expression_type(right);

        let left_is_int =
            matches!(left_type, Some(HirType::Int) | Some(HirType::UnsignedInt));
        let right_is_int =
            matches!(right_type, Some(HirType::Int) | Some(HirType::UnsignedInt));
        let left_is_float = matches!(left_type, Some(HirType::Float));
        let right_is_float = matches!(right_type, Some(HirType::Float));
        let left_is_double = matches!(left_type, Some(HirType::Double));
        let right_is_double = matches!(right_type, Some(HirType::Double));

        if (left_is_int && right_is_float) || (left_is_float && right_is_int) {
            let left_cast = if left_is_int {
                format!("({} as f32)", left_str)
            } else {
                left_str.to_string()
            };
            let right_cast = if right_is_int {
                format!("({} as f32)", right_str)
            } else {
                right_str.to_string()
            };
            return Some(format!("{} {} {}", left_cast, op_str, right_cast));
        }

        if (left_is_int && right_is_double) || (left_is_double && right_is_int) {
            let left_cast = if left_is_int {
                format!("({} as f64)", left_str)
            } else {
                left_str.to_string()
            };
            let right_cast = if right_is_int {
                format!("({} as f64)", right_str)
            } else {
                right_str.to_string()
            };
            return Some(format!("{} {} {}", left_cast, op_str, right_cast));
        }

        if (left_is_float && right_is_double) || (left_is_double && right_is_float) {
            let left_cast = if left_is_float {
                format!("({} as f64)", left_str)
            } else {
                left_str.to_string()
            };
            let right_cast = if right_is_float {
                format!("({} as f64)", right_str)
            } else {
                right_str.to_string()
            };
            return Some(format!("{} {} {}", left_cast, op_str, right_cast));
        }

        None
    }

    #[allow(clippy::too_many_arguments)]
    fn gen_expr_binary_chained_comparison(
        &self,
        left: &HirExpression,
        right: &HirExpression,
        left_str: &str,
        right_str: &str,
        op_str: &str,
        _ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> Option<String> {
        let left_is_comparison = Self::is_boolean_expression(left);
        let right_is_comparison = Self::is_boolean_expression(right);

        if left_is_comparison || right_is_comparison {
            let left_code = if left_is_comparison {
                format!("(({}) as i32)", left_str)
            } else {
                left_str.to_string()
            };
            let right_code = if right_is_comparison {
                format!("(({}) as i32)", right_str)
            } else {
                right_str.to_string()
            };
            if let Some(HirType::Int) = target_type {
                return Some(format!("({} {} {}) as i32", left_code, op_str, right_code));
            }
            return Some(format!("{} {} {}", left_code, op_str, right_code));
        }
        None
    }

    #[allow(clippy::too_many_arguments)]
    fn gen_expr_binary_signed_unsigned_comparison(
        &self,
        left: &HirExpression,
        right: &HirExpression,
        left_str: &str,
        right_str: &str,
        op_str: &str,
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> Option<String> {
        let left_type = ctx.infer_expression_type(left);
        let right_type = ctx.infer_expression_type(right);

        let left_is_signed = matches!(left_type, Some(HirType::Int));
        let left_is_unsigned = matches!(left_type, Some(HirType::UnsignedInt));
        let right_is_signed = matches!(right_type, Some(HirType::Int));
        let right_is_unsigned = matches!(right_type, Some(HirType::UnsignedInt));

        if (left_is_signed && right_is_unsigned)
            || (left_is_unsigned && right_is_signed)
        {
            let left_code = format!("({} as i64)", left_str);
            let right_code = format!("({} as i64)", right_str);
            if let Some(HirType::Int) = target_type {
                return Some(format!("({} {} {}) as i32", left_code, op_str, right_code));
            }
            return Some(format!("{} {} {}", left_code, op_str, right_code));
        }
        None
    }

    #[allow(clippy::too_many_arguments)]
    fn gen_expr_binary_arithmetic_target_cast(
        &self,
        left: &HirExpression,
        right: &HirExpression,
        left_str: &str,
        right_str: &str,
        op_str: &str,
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> Option<String> {
        let left_type = ctx.infer_expression_type(left);
        let right_type = ctx.infer_expression_type(right);

        let result_is_int =
            matches!(left_type, Some(HirType::Int) | Some(HirType::UnsignedInt))
                && matches!(
                    right_type,
                    Some(HirType::Int) | Some(HirType::UnsignedInt)
                );

        if result_is_int {
            if let Some(HirType::Float) = target_type {
                return Some(format!("({} {} {}) as f32", left_str, op_str, right_str));
            }
            if let Some(HirType::Double) = target_type {
                return Some(format!("({} {} {}) as f64", left_str, op_str, right_str));
            }
        }
        None
    }

    fn gen_expr_binary_bitwise_bool(
        &self,
        left: &HirExpression,
        right: &HirExpression,
        left_str: &str,
        right_str: &str,
        op_str: &str,
        ctx: &TypeContext,
    ) -> Option<String> {
        let left_is_bool = Self::is_boolean_expression(left);
        let right_is_bool = Self::is_boolean_expression(right);
        let left_type = ctx.infer_expression_type(left);
        let right_type = ctx.infer_expression_type(right);
        let left_is_unsigned = matches!(left_type, Some(HirType::UnsignedInt));
        let right_is_unsigned = matches!(right_type, Some(HirType::UnsignedInt));

        if left_is_bool || right_is_bool {
            let left_code = if left_is_bool {
                format!("({}) as i32", left_str)
            } else if left_is_unsigned {
                format!("({} as i32)", left_str)
            } else {
                left_str.to_string()
            };
            let right_code = if right_is_bool {
                format!("({}) as i32", right_str)
            } else if right_is_unsigned {
                format!("({} as i32)", right_str)
            } else {
                right_str.to_string()
            };
            let result = format!("{} {} {}", left_code, op_str, right_code);
            if left_is_unsigned || right_is_unsigned {
                return Some(format!("({}) as u32", result));
            }
            return Some(result);
        }
        None
    }

    fn gen_expr_dereference(&self, inner: &HirExpression, ctx: &TypeContext) -> String {
        if let HirExpression::Variable(var_name) = inner {
            if let Some(idx_var) = ctx.get_string_iter_index(var_name) {
                return format!("{}[{}]", var_name, idx_var);
            }

            if let Some(var_type) = ctx.get_type(var_name) {
                if matches!(var_type, HirType::StringReference | HirType::StringLiteral) {
                    return format!("{}.as_bytes()[0] as i32", var_name);
                }
            }
        }

        if let HirExpression::PostIncrement { operand } = inner {
            if let HirExpression::Variable(var_name) = &**operand {
                if let Some(var_type) = ctx.get_type(var_name) {
                    if matches!(var_type, HirType::StringReference | HirType::StringLiteral)
                    {
                        return self.generate_expression_with_context(inner, ctx);
                    }
                }
            }
        }

        let inner_code = self.generate_expression_with_context(inner, ctx);

        let needs_unsafe = match inner {
            HirExpression::Variable(var_name) => ctx.is_pointer(var_name),
            HirExpression::BinaryOp { left, .. } => {
                if let HirExpression::Variable(var_name) = &**left {
                    ctx.is_pointer(var_name)
                } else {
                    false
                }
            }
            _ => false,
        };

        if needs_unsafe {
            return Self::unsafe_block(
                &format!("*{}", inner_code),
                "pointer is valid and properly aligned from caller contract",
            );
        }

        format!("*{}", inner_code)
    }

    fn gen_expr_unary_op(
        &self,
        op: &decy_hir::UnaryOperator,
        operand: &HirExpression,
        ctx: &TypeContext,
    ) -> String {
        use decy_hir::UnaryOperator;
        match op {
            UnaryOperator::PostIncrement => {
                let operand_code = self.generate_expression_with_context(operand, ctx);
                let operand_type = ctx.infer_expression_type(operand);
                if matches!(operand_type, Some(HirType::Pointer(_))) {
                    format!(
                        "{{ let __tmp = {}; {} = {}.wrapping_add(1); __tmp }}",
                        operand_code, operand_code, operand_code
                    )
                } else {
                    format!(
                        "{{ let __tmp = {}; {} += 1; __tmp }}",
                        operand_code, operand_code
                    )
                }
            }
            UnaryOperator::PostDecrement => {
                let operand_code = self.generate_expression_with_context(operand, ctx);
                let operand_type = ctx.infer_expression_type(operand);
                if matches!(operand_type, Some(HirType::Pointer(_))) {
                    format!(
                        "{{ let __tmp = {}; {} = {}.wrapping_sub(1); __tmp }}",
                        operand_code, operand_code, operand_code
                    )
                } else {
                    format!(
                        "{{ let __tmp = {}; {} -= 1; __tmp }}",
                        operand_code, operand_code
                    )
                }
            }
            UnaryOperator::PreIncrement => {
                let operand_code = self.generate_expression_with_context(operand, ctx);
                let operand_type = ctx.infer_expression_type(operand);
                if matches!(operand_type, Some(HirType::Pointer(_))) {
                    format!(
                        "{{ {} = {}.wrapping_add(1); {} }}",
                        operand_code, operand_code, operand_code
                    )
                } else {
                    format!("{{ {} += 1; {} }}", operand_code, operand_code)
                }
            }
            UnaryOperator::PreDecrement => {
                let operand_code = self.generate_expression_with_context(operand, ctx);
                let operand_type = ctx.infer_expression_type(operand);
                if matches!(operand_type, Some(HirType::Pointer(_))) {
                    format!(
                        "{{ {} = {}.wrapping_sub(1); {} }}",
                        operand_code, operand_code, operand_code
                    )
                } else {
                    format!("{{ {} -= 1; {} }}", operand_code, operand_code)
                }
            }
            UnaryOperator::LogicalNot => {
                let operand_code = self.generate_expression_with_context(operand, ctx);
                if Self::is_boolean_expression(operand) {
                    format!("!{}", operand_code)
                } else {
                    format!("({} == 0) as i32", operand_code)
                }
            }
            _ => {
                let op_str = Self::unary_operator_to_string(op);
                let operand_code = self.generate_expression_with_context(operand, ctx);
                format!("{}{}", op_str, operand_code)
            }
        }
    }

    fn gen_expr_function_call(
        &self,
        function: &str,
        arguments: &[HirExpression],
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> String {
        match function {
            "strlen" => self.gen_call_strlen(function, arguments, ctx),
            "strcpy" => self.gen_call_strcpy(function, arguments, ctx),
            "malloc" => self.gen_call_malloc(arguments, ctx, target_type),
            "calloc" => self.gen_call_calloc(arguments, ctx, target_type),
            "realloc" => self.gen_call_realloc(arguments, ctx, target_type),
            "free" => self.gen_call_free(arguments, ctx),
            "fopen" => self.gen_call_fopen(arguments, ctx),
            "fclose" => self.gen_call_fclose(arguments, ctx),
            "fgetc" | "getc" => self.gen_call_fgetc(arguments, ctx),
            "fputc" | "putc" => self.gen_call_fputc(arguments, ctx),
            "fprintf" => self.gen_call_fprintf(arguments, ctx),
            "printf" => self.gen_call_printf(arguments, ctx),
            "fread" => self.gen_call_fread(arguments, ctx),
            "fwrite" => self.gen_call_fwrite(arguments, ctx),
            "fputs" => self.gen_call_fputs(arguments, ctx),
            "fork" => "/* fork() transformed to Command API */ 0".to_string(),
            "execl" | "execlp" | "execle" | "execv" | "execvp" | "execve" => {
                self.gen_call_exec(arguments, ctx)
            }
            "waitpid" | "wait3" | "wait4" => {
                "/* waitpid handled by Command API */ child.wait().expect(\"wait failed\")"
                    .to_string()
            }
            "wait" => "child.wait().expect(\"wait failed\")".to_string(),
            "WEXITSTATUS" => self.gen_call_status_macro(arguments, ctx, |s| format!("{}.code().unwrap_or(-1)", s)),
            "WIFEXITED" => self.gen_call_status_macro(arguments, ctx, |s| format!("{}.success()", s)),
            "WIFSIGNALED" => self.gen_call_status_macro(arguments, ctx, |s| format!("{}.signal().is_some()", s)),
            "WTERMSIG" => self.gen_call_status_macro(arguments, ctx, |s| format!("{}.signal().unwrap_or(0)", s)),
            "atoi" => self.gen_call_parse(arguments, ctx, "i32", "0"),
            "atof" => self.gen_call_parse(arguments, ctx, "f64", "0.0"),
            "abs" => {
                if arguments.len() == 1 {
                    let x = self.generate_expression_with_context(&arguments[0], ctx);
                    format!("({}).abs()", x)
                } else {
                    "0 /* abs requires 1 arg */".to_string()
                }
            }
            "exit" => {
                if arguments.len() == 1 {
                    let code = self.generate_expression_with_context(&arguments[0], ctx);
                    format!("std::process::exit({})", code)
                } else {
                    "std::process::exit(1)".to_string()
                }
            }
            "puts" => {
                if arguments.len() == 1 {
                    let s = self.generate_expression_with_context(&arguments[0], ctx);
                    format!("println!(\"{{}}\", {})", s)
                } else {
                    "println!()".to_string()
                }
            }
            "snprintf" => self.gen_call_snprintf(arguments, ctx),
            "sprintf" => self.gen_call_sprintf(arguments, ctx),
            "qsort" => {
                if arguments.len() == 4 {
                    let base = self.generate_expression_with_context(&arguments[0], ctx);
                    let n = self.generate_expression_with_context(&arguments[1], ctx);
                    let cmp = self.generate_expression_with_context(&arguments[3], ctx);
                    format!("{}[..{} as usize].sort_by(|a, b| {}(a, b))", base, n, cmp)
                } else {
                    "/* qsort requires 4 args */".to_string()
                }
            }
            _ => self.gen_call_default(function, arguments, ctx),
        }
    }

    fn gen_call_strlen(
        &self,
        function: &str,
        arguments: &[HirExpression],
        ctx: &TypeContext,
    ) -> String {
        if arguments.len() == 1 {
            format!(
                "{}.len() as i32",
                self.generate_expression_with_context(&arguments[0], ctx)
            )
        } else {
            let args: Vec<String> = arguments
                .iter()
                .map(|arg| self.generate_expression_with_context(arg, ctx))
                .collect();
            format!("{}({})", function, args.join(", "))
        }
    }

    fn gen_call_strcpy(
        &self,
        function: &str,
        arguments: &[HirExpression],
        ctx: &TypeContext,
    ) -> String {
        if arguments.len() == 2 {
            let src_code =
                self.generate_expression_with_context(&arguments[1], ctx);
            let is_raw_pointer = src_code.contains("(*")
                || src_code.contains(").")
                || src_code.contains("as *");
            if is_raw_pointer {
                format!(
                    "unsafe {{ std::ffi::CStr::from_ptr({} as *const i8).to_str().unwrap_or(\"\").to_string() }}",
                    src_code
                )
            } else {
                format!("{}.to_string()", src_code)
            }
        } else {
            let args: Vec<String> = arguments
                .iter()
                .map(|arg| self.generate_expression_with_context(arg, ctx))
                .collect();
            format!("{}({})", function, args.join(", "))
        }
    }

    fn gen_call_malloc(
        &self,
        arguments: &[HirExpression],
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> String {
        if arguments.len() == 1 {
            let size_code =
                self.generate_expression_with_context(&arguments[0], ctx);

            if let Some(HirType::Vec(elem_type)) = target_type {
                let elem_type_str = Self::map_type(elem_type);
                let default_val = Self::default_value_for_type(elem_type);
                if let HirExpression::BinaryOp {
                    op: BinaryOperator::Multiply,
                    left,
                    ..
                } = &arguments[0]
                {
                    let count_code =
                        self.generate_expression_with_context(left, ctx);
                    return format!(
                        "vec![{}; ({}) as usize]",
                        default_val, count_code
                    );
                } else {
                    return format!(
                        "Vec::<{}>::with_capacity(({}) as usize)",
                        elem_type_str, size_code
                    );
                }
            }

            if let Some(HirType::Pointer(inner)) = target_type {
                if matches!(inner.as_ref(), HirType::Char) {
                    return format!(
                        "Box::leak(vec![0u8; ({}) as usize].into_boxed_slice()).as_mut_ptr()",
                        size_code
                    );
                }
                if let HirType::Struct(struct_name) = inner.as_ref() {
                    return format!(
                        "Box::into_raw(Box::<{}>::default())",
                        struct_name
                    );
                }
                let elem_type_str = Self::map_type(inner);
                let default_val = Self::default_value_for_type(inner);
                if let HirExpression::BinaryOp {
                    op: BinaryOperator::Multiply,
                    left,
                    ..
                } = &arguments[0]
                {
                    let count_code =
                        self.generate_expression_with_context(left, ctx);
                    return format!(
                        "Box::leak(vec![{}; ({}) as usize].into_boxed_slice()).as_mut_ptr() as *mut {}",
                        default_val, count_code, elem_type_str
                    );
                } else {
                    return format!(
                        "Box::leak(vec![{}; ({}) as usize].into_boxed_slice()).as_mut_ptr() as *mut {}",
                        default_val, size_code, elem_type_str
                    );
                }
            }

            if let HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left,
                ..
            } = &arguments[0]
            {
                let count_code = self.generate_expression_with_context(left, ctx);
                format!("vec![0i32; ({}) as usize]", count_code)
            } else {
                format!("Vec::<u8>::with_capacity(({}) as usize)", size_code)
            }
        } else {
            "Vec::new()".to_string()
        }
    }

    fn gen_call_calloc(
        &self,
        arguments: &[HirExpression],
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> String {
        if arguments.len() == 2 {
            let count_code =
                self.generate_expression_with_context(&arguments[0], ctx);

            if let Some(HirType::Vec(elem_type)) = target_type {
                let default_val = Self::default_value_for_type(elem_type);
                return format!("vec![{}; {} as usize]", default_val, count_code);
            }

            if let Some(HirType::Pointer(inner)) = target_type {
                let elem_type_str = Self::map_type(inner);
                let default_val = Self::default_value_for_type(inner);
                return format!(
                    "Box::leak(vec![{}; {} as usize].into_boxed_slice()).as_mut_ptr() as *mut {}",
                    default_val, count_code, elem_type_str
                );
            }

            format!("vec![0i32; {} as usize]", count_code)
        } else {
            "Vec::new()".to_string()
        }
    }

    fn gen_call_realloc(
        &self,
        arguments: &[HirExpression],
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> String {
        if arguments.len() == 2 {
            let ptr_code =
                self.generate_expression_with_context(&arguments[0], ctx);
            let size_code =
                self.generate_expression_with_context(&arguments[1], ctx);
            let realloc_call =
                format!("realloc({} as *mut (), {})", ptr_code, size_code);

            if let Some(HirType::Pointer(inner)) = target_type {
                let target_ptr_type =
                    Self::map_type(&HirType::Pointer(inner.clone()));
                format!("{} as {}", realloc_call, target_ptr_type)
            } else {
                realloc_call
            }
        } else {
            "std::ptr::null_mut()".to_string()
        }
    }

    fn gen_call_free(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if arguments.len() == 1 {
            let ptr_code =
                self.generate_expression_with_context(&arguments[0], ctx);
            format!("drop({})", ptr_code)
        } else {
            "/* free() */".to_string()
        }
    }

    fn gen_call_fopen(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if arguments.len() == 2 {
            let filename =
                self.generate_expression_with_context(&arguments[0], ctx);
            let mode = self.generate_expression_with_context(&arguments[1], ctx);
            if mode.contains('w') || mode.contains('a') {
                format!("std::fs::File::create({}).ok()", filename)
            } else {
                format!("std::fs::File::open({}).ok()", filename)
            }
        } else {
            "None /* fopen requires 2 args */".to_string()
        }
    }

    fn gen_call_fclose(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if arguments.len() == 1 {
            let file_code =
                self.generate_expression_with_context(&arguments[0], ctx);
            format!("drop({})", file_code)
        } else {
            "/* fclose() */".to_string()
        }
    }

    fn gen_call_fgetc(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if arguments.len() == 1 {
            let file_code =
                self.generate_expression_with_context(&arguments[0], ctx);
            format!(
                "{{ use std::io::Read; let mut buf = [0u8; 1]; {}.read(&mut buf).map(|_| buf[0] as i32).unwrap_or(-1) }}",
                file_code
            )
        } else {
            "-1 /* fgetc requires 1 arg */".to_string()
        }
    }

    fn gen_call_fputc(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if arguments.len() == 2 {
            let char_code =
                self.generate_expression_with_context(&arguments[0], ctx);
            let file_code =
                self.generate_expression_with_context(&arguments[1], ctx);
            format!(
                "{{ use std::io::Write; {}.write(&[{} as u8]).map(|_| {} as i32).unwrap_or(-1) }}",
                file_code, char_code, char_code
            )
        } else {
            "-1 /* fputc requires 2 args */".to_string()
        }
    }

    fn gen_call_fprintf(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if arguments.len() >= 2 {
            let file_code =
                self.generate_expression_with_context(&arguments[0], ctx);
            let fmt = self.generate_expression_with_context(&arguments[1], ctx);
            let rust_fmt = Self::convert_c_format_to_rust(&fmt);
            if arguments.len() == 2 {
                format!(
                    "{{ use std::io::Write; write!({}, {}).map(|_| 0).unwrap_or(-1) }}",
                    file_code, rust_fmt
                )
            } else {
                let s_positions = Self::find_string_format_positions(&fmt);
                let args: Vec<String> = arguments[2..]
                    .iter()
                    .enumerate()
                    .map(|(i, a)| {
                        let arg_code = self.generate_expression_with_context(a, ctx);
                        if s_positions.contains(&i) {
                            format!("unsafe {{ std::ffi::CStr::from_ptr({} as *const i8).to_str().unwrap_or(\"\") }}", arg_code)
                        } else {
                            arg_code
                        }
                    })
                    .collect();
                format!(
                    "{{ use std::io::Write; write!({}, {}, {}).map(|_| 0).unwrap_or(-1) }}",
                    file_code, rust_fmt, args.join(", ")
                )
            }
        } else {
            "-1 /* fprintf requires 2+ args */".to_string()
        }
    }

    fn gen_call_printf(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if !arguments.is_empty() {
            let fmt = self.generate_expression_with_context(&arguments[0], ctx);
            let rust_fmt = Self::convert_c_format_to_rust(&fmt);
            if arguments.len() == 1 {
                format!("print!({})", rust_fmt)
            } else {
                let s_positions = Self::find_string_format_positions(&fmt);
                let args: Vec<String> = arguments[1..]
                    .iter()
                    .enumerate()
                    .map(|(i, a)| {
                        let arg_code =
                            self.generate_expression_with_context(a, ctx);
                        if s_positions.contains(&i) && !Self::is_string_ternary(a) {
                            let arg_type = ctx.infer_expression_type(a);
                            let is_raw_pointer =
                                matches!(arg_type, Some(HirType::Pointer(_)));
                            let is_function_call =
                                matches!(a, HirExpression::FunctionCall { .. });
                            if is_raw_pointer || is_function_call {
                                Self::wrap_raw_ptr_with_cstr(&arg_code)
                            } else {
                                Self::wrap_with_cstr(&arg_code)
                            }
                        } else {
                            arg_code
                        }
                    })
                    .collect();
                format!("print!({}, {})", rust_fmt, args.join(", "))
            }
        } else {
            "print!(\"\")".to_string()
        }
    }

    fn gen_call_fread(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if arguments.len() == 4 {
            let buf_code =
                self.generate_expression_with_context(&arguments[0], ctx);
            let file_code =
                self.generate_expression_with_context(&arguments[3], ctx);
            format!(
                "{{ use std::io::Read; {}.read(&mut {}).unwrap_or(0) }}",
                file_code, buf_code
            )
        } else {
            "0 /* fread requires 4 args */".to_string()
        }
    }

    fn gen_call_fwrite(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if arguments.len() == 4 {
            let data_code =
                self.generate_expression_with_context(&arguments[0], ctx);
            let file_code =
                self.generate_expression_with_context(&arguments[3], ctx);
            format!(
                "{{ use std::io::Write; {}.write(&{}).unwrap_or(0) }}",
                file_code, data_code
            )
        } else {
            "0 /* fwrite requires 4 args */".to_string()
        }
    }

    fn gen_call_fputs(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if arguments.len() == 2 {
            let str_code =
                self.generate_expression_with_context(&arguments[0], ctx);
            let file_code =
                self.generate_expression_with_context(&arguments[1], ctx);
            format!(
                "{{ use std::io::Write; {}.write_all({}.as_bytes()).map(|_| 0).unwrap_or(-1) }}",
                file_code, str_code
            )
        } else {
            "-1 /* fputs requires 2 args */".to_string()
        }
    }

    fn gen_call_exec(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if !arguments.is_empty() {
            let cmd = self.generate_expression_with_context(&arguments[0], ctx);
            let args: Vec<String> = arguments
                .iter()
                .skip(2)
                .filter(|a| !matches!(a, HirExpression::NullLiteral))
                .map(|a| self.generate_expression_with_context(a, ctx))
                .collect();
            if args.is_empty() {
                format!(
                    "{{ use std::process::Command; Command::new({}).status().expect(\"command failed\"); }}",
                    cmd
                )
            } else {
                let arg_chain: String =
                    args.iter().map(|a| format!(".arg({})", a)).collect();
                format!(
                    "{{ use std::process::Command; Command::new({}){}.status().expect(\"command failed\"); }}",
                    cmd, arg_chain
                )
            }
        } else {
            "/* exec requires args */".to_string()
        }
    }

    fn gen_call_status_macro(
        &self,
        arguments: &[HirExpression],
        ctx: &TypeContext,
        fmt_fn: impl Fn(String) -> String,
    ) -> String {
        if !arguments.is_empty() {
            let status_var =
                self.generate_expression_with_context(&arguments[0], ctx);
            fmt_fn(status_var)
        } else {
            "/* macro requires status arg */".to_string()
        }
    }

    fn gen_call_parse(
        &self,
        arguments: &[HirExpression],
        ctx: &TypeContext,
        rust_type: &str,
        default: &str,
    ) -> String {
        if arguments.len() == 1 {
            let s = self.generate_expression_with_context(&arguments[0], ctx);
            format!("{}.parse::<{}>().unwrap_or({})", s, rust_type, default)
        } else {
            format!("{} /* parse requires 1 arg */", default)
        }
    }

    fn gen_call_snprintf(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if arguments.len() >= 3 {
            let fmt = self.generate_expression_with_context(&arguments[2], ctx);
            let rust_fmt = Self::convert_c_format_to_rust(&fmt);
            if arguments.len() == 3 {
                format!("format!({})", rust_fmt)
            } else {
                let args: Vec<String> = arguments[3..]
                    .iter()
                    .map(|a| self.generate_expression_with_context(a, ctx))
                    .collect();
                format!("format!({}, {})", rust_fmt, args.join(", "))
            }
        } else {
            "String::new() /* snprintf requires 3+ args */".to_string()
        }
    }

    fn gen_call_sprintf(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if arguments.len() >= 2 {
            let fmt = self.generate_expression_with_context(&arguments[1], ctx);
            let rust_fmt = Self::convert_c_format_to_rust(&fmt);
            if arguments.len() == 2 {
                format!("format!({})", rust_fmt)
            } else {
                let args: Vec<String> = arguments[2..]
                    .iter()
                    .map(|a| self.generate_expression_with_context(a, ctx))
                    .collect();
                format!("format!({}, {})", rust_fmt, args.join(", "))
            }
        } else {
            "String::new() /* sprintf requires 2+ args */".to_string()
        }
    }

    fn gen_call_default(
        &self,
        function: &str,
        arguments: &[HirExpression],
        ctx: &TypeContext,
    ) -> String {
        let slice_mappings = ctx.get_slice_func_len_indices(function);
        let len_indices_to_skip: std::collections::HashSet<usize> = slice_mappings
            .map(|mappings| mappings.iter().map(|(_, len_idx)| *len_idx).collect())
            .unwrap_or_default();
        let array_indices: std::collections::HashSet<usize> = slice_mappings
            .map(|mappings| mappings.iter().map(|(arr_idx, _)| *arr_idx).collect())
            .unwrap_or_default();

        let args: Vec<String> = arguments
            .iter()
            .enumerate()
            .filter_map(|(i, arg)| {
                if len_indices_to_skip.contains(&i) {
                    return None;
                }

                if array_indices.contains(&i) {
                    let arg_code = self.generate_expression_with_context(arg, ctx);
                    return Some(format!("&{}", arg_code));
                }

                let is_address_of = matches!(arg, HirExpression::AddressOf(_))
                    || matches!(
                        arg,
                        HirExpression::UnaryOp {
                            op: decy_hir::UnaryOperator::AddressOf,
                            ..
                        }
                    );

                if is_address_of {
                    let inner = match arg {
                        HirExpression::AddressOf(inner) => inner.as_ref(),
                        HirExpression::UnaryOp { operand, .. } => operand.as_ref(),
                        _ => unreachable!(),
                    };

                    let expects_mut = ctx
                        .get_function_param_type(function, i)
                        .map(|t| {
                            matches!(t, HirType::Reference { mutable: true, .. })
                        })
                        .unwrap_or(true);

                    let inner_code =
                        self.generate_expression_with_context(inner, ctx);
                    if expects_mut {
                        Some(format!("&mut {}", inner_code))
                    } else {
                        Some(format!("&{}", inner_code))
                    }
                } else {
                    if let Some(string_iter_params) =
                        ctx.get_string_iter_func(function)
                    {
                        if let Some((_, is_mutable)) =
                            string_iter_params.iter().find(|(idx, _)| *idx == i)
                        {
                            if let HirExpression::Variable(var_name) = arg {
                                let var_type = ctx.get_type(var_name);
                                if matches!(var_type, Some(HirType::Array { .. })) {
                                    if *is_mutable {
                                        return Some(format!("&mut {}", var_name));
                                    } else {
                                        return Some(format!("&{}", var_name));
                                    }
                                }
                            }
                            if let HirExpression::StringLiteral(s) = arg {
                                return Some(format!("b\"{}\"", s));
                            }
                            if let HirExpression::AddressOf(inner) = arg {
                                let inner_code = self
                                    .generate_expression_with_context(inner, ctx);
                                if *is_mutable {
                                    return Some(format!("&mut {}", inner_code));
                                } else {
                                    return Some(format!("&{}", inner_code));
                                }
                            }
                        }
                    }

                    let param_type = ctx.get_function_param_type(function, i);
                    let is_raw_pointer_param = param_type
                        .map(|t| matches!(t, HirType::Pointer(_)))
                        .unwrap_or(false);

                    if is_raw_pointer_param {
                        if let HirExpression::Variable(var_name) = arg {
                            let var_type = ctx.get_type(var_name);
                            if matches!(var_type, Some(HirType::Array { .. })) {
                                return Some(format!("{}.as_mut_ptr()", var_name));
                            }
                        }
                        if let HirExpression::StringLiteral(s) = arg {
                            return Some(format!("\"{}\".as_ptr() as *mut u8", s));
                        }
                    }

                    let is_ref_param = param_type
                        .map(|t| matches!(t, HirType::Reference { .. }))
                        .unwrap_or(false);
                    if is_ref_param {
                        if let HirExpression::Variable(var_name) = arg {
                            let var_type = ctx.get_type(var_name);
                            if matches!(var_type, Some(HirType::Pointer(_))) {
                                return Some(Self::unsafe_block(
                                    &format!("&mut *{}", var_name),
                                    "pointer is non-null and valid for the duration of the call",
                                ));
                            }
                        }
                    }

                    let is_slice_param = param_type
                        .map(|t| matches!(t, HirType::Array { size: None, .. }))
                        .unwrap_or(false);
                    if is_slice_param {
                        if let HirExpression::Variable(var_name) = arg {
                            let var_type = ctx.get_type(var_name);
                            if matches!(var_type, Some(HirType::Array { size: Some(_), .. })) {
                                return Some(format!("&mut {}", var_name));
                            }
                        }
                    }

                    let is_int_param = param_type
                        .map(|t| matches!(t, HirType::Int))
                        .unwrap_or(false);
                    if is_int_param {
                        if let HirExpression::CharLiteral(c) = arg {
                            return Some(format!("{}i32", *c as i32));
                        }
                    }

                    let is_string_param = param_type
                        .map(|t| matches!(t, HirType::StringReference | HirType::StringLiteral))
                        .unwrap_or(false);
                    let is_string_func = matches!(
                        function,
                        "strcmp" | "strncmp" | "strchr" | "strrchr" | "strstr" | "strlen"
                    );
                    if is_string_param || is_string_func {
                        if let HirExpression::PointerFieldAccess { pointer, field } = arg {
                            let ptr_code = self.generate_expression_with_context(pointer, ctx);
                            return Some(Self::unsafe_block(
                                &format!("std::ffi::CStr::from_ptr((*{}).{} as *const i8).to_str().unwrap_or(\"\")", ptr_code, field),
                                "string pointer is null-terminated and valid",
                            ));
                        }
                    }

                    Some(self.generate_expression_with_context(arg, ctx))
                }
            })
            .collect();
        let safe_function = match function {
            "write" => "c_write",
            "read" => "c_read",
            "type" => "c_type",
            "match" => "c_match",
            "self" => "c_self",
            "in" => "c_in",
            _ => function,
        };
        format!("{}({})", safe_function, args.join(", "))
    }

    fn gen_expr_pointer_field_access(
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

    fn gen_expr_array_index(
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

    fn gen_expr_sizeof(&self, type_name: &str, ctx: &TypeContext) -> String {
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

    fn gen_expr_calloc(
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

    fn gen_expr_malloc(&self, size: &HirExpression, ctx: &TypeContext) -> String {
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

    fn gen_expr_realloc(
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

    fn gen_expr_string_method_call(
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

    fn gen_expr_cast(
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

    fn gen_expr_compound_literal(
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

    fn gen_expr_post_increment(&self, operand: &HirExpression, ctx: &TypeContext) -> String {
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

    fn gen_expr_pre_increment(&self, operand: &HirExpression, ctx: &TypeContext) -> String {
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

    fn gen_expr_post_decrement(&self, operand: &HirExpression, ctx: &TypeContext) -> String {
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

    fn gen_expr_pre_decrement(&self, operand: &HirExpression, ctx: &TypeContext) -> String {
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

    fn gen_expr_ternary(
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
    fn unary_operator_to_string(op: &decy_hir::UnaryOperator) -> &'static str {
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
    fn binary_operator_to_string(op: &BinaryOperator) -> &'static str {
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
    fn is_malloc_expression(expr: &HirExpression) -> bool {
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
    fn convert_c_format_to_rust(c_fmt: &str) -> String {
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
    fn convert_format_specifiers(input: &str) -> String {
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
    fn find_string_format_positions(fmt: &str) -> Vec<usize> {
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
    fn wrap_with_cstr(arg: &str) -> String {
        format!(
            "unsafe {{ std::ffi::CStr::from_ptr({}.as_ptr() as *const i8).to_str().unwrap_or(\"\") }}",
            arg
        )
    }

    /// DECY-221: Wrap a raw pointer (*mut u8) with CStr conversion for safe printing.
    /// Unlike wrap_with_cstr, this doesn't call .as_ptr() since the arg is already a pointer.
    fn wrap_raw_ptr_with_cstr(arg: &str) -> String {
        format!(
            "unsafe {{ std::ffi::CStr::from_ptr({} as *const i8).to_str().unwrap_or(\"\") }}",
            arg
        )
    }

    /// DECY-192: Check if expression is a ternary that returns string literals.
    /// Such expressions should not be wrapped with CStr since they return &str directly in Rust.
    fn is_string_ternary(expr: &HirExpression) -> bool {
        if let HirExpression::Ternary { then_expr, else_expr, .. } = expr {
            matches!(**then_expr, HirExpression::StringLiteral(_))
                && matches!(**else_expr, HirExpression::StringLiteral(_))
        } else {
            false
        }
    }
}
