/* K&R C Chapter 3: Comma Operator
 * K&R §3.6: Expression separator
 * Transpiled to safe Rust (Rust has no comma operator)
 */

fn demo_basic_comma() {
    println!("=== Basic Comma Operator (Rust equivalent) ===");

    let a: i32;
    let b: i32;
    let c: i32;

    // Rust doesn't have comma operator
    // Use block expression or separate statements
    b = 2;
    c = 3;
    a = b + c;
    println!("a = (b=2, c=3, b+c): a={}, b={}, c={}", a, b, c);
    println!();
}

fn demo_for_loop() {
    println!("=== Multiple Variables in for Loop ===");

    // Rust allows tuple initialization
    let mut i = 0;
    let mut j = 10;
    while i < j {
        println!("i={}, j={}", i, j);
        i += 1;
        j -= 1;
    }
    println!();
}

fn demo_swap() {
    println!("=== Swap (Rust way) ===");

    let mut x = 5;
    let mut y = 10;
    println!("Before: x={}, y={}", x, y);

    // Rust has tuple assignment (no comma operator needed)
    (x, y) = (y, x);
    println!("After:  x={}, y={}", x, y);
    println!();
}

fn main() {
    println!("=== Comma Operator (Rust alternatives) ===\n");

    demo_basic_comma();
    demo_for_loop();
    demo_swap();

    println!("Rust alternatives to C comma operator:");
    println!("  - Block expressions: {{ stmt1; stmt2; expr }}");
    println!("  - Tuple assignment: (a, b) = (1, 2)");
    println!("  - Separate statements");
    println!("  - No single comma operator");
}

// Demonstrate block expressions as comma operator alternative
fn demonstrate_blocks() {
    let result = {
        let b = 2;
        let c = 3;
        b + c  // Last expression is return value
    };
    println!("Block result: {}", result);
}
