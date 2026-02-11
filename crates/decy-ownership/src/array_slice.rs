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
        // DECY-163: Don't remove length params when array uses pointer arithmetic
        let mut array_param_map: HashMap<String, Option<String>> = HashMap::new();
        let mut length_params_to_remove: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for (array_param, length_param) in &array_params {
            // DECY-163: Skip arrays that use pointer arithmetic
            // Raw pointers don't have .len(), so we keep the size param as-is
            if Self::uses_pointer_arithmetic(func, array_param) {
                continue;
            }

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
                    // DECY-161: Check if parameter uses pointer arithmetic
                    // If so, it must stay as raw pointer (slices don't support arr++)
                    if Self::uses_pointer_arithmetic(func, param.name()) {
                        // Keep as raw pointer
                        return Some(param.clone());
                    }

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
                // DECY-224: Transform all init statements
                init: init
                    .iter()
                    .map(|s| Self::transform_statement(s, array_param_map))
                    .collect(),
                condition: condition
                    .as_ref()
                    .map(|c| Self::transform_expression(c, array_param_map)),
                // DECY-224: Transform all increment statements
                increment: increment
                    .iter()
                    .map(|s| Self::transform_statement(s, array_param_map))
                    .collect(),
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

impl ArrayParameterTransformer {
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

impl Default for ArrayParameterTransformer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use decy_hir::{
        BinaryOperator, HirExpression, HirFunction, HirParameter, HirStatement, HirType,
    };

    // ============================================================================
    // ARRAY PARAMETER INFO TESTS
    // ============================================================================

    #[test]
    fn test_array_parameter_info_creation() {
        let info = ArrayParameterInfo {
            array_param: "arr".to_string(),
            length_param: Some("len".to_string()),
            is_mutable: true,
        };
        assert_eq!(info.array_param, "arr");
        assert_eq!(info.length_param, Some("len".to_string()));
        assert!(info.is_mutable);
    }

    #[test]
    fn test_array_parameter_info_no_length() {
        let info = ArrayParameterInfo {
            array_param: "data".to_string(),
            length_param: None,
            is_mutable: false,
        };
        assert_eq!(info.array_param, "data");
        assert!(info.length_param.is_none());
        assert!(!info.is_mutable);
    }

    #[test]
    fn test_array_parameter_info_clone() {
        let info = ArrayParameterInfo {
            array_param: "arr".to_string(),
            length_param: Some("size".to_string()),
            is_mutable: true,
        };
        let cloned = info.clone();
        assert_eq!(cloned.array_param, info.array_param);
        assert_eq!(cloned.length_param, info.length_param);
        assert_eq!(cloned.is_mutable, info.is_mutable);
    }

    #[test]
    fn test_array_parameter_info_debug() {
        let info = ArrayParameterInfo {
            array_param: "arr".to_string(),
            length_param: Some("len".to_string()),
            is_mutable: false,
        };
        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("ArrayParameterInfo"));
        assert!(debug_str.contains("arr"));
    }

    // ============================================================================
    // TRANSFORMER CONSTRUCTION TESTS
    // ============================================================================

    #[test]
    fn test_transformer_new() {
        let transformer = ArrayParameterTransformer::new();
        assert!(std::mem::size_of_val(&transformer) == 0);
    }

    #[test]
    fn test_transformer_default() {
        let transformer: ArrayParameterTransformer = Default::default();
        assert!(std::mem::size_of_val(&transformer) == 0);
    }

    // ============================================================================
    // TRANSFORM EXPRESSION TESTS
    // ============================================================================

    #[test]
    fn test_transform_expression_variable_not_length_param() {
        let map: HashMap<String, Option<String>> = HashMap::new();
        let expr = HirExpression::Variable("x".to_string());
        let result = ArrayParameterTransformer::transform_expression(&expr, &map);
        assert!(matches!(result, HirExpression::Variable(name) if name == "x"));
    }

    #[test]
    fn test_transform_expression_variable_is_length_param() {
        let mut map: HashMap<String, Option<String>> = HashMap::new();
        map.insert("arr".to_string(), Some("len".to_string()));

        let expr = HirExpression::Variable("len".to_string());
        let result = ArrayParameterTransformer::transform_expression(&expr, &map);

        // Should transform to arr.len()
        match result {
            HirExpression::StringMethodCall {
                receiver,
                method,
                arguments,
            } => {
                assert_eq!(method, "len");
                assert!(arguments.is_empty());
                match *receiver {
                    HirExpression::Variable(name) => assert_eq!(name, "arr"),
                    _ => panic!("Expected Variable receiver"),
                }
            }
            _ => panic!("Expected StringMethodCall"),
        }
    }

    #[test]
    fn test_transform_expression_binary_op() {
        let map: HashMap<String, Option<String>> = HashMap::new();
        let expr = HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        };
        let result = ArrayParameterTransformer::transform_expression(&expr, &map);
        assert!(matches!(result, HirExpression::BinaryOp { .. }));
    }

    #[test]
    fn test_transform_expression_function_call() {
        let map: HashMap<String, Option<String>> = HashMap::new();
        let expr = HirExpression::FunctionCall {
            function: "test".to_string(),
            arguments: vec![HirExpression::Variable("x".to_string())],
        };
        let result = ArrayParameterTransformer::transform_expression(&expr, &map);
        match result {
            HirExpression::FunctionCall {
                function,
                arguments,
            } => {
                assert_eq!(function, "test");
                assert_eq!(arguments.len(), 1);
            }
            _ => panic!("Expected FunctionCall"),
        }
    }

    #[test]
    fn test_transform_expression_array_index() {
        let map: HashMap<String, Option<String>> = HashMap::new();
        let expr = HirExpression::ArrayIndex {
            array: Box::new(HirExpression::Variable("arr".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
        };
        let result = ArrayParameterTransformer::transform_expression(&expr, &map);
        assert!(matches!(result, HirExpression::ArrayIndex { .. }));
    }

    #[test]
    fn test_transform_expression_cast() {
        let map: HashMap<String, Option<String>> = HashMap::new();
        let expr = HirExpression::Cast {
            expr: Box::new(HirExpression::Variable("x".to_string())),
            target_type: HirType::Int,
        };
        let result = ArrayParameterTransformer::transform_expression(&expr, &map);
        assert!(matches!(result, HirExpression::Cast { .. }));
    }

    #[test]
    fn test_transform_expression_dereference() {
        let map: HashMap<String, Option<String>> = HashMap::new();
        let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("ptr".to_string())));
        let result = ArrayParameterTransformer::transform_expression(&expr, &map);
        assert!(matches!(result, HirExpression::Dereference(_)));
    }

    #[test]
    fn test_transform_expression_address_of() {
        let map: HashMap<String, Option<String>> = HashMap::new();
        let expr = HirExpression::AddressOf(Box::new(HirExpression::Variable("x".to_string())));
        let result = ArrayParameterTransformer::transform_expression(&expr, &map);
        assert!(matches!(result, HirExpression::AddressOf(_)));
    }

    #[test]
    fn test_transform_expression_int_literal_passthrough() {
        let map: HashMap<String, Option<String>> = HashMap::new();
        let expr = HirExpression::IntLiteral(42);
        let result = ArrayParameterTransformer::transform_expression(&expr, &map);
        assert!(matches!(result, HirExpression::IntLiteral(42)));
    }

    // ============================================================================
    // TRANSFORM STATEMENT TESTS
    // ============================================================================

    #[test]
    fn test_transform_statement_variable_declaration() {
        let map: HashMap<String, Option<String>> = HashMap::new();
        let stmt = HirStatement::VariableDeclaration {
            name: "x".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(10)),
        };
        let result = ArrayParameterTransformer::transform_statement(&stmt, &map);
        match result {
            HirStatement::VariableDeclaration {
                name, initializer, ..
            } => {
                assert_eq!(name, "x");
                assert!(initializer.is_some());
            }
            _ => panic!("Expected VariableDeclaration"),
        }
    }

    #[test]
    fn test_transform_statement_assignment() {
        let map: HashMap<String, Option<String>> = HashMap::new();
        let stmt = HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::IntLiteral(5),
        };
        let result = ArrayParameterTransformer::transform_statement(&stmt, &map);
        assert!(matches!(result, HirStatement::Assignment { .. }));
    }

    #[test]
    fn test_transform_statement_if() {
        let map: HashMap<String, Option<String>> = HashMap::new();
        let stmt = HirStatement::If {
            condition: HirExpression::Variable("cond".to_string()),
            then_block: vec![HirStatement::Break],
            else_block: None,
        };
        let result = ArrayParameterTransformer::transform_statement(&stmt, &map);
        match result {
            HirStatement::If {
                then_block,
                else_block,
                ..
            } => {
                assert_eq!(then_block.len(), 1);
                assert!(else_block.is_none());
            }
            _ => panic!("Expected If"),
        }
    }

    #[test]
    fn test_transform_statement_if_else() {
        let map: HashMap<String, Option<String>> = HashMap::new();
        let stmt = HirStatement::If {
            condition: HirExpression::Variable("cond".to_string()),
            then_block: vec![HirStatement::Break],
            else_block: Some(vec![HirStatement::Continue]),
        };
        let result = ArrayParameterTransformer::transform_statement(&stmt, &map);
        match result {
            HirStatement::If { else_block, .. } => {
                assert!(else_block.is_some());
                assert_eq!(else_block.unwrap().len(), 1);
            }
            _ => panic!("Expected If"),
        }
    }

    #[test]
    fn test_transform_statement_while() {
        let map: HashMap<String, Option<String>> = HashMap::new();
        let stmt = HirStatement::While {
            condition: HirExpression::Variable("running".to_string()),
            body: vec![HirStatement::Break],
        };
        let result = ArrayParameterTransformer::transform_statement(&stmt, &map);
        assert!(matches!(result, HirStatement::While { .. }));
    }

    #[test]
    fn test_transform_statement_for() {
        let map: HashMap<String, Option<String>> = HashMap::new();
        let stmt = HirStatement::For {
            init: vec![HirStatement::VariableDeclaration {
                name: "i".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            }],
            condition: Some(HirExpression::BinaryOp {
                op: BinaryOperator::LessThan,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::IntLiteral(10)),
            }),
            increment: vec![HirStatement::Assignment {
                target: "i".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("i".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            }],
            body: vec![HirStatement::Break],
        };
        let result = ArrayParameterTransformer::transform_statement(&stmt, &map);
        assert!(matches!(result, HirStatement::For { .. }));
    }

    #[test]
    fn test_transform_statement_return() {
        let map: HashMap<String, Option<String>> = HashMap::new();
        let stmt = HirStatement::Return(Some(HirExpression::Variable("result".to_string())));
        let result = ArrayParameterTransformer::transform_statement(&stmt, &map);
        assert!(matches!(result, HirStatement::Return(Some(_))));
    }

    #[test]
    fn test_transform_statement_return_void() {
        let map: HashMap<String, Option<String>> = HashMap::new();
        let stmt = HirStatement::Return(None);
        let result = ArrayParameterTransformer::transform_statement(&stmt, &map);
        assert!(matches!(result, HirStatement::Return(None)));
    }

    #[test]
    fn test_transform_statement_expression() {
        let map: HashMap<String, Option<String>> = HashMap::new();
        let stmt = HirStatement::Expression(HirExpression::FunctionCall {
            function: "print".to_string(),
            arguments: vec![],
        });
        let result = ArrayParameterTransformer::transform_statement(&stmt, &map);
        assert!(matches!(result, HirStatement::Expression(_)));
    }

    #[test]
    fn test_transform_statement_break_passthrough() {
        let map: HashMap<String, Option<String>> = HashMap::new();
        let stmt = HirStatement::Break;
        let result = ArrayParameterTransformer::transform_statement(&stmt, &map);
        assert!(matches!(result, HirStatement::Break));
    }

    #[test]
    fn test_transform_statement_continue_passthrough() {
        let map: HashMap<String, Option<String>> = HashMap::new();
        let stmt = HirStatement::Continue;
        let result = ArrayParameterTransformer::transform_statement(&stmt, &map);
        assert!(matches!(result, HirStatement::Continue));
    }

    // ============================================================================
    // POINTER ARITHMETIC DETECTION TESTS
    // ============================================================================

    #[test]
    fn test_uses_pointer_arithmetic_empty_body() {
        let func = HirFunction::new(
            "test".to_string(),
            HirType::Void,
            vec![HirParameter::new(
                "arr".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            )],
        );
        assert!(!ArrayParameterTransformer::uses_pointer_arithmetic(
            &func, "arr"
        ));
    }

    #[test]
    fn test_expression_uses_pointer_arithmetic_post_increment() {
        let expr = HirExpression::PostIncrement {
            operand: Box::new(HirExpression::Variable("ptr".to_string())),
        };
        assert!(ArrayParameterTransformer::expression_uses_pointer_arithmetic(&expr, "ptr"));
        assert!(!ArrayParameterTransformer::expression_uses_pointer_arithmetic(&expr, "other"));
    }

    #[test]
    fn test_expression_uses_pointer_arithmetic_pre_increment() {
        let expr = HirExpression::PreIncrement {
            operand: Box::new(HirExpression::Variable("ptr".to_string())),
        };
        assert!(ArrayParameterTransformer::expression_uses_pointer_arithmetic(&expr, "ptr"));
    }

    #[test]
    fn test_expression_uses_pointer_arithmetic_post_decrement() {
        let expr = HirExpression::PostDecrement {
            operand: Box::new(HirExpression::Variable("ptr".to_string())),
        };
        assert!(ArrayParameterTransformer::expression_uses_pointer_arithmetic(&expr, "ptr"));
    }

    #[test]
    fn test_expression_uses_pointer_arithmetic_pre_decrement() {
        let expr = HirExpression::PreDecrement {
            operand: Box::new(HirExpression::Variable("ptr".to_string())),
        };
        assert!(ArrayParameterTransformer::expression_uses_pointer_arithmetic(&expr, "ptr"));
    }

    #[test]
    fn test_expression_uses_pointer_arithmetic_other_expr() {
        let expr = HirExpression::Variable("ptr".to_string());
        assert!(!ArrayParameterTransformer::expression_uses_pointer_arithmetic(&expr, "ptr"));
    }

    #[test]
    fn test_statement_uses_pointer_arithmetic_assignment_add() {
        let stmt = HirStatement::Assignment {
            target: "arr".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("arr".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        };
        assert!(ArrayParameterTransformer::statement_uses_pointer_arithmetic(&stmt, "arr"));
    }

    #[test]
    fn test_statement_uses_pointer_arithmetic_assignment_subtract() {
        let stmt = HirStatement::Assignment {
            target: "arr".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Subtract,
                left: Box::new(HirExpression::Variable("arr".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        };
        assert!(ArrayParameterTransformer::statement_uses_pointer_arithmetic(&stmt, "arr"));
    }

    #[test]
    fn test_statement_uses_pointer_arithmetic_different_variable() {
        let stmt = HirStatement::Assignment {
            target: "other".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("other".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        };
        assert!(!ArrayParameterTransformer::statement_uses_pointer_arithmetic(&stmt, "arr"));
    }

    #[test]
    fn test_statement_uses_pointer_arithmetic_in_if_then() {
        let stmt = HirStatement::If {
            condition: HirExpression::Variable("cond".to_string()),
            then_block: vec![HirStatement::Expression(HirExpression::PostIncrement {
                operand: Box::new(HirExpression::Variable("ptr".to_string())),
            })],
            else_block: None,
        };
        assert!(ArrayParameterTransformer::statement_uses_pointer_arithmetic(&stmt, "ptr"));
    }

    #[test]
    fn test_statement_uses_pointer_arithmetic_in_if_else() {
        let stmt = HirStatement::If {
            condition: HirExpression::Variable("cond".to_string()),
            then_block: vec![],
            else_block: Some(vec![HirStatement::Expression(
                HirExpression::PostIncrement {
                    operand: Box::new(HirExpression::Variable("ptr".to_string())),
                },
            )]),
        };
        assert!(ArrayParameterTransformer::statement_uses_pointer_arithmetic(&stmt, "ptr"));
    }

    #[test]
    fn test_statement_uses_pointer_arithmetic_in_while() {
        let stmt = HirStatement::While {
            condition: HirExpression::Variable("cond".to_string()),
            body: vec![HirStatement::Expression(HirExpression::PostIncrement {
                operand: Box::new(HirExpression::Variable("ptr".to_string())),
            })],
        };
        assert!(ArrayParameterTransformer::statement_uses_pointer_arithmetic(&stmt, "ptr"));
    }

    #[test]
    fn test_statement_uses_pointer_arithmetic_in_for() {
        let stmt = HirStatement::For {
            init: vec![],
            condition: Some(HirExpression::Variable("cond".to_string())),
            increment: vec![],
            body: vec![HirStatement::Expression(HirExpression::PostIncrement {
                operand: Box::new(HirExpression::Variable("ptr".to_string())),
            })],
        };
        assert!(ArrayParameterTransformer::statement_uses_pointer_arithmetic(&stmt, "ptr"));
    }

    // ============================================================================
    // ADDITIONAL COVERAGE: ArrayIndexAssignment, UnaryOp, edge cases
    // ============================================================================

    #[test]
    fn test_transform_statement_array_index_assignment() {
        let mut map: HashMap<String, Option<String>> = HashMap::new();
        map.insert("arr".to_string(), Some("len".to_string()));

        let stmt = HirStatement::ArrayIndexAssignment {
            array: Box::new(HirExpression::Variable("arr".to_string())),
            index: Box::new(HirExpression::Variable("len".to_string())),
            value: HirExpression::IntLiteral(42),
        };
        let result = ArrayParameterTransformer::transform_statement(&stmt, &map);
        match result {
            HirStatement::ArrayIndexAssignment {
                array,
                index,
                value,
            } => {
                // array stays as Variable (not a length param)
                assert!(matches!(*array, HirExpression::Variable(_)));
                // index was "len" → should be transformed to arr.len()
                assert!(matches!(*index, HirExpression::StringMethodCall { .. }));
                assert!(matches!(value, HirExpression::IntLiteral(42)));
            }
            _ => panic!("Expected ArrayIndexAssignment"),
        }
    }

    #[test]
    fn test_transform_expression_unary_op() {
        let map: HashMap<String, Option<String>> = HashMap::new();
        let expr = HirExpression::UnaryOp {
            op: decy_hir::UnaryOperator::Minus,
            operand: Box::new(HirExpression::Variable("x".to_string())),
        };
        let result = ArrayParameterTransformer::transform_expression(&expr, &map);
        match result {
            HirExpression::UnaryOp { op, operand } => {
                assert!(matches!(op, decy_hir::UnaryOperator::Minus));
                assert!(matches!(*operand, HirExpression::Variable(ref n) if n == "x"));
            }
            _ => panic!("Expected UnaryOp"),
        }
    }

    #[test]
    fn test_transform_statement_var_decl_no_initializer() {
        let map: HashMap<String, Option<String>> = HashMap::new();
        let stmt = HirStatement::VariableDeclaration {
            name: "x".to_string(),
            var_type: HirType::Int,
            initializer: None,
        };
        let result = ArrayParameterTransformer::transform_statement(&stmt, &map);
        match result {
            HirStatement::VariableDeclaration { initializer, .. } => {
                assert!(initializer.is_none());
            }
            _ => panic!("Expected VariableDeclaration"),
        }
    }

    #[test]
    fn test_transform_expression_binary_op_with_length_param() {
        let mut map: HashMap<String, Option<String>> = HashMap::new();
        map.insert("arr".to_string(), Some("len".to_string()));

        let expr = HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::Variable("len".to_string())),
        };
        let result = ArrayParameterTransformer::transform_expression(&expr, &map);
        match result {
            HirExpression::BinaryOp { right, .. } => {
                // "len" should be transformed to arr.len()
                assert!(matches!(*right, HirExpression::StringMethodCall { .. }));
            }
            _ => panic!("Expected BinaryOp"),
        }
    }

    #[test]
    fn test_transform_expression_function_call_with_length_param() {
        let mut map: HashMap<String, Option<String>> = HashMap::new();
        map.insert("arr".to_string(), Some("size".to_string()));

        let expr = HirExpression::FunctionCall {
            function: "process".to_string(),
            arguments: vec![
                HirExpression::Variable("arr".to_string()),
                HirExpression::Variable("size".to_string()),
            ],
        };
        let result = ArrayParameterTransformer::transform_expression(&expr, &map);
        match result {
            HirExpression::FunctionCall { arguments, .. } => {
                assert_eq!(arguments.len(), 2);
                // "size" arg should be transformed to arr.len()
                assert!(matches!(arguments[1], HirExpression::StringMethodCall { .. }));
            }
            _ => panic!("Expected FunctionCall"),
        }
    }

    #[test]
    fn test_transform_expression_cast_with_length_param() {
        let mut map: HashMap<String, Option<String>> = HashMap::new();
        map.insert("arr".to_string(), Some("n".to_string()));

        let expr = HirExpression::Cast {
            expr: Box::new(HirExpression::Variable("n".to_string())),
            target_type: HirType::Int,
        };
        let result = ArrayParameterTransformer::transform_expression(&expr, &map);
        match result {
            HirExpression::Cast { expr, .. } => {
                assert!(matches!(*expr, HirExpression::StringMethodCall { .. }));
            }
            _ => panic!("Expected Cast"),
        }
    }

    #[test]
    fn test_transform_expression_dereference_with_length_param() {
        let mut map: HashMap<String, Option<String>> = HashMap::new();
        map.insert("arr".to_string(), Some("len".to_string()));

        let expr =
            HirExpression::Dereference(Box::new(HirExpression::Variable("len".to_string())));
        let result = ArrayParameterTransformer::transform_expression(&expr, &map);
        match result {
            HirExpression::Dereference(inner) => {
                assert!(matches!(*inner, HirExpression::StringMethodCall { .. }));
            }
            _ => panic!("Expected Dereference"),
        }
    }

    #[test]
    fn test_transform_expression_address_of_with_length_param() {
        let mut map: HashMap<String, Option<String>> = HashMap::new();
        map.insert("arr".to_string(), Some("len".to_string()));

        let expr = HirExpression::AddressOf(Box::new(HirExpression::Variable("len".to_string())));
        let result = ArrayParameterTransformer::transform_expression(&expr, &map);
        match result {
            HirExpression::AddressOf(inner) => {
                assert!(matches!(*inner, HirExpression::StringMethodCall { .. }));
            }
            _ => panic!("Expected AddressOf"),
        }
    }

    #[test]
    fn test_transform_expression_array_index_with_length_param() {
        let mut map: HashMap<String, Option<String>> = HashMap::new();
        map.insert("arr".to_string(), Some("len".to_string()));

        let expr = HirExpression::ArrayIndex {
            array: Box::new(HirExpression::Variable("arr".to_string())),
            index: Box::new(HirExpression::Variable("len".to_string())),
        };
        let result = ArrayParameterTransformer::transform_expression(&expr, &map);
        match result {
            HirExpression::ArrayIndex { index, .. } => {
                assert!(matches!(*index, HirExpression::StringMethodCall { .. }));
            }
            _ => panic!("Expected ArrayIndex"),
        }
    }

    #[test]
    fn test_statement_uses_pointer_arithmetic_non_matching_assignment() {
        // Assignment with non-arithmetic operation (e.g., multiply)
        let stmt = HirStatement::Assignment {
            target: "arr".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(HirExpression::Variable("arr".to_string())),
                right: Box::new(HirExpression::IntLiteral(2)),
            },
        };
        assert!(!ArrayParameterTransformer::statement_uses_pointer_arithmetic(&stmt, "arr"));
    }

    #[test]
    fn test_statement_uses_pointer_arithmetic_assignment_different_left() {
        // target matches but left side of BinaryOp is different variable
        let stmt = HirStatement::Assignment {
            target: "arr".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("other".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        };
        assert!(!ArrayParameterTransformer::statement_uses_pointer_arithmetic(&stmt, "arr"));
    }

    #[test]
    fn test_statement_uses_pointer_arithmetic_non_binary_value() {
        // Assignment where value is not a BinaryOp
        let stmt = HirStatement::Assignment {
            target: "arr".to_string(),
            value: HirExpression::IntLiteral(0),
        };
        assert!(!ArrayParameterTransformer::statement_uses_pointer_arithmetic(&stmt, "arr"));
    }

    #[test]
    fn test_statement_uses_pointer_arithmetic_if_no_else() {
        // If with no else block, no arithmetic in then
        let stmt = HirStatement::If {
            condition: HirExpression::Variable("flag".to_string()),
            then_block: vec![HirStatement::Break],
            else_block: None,
        };
        assert!(!ArrayParameterTransformer::statement_uses_pointer_arithmetic(&stmt, "ptr"));
    }

    #[test]
    fn test_statement_uses_pointer_arithmetic_break_passthrough() {
        let stmt = HirStatement::Break;
        assert!(!ArrayParameterTransformer::statement_uses_pointer_arithmetic(&stmt, "ptr"));
    }

    #[test]
    fn test_uses_pointer_arithmetic_multiple_statements() {
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![HirParameter::new(
                "arr".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            )],
            vec![
                HirStatement::VariableDeclaration {
                    name: "x".to_string(),
                    var_type: HirType::Int,
                    initializer: Some(HirExpression::IntLiteral(0)),
                },
                HirStatement::Assignment {
                    target: "arr".to_string(),
                    value: HirExpression::BinaryOp {
                        op: BinaryOperator::Add,
                        left: Box::new(HirExpression::Variable("arr".to_string())),
                        right: Box::new(HirExpression::IntLiteral(1)),
                    },
                },
            ],
        );
        assert!(ArrayParameterTransformer::uses_pointer_arithmetic(
            &func, "arr"
        ));
    }

    #[test]
    fn test_transform_statement_if_with_length_params() {
        let mut map: HashMap<String, Option<String>> = HashMap::new();
        map.insert("arr".to_string(), Some("len".to_string()));

        let stmt = HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::LessThan,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::Variable("len".to_string())),
            },
            then_block: vec![HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::Variable("len".to_string()),
            }],
            else_block: Some(vec![HirStatement::Assignment {
                target: "y".to_string(),
                value: HirExpression::Variable("len".to_string()),
            }]),
        };
        let result = ArrayParameterTransformer::transform_statement(&stmt, &map);
        match result {
            HirStatement::If {
                condition,
                then_block,
                else_block,
            } => {
                // condition should have len transformed
                if let HirExpression::BinaryOp { right, .. } = condition {
                    assert!(matches!(*right, HirExpression::StringMethodCall { .. }));
                }
                // then_block assignment value should be transformed
                if let HirStatement::Assignment { value, .. } = &then_block[0] {
                    assert!(matches!(value, HirExpression::StringMethodCall { .. }));
                }
                // else_block assignment value should be transformed
                let else_stmts = else_block.unwrap();
                if let HirStatement::Assignment { value, .. } = &else_stmts[0] {
                    assert!(matches!(value, HirExpression::StringMethodCall { .. }));
                }
            }
            _ => panic!("Expected If"),
        }
    }

    #[test]
    fn test_transform_statement_while_with_length_param() {
        let mut map: HashMap<String, Option<String>> = HashMap::new();
        map.insert("arr".to_string(), Some("n".to_string()));

        let stmt = HirStatement::While {
            condition: HirExpression::Variable("n".to_string()),
            body: vec![HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::Variable("n".to_string()),
            }],
        };
        let result = ArrayParameterTransformer::transform_statement(&stmt, &map);
        match result {
            HirStatement::While { condition, body } => {
                assert!(matches!(condition, HirExpression::StringMethodCall { .. }));
                if let HirStatement::Assignment { value, .. } = &body[0] {
                    assert!(matches!(value, HirExpression::StringMethodCall { .. }));
                }
            }
            _ => panic!("Expected While"),
        }
    }

    #[test]
    fn test_transform_statement_for_with_length_param() {
        let mut map: HashMap<String, Option<String>> = HashMap::new();
        map.insert("arr".to_string(), Some("n".to_string()));

        let stmt = HirStatement::For {
            init: vec![HirStatement::VariableDeclaration {
                name: "i".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            }],
            condition: Some(HirExpression::BinaryOp {
                op: BinaryOperator::LessThan,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::Variable("n".to_string())),
            }),
            increment: vec![HirStatement::Assignment {
                target: "i".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("i".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            }],
            body: vec![HirStatement::Return(Some(HirExpression::Variable(
                "n".to_string(),
            )))],
        };
        let result = ArrayParameterTransformer::transform_statement(&stmt, &map);
        match result {
            HirStatement::For {
                condition, body, ..
            } => {
                // Condition should have "n" transformed
                let cond = condition.unwrap();
                if let HirExpression::BinaryOp { right, .. } = cond {
                    assert!(matches!(*right, HirExpression::StringMethodCall { .. }));
                }
                // Return in body should have "n" transformed
                if let HirStatement::Return(Some(expr)) = &body[0] {
                    assert!(matches!(expr, HirExpression::StringMethodCall { .. }));
                }
            }
            _ => panic!("Expected For"),
        }
    }

    #[test]
    fn test_transform_statement_expression_with_length_param() {
        let mut map: HashMap<String, Option<String>> = HashMap::new();
        map.insert("arr".to_string(), Some("sz".to_string()));

        let stmt = HirStatement::Expression(HirExpression::FunctionCall {
            function: "print".to_string(),
            arguments: vec![HirExpression::Variable("sz".to_string())],
        });
        let result = ArrayParameterTransformer::transform_statement(&stmt, &map);
        match result {
            HirStatement::Expression(HirExpression::FunctionCall { arguments, .. }) => {
                assert!(matches!(
                    arguments[0],
                    HirExpression::StringMethodCall { .. }
                ));
            }
            _ => panic!("Expected Expression(FunctionCall)"),
        }
    }

    #[test]
    fn test_transform_expression_variable_array_param_no_length() {
        // Array param with None length — variable should not be transformed
        let mut map: HashMap<String, Option<String>> = HashMap::new();
        map.insert("arr".to_string(), None);

        let expr = HirExpression::Variable("arr".to_string());
        let result = ArrayParameterTransformer::transform_expression(&expr, &map);
        assert!(matches!(result, HirExpression::Variable(ref n) if n == "arr"));
    }

    // ============================================================================
    // TRANSFORM METHOD INTEGRATION TESTS (covers lines 49-136)
    // ============================================================================

    #[test]
    fn test_transform_array_param_to_immutable_slice() {
        // void process(int* arr, int len) { return len; }
        // → fn process(arr: &[i32]) { return arr.len(); }
        let func = HirFunction::new_with_body(
            "process".to_string(),
            HirType::Void,
            vec![
                HirParameter::new(
                    "arr".to_string(),
                    HirType::Pointer(Box::new(HirType::Int)),
                ),
                HirParameter::new("len".to_string(), HirType::Int),
            ],
            vec![HirStatement::Return(Some(HirExpression::Variable(
                "len".to_string(),
            )))],
        );

        let dfg = crate::dataflow::DataflowAnalyzer::new().analyze(&func);

        let transformer = ArrayParameterTransformer::new();
        let result = transformer.transform(&func, &dfg);

        // arr parameter should be transformed to a slice
        assert_eq!(result.parameters().len(), 1, "len param should be removed");
        assert_eq!(result.parameters()[0].name(), "arr");
        match result.parameters()[0].param_type() {
            HirType::Reference { inner, mutable } => {
                assert!(!mutable, "Should be immutable slice (not modified)");
                assert!(matches!(**inner, HirType::Array { ref element_type, size: None }
                    if matches!(**element_type, HirType::Int)));
            }
            other => panic!("Expected Reference, got {:?}", other),
        }

        // Body should have len replaced with arr.len()
        match &result.body()[0] {
            HirStatement::Return(Some(HirExpression::StringMethodCall {
                method, ..
            })) => {
                assert_eq!(method, "len");
            }
            other => panic!("Expected Return with .len() call, got {:?}", other),
        }
    }

    #[test]
    fn test_transform_mutable_array_param_to_mut_slice() {
        // void modify(int* data, int size) { data[0] = 42; }
        // → fn modify(data: &mut [i32]) { data[0] = 42; }
        let func = HirFunction::new_with_body(
            "modify".to_string(),
            HirType::Void,
            vec![
                HirParameter::new(
                    "data".to_string(),
                    HirType::Pointer(Box::new(HirType::Int)),
                ),
                HirParameter::new("size".to_string(), HirType::Int),
            ],
            vec![HirStatement::ArrayIndexAssignment {
                array: Box::new(HirExpression::Variable("data".to_string())),
                index: Box::new(HirExpression::IntLiteral(0)),
                value: HirExpression::IntLiteral(42),
            }],
        );

        let dfg = crate::dataflow::DataflowAnalyzer::new().analyze(&func);
        let transformer = ArrayParameterTransformer::new();
        let result = transformer.transform(&func, &dfg);

        // data param should be &mut [i32] (it's modified via array index assignment)
        assert_eq!(result.parameters().len(), 1, "size param should be removed");
        assert_eq!(result.parameters()[0].name(), "data");
        match result.parameters()[0].param_type() {
            HirType::Reference { mutable, .. } => {
                assert!(mutable, "Should be mutable slice (data is modified)");
            }
            other => panic!("Expected Reference, got {:?}", other),
        }
    }

    #[test]
    fn test_transform_skips_pointer_arithmetic_params() {
        // void walk(int* arr, int len) { arr = arr + 1; }
        // Pointer arithmetic → keep as raw pointer, don't transform
        let func = HirFunction::new_with_body(
            "walk".to_string(),
            HirType::Void,
            vec![
                HirParameter::new(
                    "arr".to_string(),
                    HirType::Pointer(Box::new(HirType::Int)),
                ),
                HirParameter::new("len".to_string(), HirType::Int),
            ],
            vec![HirStatement::Assignment {
                target: "arr".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("arr".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            }],
        );

        let dfg = crate::dataflow::DataflowAnalyzer::new().analyze(&func);
        let transformer = ArrayParameterTransformer::new();
        let result = transformer.transform(&func, &dfg);

        // Both params should be kept (no transformation due to pointer arithmetic)
        assert_eq!(
            result.parameters().len(),
            2,
            "Both params should be kept when pointer arithmetic is used"
        );
        assert_eq!(result.parameters()[0].name(), "arr");
        assert_eq!(result.parameters()[1].name(), "len");
        // arr should still be a raw pointer, not a slice
        assert!(
            matches!(result.parameters()[0].param_type(), HirType::Pointer(_)),
            "arr should remain as Pointer type"
        );
    }

    #[test]
    fn test_transform_no_array_params_returns_unchanged() {
        // void add(int a, int b) { return a + b; }
        // No array params → function returned unchanged
        let func = HirFunction::new_with_body(
            "add".to_string(),
            HirType::Int,
            vec![
                HirParameter::new("a".to_string(), HirType::Int),
                HirParameter::new("b".to_string(), HirType::Int),
            ],
            vec![HirStatement::Return(Some(HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            }))],
        );

        let dfg = crate::dataflow::DataflowAnalyzer::new().analyze(&func);
        let transformer = ArrayParameterTransformer::new();
        let result = transformer.transform(&func, &dfg);

        // Should return unchanged (no array params)
        assert_eq!(result.parameters().len(), 2);
        assert_eq!(result.parameters()[0].name(), "a");
        assert_eq!(result.parameters()[1].name(), "b");
    }
}
