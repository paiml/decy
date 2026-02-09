//! Borrow code generation from ownership inference.
//!
//! This module generates Rust borrow syntax (&T, &mut T) from ownership
//! inference results, transforming C pointers into safe Rust references.

use crate::inference::{OwnershipInference, OwnershipKind};
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType};
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
    /// - Owning pointers → `Box<T>`
    /// - ImmutableBorrow → `&T`
    /// - MutableBorrow → `&mut T`
    /// - Unknown → `*mut T` (falls back to raw pointer)
    /// - ArrayPointer → kept as pointer (handled by slice transformation)
    ///
    /// DECY-180: Re-enabled ownership-based transformation for single-shot compile.
    /// DECY-041: Pointer arithmetic cases are handled separately in transform_function.
    pub fn transform_type(
        &self,
        hir_type: &HirType,
        variable_name: &str,
        inferences: &HashMap<String, OwnershipInference>,
    ) -> HirType {
        match hir_type {
            HirType::Pointer(inner) => {
                // DECY-180: Look up ownership inference for this variable
                if let Some(inference) = inferences.get(variable_name) {
                    match &inference.kind {
                        OwnershipKind::Owning => {
                            // Owning pointers become Box<T>
                            HirType::Box(inner.clone())
                        }
                        OwnershipKind::ImmutableBorrow => {
                            // Immutable borrows become &T
                            HirType::Reference {
                                inner: inner.clone(),
                                mutable: false,
                            }
                        }
                        OwnershipKind::MutableBorrow => {
                            // Mutable borrows become &mut T
                            HirType::Reference {
                                inner: inner.clone(),
                                mutable: true,
                            }
                        }
                        OwnershipKind::ArrayPointer { .. } => {
                            // Array pointers handled by slice transformation
                            HirType::Pointer(inner.clone())
                        }
                        OwnershipKind::Unknown => {
                            // Unknown cases fall back to raw pointer
                            HirType::Pointer(inner.clone())
                        }
                    }
                } else {
                    // No inference available, keep as pointer
                    HirType::Pointer(inner.clone())
                }
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
        // DECY-161: Also pass function to check for pointer arithmetic
        let (transformed_params, length_params_to_remove) =
            self.transform_parameters_with_array_detection(func, inferences, &dataflow_graph);

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
    /// DECY-161: Skip slice transformation if parameter uses pointer arithmetic.
    /// Returns (transformed_params, length_params_to_remove)
    fn transform_parameters_with_array_detection(
        &self,
        func: &HirFunction,
        inferences: &HashMap<String, OwnershipInference>,
        dataflow_graph: &crate::dataflow::DataflowGraph,
    ) -> (Vec<HirParameter>, HashMap<String, String>) {
        let params = func.parameters();
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
                // DECY-161: Check if parameter uses pointer arithmetic
                // If so, keep as raw pointer (slices don't support arr++ or arr + n)
                if Self::uses_pointer_arithmetic(func, param.name()) {
                    // Keep as raw pointer
                    transformed_params.push(param.clone());
                    continue;
                }

                // This is an array parameter!
                // Transform pointer to slice type
                // DECY-135: Use with_type to preserve is_pointee_const
                let slice_type = self.transform_to_slice_type(
                    param.param_type(),
                    param.name(),
                    inferences,
                    dataflow_graph,
                );
                transformed_params.push(param.with_type(slice_type));

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
                // DECY-184: Check if parameter uses pointer arithmetic
                // If so, keep as raw pointer (borrow transformation doesn't work with ptr = ptr + 1)
                // This allows codegen's string iteration detection to generate &mut [u8] / &[u8]
                if Self::uses_pointer_arithmetic(func, param.name()) {
                    // Keep as raw pointer - codegen will handle string iteration pattern
                    transformed_params.push(param.clone());
                } else {
                    // Not an array parameter - keep as is (with normal transformation)
                    // DECY-135: Use with_type to preserve is_pointee_const for const char* → &str
                    let transformed_type =
                        self.transform_type(param.param_type(), param.name(), inferences);
                    transformed_params.push(param.with_type(transformed_type));
                }
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
                // DECY-224: Transform all init statements
                init: init
                    .iter()
                    .map(|s| {
                        self.transform_statement_with_length_replacement(
                            s,
                            inferences,
                            length_params_to_remove,
                        )
                    })
                    .collect(),
                condition: self.transform_expression_with_length_replacement(
                    condition,
                    inferences,
                    length_params_to_remove,
                ),
                // DECY-224: Transform all increment statements
                increment: increment
                    .iter()
                    .map(|s| {
                        self.transform_statement_with_length_replacement(
                            s,
                            inferences,
                            length_params_to_remove,
                        )
                    })
                    .collect(),
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
            // DECY-197: Inline assembly passes through unchanged
            HirStatement::InlineAsm { .. } => stmt.clone(),
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
            // DECY-139: Handle increment/decrement expressions
            HirExpression::PostIncrement { operand } => HirExpression::PostIncrement {
                operand: Box::new(self.transform_expression_with_length_replacement(
                    operand,
                    inferences,
                    length_params_to_remove,
                )),
            },
            HirExpression::PreIncrement { operand } => HirExpression::PreIncrement {
                operand: Box::new(self.transform_expression_with_length_replacement(
                    operand,
                    inferences,
                    length_params_to_remove,
                )),
            },
            HirExpression::PostDecrement { operand } => HirExpression::PostDecrement {
                operand: Box::new(self.transform_expression_with_length_replacement(
                    operand,
                    inferences,
                    length_params_to_remove,
                )),
            },
            HirExpression::PreDecrement { operand } => HirExpression::PreDecrement {
                operand: Box::new(self.transform_expression_with_length_replacement(
                    operand,
                    inferences,
                    length_params_to_remove,
                )),
            },
            // DECY-192: Ternary expressions - transform all sub-expressions
            HirExpression::Ternary {
                condition,
                then_expr,
                else_expr,
            } => HirExpression::Ternary {
                condition: Box::new(self.transform_expression_with_length_replacement(
                    condition,
                    inferences,
                    length_params_to_remove,
                )),
                then_expr: Box::new(self.transform_expression_with_length_replacement(
                    then_expr,
                    inferences,
                    length_params_to_remove,
                )),
                else_expr: Box::new(self.transform_expression_with_length_replacement(
                    else_expr,
                    inferences,
                    length_params_to_remove,
                )),
            },
            // Leaf expressions - no transformation needed
            HirExpression::IntLiteral(_)
            | HirExpression::FloatLiteral(_)
            | HirExpression::StringLiteral(_)
            | HirExpression::CharLiteral(_)
            | HirExpression::Variable(_)
            | HirExpression::Sizeof { .. }
            | HirExpression::NullLiteral => expr.clone(),
        }
    }

    /// DECY-161: Check if a parameter uses pointer arithmetic in the function body.
    ///
    /// Parameters that use pointer arithmetic (arr++, arr = arr + n) cannot be
    /// transformed to slices because slices don't support these operations.
    fn uses_pointer_arithmetic(func: &HirFunction, param_name: &str) -> bool {
        for stmt in func.body() {
            if Self::statement_uses_pointer_arithmetic(stmt, param_name) {
                return true;
            }
        }
        false
    }

    /// Recursively check if a statement uses pointer arithmetic on a variable.
    fn statement_uses_pointer_arithmetic(stmt: &HirStatement, var_name: &str) -> bool {
        use decy_hir::BinaryOperator;
        match stmt {
            HirStatement::Assignment { target, value } => {
                // Check if this is var = var + n or var = var - n (pointer arithmetic)
                if target == var_name {
                    if let HirExpression::BinaryOp { op, left, .. } = value {
                        if matches!(op, BinaryOperator::Add | BinaryOperator::Subtract) {
                            if let HirExpression::Variable(name) = &**left {
                                if name == var_name {
                                    return true;
                                }
                            }
                        }
                    }
                }
                false
            }
            // DECY-164: Also check for post/pre increment/decrement on the variable
            HirStatement::Expression(expr) => {
                Self::expression_uses_pointer_arithmetic(expr, var_name)
            }
            HirStatement::If {
                then_block,
                else_block,
                ..
            } => {
                then_block
                    .iter()
                    .any(|s| Self::statement_uses_pointer_arithmetic(s, var_name))
                    || else_block.as_ref().is_some_and(|blk| {
                        blk.iter()
                            .any(|s| Self::statement_uses_pointer_arithmetic(s, var_name))
                    })
            }
            HirStatement::While { body, .. } | HirStatement::For { body, .. } => body
                .iter()
                .any(|s| Self::statement_uses_pointer_arithmetic(s, var_name)),
            _ => false,
        }
    }

    /// DECY-164: Check if an expression uses pointer arithmetic on a variable.
    /// Catches str++, ++str, str--, --str patterns.
    fn expression_uses_pointer_arithmetic(expr: &HirExpression, var_name: &str) -> bool {
        match expr {
            HirExpression::PostIncrement { operand }
            | HirExpression::PreIncrement { operand }
            | HirExpression::PostDecrement { operand }
            | HirExpression::PreDecrement { operand } => {
                matches!(&**operand, HirExpression::Variable(name) if name == var_name)
            }
            _ => false,
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
