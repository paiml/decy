//! `Box<T>` transformations for replacing malloc/free patterns.
//!
//! Transforms malloc() calls into safe Rust `Box::new()` expressions.

use decy_analyzer::patterns::BoxCandidate;
use decy_hir::{HirExpression, HirStatement, HirType};

/// Transformer for converting malloc patterns to `Box::new()`.
#[derive(Debug, Clone)]
pub struct BoxTransformer;

impl BoxTransformer {
    /// Create a new Box transformer.
    pub fn new() -> Self {
        Self
    }

    /// Transform a malloc expression into a `Box::new()` expression.
    ///
    /// Converts patterns like:
    /// - `malloc(sizeof(T))` → `Box::new(T::default())`
    /// - `malloc(size)` → `Box::new(T::default())`
    ///
    /// For now, we generate a default-initialized Box since we don't have
    /// sizeof analysis yet. This will be enhanced in future phases.
    pub fn transform_malloc_to_box(
        &self,
        _malloc_expr: &HirExpression,
        pointee_type: &HirType,
    ) -> HirExpression {
        // Generate Box::new(default_value) based on pointee type
        let default_value = self.default_value_for_type(pointee_type);

        HirExpression::FunctionCall {
            function: "Box::new".to_string(),
            arguments: vec![default_value],
        }
    }

    /// Generate a default value expression for a type.
    fn default_value_for_type(&self, hir_type: &HirType) -> HirExpression {
        match hir_type {
            HirType::Int | HirType::UnsignedInt => HirExpression::IntLiteral(0), // DECY-158
            HirType::Float | HirType::Double => {
                // We'll use 0 for now since we only have IntLiteral
                // Future: Add FloatLiteral to HirExpression
                HirExpression::IntLiteral(0)
            }
            HirType::Char => HirExpression::IntLiteral(0),
            HirType::Option(_) => {
                // Option types default to None
                HirExpression::NullLiteral
            }
            HirType::Void
            | HirType::Pointer(_)
            | HirType::Box(_)
            | HirType::Vec(_)
            | HirType::Reference { .. }
            | HirType::Struct(_)
            | HirType::Enum(_)
            | HirType::Union(_)
            | HirType::Array { .. }
            | HirType::FunctionPointer { .. }
            | HirType::StringLiteral
            | HirType::OwnedString
            | HirType::StringReference => {
                // Fallback for types that don't have simple defaults
                HirExpression::IntLiteral(0)
            }
        }
    }

    /// Transform a statement containing malloc into one using Box::new().
    ///
    /// Takes a VariableDeclaration or Assignment with malloc and transforms it.
    /// Converts both the malloc expression AND the variable type from Pointer to Box.
    pub fn transform_statement(
        &self,
        stmt: &HirStatement,
        _candidate: &BoxCandidate,
    ) -> HirStatement {
        match stmt {
            HirStatement::VariableDeclaration {
                name,
                var_type,
                initializer,
            } => {
                if let Some(HirExpression::FunctionCall { function, .. }) = initializer {
                    if function == "malloc" {
                        // Extract pointee type from pointer type
                        if let HirType::Pointer(pointee) = var_type {
                            let box_expr = self
                                .transform_malloc_to_box(initializer.as_ref().unwrap(), pointee);

                            // Convert Pointer type to Box type
                            let box_type = HirType::Box(pointee.clone());

                            return HirStatement::VariableDeclaration {
                                name: name.clone(),
                                var_type: box_type,
                                initializer: Some(box_expr),
                            };
                        }
                    }
                }
                stmt.clone()
            }
            HirStatement::Assignment { target, value } => {
                if let HirExpression::FunctionCall { function, .. } = value {
                    if function == "malloc" {
                        // For assignments, we don't have type info directly
                        // Assume int pointer for now (will be enhanced with type inference)
                        let box_expr = self.transform_malloc_to_box(value, &HirType::Int);

                        return HirStatement::Assignment {
                            target: target.clone(),
                            value: box_expr,
                        };
                    }
                }
                stmt.clone()
            }
            _ => stmt.clone(),
        }
    }
}

