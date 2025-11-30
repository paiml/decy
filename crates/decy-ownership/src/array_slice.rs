//! Array Parameter to Slice Transformation
//!
//! **DECY-072 GREEN**: Transforms C array parameters to safe Rust slices
//!
//! This module transforms function signatures like:
//! - `void process(int* arr, int len)` → `fn process(arr: &[i32])`
//! - `void modify(int* arr, int len)` → `fn modify(arr: &mut [i32])`
//!
//! It also transforms the function body to replace length parameter
//! references with `.len()` calls on the slice.

use crate::dataflow::DataflowGraph;
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType};
use std::collections::HashMap;

/// Information about an array parameter transformation
#[derive(Debug, Clone)]
pub struct ArrayParameterInfo {
    /// Name of the array parameter
    pub array_param: String,
    /// Name of the corresponding length parameter (if any)
    pub length_param: Option<String>,
    /// Whether the array is mutable
    pub is_mutable: bool,
}

/// Transforms a function to use slices for array parameters
pub struct ArrayParameterTransformer;

impl ArrayParameterTransformer {
    /// Create a new array parameter transformer
    pub fn new() -> Self {
        Self
    }

    /// Transform a function to use slices for array parameters
    ///
    /// # Arguments
    ///
    /// * `func` - The HIR function to transform
    /// * `dataflow` - Dataflow graph with array parameter detection
    ///
    /// # Returns
    ///
    /// A new HIR function with:
    /// - Array parameters transformed to slices (`&[T]` or `&mut [T]`)
    /// - Length parameters removed
    /// - Function body updated to use `.len()` instead of length param
    pub fn transform(&self, func: &HirFunction, dataflow: &DataflowGraph) -> HirFunction {
        // Get array parameter information
        let array_params = dataflow.get_array_parameters();

        if array_params.is_empty() {
            // No array parameters, return function unchanged
            return func.clone();
        }

        // Build map of array params and length params to remove
        let mut array_param_map: HashMap<String, Option<String>> = HashMap::new();
        let mut length_params_to_remove: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for (array_param, length_param) in &array_params {
            array_param_map.insert(array_param.clone(), length_param.clone());
            if let Some(len_param) = length_param {
                length_params_to_remove.insert(len_param.clone());
            }
        }

        // Transform parameters
        let new_parameters: Vec<HirParameter> = func
            .parameters()
            .iter()
            .filter_map(|param| {
                // Skip length parameters
                if length_params_to_remove.contains(param.name()) {
                    return None;
                }

                // Transform array parameters to slices
                if array_param_map.contains_key(param.name()) {
                    // Get element type from pointer
                    if let HirType::Pointer(inner) = param.param_type() {
                        // Check if array is modified (need &mut) or read-only (need &)
                        // DECY-072: Check dataflow to determine mutability
                        let is_mutable = dataflow.is_modified(param.name());

                        // Create slice type: &[T] or &mut [T]
                        // In HIR, a slice is represented as a Reference to an Array with size=None
                        let slice_type = HirType::Reference {
                            inner: Box::new(HirType::Array {
                                element_type: inner.clone(),
                                size: None, // None means unsized (slice)
                            }),
                            mutable: is_mutable,
                        };

                        // DECY-135: Use with_type to preserve is_pointee_const
                        return Some(param.with_type(slice_type));
                    }
                }

                // Keep parameter unchanged
                Some(param.clone())
            })
            .collect();

        // Transform function body to replace length parameter references with .len()
        let new_body: Vec<HirStatement> = func
            .body()
            .iter()
            .map(|stmt| Self::transform_statement(stmt, &array_param_map))
            .collect();

        // Create new function with transformed parameters and body
        HirFunction::new_with_body(
            func.name().to_string(),
            func.return_type().clone(),
            new_parameters,
            new_body,
        )
    }

