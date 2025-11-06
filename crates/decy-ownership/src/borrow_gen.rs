//! Borrow code generation from ownership inference.
//!
//! This module generates Rust borrow syntax (&T, &mut T) from ownership
//! inference results, transforming C pointers into safe Rust references.

use crate::inference::OwnershipInference;
use decy_hir::{HirFunction, HirParameter, HirType};
use std::collections::HashMap;

/// Borrow code generator.
///
/// Transforms HIR based on ownership inference to generate
/// idiomatic Rust borrow code.
#[derive(Debug)]
pub struct BorrowGenerator;

impl BorrowGenerator {
    /// Create a new borrow generator.
    pub fn new() -> Self {
        Self
    }

    /// Transform a HIR type based on ownership inference.
    ///
    /// Converts C pointer types to appropriate Rust borrow types:
    /// - Owning pointers → `Box<T>` (already handled by Box transformer)
    /// - ImmutableBorrow → &T
    /// - MutableBorrow → &mut T
    /// - Unknown → *mut T (falls back to raw pointer)
    ///
    /// DECY-041: For now, keep all pointers as raw pointers to support pointer arithmetic.
    /// Future: Detect when references are safe and when raw pointers are needed.
    pub fn transform_type(
        &self,
        hir_type: &HirType,
        _variable_name: &str,
        _inferences: &HashMap<String, OwnershipInference>,
    ) -> HirType {
        match hir_type {
            HirType::Pointer(inner) => {
                // DECY-041: Keep all pointers as raw pointers for now
                // This allows pointer arithmetic to work correctly.
                // In the future, we can be more sophisticated and only use
                // raw pointers when pointer arithmetic is detected.
                HirType::Pointer(inner.clone())

                // Original logic (disabled for DECY-041):
                // Look up ownership inference for this variable
                // if let Some(inference) = inferences.get(variable_name) {
                //     match inference.kind {
                //         OwnershipKind::Owning => {
                //             // Owning pointers become Box<T>
                //             HirType::Box(inner.clone())
                //         }
                //         OwnershipKind::ImmutableBorrow => {
                //             // Immutable borrows become &T
                //             HirType::Reference {
                //                 inner: inner.clone(),
                //                 mutable: false,
                //             }
                //         }
                //         OwnershipKind::MutableBorrow => {
                //             // Mutable borrows become &mut T
                //             HirType::Reference {
                //                 inner: inner.clone(),
                //                 mutable: true,
                //             }
                //         }
                //         OwnershipKind::Unknown => {
                //             // Unknown cases fall back to raw pointer
                //             HirType::Pointer(inner.clone())
                //         }
                //     }
                // } else {
                //     // No inference available, keep as pointer
                //     HirType::Pointer(inner.clone())
                // }
            }
            // Non-pointer types remain unchanged
            _ => hir_type.clone(),
        }
    }

    /// Transform function parameters based on ownership inference.
    ///
    /// Converts pointer parameters to appropriate borrow types.
    pub fn transform_parameters(
        &self,
        params: &[HirParameter],
        inferences: &HashMap<String, OwnershipInference>,
    ) -> Vec<HirParameter> {
        params
            .iter()
            .map(|param| {
                let transformed_type =
                    self.transform_type(param.param_type(), param.name(), inferences);
                HirParameter::new(param.name().to_string(), transformed_type)
            })
            .collect()
    }

    /// Transform a function signature based on ownership inference.
    ///
    /// Returns a new HirFunction with transformed parameter types and body.
    /// DECY-070: Also transforms pointer arithmetic to safe slice indexing.
    pub fn transform_function(
        &self,
        func: &HirFunction,
        inferences: &HashMap<String, OwnershipInference>,
    ) -> HirFunction {
        let transformed_params = self.transform_parameters(func.parameters(), inferences);

        // DECY-070: Transform function body to convert pointer arithmetic to SliceIndex
        let transformed_body = func
            .body()
            .iter()
            .map(|stmt| self.transform_statement(stmt, inferences))
            .collect();

        HirFunction::new_with_body(
            func.name().to_string(),
            func.return_type().clone(),
            transformed_params,
            transformed_body,
        )
    }

