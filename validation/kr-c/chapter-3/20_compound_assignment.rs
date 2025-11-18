/* K&R C Chapter 3: Compound Assignment Operators
 * K&R §3.11: Assignment shorthand operators
 * Transpiled to safe Rust
 */

fn demo_arithmetic_compound() {
    println!("=== Arithmetic Compound Assignment ===");

    let mut x = 10;
    println!("x = {}", x);

    x += 5;
    println!("x += 5:  x = {}", x);

    x -= 3;
    println!("x -= 3:  x = {}", x);

    x *= 2;
    println!("x *= 2:  x = {}", x);

    x /= 4;
    println!("x /= 4:  x = {}", x);

    x %= 5;
    println!("x %= 5:  x = {}", x);

    println!();
}

fn demo_bitwise_compound() {
    println!("=== Bitwise Compound Assignment ===");

    let mut flags: u32 = 0b11110000;
    println!("flags = 0x{:02X}", flags);

    flags |= 0b00001111;
    println!("flags |= 0x0F: flags = 0x{:02X}", flags);

    flags &= 0b10101010;
    println!("flags &= 0xAA: flags = 0x{:02X}", flags);

    flags ^= 0b11111111;
    println!("flags ^= 0xFF: flags = 0x{:02X}", flags);

    println!();
}

fn demo_shift_compound() {
    println!("=== Shift Compound Assignment ===");

    let mut x: u32 = 4;
    println!("x = {}", x);

    x <<= 2;
    println!("x <<= 2: x = {} (multiply by 4)", x);

    x >>= 1;
    println!("x >>= 1: x = {} (divide by 2)", x);

    println!();
}

fn demo_array_operations() {
    println!("=== Array Operations ===");

    let mut arr = vec![1, 2, 3, 4, 5];

    print!("Original: ");
    for &val in &arr {
        print!("{} ", val);
    }
    println!();

    // Double all elements
    for val in arr.iter_mut() {
        *val *= 2;
    }

    print!("After *=2: ");
    for &val in &arr {
        print!("{} ", val);
    }
    println!("\n");
}

fn demo_complex_expression() {
    println!("=== Complex Expressions ===");

    let mut x = 5;
    println!("x = {}", x);

    // Equivalent to: x = x * 2 + 3
    x = x * 2;
    x += 3;
    println!("x = x * 2 + 3: x = {}", x);

    // x *= 2 + 3 means x = x * (2 + 3)
    let mut x = 5;
    x *= 2 + 3;
    println!("x *= 2 + 3: x = {} (x = x * (2 + 3))", x);

    println!();
}

fn demo_slice_indexing() {
    println!("=== Slice Indexing ===");

    let mut arr = vec![10, 20, 30, 40, 50];
    let mut idx = 0;

    println!("arr[{}] = {}", idx, arr[idx]);

    idx += 2;
    println!("After idx += 2: arr[{}] = {}", idx, arr[idx]);

    idx -= 1;
    println!("After idx -= 1: arr[{}] = {}", idx, arr[idx]);

    println!();
}

fn main() {
    println!("=== Compound Assignment Operators ===\n");

    demo_arithmetic_compound();
    demo_bitwise_compound();
    demo_shift_compound();
    demo_array_operations();
    demo_complex_expression();
    demo_slice_indexing();

    println!("Compound assignment operators:");
    println!("  +=  -=  *=  /=  %=");
    println!("  &=  |=  ^=  <<=  >>=");
    println!("\nBehavior (same as C):");
    println!("  x op= y is equivalent to x = x op y");
    println!("  But x is evaluated only once");
    println!("  More concise and potentially more efficient");
}