    /// Transform a statement to replace length parameter references with .len()
    fn transform_statement(
        stmt: &HirStatement,
        array_param_map: &HashMap<String, Option<String>>,
    ) -> HirStatement {
        match stmt {
            HirStatement::VariableDeclaration {
                name,
                var_type,
                initializer,
            } => HirStatement::VariableDeclaration {
                name: name.clone(),
                var_type: var_type.clone(),
                initializer: initializer
                    .as_ref()
                    .map(|expr| Self::transform_expression(expr, array_param_map)),
            },
            HirStatement::Assignment { target, value } => HirStatement::Assignment {
                target: target.clone(),
                value: Self::transform_expression(value, array_param_map),
            },
            HirStatement::If {
                condition,
                then_block,
                else_block,
            } => HirStatement::If {
                condition: Self::transform_expression(condition, array_param_map),
                then_block: then_block
                    .iter()
                    .map(|s| Self::transform_statement(s, array_param_map))
                    .collect(),
                else_block: else_block.as_ref().map(|block| {
                    block
                        .iter()
                        .map(|s| Self::transform_statement(s, array_param_map))
                        .collect()
                }),
            },
            HirStatement::While { condition, body } => HirStatement::While {
                condition: Self::transform_expression(condition, array_param_map),
                body: body
                    .iter()
                    .map(|s| Self::transform_statement(s, array_param_map))
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
                    .map(|s| Box::new(Self::transform_statement(s, array_param_map))),
                condition: Self::transform_expression(condition, array_param_map),
                increment: increment
                    .as_ref()
                    .map(|s| Box::new(Self::transform_statement(s, array_param_map))),
                body: body
                    .iter()
                    .map(|s| Self::transform_statement(s, array_param_map))
                    .collect(),
            },
            HirStatement::Return(Some(expr)) => {
                HirStatement::Return(Some(Self::transform_expression(expr, array_param_map)))
            }
            HirStatement::ArrayIndexAssignment {
                array,
                index,
                value,
            } => HirStatement::ArrayIndexAssignment {
                array: Box::new(Self::transform_expression(array, array_param_map)),
                index: Box::new(Self::transform_expression(index, array_param_map)),
                value: Self::transform_expression(value, array_param_map),
            },
            HirStatement::Expression(expr) => {
                HirStatement::Expression(Self::transform_expression(expr, array_param_map))
            }
            // Other statements pass through unchanged
            _ => stmt.clone(),
        }
    }

    /// Transform an expression to replace length parameter references with .len()
    ///
    /// Note: For now, we don't transform length parameter references in the body.
    /// This will be handled by the code generator during Rust code generation.
    fn transform_expression(
        expr: &HirExpression,
        array_param_map: &HashMap<String, Option<String>>,
    ) -> HirExpression {
        match expr {
            HirExpression::Variable(name) => {
                // Transform length parameter references to .len() calls
                // Find if this variable is a length parameter for any array
                for (array_name, length_param) in array_param_map {
                    if let Some(len_name) = length_param {
                        if len_name == name {
                            // Replace with array.len()
                            return HirExpression::StringMethodCall {
                                receiver: Box::new(HirExpression::Variable(array_name.clone())),
                                method: "len".to_string(),
                                arguments: vec![],
                            };
                        }
                    }
                }
                // Not a length parameter, keep as-is
                expr.clone()
            }
            HirExpression::BinaryOp { op, left, right } => HirExpression::BinaryOp {
                op: *op,
                left: Box::new(Self::transform_expression(left, array_param_map)),
                right: Box::new(Self::transform_expression(right, array_param_map)),
            },
            HirExpression::UnaryOp { op, operand } => HirExpression::UnaryOp {
                op: *op,
                operand: Box::new(Self::transform_expression(operand, array_param_map)),
            },
            HirExpression::FunctionCall {
                function,
                arguments,
            } => HirExpression::FunctionCall {
                function: function.clone(),
                arguments: arguments
                    .iter()
                    .map(|arg| Self::transform_expression(arg, array_param_map))
                    .collect(),
            },
            HirExpression::ArrayIndex { array, index } => HirExpression::ArrayIndex {
                array: Box::new(Self::transform_expression(array, array_param_map)),
                index: Box::new(Self::transform_expression(index, array_param_map)),
            },
            HirExpression::Cast { expr, target_type } => HirExpression::Cast {
                expr: Box::new(Self::transform_expression(expr, array_param_map)),
                target_type: target_type.clone(),
            },
            HirExpression::Dereference(inner) => HirExpression::Dereference(Box::new(
                Self::transform_expression(inner, array_param_map),
            )),
            HirExpression::AddressOf(inner) => HirExpression::AddressOf(Box::new(
                Self::transform_expression(inner, array_param_map),
            )),
            // Literals and other expressions pass through unchanged
            _ => expr.clone(),
        }
    }
}

impl Default for ArrayParameterTransformer {
    fn default() -> Self {
        Self::new()
    }
}
