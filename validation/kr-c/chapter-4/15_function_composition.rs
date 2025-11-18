/* K&R C Chapter 4: Function Composition
 * Composing functions together
 * Transpiled to safe Rust (using closures and functional patterns)
 */

// Simple mathematical functions
fn add_one(x: i32) -> i32 {
    x + 1
}

fn double_it(x: i32) -> i32 {
    x * 2
}

fn square_it(x: i32) -> i32 {
    x * x
}

fn negate_it(x: i32) -> i32 {
    -x
}

// Function composition using function pointers
type UnaryFunc = fn(i32) -> i32;

fn compose(f: UnaryFunc, g: UnaryFunc, x: i32) -> i32 {
    f(g(x))
}

fn compose3(f: UnaryFunc, g: UnaryFunc, h: UnaryFunc, x: i32) -> i32 {
    f(g(h(x)))
}

// Pipeline: apply functions in sequence
fn pipeline(x: i32, functions: &[UnaryFunc]) -> i32 {
    let mut result = x;
    for &func in functions {
        result = func(result);
    }
    result
}

// Higher-order function: returns a function
fn select_operation(op: char) -> Option<UnaryFunc> {
    match op {
        '+' => Some(add_one),
        '*' => Some(double_it),
        '^' => Some(square_it),
        '-' => Some(negate_it),
        _ => None,
    }
}

// Map function over array
fn map(arr: &mut [i32], f: UnaryFunc) {
    for val in arr.iter_mut() {
        *val = f(*val);
    }
}

// Reduce/fold array
fn reduce<F>(arr: &[i32], f: F, initial: i32) -> i32
where
    F: Fn(i32, i32) -> i32,
{
    let mut result = initial;
    for &val in arr {
        result = f(result, val);
    }
    result
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

fn max_func(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

fn main() {
    println!("=== Function Composition ===\n");

    // Simple composition
    let x = 5;
    println!("x = {}", x);
    println!("add_one(x) = {}", add_one(x));
    println!("double_it(x) = {}", double_it(x));
    println!("square_it(x) = {}", square_it(x));

    // Compose two functions
    println!("\nComposed (double then add_one):");
    println!("  compose(add_one, double_it, {}) = {}", x, compose(add_one, double_it, x));

    println!("Composed (add_one then double):");
    println!("  compose(double_it, add_one, {}) = {}", x, compose(double_it, add_one, x));

    // Compose three functions
    println!("\nTriple composition:");
    println!("  compose3(add_one, double_it, square_it, {}) = {}",
             x, compose3(add_one, double_it, square_it, x));

    // Pipeline
    println!("\nPipeline:");
    let pipeline_funcs: Vec<UnaryFunc> = vec![square_it, double_it, add_one];
    println!("  square -> double -> add_one: {} -> {}",
             x, pipeline(x, &pipeline_funcs));

    // Dynamic function selection
    println!("\nDynamic selection:");
    let ops = vec!['+', '*', '^', '-'];
    for &op in &ops {
        if let Some(f) = select_operation(op) {
            println!("  op '{}': {} -> {}", op, x, f(x));
        }
    }

    // Map
    let arr = vec![1, 2, 3, 4, 5];

    println!("\nMap operations:");
    println!("Original: {:?}", arr);

    let mut arr_copy = arr.clone();
    map(&mut arr_copy, double_it);
    println!("After map(double_it): {:?}", arr_copy);

    // Reduce
    println!("\nReduce operations:");
    println!("Sum: {}", reduce(&arr, add, 0));
    println!("Product: {}", reduce(&arr, multiply, 1));
    println!("Max: {}", reduce(&arr, max_func, arr[0]));
}

// Demonstrate idiomatic Rust iterator composition
#[allow(dead_code)]
fn idiomatic_composition() {
    let arr = vec![1, 2, 3, 4, 5];

    // Chain operations using iterators
    let result: Vec<i32> = arr.iter()
        .map(|&x| square_it(x))
        .map(|x| double_it(x))
        .map(|x| add_one(x))
        .collect();

    println!("Chained: {:?}", result);

    // Reduce using fold
    let sum: i32 = arr.iter().fold(0, |acc, &x| acc + x);
    let product: i32 = arr.iter().fold(1, |acc, &x| acc * x);

    println!("Sum: {}, Product: {}", sum, product);
}
