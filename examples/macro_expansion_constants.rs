//! Macro Expansion Examples - Object-Like Macros (Constants)
//!
//! This example demonstrates DECY's transpilation of C object-like macros
//! to Rust const declarations.
//!
//! Reference: K&R §4.11, ISO C99 §6.10.3

use decy_codegen::CodeGenerator;
use decy_hir::HirMacroDefinition;

fn main() {
    println!("=== Macro Expansion: Object-Like Macros ===\n");

    // ANCHOR: integer_constants
    // Integer constants
    let max_def = HirMacroDefinition::new_object_like("MAX".to_string(), "100".to_string());
    let min_def = HirMacroDefinition::new_object_like("MIN".to_string(), "-50".to_string());

    let generator = CodeGenerator::new();

    let max_rust = generator
        .generate_macro(&max_def)
        .expect("Failed to generate MAX");
    let min_rust = generator
        .generate_macro(&min_def)
        .expect("Failed to generate MIN");

    println!("C:    #define MAX 100");
    println!("Rust: {}\n", max_rust);

    println!("C:    #define MIN -50");
    println!("Rust: {}\n", min_rust);

    assert_eq!(max_rust, "const MAX: i32 = 100;");
    assert_eq!(min_rust, "const MIN: i32 = -50;");
    // ANCHOR_END: integer_constants

    // ANCHOR: float_constants
    // Floating point constants
    let pi_def = HirMacroDefinition::new_object_like("PI".to_string(), "3.14159".to_string());
    let e_def = HirMacroDefinition::new_object_like("E".to_string(), "2.71828".to_string());

    let pi_rust = generator
        .generate_macro(&pi_def)
        .expect("Failed to generate PI");
    let e_rust = generator
        .generate_macro(&e_def)
        .expect("Failed to generate E");

    println!("C:    #define PI 3.14159");
    println!("Rust: {}\n", pi_rust);

    println!("C:    #define E 2.71828");
    println!("Rust: {}\n", e_rust);

    assert_eq!(pi_rust, "const PI: f64 = 3.14159;");
    assert_eq!(e_rust, "const E: f64 = 2.71828;");
    // ANCHOR_END: float_constants

    // ANCHOR: string_constants
    // String constants
    let greeting_def =
        HirMacroDefinition::new_object_like("GREETING".to_string(), "\"Hello, World!\"".to_string());
    let version_def =
        HirMacroDefinition::new_object_like("VERSION".to_string(), "\"v1.0.0\"".to_string());

    let greeting_rust = generator
        .generate_macro(&greeting_def)
        .expect("Failed to generate GREETING");
    let version_rust = generator
        .generate_macro(&version_def)
        .expect("Failed to generate VERSION");

    println!("C:    #define GREETING \"Hello, World!\"");
    println!("Rust: {}\n", greeting_rust);

    println!("C:    #define VERSION \"v1.0.0\"");
    println!("Rust: {}\n", version_rust);

    assert_eq!(greeting_rust, "const GREETING: &str = \"Hello, World!\";");
    assert_eq!(version_rust, "const VERSION: &str = \"v1.0.0\";");
    // ANCHOR_END: string_constants

    // ANCHOR: char_constants
    // Character constants
    let newline_def =
        HirMacroDefinition::new_object_like("NEWLINE".to_string(), "'\\n'".to_string());
    let tab_def = HirMacroDefinition::new_object_like("TAB".to_string(), "'\\t'".to_string());

    let newline_rust = generator
        .generate_macro(&newline_def)
        .expect("Failed to generate NEWLINE");
    let tab_rust = generator
        .generate_macro(&tab_def)
        .expect("Failed to generate TAB");

    println!("C:    #define NEWLINE '\\n'");
    println!("Rust: {}\n", newline_rust);

    println!("C:    #define TAB '\\t'");
    println!("Rust: {}\n", tab_rust);

    assert_eq!(newline_rust, "const NEWLINE: char = '\\n';");
    assert_eq!(tab_rust, "const TAB: char = '\\t';");
    // ANCHOR_END: char_constants

    // ANCHOR: hex_constants
    // Hexadecimal constants
    let flags_def = HirMacroDefinition::new_object_like("FLAGS".to_string(), "0xFF".to_string());
    let mask_def = HirMacroDefinition::new_object_like("MASK".to_string(), "0x0F".to_string());

    let flags_rust = generator
        .generate_macro(&flags_def)
        .expect("Failed to generate FLAGS");
    let mask_rust = generator
        .generate_macro(&mask_def)
        .expect("Failed to generate MASK");

    println!("C:    #define FLAGS 0xFF");
    println!("Rust: {}\n", flags_rust);

    println!("C:    #define MASK 0x0F");
    println!("Rust: {}\n", mask_rust);

    assert_eq!(flags_rust, "const FLAGS: i32 = 0xFF;");
    assert_eq!(mask_rust, "const MASK: i32 = 0x0F;");
    // ANCHOR_END: hex_constants

    println!("\n✅ All constant macro transformations verified!");
}
