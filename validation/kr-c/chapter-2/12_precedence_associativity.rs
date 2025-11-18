/* K&R C Chapter 2.12: Precedence and Order of Evaluation
 * Page 52-53
 * Operator precedence and associativity
 * Transpiled to safe Rust
 */

fn main() {
    let a = 10;
    let b = 20;
    let c = 30;

    // Arithmetic precedence
    println!("Arithmetic precedence:");
    println!("a + b * c = {} (b*c first)", a + b * c);
    println!("(a + b) * c = {} (explicit grouping)", (a + b) * c);
    println!("a * b + c = {} (a*b first)", a * b + c);

    // Mixed operators
    println!("\nMixed operators:");
    let result1 = 5 + 3 * 2;  // 5 + 6 = 11
    println!("5 + 3 * 2 = {}", result1);

    let result2 = 20 / 4 * 2;  // Left-to-right: 5 * 2 = 10
    println!("20 / 4 * 2 = {}", result2);

    let result3 = 20 / (4 * 2);  // 20 / 8 = 2
    println!("20 / (4 * 2) = {}", result3);

    // Comparison and logical
    println!("\nComparison and logical:");
    let x = 5;
    let y = 10;
    let z = 15;

    let cond1 = x < y && y < z;
    println!("x < y && y < z = {}", cond1);

    let cond2 = x < y || z < y;
    println!("x < y || z < y = {}", cond2);

    // Rust doesn't allow chained comparisons like C
    // This would be a type error: x < y < z
    let cond3 = (x < y) && (y < z);  // Correct way in Rust
    println!("x < y && y < z = {} (explicit)", cond3);

    // Bitwise vs logical
    println!("\nBitwise vs logical:");
    let bit_and = 5 & 3;      // 0101 & 0011 = 0001
    let logical_and = (5 != 0) && (3 != 0);  // true && true = true
    println!("5 & 3 = {} (bitwise)", bit_and);
    println!("(5 != 0) && (3 != 0) = {} (logical)", logical_and);

    // Assignment associativity (right-to-left)
    println!("\nAssignment associativity:");
    let p: i32;
    let q: i32;
    let r = 42;
    q = r;
    p = q;
    println!("p = q = r = 42 -> p={}, q={}, r={}", p, q, r);

    // Increment and arithmetic
    println!("\nIncrement and arithmetic:");
    let mut n = 5;
    n += 1;
    let res1 = n * 2;  // 6 * 2 = 12
    println!("++n * 2 = {} (n={})", res1, n);

    let mut n = 5;
    let res2 = n * 2;  // 5 * 2 = 10
    n += 1;  // then n=6
    println!("n++ * 2 = {} (n={})", res2, n);
}

// Key differences from C:
// 1. Rust requires explicit bool in logical contexts (no implicit int->bool)
// 2. Cannot chain comparisons like C: x < y < z (use x < y && y < z)
// 3. Operator precedence is similar to C but more explicit
// 4. No ++ or -- operators (use += 1 or -= 1)
// 5. Better type safety prevents many precedence-related bugs
