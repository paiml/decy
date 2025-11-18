/* K&R C Chapter 4: Function Currying Simulation
 * Partial application of functions
 * Transpiled to safe Rust (using closures)
 */

// Regular function
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Curried version using closures (Rust idiomatic way)
fn add_curried(x: i32) -> impl Fn(i32) -> i32 {
    move |y| x + y
}

// Multiply with partial application
fn multiply_by(multiplier: i32) -> impl Fn(i32) -> i32 {
    move |value| multiplier * value
}

// Power function
fn power_base(base: i32) -> impl Fn(i32) -> i32 {
    move |exponent| {
        let mut result = 1;
        for _ in 0..exponent {
            result *= base;
        }
        result
    }
}

// Comparison with threshold
fn greater_than(threshold: i32) -> impl Fn(i32) -> bool {
    move |value| value > threshold
}

fn main() {
    println!("=== Function Currying ===\n");

    // Regular function
    println!("Regular add(5, 3) = {}", add(5, 3));

    // Partial application using closures
    println!("\nPartial application:");
    let add5 = add_curried(5);
    println!("  add5 = add_curried(5)");
    println!("  add5(3) = {}", add5(3));
    println!("  add5(10) = {}", add5(10));

    let add10 = add_curried(10);
    println!("  add10 = add_curried(10)");
    println!("  add10(3) = {}", add10(3));

    // Multiply partial
    println!("\nMultiplication:");
    let double_it = multiply_by(2);
    let triple_it = multiply_by(3);

    println!("  double(5) = {}", double_it(5));
    println!("  double(10) = {}", double_it(10));
    println!("  triple(5) = {}", triple_it(5));

    // Power partial
    println!("\nPower functions:");
    let powers_of_2 = power_base(2);
    let powers_of_3 = power_base(3);

    print!("  Powers of 2: ");
    for i in 0..=5 {
        print!("{} ", powers_of_2(i));
    }
    println!();

    print!("  Powers of 3: ");
    for i in 0..=5 {
        print!("{} ", powers_of_3(i));
    }
    println!();

    // Comparison partial
    println!("\nComparison predicates:");
    let gt10 = greater_than(10);
    let gt50 = greater_than(50);

    let values = [5, 15, 45, 55, 100];
    for &val in &values {
        println!("  {}: gt10={}, gt50={}",
                 val, gt10(val), gt50(val));
    }
}

// Demonstrate more advanced currying
fn demonstrate_advanced_currying() {
    // Triple-argument function curried
    let add3 = |x| move |y| move |z| x + y + z;

    let add5_and = add3(5);
    let add5_and_3_and = add5_and(3);
    let result = add5_and_3_and(2);

    println!("Curried 3-arg: add3(5)(3)(2) = {}", result);

    // Using filter with curried predicate
    let numbers = vec![1, 5, 10, 15, 20, 50, 100];
    let gt10 = |x: &i32| *x > 10;

    let filtered: Vec<i32> = numbers.iter()
        .copied()
        .filter(gt10)
        .collect();

    println!("Numbers > 10: {:?}", filtered);
}

// Key differences from C:
// 1. Closures capture environment (move keyword)
// 2. impl Trait for returning closures
// 3. No manual struct wrapping needed
// 4. Type inference works seamlessly
// 5. Closures are first-class values
// 6. Can chain curried functions naturally
