//! Binary operation expression generation.

use super::{CodeGenerator, TypeContext};
use decy_hir::{BinaryOperator, HirExpression, HirType};

impl CodeGenerator {
    pub(crate) fn gen_expr_binary_op(
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

    pub(crate) fn gen_expr_binary_assign(
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

    pub(crate) fn gen_expr_binary_equality_special(
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

    pub(crate) fn gen_expr_binary_char_int_comparison(
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

    pub(crate) fn gen_expr_binary_char_arithmetic(
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

    pub(crate) fn gen_expr_binary_pointer_arithmetic(
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
    pub(crate) fn gen_expr_binary_logical(
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
    pub(crate) fn gen_expr_binary_arithmetic_coercion(
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
    pub(crate) fn gen_expr_binary_chained_comparison(
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
    pub(crate) fn gen_expr_binary_signed_unsigned_comparison(
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
    pub(crate) fn gen_expr_binary_arithmetic_target_cast(
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

    pub(crate) fn gen_expr_binary_bitwise_bool(
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
}
