//! Unit tests for code generator (DECY-003 RED phase).
//!
//! These tests are intentionally failing to follow EXTREME TDD methodology.

use super::*;

#[cfg(test)]
mod tests {
    use super::*;
    use decy_hir::{HirFunction, HirType, HirParameter};

    #[test]
    fn test_generate_function_signature() {
        // RED PHASE: This test will FAIL
        let func = HirFunction::new(
            "main".to_string(),
            HirType::Int,
            vec![],
        );

        let codegen = CodeGenerator::new();
        let signature = codegen.generate_signature(&func);

        assert_eq!(signature, "fn main() -> i32");
    }

    #[test]
    fn test_generate_function_with_parameters() {
        // RED PHASE: This test will FAIL
        let func = HirFunction::new(
            "add".to_string(),
            HirType::Int,
            vec![
                HirParameter::new("a".to_string(), HirType::Int),
                HirParameter::new("b".to_string(), HirType::Int),
            ],
        );

        let codegen = CodeGenerator::new();
        let signature = codegen.generate_signature(&func);

        assert_eq!(signature, "fn add(a: i32, b: i32) -> i32");
    }

    #[test]
    fn test_type_mapping_int_to_i32() {
        // RED PHASE: This test will FAIL
        let codegen = CodeGenerator::new();

        assert_eq!(codegen.map_type(&HirType::Int), "i32");
    }

    #[test]
    fn test_type_mapping_float_to_f32() {
        // RED PHASE: This test will FAIL
        let codegen = CodeGenerator::new();

        assert_eq!(codegen.map_type(&HirType::Float), "f32");
    }

    #[test]
    fn test_type_mapping_double_to_f64() {
        // RED PHASE: This test will FAIL
        let codegen = CodeGenerator::new();

        assert_eq!(codegen.map_type(&HirType::Double), "f64");
    }

    #[test]
    fn test_type_mapping_void_to_unit() {
        // RED PHASE: This test will FAIL
        let codegen = CodeGenerator::new();

        assert_eq!(codegen.map_type(&HirType::Void), "()");
    }

    #[test]
    fn test_type_mapping_char_to_u8() {
        // RED PHASE: This test will FAIL
        let codegen = CodeGenerator::new();

        assert_eq!(codegen.map_type(&HirType::Char), "u8");
    }

    #[test]
    fn test_type_mapping_pointer() {
        // RED PHASE: This test will FAIL
        let codegen = CodeGenerator::new();
        let ptr_type = HirType::Pointer(Box::new(HirType::Int));

        assert_eq!(codegen.map_type(&ptr_type), "*mut i32");
    }

    #[test]
    fn test_generate_void_function() {
        // RED PHASE: This test will FAIL
        let func = HirFunction::new(
            "print_hello".to_string(),
            HirType::Void,
            vec![],
        );

        let codegen = CodeGenerator::new();
        let signature = codegen.generate_signature(&func);

        assert_eq!(signature, "fn print_hello()");
    }

    #[test]
    fn test_generate_complete_function() {
        // RED PHASE: This test will FAIL
        let func = HirFunction::new(
            "add".to_string(),
            HirType::Int,
            vec![
                HirParameter::new("a".to_string(), HirType::Int),
                HirParameter::new("b".to_string(), HirType::Int),
            ],
        );

        let codegen = CodeGenerator::new();
        let code = codegen.generate_function(&func);

        // Should generate a complete function with stub body
        assert!(code.contains("fn add(a: i32, b: i32) -> i32"));
        assert!(code.contains("{")); // Has body
        assert!(code.contains("}")); // Closes body
    }

    #[test]
    fn test_generate_return_statement() {
        // RED PHASE: This test will FAIL
        let codegen = CodeGenerator::new();
        let return_stmt = codegen.generate_return(&HirType::Int);

        // Should generate a default return for the type
        assert!(return_stmt.contains("return"));
        assert!(return_stmt.contains("0")); // Default i32 value
    }

    #[test]
    fn test_generate_function_with_pointer_parameter() {
        // RED PHASE: This test will FAIL
        let func = HirFunction::new(
            "process".to_string(),
            HirType::Void,
            vec![
                HirParameter::new("data".to_string(),
                    HirType::Pointer(Box::new(HirType::Int))),
            ],
        );

        let codegen = CodeGenerator::new();
        let signature = codegen.generate_signature(&func);

        assert_eq!(signature, "fn process(data: *mut i32)");
    }

    #[test]
    fn test_end_to_end_simple_function() {
        // RED PHASE: This test will FAIL
        let func = HirFunction::new(
            "add".to_string(),
            HirType::Int,
            vec![
                HirParameter::new("a".to_string(), HirType::Int),
                HirParameter::new("b".to_string(), HirType::Int),
            ],
        );

        let codegen = CodeGenerator::new();
        let code = codegen.generate_function(&func);

        // Verify generated code has proper structure
        assert!(code.starts_with("fn add"));
        assert!(code.contains("a: i32"));
        assert!(code.contains("b: i32"));
        assert!(code.contains("-> i32"));

        // Should have a function body (even if stub)
        let open_braces = code.matches('{').count();
        let close_braces = code.matches('}').count();
        assert_eq!(open_braces, close_braces);
        assert!(open_braces > 0);
    }

    #[test]
    fn test_generated_code_compiles() {
        // RED PHASE: This test will FAIL
        // This is an integration test to ensure generated code is valid Rust
        let func = HirFunction::new(
            "test_fn".to_string(),
            HirType::Int,
            vec![],
        );

        let codegen = CodeGenerator::new();
        let code = codegen.generate_function(&func);

        // The generated code should be syntactically valid
        // We'll verify this by checking it has all required parts
        assert!(code.contains("fn test_fn"));
        assert!(code.contains("-> i32"));
        assert!(code.contains("{"));
        assert!(code.contains("}"));
    }
}
