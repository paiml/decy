/* K&R C Chapter 4.5: Header Files
 * Page 82-83
 * Using standard headers and function declarations
 * Transpiled to safe Rust (using standard library modules)
 */

// In Rust, we use 'use' statements instead of #include
// Rust modules are namespaced and explicit

fn main() {
    let test_str = "Hello, World!";
    let test_num = 2.5;
    let test_char = 'A';

    println!("=== String Functions (string methods) ===");
    print_string_info(test_str);

    println!("\n=== Math Functions (f64 methods) ===");
    print_math_functions(test_num);

    println!("\n=== Character Functions (char methods) ===");
    print_char_info(test_char);
    print_char_info('a');
    print_char_info('5');
    print_char_info(' ');
}

fn print_string_info(s: &str) {
    println!("String: \"{}\"", s);
    println!("Length: {}", s.len());
    println!("First char: '{}'", s.chars().next().unwrap_or(' '));

    // String copy in Rust
    let copy = s.to_string();
    println!("Copy: \"{}\"", copy);

    if s == copy {
        println!("Strings are equal");
    }
}

fn print_math_functions(x: f64) {
    println!("x = {:.2}", x);
    println!("sqrt(x) = {:.4}", x.sqrt());
    println!("pow(x, 3) = {:.4}", x.powi(3));
    println!("sin(x) = {:.4}", x.sin());
    println!("cos(x) = {:.4}", x.cos());
    println!("exp(x) = {:.4}", x.exp());
    println!("ln(x) = {:.4}", x.ln());
    println!("ceil(x) = {:.0}", x.ceil());
    println!("floor(x) = {:.0}", x.floor());
}

fn print_char_info(c: char) {
    println!("\nChar: '{}' (Unicode {:X})", c, c as u32);
    println!("  is_alphabetic: {}", c.is_alphabetic());
    println!("  is_numeric: {}", c.is_numeric());
    println!("  is_whitespace: {}", c.is_whitespace());
    println!("  is_uppercase: {}", c.is_uppercase());
    println!("  is_lowercase: {}", c.is_lowercase());
    println!("  to_uppercase: '{}'", c.to_uppercase());
    println!("  to_lowercase: '{}'", c.to_lowercase());
}

// Key differences from C headers:
// 1. No header files - modules instead
// 2. Methods on types (x.sqrt()) instead of functions (sqrt(x))
// 3. No null-terminated strings - UTF-8 String/&str
// 4. No manual memory management
// 5. Traits instead of function pointers for polymorphism
