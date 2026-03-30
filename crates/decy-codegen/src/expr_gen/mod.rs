//! Expression code generation methods for CodeGenerator.
//!
//! Contains all methods related to generating Rust code from HIR expressions,
//! including binary operations, unary operations, function calls, literals,
//! format string handling, and type coercion.

mod literals;
mod binary_ops;
mod calls;
mod misc;

use super::{escape_rust_keyword, CodeGenerator, TypeContext};
use decy_hir::{HirExpression, HirType};

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
            // DECY-207: C++ new T(args) -> Box::new(T::new(args)) or Box::new(T::default())
            HirExpression::CxxNew { allocated_type, arguments } => {
                let type_name = Self::map_type(allocated_type);
                if arguments.is_empty() {
                    format!("Box::new({}::default())", type_name)
                } else {
                    let args: Vec<String> = arguments
                        .iter()
                        .map(|a| self.generate_expression_with_context(a, ctx))
                        .collect();
                    format!("Box::new({}::new({}))", type_name, args.join(", "))
                }
            }
            // DECY-207: C++ delete ptr -> drop(ptr) (Box drops automatically)
            HirExpression::CxxDelete { operand } => {
                let operand_code = self.generate_expression_with_context(operand, ctx);
                format!("drop({})", operand_code)
            }
        }
    }
}