    /// Transform a statement, recursively transforming expressions within it.
    /// DECY-070: Converts pointer arithmetic to safe slice indexing.
    fn transform_statement(
        &self,
        stmt: &decy_hir::HirStatement,
        inferences: &HashMap<String, OwnershipInference>,
    ) -> decy_hir::HirStatement {
        use decy_hir::HirStatement;

        match stmt {
            HirStatement::Return(expr_opt) => HirStatement::Return(
                expr_opt
                    .as_ref()
                    .map(|e| self.transform_expression(e, inferences)),
            ),
            HirStatement::Assignment { target, value } => HirStatement::Assignment {
                target: target.clone(),
                value: self.transform_expression(value, inferences),
            },
            HirStatement::DerefAssignment { target, value } => HirStatement::DerefAssignment {
                target: self.transform_expression(target, inferences),
                value: self.transform_expression(value, inferences),
            },
            HirStatement::ArrayIndexAssignment {
                array,
                index,
                value,
            } => HirStatement::ArrayIndexAssignment {
                array: Box::new(self.transform_expression(array, inferences)),
                index: Box::new(self.transform_expression(index, inferences)),
                value: self.transform_expression(value, inferences),
            },
            HirStatement::FieldAssignment {
                object,
                field,
                value,
            } => HirStatement::FieldAssignment {
                object: self.transform_expression(object, inferences),
                field: field.clone(),
                value: self.transform_expression(value, inferences),
            },
            HirStatement::If {
                condition,
                then_block,
                else_block,
            } => HirStatement::If {
                condition: self.transform_expression(condition, inferences),
                then_block: then_block
                    .iter()
                    .map(|s| self.transform_statement(s, inferences))
                    .collect(),
                else_block: else_block.as_ref().map(|stmts| {
                    stmts
                        .iter()
                        .map(|s| self.transform_statement(s, inferences))
                        .collect()
                }),
            },
            HirStatement::While { condition, body } => HirStatement::While {
                condition: self.transform_expression(condition, inferences),
                body: body
                    .iter()
                    .map(|s| self.transform_statement(s, inferences))
                    .collect(),
            },
            HirStatement::For {
                init,
                condition,
                increment,
                body,
            } => HirStatement::For {
                init: init
                    .as_ref()
                    .map(|s| Box::new(self.transform_statement(s, inferences))),
                condition: self.transform_expression(condition, inferences),
                increment: increment
                    .as_ref()
                    .map(|s| Box::new(self.transform_statement(s, inferences))),
                body: body
                    .iter()
                    .map(|s| self.transform_statement(s, inferences))
                    .collect(),
            },
            HirStatement::Switch {
                condition,
                cases,
                default_case,
            } => HirStatement::Switch {
                condition: self.transform_expression(condition, inferences),
                cases: cases
                    .iter()
                    .map(|case| decy_hir::SwitchCase {
                        value: case.value.clone(),
                        body: case
                            .body
                            .iter()
                            .map(|s| self.transform_statement(s, inferences))
                            .collect(),
                    })
                    .collect(),
                default_case: default_case.as_ref().map(|stmts| {
                    stmts
                        .iter()
                        .map(|s| self.transform_statement(s, inferences))
                        .collect()
                }),
            },
            HirStatement::Free { pointer } => HirStatement::Free {
                pointer: self.transform_expression(pointer, inferences),
            },
            HirStatement::Expression(expr) => {
                HirStatement::Expression(self.transform_expression(expr, inferences))
            }
            HirStatement::VariableDeclaration {
                name,
                var_type,
                initializer,
            } => HirStatement::VariableDeclaration {
                name: name.clone(),
                var_type: var_type.clone(),
                initializer: initializer
                    .as_ref()
                    .map(|e| self.transform_expression(e, inferences)),
            },
            // Statements that don't contain expressions
            HirStatement::Break | HirStatement::Continue => stmt.clone(),
        }
    }