impl Default for BoxTransformer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_malloc_to_box_int() {
        let transformer = BoxTransformer::new();
        let malloc_expr = HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::IntLiteral(100)],
        };

        let box_expr = transformer.transform_malloc_to_box(&malloc_expr, &HirType::Int);

        match box_expr {
            HirExpression::FunctionCall {
                function,
                arguments,
            } => {
                assert_eq!(function, "Box::new");
                assert_eq!(arguments.len(), 1);
                assert_eq!(arguments[0], HirExpression::IntLiteral(0));
            }
            _ => panic!("Expected FunctionCall"),
        }
    }

    #[test]
    fn test_transform_variable_declaration_with_malloc() {
        let transformer = BoxTransformer::new();
        let candidate = BoxCandidate {
            variable: "ptr".to_string(),
            malloc_index: 0,
            free_index: None,
        };

        let stmt = HirStatement::VariableDeclaration {
            name: "ptr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::IntLiteral(100)],
            }),
        };

        let transformed = transformer.transform_statement(&stmt, &candidate);

        match transformed {
            HirStatement::VariableDeclaration {
                name,
                initializer: Some(HirExpression::FunctionCall { function, .. }),
                ..
            } => {
                assert_eq!(name, "ptr");
                assert_eq!(function, "Box::new");
            }
            _ => panic!("Expected VariableDeclaration with Box::new"),
        }
    }

    #[test]
    fn test_transform_assignment_with_malloc() {
        let transformer = BoxTransformer::new();
        let candidate = BoxCandidate {
            variable: "ptr".to_string(),
            malloc_index: 0,
            free_index: None,
        };

        let stmt = HirStatement::Assignment {
            target: "ptr".to_string(),
            value: HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::IntLiteral(50)],
            },
        };

        let transformed = transformer.transform_statement(&stmt, &candidate);

        match transformed {
            HirStatement::Assignment {
                target,
                value: HirExpression::FunctionCall { function, .. },
            } => {
                assert_eq!(target, "ptr");
                assert_eq!(function, "Box::new");
            }
            _ => panic!("Expected Assignment with Box::new"),
        }
    }

    #[test]
    fn test_non_malloc_statement_unchanged() {
        let transformer = BoxTransformer::new();
        let candidate = BoxCandidate {
            variable: "x".to_string(),
            malloc_index: 0,
            free_index: None,
        };

        let stmt = HirStatement::VariableDeclaration {
            name: "x".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(42)),
        };

        let transformed = transformer.transform_statement(&stmt, &candidate);
        assert_eq!(transformed, stmt);
    }

    #[test]
    fn test_default_value_generation() {
        let transformer = BoxTransformer::new();

        let int_default = transformer.default_value_for_type(&HirType::Int);
        assert_eq!(int_default, HirExpression::IntLiteral(0));

        let char_default = transformer.default_value_for_type(&HirType::Char);
        assert_eq!(char_default, HirExpression::IntLiteral(0));
    }

    #[test]
    fn test_transform_generates_box_type() {
        let transformer = BoxTransformer::new();
        let candidate = BoxCandidate {
            variable: "ptr".to_string(),
            malloc_index: 0,
            free_index: None,
        };

        let stmt = HirStatement::VariableDeclaration {
            name: "ptr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::IntLiteral(100)],
            }),
        };

        let transformed = transformer.transform_statement(&stmt, &candidate);

        match transformed {
            HirStatement::VariableDeclaration {
                name,
                var_type: HirType::Box(inner),
                initializer: Some(HirExpression::FunctionCall { function, .. }),
                ..
            } => {
                assert_eq!(name, "ptr");
                assert_eq!(function, "Box::new");
                assert_eq!(*inner, HirType::Int);
            }
            _ => panic!("Expected VariableDeclaration with Box<T> type"),
        }
    }

    #[test]
    fn test_box_type_with_different_pointee() {
        let transformer = BoxTransformer::new();
        let candidate = BoxCandidate {
            variable: "data".to_string(),
            malloc_index: 0,
            free_index: None,
        };

        let stmt = HirStatement::VariableDeclaration {
            name: "data".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Char)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::IntLiteral(50)],
            }),
        };

        let transformed = transformer.transform_statement(&stmt, &candidate);

        match transformed {
            HirStatement::VariableDeclaration {
                var_type: HirType::Box(inner),
                ..
            } => {
                assert_eq!(*inner, HirType::Char);
            }
            _ => panic!("Expected Box<char> type"),
        }
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Transform never panics
        #[test]
        fn property_transform_never_panics(
            var_name in "[a-z_][a-z0-9_]{0,10}",
            size in 1i32..1000
        ) {
            let transformer = BoxTransformer::new();
            let candidate = BoxCandidate {
                variable: var_name.clone(),
                malloc_index: 0,
                free_index: None,
            };

            let stmt = HirStatement::VariableDeclaration {
                name: var_name,
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(size)],
                }),
            };

            let _transformed = transformer.transform_statement(&stmt, &candidate);
            // If we get here without panic, test passes
        }

        /// Property: Transformed malloc always becomes Box::new
        #[test]
        fn property_malloc_becomes_box_new(
            size in 1i32..1000
        ) {
            let transformer = BoxTransformer::new();
            let malloc_expr = HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::IntLiteral(size)],
            };

            let box_expr = transformer.transform_malloc_to_box(&malloc_expr, &HirType::Int);

            match box_expr {
                HirExpression::FunctionCall { function, .. } => {
                    prop_assert_eq!(function, "Box::new");
                }
                _ => prop_assert!(false, "Expected FunctionCall"),
            }
        }

        /// Property: Box::new always has exactly one argument
        #[test]
        fn property_box_new_has_one_arg(
            size in 1i32..1000
        ) {
            let transformer = BoxTransformer::new();
            let malloc_expr = HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::IntLiteral(size)],
            };

            let box_expr = transformer.transform_malloc_to_box(&malloc_expr, &HirType::Int);

            match box_expr {
                HirExpression::FunctionCall { arguments, .. } => {
                    prop_assert_eq!(arguments.len(), 1);
                }
                _ => prop_assert!(false, "Expected FunctionCall"),
            }
        }

        /// Property: Transform preserves variable name
        #[test]
        fn property_transform_preserves_name(
            var_name in "[a-z_][a-z0-9_]{0,10}",
            size in 1i32..1000
        ) {
            let transformer = BoxTransformer::new();
            let candidate = BoxCandidate {
                variable: var_name.clone(),
                malloc_index: 0,
                free_index: None,
            };

            let stmt = HirStatement::VariableDeclaration {
                name: var_name.clone(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(size)],
                }),
            };

            let transformed = transformer.transform_statement(&stmt, &candidate);

            match transformed {
                HirStatement::VariableDeclaration { name, .. } => {
                    prop_assert_eq!(&name, &var_name);
                }
                _ => prop_assert!(false, "Expected VariableDeclaration"),
            }
        }
    }
}
