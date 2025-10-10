//! End-to-end integration tests for the complete C-to-Rust pipeline.
//!
//! These tests verify the entire transformation flow:
//! C source → Parser → HIR → Box Transform → Rust code

use decy_analyzer::patterns::PatternDetector;
use decy_codegen::CodeGenerator;
use decy_hir::HirFunction;
use decy_parser::CParser;

#[test]
fn test_simple_c_to_rust_function_parsing() {
    // Parse simple C function to verify basic pipeline
    let c_source = "int main() { return 0; }";

    let parser = CParser::new().expect("Failed to create parser");
    let ast = parser.parse(c_source).expect("Failed to parse C source");

    assert_eq!(ast.functions().len(), 1);
    let c_func = &ast.functions()[0];

    // Convert to HIR
    let hir_func = HirFunction::from_ast_function(c_func);

    assert_eq!(hir_func.name(), "main");

    // Generate Rust code
    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_function(&hir_func);

    assert!(rust_code.contains("fn main()"));
    assert!(rust_code.contains("i32"));
}

#[test]
fn test_function_with_parameters() {
    let c_source = "int add(int a, int b) { return 0; }";

    let parser = CParser::new().expect("Failed to create parser");
    let ast = parser.parse(c_source).expect("Failed to parse C source");

    let hir_func = HirFunction::from_ast_function(&ast.functions()[0]);

    assert_eq!(hir_func.parameters().len(), 2);
    assert_eq!(hir_func.parameters()[0].name(), "a");
    assert_eq!(hir_func.parameters()[1].name(), "b");

    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_function(&hir_func);

    assert!(rust_code.contains("fn add"));
    assert!(rust_code.contains("a: i32"));
    assert!(rust_code.contains("b: i32"));
}

#[test]
fn test_function_with_pointer_parameter() {
    let c_source = "void process(int* data) { }";

    let parser = CParser::new().expect("Failed to create parser");
    let ast = parser.parse(c_source).expect("Failed to parse C source");

    let hir_func = HirFunction::from_ast_function(&ast.functions()[0]);

    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_function(&hir_func);

    assert!(rust_code.contains("fn process"));
    assert!(rust_code.contains("data: *mut i32"));
}

#[test]
fn test_multiple_functions() {
    let c_source = r#"
        int add(int a, int b) { return 0; }
        int subtract(int a, int b) { return 0; }
        float multiply(float x, float y) { return 0.0; }
    "#;

    let parser = CParser::new().expect("Failed to create parser");
    let ast = parser.parse(c_source).expect("Failed to parse C source");

    assert_eq!(ast.functions().len(), 3);

    let codegen = CodeGenerator::new();
    let mut all_rust_code = String::new();

    for c_func in ast.functions() {
        let hir_func = HirFunction::from_ast_function(c_func);
        let rust_code = codegen.generate_function(&hir_func);
        all_rust_code.push_str(&rust_code);
        all_rust_code.push('\n');
    }

    assert!(all_rust_code.contains("fn add"));
    assert!(all_rust_code.contains("fn subtract"));
    assert!(all_rust_code.contains("fn multiply"));
}

/// This test demonstrates the complete malloc-to-Box transformation pipeline.
#[test]
fn test_malloc_to_box_transformation_end_to_end() {
    let c_source = r#"
        void process() {
            int* number = malloc(sizeof(int));
            char* letter = malloc(sizeof(char));
        }
    "#;

    let parser = CParser::new().expect("Failed to create parser");
    let ast = parser.parse(c_source).expect("Failed to parse C source");

    let hir_func = HirFunction::from_ast_function(&ast.functions()[0]);

    // Detect Box candidates
    let detector = PatternDetector::new();
    let candidates = detector.find_box_candidates(&hir_func);

    assert_eq!(candidates.len(), 2, "Should detect 2 malloc patterns");

    // Generate transformed Rust code
    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_function_with_box_transform(&hir_func, &candidates);

    // Verify transformations
    assert!(rust_code.contains("Box<i32>"));
    assert!(rust_code.contains("Box<u8>"));
    assert!(rust_code.contains("Box::new(0)"));
    assert!(!rust_code.contains("malloc"));
}
