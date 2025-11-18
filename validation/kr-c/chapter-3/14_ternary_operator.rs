/* K&R C Chapter 3: Ternary Operator
 * K&R §3.5: Conditional expression (?:)
 * Tests ternary operator usage
 * Transpiled to safe Rust using if expressions
 */

fn max(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

fn min(a: i32, b: i32) -> i32 {
    if a < b { a } else { b }
}

fn abs_value(x: i32) -> i32 {
    if x < 0 { -x } else { x }
}

fn get_grade(score: i32) -> &'static str {
    if score >= 90 {
        "A"
    } else if score >= 80 {
        "B"
    } else if score >= 70 {
        "C"
    } else if score >= 60 {
        "D"
    } else {
        "F"
    }
}

fn main() {
    println!("=== Ternary Operator ===\n");

    println!("max(10, 20) = {}", max(10, 20));
    println!("min(10, 20) = {}", min(10, 20));
    println!("abs(-15) = {}\n", abs_value(-15));

    println!("Grading:");
    let scores: [i32; 5] = [95, 82, 73, 61, 45];
    for i in 0..5 {
        println!("  Score {}: Grade {}", scores[i], get_grade(scores[i]));
    }
    println!();

    let x: i32 = 5;
    println!("x={} is {}", x, if x % 2 == 0 { "even" } else { "odd" });

    println!("\nTernary operator:");
    println!("  Syntax: condition ? expr1 : expr2");
    println!("  Returns expr1 if condition true, else expr2");
    println!("  More concise than if-else for simple cases");
}