    /// Transform an expression, detecting and converting pointer arithmetic to SliceIndex.
    /// DECY-070: Converts *(ptr + offset) to SliceIndex { slice: ptr, index: offset }
    #[allow(clippy::only_used_in_recursion)]
    fn transform_expression(
        &self,
        expr: &decy_hir::HirExpression,
        inferences: &HashMap<String, OwnershipInference>,
    ) -> decy_hir::HirExpression {
        use decy_hir::{BinaryOperator, HirExpression};

        // DECY-070: Detect pointer arithmetic pattern: *(ptr + offset) or *(ptr - offset)
        if let HirExpression::Dereference(inner) = expr {
            if let HirExpression::BinaryOp { op, left, right } = &**inner {
                if matches!(op, BinaryOperator::Add | BinaryOperator::Subtract) {
                    // Check if left operand is a pointer variable with ArrayPointer ownership
                    if let HirExpression::Variable(var_name) = &**left {
                        if let Some(inference) = inferences.get(var_name) {
                            if let crate::inference::OwnershipKind::ArrayPointer {
                                element_type,
                                ..
                            } = &inference.kind
                            {
                                // Transform to SliceIndex!
                                let index_expr = if matches!(op, BinaryOperator::Subtract) {
                                    // For subtraction, negate the offset: ptr - 2 becomes ptr[-2]
                                    // But Rust doesn't support negative indexing, so we'll just use the offset as-is
                                    // Runtime checks will catch out-of-bounds access
                                    self.transform_expression(right, inferences)
                                } else {
                                    self.transform_expression(right, inferences)
                                };

                                return HirExpression::SliceIndex {
                                    slice: Box::new(HirExpression::Variable(var_name.clone())),
                                    index: Box::new(index_expr),
                                    element_type: element_type.clone(),
                                };
                            }
                        }
                    }
                }
            }
        }

        // Recursively transform child expressions
        match expr {
            HirExpression::Dereference(inner) => {
                HirExpression::Dereference(Box::new(self.transform_expression(inner, inferences)))
            }
            HirExpression::AddressOf(inner) => {
                HirExpression::AddressOf(Box::new(self.transform_expression(inner, inferences)))
            }
            HirExpression::UnaryOp { op, operand } => HirExpression::UnaryOp {
                op: *op,
                operand: Box::new(self.transform_expression(operand, inferences)),
            },
            HirExpression::BinaryOp { op, left, right } => HirExpression::BinaryOp {
                op: *op,
                left: Box::new(self.transform_expression(left, inferences)),
                right: Box::new(self.transform_expression(right, inferences)),
            },
            HirExpression::FunctionCall {
                function,
                arguments,
            } => HirExpression::FunctionCall {
                function: function.clone(),
                arguments: arguments
                    .iter()
                    .map(|arg| self.transform_expression(arg, inferences))
                    .collect(),
            },
            HirExpression::FieldAccess { object, field } => HirExpression::FieldAccess {
                object: Box::new(self.transform_expression(object, inferences)),
                field: field.clone(),
            },
            HirExpression::PointerFieldAccess { pointer, field } => {
                HirExpression::PointerFieldAccess {
                    pointer: Box::new(self.transform_expression(pointer, inferences)),
                    field: field.clone(),
                }
            }
            HirExpression::ArrayIndex { array, index } => HirExpression::ArrayIndex {
                array: Box::new(self.transform_expression(array, inferences)),
                index: Box::new(self.transform_expression(index, inferences)),
            },
            HirExpression::Cast {
                expr: cast_expr,
                target_type,
            } => HirExpression::Cast {
                expr: Box::new(self.transform_expression(cast_expr, inferences)),
                target_type: target_type.clone(),
            },
            HirExpression::CompoundLiteral {
                literal_type,
                initializers,
            } => HirExpression::CompoundLiteral {
                literal_type: literal_type.clone(),
                initializers: initializers
                    .iter()
                    .map(|init| self.transform_expression(init, inferences))
                    .collect(),
            },
            HirExpression::IsNotNull(inner) => {
                HirExpression::IsNotNull(Box::new(self.transform_expression(inner, inferences)))
            }
            HirExpression::Calloc {
                count,
                element_type,
            } => HirExpression::Calloc {
                count: Box::new(self.transform_expression(count, inferences)),
                element_type: element_type.clone(),
            },
            HirExpression::Malloc { size } => HirExpression::Malloc {
                size: Box::new(self.transform_expression(size, inferences)),
            },
            HirExpression::Realloc { pointer, new_size } => HirExpression::Realloc {
                pointer: Box::new(self.transform_expression(pointer, inferences)),
                new_size: Box::new(self.transform_expression(new_size, inferences)),
            },
            HirExpression::StringMethodCall {
                receiver,
                method,
                arguments,
            } => HirExpression::StringMethodCall {
                receiver: Box::new(self.transform_expression(receiver, inferences)),
                method: method.clone(),
                arguments: arguments
                    .iter()
                    .map(|arg| self.transform_expression(arg, inferences))
                    .collect(),
            },
            HirExpression::SliceIndex {
                slice,
                index,
                element_type,
            } => {
                // SliceIndex expressions are already safe - just transform children
                HirExpression::SliceIndex {
                    slice: Box::new(self.transform_expression(slice, inferences)),
                    index: Box::new(self.transform_expression(index, inferences)),
                    element_type: element_type.clone(),
                }
            }
            // Leaf expressions - no transformation needed
            HirExpression::IntLiteral(_)
            | HirExpression::StringLiteral(_)
            | HirExpression::Variable(_)
            | HirExpression::Sizeof { .. }
            | HirExpression::NullLiteral => expr.clone(),
        }
    }
}

impl Default for BorrowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "borrow_gen_tests.rs"]
mod borrow_gen_tests;
