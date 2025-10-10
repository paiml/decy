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
        let func = HirFunction::new("main".to_string(), HirType::Int, vec![]);

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

        let func = HirFunction::new("add".to_string(), HirType::Int, params);

        assert_eq!(func.name(), "add");
        assert_eq!(func.parameters().len(), 2);
        assert_eq!(func.parameters()[0].name(), "a");
        assert_eq!(func.parameters()[1].name(), "b");
    }

    #[test]
    fn test_hir_type_variants() {
        // RED PHASE: This test will FAIL
        let types = [
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
        let func = HirFunction::new("test".to_string(), HirType::Void, vec![]);

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
        use decy_parser::parser::{Function, Parameter, Type};

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
            assert_eq!(*inner, HirType::Int);
        } else {
            panic!("Expected pointer type");
        }
    }

    #[test]
    fn test_hir_type_equality() {
        // Test type equality
        assert_eq!(HirType::Void, HirType::Void);
        assert_eq!(HirType::Int, HirType::Int);
        assert_ne!(HirType::Int, HirType::Float);

        // Test pointer equality
        let ptr1 = HirType::Pointer(Box::new(HirType::Int));
        let ptr2 = HirType::Pointer(Box::new(HirType::Int));
        assert_eq!(ptr1, ptr2);
    }

    #[test]
    fn test_hir_parameter_from_ast() {
        use decy_parser::parser::{Parameter, Type};

        let ast_param = Parameter::new("x".to_string(), Type::Float);
        let hir_param = HirParameter::from_ast_parameter(&ast_param);

        assert_eq!(hir_param.name(), "x");
        assert_eq!(hir_param.param_type(), &HirType::Float);
    }

    #[test]
    fn test_hir_function_with_complex_params() {
        let params = vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Float),
            HirParameter::new("c".to_string(), HirType::Pointer(Box::new(HirType::Char))),
        ];

        let func = HirFunction::new("complex".to_string(), HirType::Void, params);

        assert_eq!(func.name(), "complex");
        assert_eq!(func.return_type(), &HirType::Void);
        assert_eq!(func.parameters().len(), 3);
        assert_eq!(func.parameters()[0].param_type(), &HirType::Int);
        assert_eq!(func.parameters()[1].param_type(), &HirType::Float);

        if let HirType::Pointer(inner) = func.parameters()[2].param_type() {
            assert_eq!(**inner, HirType::Char);
        } else {
            panic!("Expected pointer type");
        }
    }

    #[test]
    fn test_nested_pointer_conversion() {
        use decy_parser::parser::Type;

        // Test double pointer: int**
        let ast_double_ptr = Type::Pointer(Box::new(Type::Pointer(Box::new(Type::Int))));
        let hir_double_ptr = HirType::from_ast_type(&ast_double_ptr);

        if let HirType::Pointer(outer) = hir_double_ptr {
            if let HirType::Pointer(inner) = *outer {
                assert_eq!(*inner, HirType::Int);
            } else {
                panic!("Expected nested pointer");
            }
        } else {
            panic!("Expected pointer type");
        }
    }

    #[test]
    fn test_ast_to_hir_with_multiple_param_types() {
        use decy_parser::parser::{Function, Parameter, Type};

        let ast_func = Function::new(
            "multi".to_string(),
            Type::Double,
            vec![
                Parameter::new("i".to_string(), Type::Int),
                Parameter::new("f".to_string(), Type::Float),
                Parameter::new("d".to_string(), Type::Double),
                Parameter::new("c".to_string(), Type::Char),
            ],
        );

        let hir_func = HirFunction::from_ast_function(&ast_func);

        assert_eq!(hir_func.name(), "multi");
        assert_eq!(hir_func.return_type(), &HirType::Double);
        assert_eq!(hir_func.parameters().len(), 4);
        assert_eq!(hir_func.parameters()[0].param_type(), &HirType::Int);
        assert_eq!(hir_func.parameters()[1].param_type(), &HirType::Float);
        assert_eq!(hir_func.parameters()[2].param_type(), &HirType::Double);
        assert_eq!(hir_func.parameters()[3].param_type(), &HirType::Char);
    }
}
