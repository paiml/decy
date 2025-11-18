/* K&R C Chapter 3: Sequence Points
 * K&R §3.9: Undefined behavior and evaluation order
 * Transpiled to safe Rust
 */

fn demo_safe_mutations() {
    println!("=== Safe Mutations (No Sequence Points Needed) ===");

    let mut i = 5;

    // Rust prevents multiple mutable accesses
    // This would be a compile error:
    // let bad = i + i++;  // ERROR: cannot use while mutating

    // Correct way - separate statements
    let a = i;
    i += 1;
    let b = i;
    i += 1;
    let good = a + b;
    println!("Correct way: a={}, b={}, sum={}", a, b, good);

    println!();
}

fn demo_expression_semantics() {
    println!("=== Expression Semantics ===");

    let mut x = 1;

    // Sequence guaranteed by statement boundary
    x = x + 1;  // OK: guaranteed order
    println!("After x = x + 1: x = {}", x);

    // && and || guarantee left-to-right
    let mut y = 0;
    let result = { y = 5; true } && { x = y + 1; true };
    println!("(y=5) && (x=y+1): x={}, y={} (y assigned first)", x, y);

    // if expression guarantees order
    let mut z = 10;
    z = if z > 5 { 20 } else { 30 };
    println!("if expression: z={}", z);

    println!();
}

fn demo_function_arguments() {
    println!("=== Function Call Evaluation ===");

    let mut i = 0;

    // Rust evaluates arguments in order
    let a = { let r = i; i += 1; r };
    let b = { let r = i; i += 1; r };
    let c = { let r = i; i += 1; r };
    println!("Arguments evaluated left to right: {} {} {}", a, b, c);

    println!();
}

fn demo_safe_patterns() {
    println!("=== Safe Patterns ===");

    let mut x = 5;

    // Safe: Separate mutations
    x += 1;
    x += 1;
    println!("x after two increments: {}", x);

    // Safe: Single mutation
    let mut y = x;
    y += 1;
    println!("y = x; y += 1: y={}, x={}", y, x);

    // Safe: Block expression
    let z = {
        x = 10;
        x + 1
    };
    println!("z = {{ x=10; x+1 }}: z={}, x={}", z, x);

    println!();
}

fn main() {
    println!("=== Sequence Points (Rust Safety) ===\n");

    demo_safe_mutations();
    demo_expression_semantics();
    demo_function_arguments();
    demo_safe_patterns();

    println!("Rust prevents undefined behavior:");
    println!("  - Cannot mutate while reading (borrow checker)");
    println!("  - Function arguments evaluated left-to-right");
    println!("  - Statement boundaries are clear");
    println!("  - && and || evaluate left-to-right");
    println!("  - Blocks {{ }} have defined order");
    println!("\nRust eliminates:");
    println!("  - Undefined behavior from multiple mutations");
    println!("  - Unspecified argument evaluation order");
    println!("  - Sequence point confusion");
}
