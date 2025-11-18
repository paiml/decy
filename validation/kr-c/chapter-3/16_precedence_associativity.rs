/* K&R C Chapter 3: Operator Precedence and Associativity
 * K&R §3.7: Expression evaluation order
 * Transpiled to safe Rust
 */

fn demo_arithmetic_precedence() {
    println!("=== Arithmetic Precedence ===");

    let result = 2 + 3 * 4;
    println!("2 + 3 * 4 = {} (not 20)", result);

    let result = (2 + 3) * 4;
    println!("(2 + 3) * 4 = {}", result);

    let result = 10 - 5 - 2;
    println!("10 - 5 - 2 = {} (left associative)", result);

    println!();
}

fn demo_logical_precedence() {
    println!("=== Logical Precedence ===");

    let a = true;
    let b = false;
    let c = true;

    let result = a || b && c;
    println!("true || false && true = {} (AND before OR)", result);

    let result = (a || b) && c;
    println!("(true || false) && true = {}", result);

    println!();
}

fn demo_pointer_precedence() {
    println!("=== Slice/Reference Precedence ===");

    let mut arr = [1, 2, 3];
    let mut idx = 0;

    // Rust doesn't have ++ operator
    // Demonstrate similar patterns
    println!("arr[{}] = {}", idx, arr[idx]);
    idx += 1;
    println!("After increment: arr[{}] = {}", idx, arr[idx]);

    idx = 0;
    let old_idx = idx;
    idx += 1;
    arr[old_idx] += 1;
    println!("Increment value: arr[{}] = {}", old_idx, arr[old_idx]);

    println!();
}

fn demo_bitwise_precedence() {
    println!("=== Bitwise Precedence ===");

    let x: u32 = 5;  // 0101
    let y: u32 = 3;  // 0011

    // In Rust, must be explicit about precedence
    let result = (x & y) == 1;
    println!("(x & y) == 1 = {}", result);

    println!();
}

fn demo_associativity() {
    println!("=== Associativity ===");

    // Right-to-left: assignment
    let x: i32;
    let y: i32;
    let z = 10;
    y = z;
    x = y;
    println!("x = y = z = 10: x={}, y={}, z={}", x, y, z);

    // Left-to-right: arithmetic
    let a = 16;
    let result = a / 4 / 2;
    println!("16 / 4 / 2 = {} (left-to-right)", result);

    println!();
}

fn main() {
    println!("=== Operator Precedence and Associativity ===\n");

    demo_arithmetic_precedence();
    demo_logical_precedence();
    demo_pointer_precedence();
    demo_bitwise_precedence();
    demo_associativity();

    println!("Rust precedence (similar to C):");
    println!("  1. () [] . (field access)");
    println!("  2. ! - * & (unary)");
    println!("  3. * / %");
    println!("  4. + -");
    println!("  5. << >>");
    println!("  6. < <= > >=");
    println!("  7. == !=");
    println!("  8. &");
    println!("  9. ^");
    println!(" 10. |");
    println!(" 11. &&");
    println!(" 12. ||");
    println!(" 13. = += -= etc");
}
