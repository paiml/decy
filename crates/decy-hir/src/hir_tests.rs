//! Unit tests for HIR (DECY-002 RED phase).
//!
//! These tests are intentionally failing to follow EXTREME TDD methodology.

use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hir_function_creation() {
        // RED PHASE: This test will FAIL until we define HirFunction
        let func = HirFunction::new(
            "main".to_string(),
            HirType::Int,
            vec![],
        );

        assert_eq!(func.name(), "main");
        assert_eq!(func.return_type(), &HirType::Int);
        assert_eq!(func.parameters().len(), 0);
    }

    #[test]
    fn test_hir_function_with_parameters() {
        // RED PHASE: This test will FAIL
        let params = vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ];

        let func = HirFunction::new(
            "add".to_string(),
            HirType::Int,
            params,
        );

        assert_eq!(func.name(), "add");
        assert_eq!(func.parameters().len(), 2);
        assert_eq!(func.parameters()[0].name(), "a");
        assert_eq!(func.parameters()[1].name(), "b");
    }

    #[test]
    fn test_hir_type_variants() {
        // RED PHASE: This test will FAIL
        let types = vec![
            HirType::Void,
            HirType::Int,
            HirType::Float,
            HirType::Double,
            HirType::Char,
            HirType::Pointer(Box::new(HirType::Int)),
        ];

        assert_eq!(types.len(), 6);

        // Test pointer unwrapping
        if let HirType::Pointer(inner) = &types[5] {
            assert_eq!(**inner, HirType::Int);
        } else {
            panic!("Expected pointer type");
        }
    }

    #[test]
    fn test_hir_parameter_creation() {
        // RED PHASE: This test will FAIL
        let param = HirParameter::new("x".to_string(), HirType::Float);

        assert_eq!(param.name(), "x");
        assert_eq!(param.param_type(), &HirType::Float);
    }

    #[test]
    fn test_hir_serialization() {
        // RED PHASE: This test will FAIL
        let func = HirFunction::new(
            "test".to_string(),
            HirType::Void,
            vec![],
        );

        // Test Debug trait
        let debug_str = format!("{:?}", func);
        assert!(debug_str.contains("test"));

        // Test Clone trait
        let func_clone = func.clone();
        assert_eq!(func, func_clone);
    }

    #[test]
    fn test_ast_to_hir_conversion() {
        // RED PHASE: This test will FAIL
        use decy_parser::parser::{Ast, Function, Type, Parameter};

        // Create a simple AST function
        let ast_func = Function::new(
            "add".to_string(),
            Type::Int,
            vec![
                Parameter::new("a".to_string(), Type::Int),
                Parameter::new("b".to_string(), Type::Int),
            ],
        );

        // Convert to HIR
        let hir_func = HirFunction::from_ast_function(&ast_func);

        assert_eq!(hir_func.name(), "add");
        assert_eq!(hir_func.return_type(), &HirType::Int);
        assert_eq!(hir_func.parameters().len(), 2);
    }

    #[test]
    fn test_hir_type_from_ast_type() {
        // RED PHASE: This test will FAIL
        use decy_parser::parser::Type;

        assert_eq!(HirType::from_ast_type(&Type::Void), HirType::Void);
        assert_eq!(HirType::from_ast_type(&Type::Int), HirType::Int);
        assert_eq!(HirType::from_ast_type(&Type::Float), HirType::Float);
        assert_eq!(HirType::from_ast_type(&Type::Double), HirType::Double);
        assert_eq!(HirType::from_ast_type(&Type::Char), HirType::Char);

        // Test pointer conversion
        let ptr_type = Type::Pointer(Box::new(Type::Int));
        let hir_ptr = HirType::from_ast_type(&ptr_type);

        if let HirType::Pointer(inner) = hir_ptr {
            assert_eq!(*inner, Box::new(HirType::Int));
        } else {
            panic!("Expected pointer type");
        }
    }
}
