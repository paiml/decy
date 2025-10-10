//! Example demonstrating the malloc-to-Box transformation pipeline.
//!
//! This example shows how DECY transforms C malloc/free patterns into
//! safe, idiomatic Rust Box<T> types.
//!
//! Run with: cargo run --example malloc_to_box

use decy_analyzer::patterns::PatternDetector;
use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirStatement, HirType};

fn main() {
    println!("=== DECY Malloc-to-Box Transformation Example ===\n");

    // Simulate C code:
    // void process_data() {
    //     int* number = malloc(sizeof(int));
    //     char* letter = malloc(sizeof(char));
    // }

    let func = HirFunction::new_with_body(
        "process_data".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "number".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(4)],
                }),
            },
            HirStatement::VariableDeclaration {
                name: "letter".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Char)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(1)],
                }),
            },
        ],
    );

    println!("Original C code (conceptual):");
    println!("------------------------------");
    println!("void process_data() {{");
    println!("    int* number = malloc(sizeof(int));");
    println!("    char* letter = malloc(sizeof(char));");
    println!("}}");
    println!();

    // Step 1: Pattern Detection
    println!("Step 1: Pattern Detection");
    println!("-------------------------");
    let detector = PatternDetector::new();
    let candidates = detector.find_box_candidates(&func);
    println!("Found {} malloc patterns:", candidates.len());
    for (i, candidate) in candidates.iter().enumerate() {
        println!(
            "  {}. Variable '{}' at statement index {}",
            i + 1,
            candidate.variable,
            candidate.malloc_index
        );
    }
    println!();

    // Step 2: Code Generation (without transformation)
    println!("Step 2: Naive Rust Translation (unsafe)");
    println!("----------------------------------------");
    let codegen = CodeGenerator::new();
    let code_without = codegen.generate_function(&func);
    println!("{}", code_without);
    println!();

    // Step 3: Code Generation (with Box transformation)
    println!("Step 3: Safe Rust with Box<T> (idiomatic)");
    println!("------------------------------------------");
    let code_with = codegen.generate_function_with_box_transform(&func, &candidates);
    println!("{}", code_with);
    println!();

    // Analysis
    println!("Transformation Analysis");
    println!("----------------------");
    println!("✓ Replaced malloc() with Box::new()");
    println!("✓ Changed *mut T to Box<T>");
    println!("✓ Eliminated manual memory management");
    println!("✓ Achieved memory safety through RAII");
    println!();

    // Comparison
    println!("Safety Comparison");
    println!("-----------------");
    println!("Before (unsafe):");
    println!("  • Manual malloc/free required");
    println!("  • Risk of memory leaks");
    println!("  • Risk of use-after-free");
    println!("  • Raw pointers can be null");
    println!();
    println!("After (safe):");
    println!("  • Automatic memory management");
    println!("  • No memory leaks (RAII)");
    println!("  • No use-after-free (ownership)");
    println!("  • Box<T> cannot be null");
    println!();

    println!("=== Transformation Complete! ===");
}
