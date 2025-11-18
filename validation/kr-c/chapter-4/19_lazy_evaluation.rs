/* K&R C Chapter 4: Lazy Evaluation Pattern
 * Defer computation until value is needed
 * Transpiled to safe Rust (using closures and OnceCell)
 */

use std::cell::OnceCell;

// Thunk - deferred computation
struct Thunk<T, F>
where
    F: FnOnce() -> T,
{
    compute: Option<F>,
    cached: OnceCell<T>,
}

impl<T, F> Thunk<T, F>
where
    F: FnOnce() -> T,
{
    fn new(compute: F) -> Self {
        Thunk {
            compute: Some(compute),
            cached: OnceCell::new(),
        }
    }

    fn force(&mut self) -> &T
    where
        T: std::fmt::Display,
    {
        self.cached.get_or_init(|| {
            let compute = self.compute.take().unwrap();
            let value = compute();
            println!("    [Computed value: {}]", value);
            value
        })
    }

    fn force_quiet(&mut self) -> &T {
        self.cached.get_or_init(|| {
            let compute = self.compute.take().unwrap();
            compute()
        })
    }
}

// Lazy infinite sequence (iterator-based)
struct LazyNaturals {
    current: i32,
}

impl LazyNaturals {
    fn new() -> Self {
        LazyNaturals { current: 0 }
    }
}

impl Iterator for LazyNaturals {
    type Item = i32;

    fn next(&mut self) -> Option<i32> {
        self.current += 1;
        Some(self.current)
    }
}

fn factorial(n: i32) -> i32 {
    (1..=n).product()
}

fn main() {
    println!("=== Lazy Evaluation Pattern ===\n");

    // Simple thunks
    println!("Creating thunks (not computed yet):");
    let mut lazy_sum = Thunk::new(|| 10 + 20);
    let mut lazy_fact = Thunk::new(|| factorial(5));
    println!("  Thunks created\n");

    // Force evaluation
    println!("Forcing evaluation of lazy_sum:");
    println!("  Result: {}", lazy_sum.force());
    println!("Forcing again (should use cache):");
    print!("  ");
    if lazy_sum.cached.get().is_some() {
        println!("  [Using cached value: {}]", lazy_sum.force());
    }
    println!("  Result: {}\n", lazy_sum.force());

    println!("Forcing evaluation of lazy_fact:");
    println!("  Result: {}", lazy_fact.force());
    println!("Forcing again (should use cache):");
    if lazy_fact.cached.get().is_some() {
        println!("    [Using cached value: {}]", lazy_fact.cached.get().unwrap());
    }
    println!("  Result: {}\n", lazy_fact.force());

    // Lazy infinite sequence
    println!("Lazy infinite sequence (natural numbers):");
    let naturals = LazyNaturals::new();
    let first_10: Vec<i32> = naturals.take(10).collect();
    println!("  First 10 natural numbers: {:?}", first_10);

    // Conditional evaluation
    println!("\nConditional evaluation:");
    let use_expensive = false;
    let mut expensive = Thunk::new(|| factorial(10));

    if use_expensive {
        println!("  Using expensive computation: {}", expensive.force());
    } else {
        println!("  Skipping expensive computation (never evaluated)");
    }

    println!("\nLazy evaluation benefits:");
    println!("  - Avoid unnecessary computations");
    println!("  - Handle infinite data structures");
    println!("  - Improve performance through caching");
}

// Demonstrate lazy iteration
fn demonstrate_lazy_iteration() {
    // Only the needed values are computed
    let result: i32 = (1..1_000_000)
        .filter(|x| x % 2 == 0)
        .take(5)
        .sum();

    println!("Sum of first 5 even numbers: {}", result);
    // Note: Iterator only computed 5 values, not 1 million
}

// Lazy static with once_cell
use std::sync::LazyLock;

static EXPENSIVE_COMPUTATION: LazyLock<i32> = LazyLock::new(|| {
    println!("Computing expensive value...");
    factorial(10)
});

fn demonstrate_lazy_static() {
    println!("Before accessing EXPENSIVE_COMPUTATION");
    println!("Value: {}", *EXPENSIVE_COMPUTATION);  // Computed here
    println!("Value again: {}", *EXPENSIVE_COMPUTATION);  // Cached
}

// Key differences from C:
// 1. OnceCell for thread-safe lazy initialization
// 2. Closures capture environment automatically
// 3. Iterators are inherently lazy
// 4. No manual memory management
// 5. Type-safe lazy evaluation
// 6. LazyLock for global lazy statics
