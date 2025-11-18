/* K&R C Chapter 3: Short-Circuit Evaluation
 * K&R §3.8: Logical operators and side effects
 * Transpiled to safe Rust
 */

fn side_effect_true() -> bool {
    println!("  side_effect_true() called");
    true
}

fn side_effect_false() -> bool {
    println!("  side_effect_false() called");
    false
}

fn demo_and_short_circuit() {
    println!("=== AND Short-Circuit ===");

    println!("false && side_effect_true():");
    let result = false && side_effect_true();
    println!("Result: {} (function not called)\n", result);

    println!("true && side_effect_true():");
    let result = true && side_effect_true();
    println!("Result: {} (function called)\n", result);
}

fn demo_or_short_circuit() {
    println!("=== OR Short-Circuit ===");

    println!("true || side_effect_false():");
    let result = true || side_effect_false();
    println!("Result: {} (function not called)\n", result);

    println!("false || side_effect_false():");
    let result = false || side_effect_false();
    println!("Result: {} (function called)\n", result);
}

fn demo_null_check() {
    println!("=== Option Check (Rust equivalent) ===");

    let arr = vec![1, 2, 3];
    let p = Some(&arr[0]);

    if let Some(&val) = p {
        if val > 0 {
            println!("Safe: *p = {}", val);
        }
    }

    let p: Option<&i32> = None;
    if let Some(&val) = p {
        if val > 0 {
            println!("This won't print");
        }
    } else {
        println!("None, value not accessed");
    }

    println!();
}

fn demo_range_check() {
    println!("=== Range Check ===");

    let arr = vec![10, 20, 30];
    let size = arr.len();
    let index = 5;

    // Safe bounds check
    if index < size && arr[index] > 15 {
        println!("Element: {}", arr[index]);
    } else {
        println!("Index {} out of bounds or condition false", index);
    }

    // Rust also provides .get() for safe access
    if let Some(&val) = arr.get(index) {
        if val > 15 {
            println!("Element: {}", val);
        }
    }

    println!();
}

fn main() {
    println!("=== Short-Circuit Evaluation ===\n");

    demo_and_short_circuit();
    demo_or_short_circuit();
    demo_null_check();
    demo_range_check();

    println!("Short-circuit rules (same as C):");
    println!("  - AND (&&): if left is false, right not evaluated");
    println!("  - OR (||): if left is true, right not evaluated");
    println!("  - Use for safe checks");
    println!("  - Side effects may not occur");
}
