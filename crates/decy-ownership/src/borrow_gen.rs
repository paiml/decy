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
    /// DECY-072: Transforms array parameters to safe slice parameters.
    pub fn transform_function(
        &self,
        func: &HirFunction,
        inferences: &HashMap<String, OwnershipInference>,
    ) -> HirFunction {
        // DECY-072: Build dataflow graph to detect array parameters
        let dataflow_analyzer = crate::dataflow::DataflowAnalyzer::new();
        let dataflow_graph = dataflow_analyzer.analyze(func);

        // DECY-072: Transform parameters with array detection
        let (transformed_params, length_params_to_remove) = self
            .transform_parameters_with_array_detection(
                func.parameters(),
                inferences,
                &dataflow_graph,
            );

        // DECY-070 + DECY-072: Transform function body
        // - Convert pointer arithmetic to SliceIndex (DECY-070)
        // - Replace length param usage with arr.len() (DECY-072)
        let transformed_body = func
            .body()
            .iter()
            .map(|stmt| {
                self.transform_statement_with_length_replacement(
                    stmt,
                    inferences,
                    &length_params_to_remove,
                )
            })
            .collect();

        HirFunction::new_with_body(
            func.name().to_string(),
            func.return_type().clone(),
            transformed_params,
            transformed_body,
        )
    }

    /// Transform parameters with array parameter detection.
    /// DECY-072: Detects array parameters and transforms them to slices.
    /// Returns (transformed_params, length_params_to_remove)
    fn transform_parameters_with_array_detection(
        &self,
        params: &[HirParameter],
        inferences: &HashMap<String, OwnershipInference>,
        dataflow_graph: &crate::dataflow::DataflowGraph,
    ) -> (Vec<HirParameter>, HashMap<String, String>) {
        let mut transformed_params = Vec::new();
        let mut length_params_to_remove = HashMap::new(); // length_param_name -> array_param_name
        let mut skip_next = false;

        for (i, param) in params.iter().enumerate() {
            if skip_next {
                skip_next = false;
                continue;
            }

            // Check if this is an array parameter
            if let Some(true) = dataflow_graph.is_array_parameter(param.name()) {
                // This is an array parameter!
                // Transform pointer to slice type
                let slice_type = self.transform_to_slice_type(
                    param.param_type(),
                    param.name(),
                    inferences,
                    dataflow_graph,
                );
                transformed_params.push(HirParameter::new(param.name().to_string(), slice_type));

                // DECY-113: Check if next parameter is the length parameter
                // Only treat it as length if it has a length-like name
                if i + 1 < params.len() {
                    let next_param = &params[i + 1];
                    if matches!(next_param.param_type(), HirType::Int) {
                        let param_name = next_param.name().to_lowercase();
                        // Only skip/transform if the name suggests it's a length/size parameter
                        if param_name.contains("len")
                            || param_name.contains("size")
                            || param_name.contains("count")
                            || param_name == "n"
                            || param_name == "num"
                        {
                            // This is the length parameter - skip it and record the mapping
                            length_params_to_remove
                                .insert(next_param.name().to_string(), param.name().to_string());
                            skip_next = true;
                        }
                    }
                }
            } else {
                // Not an array parameter - keep as is (with normal transformation)
                let transformed_type =
                    self.transform_type(param.param_type(), param.name(), inferences);
                transformed_params.push(HirParameter::new(
                    param.name().to_string(),
                    transformed_type,
                ));
            }
        }

        (transformed_params, length_params_to_remove)
    }

    /// Transform a pointer type to a slice type for array parameters.
    /// DECY-072: Determines mutability by checking if parameter is modified in function body.
    fn transform_to_slice_type(
        &self,
        hir_type: &HirType,
        var_name: &str,
        _inferences: &HashMap<String, OwnershipInference>,
        dataflow_graph: &crate::dataflow::DataflowGraph,
    ) -> HirType {
        if let HirType::Pointer(inner) = hir_type {
            // Check if this parameter is mutated in the function body
            let is_mutable = self.is_parameter_mutated(var_name, dataflow_graph);

            // Create slice type: &[T] or &mut [T]
            HirType::Reference {
                inner: Box::new(HirType::Vec(inner.clone())), // Use Vec as internal representation for slice
                mutable: is_mutable,
            }
        } else {
            hir_type.clone()
        }
    }

    /// Check if a parameter is mutated in the function body.
    /// DECY-072: Scans for ArrayIndexAssignment statements that modify the parameter.
    fn is_parameter_mutated(
        &self,
        var_name: &str,
        dataflow_graph: &crate::dataflow::DataflowGraph,
    ) -> bool {
        dataflow_graph
            .body()
            .iter()
            .any(|stmt| self.statement_mutates_variable(stmt, var_name))
    }

    /// Recursively check if a statement mutates a variable.
    #[allow(clippy::only_used_in_recursion)]
    fn statement_mutates_variable(&self, stmt: &decy_hir::HirStatement, var_name: &str) -> bool {
        use decy_hir::HirStatement;

        match stmt {
            HirStatement::ArrayIndexAssignment { array, .. } => {
                // Check if the array being modified is our variable
                matches!(&**array, decy_hir::HirExpression::Variable(name) if name == var_name)
            }
            HirStatement::DerefAssignment { target, .. } => {
                // Check if dereferencing our variable
                matches!(target, decy_hir::HirExpression::Variable(name) if name == var_name)
            }
            HirStatement::If {
                then_block,
                else_block,
                ..
            } => {
                then_block
                    .iter()
                    .any(|s| self.statement_mutates_variable(s, var_name))
                    || else_block
                        .as_ref()
                        .map(|stmts| {
                            stmts
                                .iter()
                                .any(|s| self.statement_mutates_variable(s, var_name))
                        })
                        .unwrap_or(false)
            }
            HirStatement::While { body, .. } | HirStatement::For { body, .. } => body
                .iter()
                .any(|s| self.statement_mutates_variable(s, var_name)),
            HirStatement::Switch {
                cases,
                default_case,
                ..
            } => {
                cases.iter().any(|case| {
                    case.body
                        .iter()
                        .any(|s| self.statement_mutates_variable(s, var_name))
                }) || default_case
                    .as_ref()
                    .map(|stmts| {
                        stmts
                            .iter()
                            .any(|s| self.statement_mutates_variable(s, var_name))
                    })
                    .unwrap_or(false)
            }
            _ => false,
        }
    }

    /// Transform a statement with length parameter replacement.
    /// DECY-072: Replaces uses of length parameters with arr.len() calls.
    fn transform_statement_with_length_replacement(
        &self,
        stmt: &decy_hir::HirStatement,
        inferences: &HashMap<String, OwnershipInference>,
        length_params_to_remove: &HashMap<String, String>,
    ) -> decy_hir::HirStatement {
        use decy_hir::HirStatement;

        match stmt {
            HirStatement::Return(expr_opt) => HirStatement::Return(expr_opt.as_ref().map(|e| {
                self.transform_expression_with_length_replacement(
                    e,
                    inferences,
                    length_params_to_remove,
                )
            })),
            HirStatement::Assignment { target, value } => HirStatement::Assignment {
                target: target.clone(),
                value: self.transform_expression_with_length_replacement(
                    value,
                    inferences,
                    length_params_to_remove,
                ),
            },
            HirStatement::DerefAssignment { target, value } => HirStatement::DerefAssignment {
                target: self.transform_expression_with_length_replacement(
                    target,
                    inferences,
                    length_params_to_remove,
                ),
                value: self.transform_expression_with_length_replacement(
                    value,
                    inferences,
                    length_params_to_remove,
                ),
            },
            HirStatement::ArrayIndexAssignment {
                array,
                index,
                value,
            } => HirStatement::ArrayIndexAssignment {
                array: Box::new(self.transform_expression_with_length_replacement(
                    array,
                    inferences,
                    length_params_to_remove,
                )),
                index: Box::new(self.transform_expression_with_length_replacement(
                    index,
                    inferences,
                    length_params_to_remove,
                )),
                value: self.transform_expression_with_length_replacement(
                    value,
                    inferences,
                    length_params_to_remove,
                ),
            },
            HirStatement::FieldAssignment {
                object,
                field,
                value,
            } => HirStatement::FieldAssignment {
                object: self.transform_expression_with_length_replacement(
                    object,
                    inferences,
                    length_params_to_remove,
                ),
                field: field.clone(),
                value: self.transform_expression_with_length_replacement(
                    value,
                    inferences,
                    length_params_to_remove,
                ),
            },
            HirStatement::If {
                condition,
                then_block,
                else_block,
            } => HirStatement::If {
                condition: self.transform_expression_with_length_replacement(
                    condition,
                    inferences,
                    length_params_to_remove,
                ),
                then_block: then_block
                    .iter()
                    .map(|s| {
                        self.transform_statement_with_length_replacement(
                            s,
                            inferences,
                            length_params_to_remove,
                        )
                    })
                    .collect(),
                else_block: else_block.as_ref().map(|stmts| {
                    stmts
                        .iter()
                        .map(|s| {
                            self.transform_statement_with_length_replacement(
                                s,
                                inferences,
                                length_params_to_remove,
                            )
                        })
                        .collect()
                }),
            },
            HirStatement::While { condition, body } => HirStatement::While {
                condition: self.transform_expression_with_length_replacement(
                    condition,
                    inferences,
                    length_params_to_remove,
                ),
                body: body
                    .iter()
                    .map(|s| {
                        self.transform_statement_with_length_replacement(
                            s,
                            inferences,
                            length_params_to_remove,
                        )
                    })
                    .collect(),
            },
            HirStatement::For {
                init,
                condition,
                increment,
                body,
            } => HirStatement::For {
                init: init.as_ref().map(|s| {
                    Box::new(self.transform_statement_with_length_replacement(
                        s,
                        inferences,
                        length_params_to_remove,
                    ))
                }),
                condition: self.transform_expression_with_length_replacement(
                    condition,
                    inferences,
                    length_params_to_remove,
                ),
                increment: increment.as_ref().map(|s| {
                    Box::new(self.transform_statement_with_length_replacement(
                        s,
                        inferences,
                        length_params_to_remove,
                    ))
                }),
                body: body
                    .iter()
                    .map(|s| {
                        self.transform_statement_with_length_replacement(
                            s,
                            inferences,
                            length_params_to_remove,
                        )
                    })
                    .collect(),
            },
            HirStatement::Switch {
                condition,
                cases,
                default_case,
            } => HirStatement::Switch {
                condition: self.transform_expression_with_length_replacement(
                    condition,
                    inferences,
                    length_params_to_remove,
                ),
                cases: cases
                    .iter()
                    .map(|case| decy_hir::SwitchCase {
                        value: case.value.clone(),
                        body: case
                            .body
                            .iter()
                            .map(|s| {
                                self.transform_statement_with_length_replacement(
                                    s,
                                    inferences,
                                    length_params_to_remove,
                                )
                            })
                            .collect(),
                    })
                    .collect(),
                default_case: default_case.as_ref().map(|stmts| {
                    stmts
                        .iter()
                        .map(|s| {
                            self.transform_statement_with_length_replacement(
                                s,
                                inferences,
                                length_params_to_remove,
                            )
                        })
                        .collect()
                }),
            },
            HirStatement::Free { pointer } => HirStatement::Free {
                pointer: self.transform_expression_with_length_replacement(
                    pointer,
                    inferences,
                    length_params_to_remove,
                ),
            },
            HirStatement::Expression(expr) => {
                HirStatement::Expression(self.transform_expression_with_length_replacement(
                    expr,
                    inferences,
                    length_params_to_remove,
                ))
            }
            HirStatement::VariableDeclaration {
                name,
                var_type,
                initializer,
            } => HirStatement::VariableDeclaration {
                name: name.clone(),
                var_type: var_type.clone(),
                initializer: initializer.as_ref().map(|e| {
                    self.transform_expression_with_length_replacement(
                        e,
                        inferences,
                        length_params_to_remove,
                    )
                }),
            },
            // Statements that don't contain expressions
            HirStatement::Break | HirStatement::Continue => stmt.clone(),
        }
    }

    /// Transform an expression with length parameter replacement.
    /// DECY-072: Replaces length parameter variable usage with arr.len() calls.
    fn transform_expression_with_length_replacement(
        &self,
        expr: &decy_hir::HirExpression,
        inferences: &HashMap<String, OwnershipInference>,
        length_params_to_remove: &HashMap<String, String>,
    ) -> decy_hir::HirExpression {
        use decy_hir::HirExpression;

        // First check if this variable is a length parameter that should be replaced
        if let HirExpression::Variable(var_name) = expr {
            if let Some(array_name) = length_params_to_remove.get(var_name) {
                // Replace length variable with arr.len() call
                return HirExpression::StringMethodCall {
                    receiver: Box::new(HirExpression::Variable(array_name.clone())),
                    method: "len".to_string(),
                    arguments: vec![],
                };
            }
        }

        // Then do the normal pointer arithmetic transformation (DECY-070)
        // But we need to recursively transform children with length replacement!
        self.transform_expression_recursive_with_length(expr, inferences, length_params_to_remove)
    }

    /// Recursively transform expression with both pointer arithmetic AND length replacement.
    /// DECY-072: This ensures length replacement happens in nested expressions.
    fn transform_expression_recursive_with_length(
        &self,
        expr: &decy_hir::HirExpression,
        inferences: &HashMap<String, OwnershipInference>,
        length_params_to_remove: &HashMap<String, String>,
    ) -> decy_hir::HirExpression {
        use decy_hir::{BinaryOperator, HirExpression};

        // DECY-070: Detect pointer arithmetic pattern: *(ptr + offset) or *(ptr - offset)
        if let HirExpression::Dereference(inner) = expr {
            if let HirExpression::BinaryOp { op, left, right } = &**inner {
                if matches!(op, BinaryOperator::Add | BinaryOperator::Subtract) {
                    if let HirExpression::Variable(var_name) = &**left {
                        if let Some(inference) = inferences.get(var_name) {
                            if let crate::inference::OwnershipKind::ArrayPointer {
                                element_type,
                                ..
                            } = &inference.kind
                            {
                                // Transform the offset expression (same for both add and subtract)
                                let index_expr = self.transform_expression_with_length_replacement(
                                    right,
                                    inferences,
                                    length_params_to_remove,
                                );

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
            HirExpression::Dereference(inner) => HirExpression::Dereference(Box::new(
                self.transform_expression_with_length_replacement(
                    inner,
                    inferences,
                    length_params_to_remove,
                ),
            )),
            HirExpression::AddressOf(inner) => HirExpression::AddressOf(Box::new(
                self.transform_expression_with_length_replacement(
                    inner,
                    inferences,
                    length_params_to_remove,
                ),
            )),
            HirExpression::UnaryOp { op, operand } => HirExpression::UnaryOp {
                op: *op,
                operand: Box::new(self.transform_expression_with_length_replacement(
                    operand,
                    inferences,
                    length_params_to_remove,
                )),
            },
            HirExpression::BinaryOp { op, left, right } => HirExpression::BinaryOp {
                op: *op,
                left: Box::new(self.transform_expression_with_length_replacement(
                    left,
                    inferences,
                    length_params_to_remove,
                )),
                right: Box::new(self.transform_expression_with_length_replacement(
                    right,
                    inferences,
                    length_params_to_remove,
                )),
            },
            HirExpression::FunctionCall {
                function,
                arguments,
            } => HirExpression::FunctionCall {
                function: function.clone(),
                arguments: arguments
                    .iter()
                    .map(|arg| {
                        self.transform_expression_with_length_replacement(
                            arg,
                            inferences,
                            length_params_to_remove,
                        )
                    })
                    .collect(),
            },
            HirExpression::FieldAccess { object, field } => HirExpression::FieldAccess {
                object: Box::new(self.transform_expression_with_length_replacement(
                    object,
                    inferences,
                    length_params_to_remove,
                )),
                field: field.clone(),
            },
            HirExpression::PointerFieldAccess { pointer, field } => {
                HirExpression::PointerFieldAccess {
                    pointer: Box::new(self.transform_expression_with_length_replacement(
                        pointer,
                        inferences,
                        length_params_to_remove,
                    )),
                    field: field.clone(),
                }
            }
            HirExpression::ArrayIndex { array, index } => HirExpression::ArrayIndex {
                array: Box::new(self.transform_expression_with_length_replacement(
                    array,
                    inferences,
                    length_params_to_remove,
                )),
                index: Box::new(self.transform_expression_with_length_replacement(
                    index,
                    inferences,
                    length_params_to_remove,
                )),
            },
            HirExpression::Cast {
                expr: cast_expr,
                target_type,
            } => HirExpression::Cast {
                expr: Box::new(self.transform_expression_with_length_replacement(
                    cast_expr,
                    inferences,
                    length_params_to_remove,
                )),
                target_type: target_type.clone(),
            },
            HirExpression::CompoundLiteral {
                literal_type,
                initializers,
            } => HirExpression::CompoundLiteral {
                literal_type: literal_type.clone(),
                initializers: initializers
                    .iter()
                    .map(|init| {
                        self.transform_expression_with_length_replacement(
                            init,
                            inferences,
                            length_params_to_remove,
                        )
                    })
                    .collect(),
            },
            HirExpression::IsNotNull(inner) => HirExpression::IsNotNull(Box::new(
                self.transform_expression_with_length_replacement(
                    inner,
                    inferences,
                    length_params_to_remove,
                ),
            )),
            HirExpression::Calloc {
                count,
                element_type,
            } => HirExpression::Calloc {
                count: Box::new(self.transform_expression_with_length_replacement(
                    count,
                    inferences,
                    length_params_to_remove,
                )),
                element_type: element_type.clone(),
            },
            HirExpression::Malloc { size } => HirExpression::Malloc {
                size: Box::new(self.transform_expression_with_length_replacement(
                    size,
                    inferences,
                    length_params_to_remove,
                )),
            },
            HirExpression::Realloc { pointer, new_size } => HirExpression::Realloc {
                pointer: Box::new(self.transform_expression_with_length_replacement(
                    pointer,
                    inferences,
                    length_params_to_remove,
                )),
                new_size: Box::new(self.transform_expression_with_length_replacement(
                    new_size,
                    inferences,
                    length_params_to_remove,
                )),
            },
            HirExpression::StringMethodCall {
                receiver,
                method,
                arguments,
            } => HirExpression::StringMethodCall {
                receiver: Box::new(self.transform_expression_with_length_replacement(
                    receiver,
                    inferences,
                    length_params_to_remove,
                )),
                method: method.clone(),
                arguments: arguments
                    .iter()
                    .map(|arg| {
                        self.transform_expression_with_length_replacement(
                            arg,
                            inferences,
                            length_params_to_remove,
                        )
                    })
                    .collect(),
            },
            HirExpression::SliceIndex {
                slice,
                index,
                element_type,
            } => HirExpression::SliceIndex {
                slice: Box::new(self.transform_expression_with_length_replacement(
                    slice,
                    inferences,
                    length_params_to_remove,
                )),
                index: Box::new(self.transform_expression_with_length_replacement(
                    index,
                    inferences,
                    length_params_to_remove,
                )),
                element_type: element_type.clone(),
            },
            // Leaf expressions - no transformation needed
            HirExpression::IntLiteral(_)
            | HirExpression::StringLiteral(_)
            | HirExpression::CharLiteral(_)
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
