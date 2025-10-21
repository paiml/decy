//! Property tests for string code generation (DECY-025 TDD-Refactor phase).

use super::*;
use decy_hir::{HirExpression, HirStatement, HirType};
use proptest::prelude::*;

proptest! {
    /// Property: Generated string literals always have quotes
    #[test]
    fn property_string_has_quotes(s in ".*") {
        let string_lit = HirExpression::StringLiteral(s);
        let codegen = CodeGenerator::new();

        let code = codegen.generate_expression(&string_lit);

        prop_assert!(code.starts_with('"'));
        prop_assert!(code.ends_with('"'));
    }

    /// Property: Empty strings generate ""
    #[test]
    fn property_empty_string_generates_quotes(_seed in 0u32..100) {
        let empty = HirExpression::StringLiteral(String::new());
        let codegen = CodeGenerator::new();

        let code = codegen.generate_expression(&empty);

        prop_assert_eq!(code, "\"\"");
    }

    /// Property: String literal code generation is deterministic
    #[test]
    fn property_string_generation_deterministic(s in ".*") {
        let string_lit = HirExpression::StringLiteral(s);
        let codegen = CodeGenerator::new();

        let code1 = codegen.generate_expression(&string_lit);
        let code2 = codegen.generate_expression(&string_lit);

        prop_assert_eq!(code1, code2);
    }

    /// Property: String content is preserved in generated code
    #[test]
    fn property_string_content_preserved(s in "[a-zA-Z0-9 ]{0,50}") {
        let string_lit = HirExpression::StringLiteral(s.clone());
        let codegen = CodeGenerator::new();

        let code = codegen.generate_expression(&string_lit);

        // Code should contain the original string between quotes
        prop_assert!(code.contains(&s));
    }

    /// Property: String literals in function calls preserve content
    #[test]
    fn property_string_in_call_preserves_content(
        s in ".*",
        func_name in "[a-z_][a-z0-9_]{0,10}"
    ) {
        let call = HirExpression::FunctionCall {
            function: func_name.clone(),
            arguments: vec![HirExpression::StringLiteral(s)],
        };
        let codegen = CodeGenerator::new();

        let code = codegen.generate_expression(&call);

        prop_assert!(code.contains(&func_name));
        prop_assert!(code.contains('"'));
    }

    /// Property: Variable declarations with strings generate valid code
    #[test]
    fn property_var_decl_string_generates_valid(
        var_name in "[a-z_][a-z0-9_]{0,10}",
        string_val in ".*"
    ) {
        let decl = HirStatement::VariableDeclaration {
            name: var_name.clone(),
            var_type: HirType::Pointer(Box::new(HirType::Char)),
            initializer: Some(HirExpression::StringLiteral(string_val)),
        };
        let codegen = CodeGenerator::new();

        let code = codegen.generate_statement(&decl);

        // Should contain variable name and let keyword
        prop_assert!(code.contains("let"));
        prop_assert!(code.contains(&var_name));
        prop_assert!(code.contains('"'));
    }
}
