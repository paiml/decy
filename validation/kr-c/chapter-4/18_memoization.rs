/* K&R C Chapter 4: Memoization Pattern
 * Caching function results for performance
 * Transpiled to safe Rust (using HashMap)
 */

use std::collections::HashMap;
use std::cell::RefCell;

// Fibonacci with memoization using HashMap
struct FibMemo {
    cache: RefCell<HashMap<i32, i64>>,
}

impl FibMemo {
    fn new() -> Self {
        FibMemo {
            cache: RefCell::new(HashMap::new()),
        }
    }

    fn compute(&self, n: i32) -> i64 {
        // Check cache first
        if let Some(&result) = self.cache.borrow().get(&n) {
            return result;
        }

        // Compute
        let result = if n <= 1 {
            n as i64
        } else {
            self.compute(n - 1) + self.compute(n - 2)
        };

        // Store in cache
        self.cache.borrow_mut().insert(n, result);
        result
    }
}

// Factorial with memoization
struct FactMemo {
    cache: RefCell<HashMap<i32, i64>>,
}

impl FactMemo {
    fn new() -> Self {
        FactMemo {
            cache: RefCell::new(HashMap::new()),
        }
    }

    fn compute(&self, n: i32) -> i64 {
        if let Some(&result) = self.cache.borrow().get(&n) {
            return result;
        }

        let result = if n <= 1 {
            1
        } else {
            n as i64 * self.compute(n - 1)
        };

        self.cache.borrow_mut().insert(n, result);
        result
    }
}

// Fibonacci without memoization (for comparison)
fn fib_slow(n: i32, call_count: &mut i32) -> i64 {
    *call_count += 1;
    if n <= 1 {
        n as i64
    } else {
        fib_slow(n - 1, call_count) + fib_slow(n - 2, call_count)
    }
}

fn main() {
    println!("=== Memoization Pattern ===\n");

    // Fibonacci with memoization
    let fib_memo = FibMemo::new();

    println!("Fibonacci with memoization:");
    for i in 0..=20 {
        println!("  fib({}) = {}", i, fib_memo.compute(i));
    }

    println!("\nCalling fib_memo(20) again (cached): {}", fib_memo.compute(20));
    println!("Calling fib_memo(15) again (cached): {}", fib_memo.compute(15));

    // Factorial with memoization
    let fact_memo = FactMemo::new();

    println!("\nFactorial with memoization:");
    for i in 0..=15 {
        println!("  {}! = {}", i, fact_memo.compute(i));
    }

    // Performance comparison
    println!("\nPerformance comparison (fib(30)):");

    // Memoized version
    let fib_memo_fresh = FibMemo::new();
    println!("  Memoized: {}", fib_memo_fresh.compute(30));

    // Naive version
    let mut call_count = 0;
    let result = fib_slow(30, &mut call_count);
    println!("  Naive (no memo): {} ({} function calls)", result, call_count);

    // Memoized on second call (instant)
    println!("  Memoized (cached): {}", fib_memo_fresh.compute(30));

    println!("\nMemoization benefits:");
    println!("  - Trades memory for speed");
    println!("  - Eliminates redundant calculations");
    println!("  - Dramatic speedup for recursive functions");
}

// Alternative: Using once_cell for lazy static memoization
use std::sync::Mutex;
use std::sync::LazyLock;

static FIB_CACHE: LazyLock<Mutex<HashMap<i32, i64>>> = LazyLock::new(|| {
    Mutex::new(HashMap::new())
});

fn fib_global_memo(n: i32) -> i64 {
    let mut cache = FIB_CACHE.lock().unwrap();

    if let Some(&result) = cache.get(&n) {
        return result;
    }

    let result = if n <= 1 {
        n as i64
    } else {
        drop(cache);  // Release lock before recursion
        let res = fib_global_memo(n - 1) + fib_global_memo(n - 2);
        FIB_CACHE.lock().unwrap().insert(n, res);
        res
    };

    result
}

// Key differences from C:
// 1. HashMap instead of static arrays
// 2. RefCell for interior mutability
// 3. No manual initialization needed
// 4. Type-safe cache lookups
// 5. Automatic memory management
// 6. Thread-safe version with Mutex
