/* K&R C Chapter 4: Closure Simulation
 * Transpiled to safe Rust (using native closures)
 */

fn main() {
    println!("=== Native Closures in Rust ===\n");

    // Counter closure - captures mutable state
    println!("Counter closure:");
    let mut count1 = 0;
    let mut count2 = 10;

    let mut increment1 = || {
        count1 += 1;
        count1
    };

    let mut increment2 = || {
        count2 += 1;
        count2
    };

    println!("  counter1: {}", increment1());
    println!("  counter1: {}", increment1());
    println!("  counter2: {}", increment2());
    println!("  counter1 value: {}", count1);
    println!("  counter2 value: {}", count2);

    // Adder closures - capture by value
    println!("\nAdder closures:");
    let add5 = |x| 5 + x;
    let add10 = |x| 10 + x;

    println!("  add5(3) = {}", add5(3));
    println!("  add5(7) = {}", add5(7));
    println!("  add10(3) = {}", add10(3));
    println!("  add10(7) = {}", add10(7));

    // Multiplier closures
    println!("\nMultiplier closures:");
    let double_it = |x| 2 * x;
    let triple_it = |x| 3 * x;

    println!("  double(5) = {}", double_it(5));
    println!("  triple(5) = {}", triple_it(5));

    // Accumulator closure - mutable capture
    println!("\nAccumulator closure:");
    let mut total = 0;
    let mut accumulate = |value| {
        total += value;
        total
    };

    println!("  Initial: 0");
    println!("  After add(10): {}", accumulate(10));
    println!("  After add(20): {}", accumulate(20));
    println!("  After add(-5): {}", accumulate(-5));

    // Advanced: Closure factories
    println!("\nClosure factories:");
    demo_closure_factories();

    // Closure traits
    println!("\nClosure traits:");
    demo_closure_traits();

    println!("\nClosure benefits:");
    println!("  - Native language support");
    println!("  - No manual memory management");
    println!("  - Type-safe capture");
    println!("  - Three traits: Fn, FnMut, FnOnce");
}

// Factory function that returns a closure
fn make_counter(initial: i32) -> impl FnMut() -> i32 {
    let mut count = initial;
    move || {
        count += 1;
        count
    }
}

fn make_adder(value: i32) -> impl Fn(i32) -> i32 {
    move |x| value + x
}

fn make_multiplier(multiplier: i32) -> impl Fn(i32) -> i32 {
    move |x| multiplier * x
}

fn make_accumulator(initial: i32) -> impl FnMut(i32) -> i32 {
    let mut total = initial;
    move |value| {
        total += value;
        total
    }
}

fn demo_closure_factories() {
    let mut counter = make_counter(0);
    println!("  factory counter: {}", counter());
    println!("  factory counter: {}", counter());

    let add100 = make_adder(100);
    println!("  factory add100(5) = {}", add100(5));

    let times10 = make_multiplier(10);
    println!("  factory times10(5) = {}", times10(5));

    let mut acc = make_accumulator(0);
    println!("  factory accumulator: {}", acc(10));
    println!("  factory accumulator: {}", acc(20));
}

fn demo_closure_traits() {
    // Fn - can be called multiple times, captures by reference
    let x = 5;
    let immutable_closure = || println!("  Fn closure: x = {}", x);
    call_fn(&immutable_closure);
    call_fn(&immutable_closure);

    // FnMut - can be called multiple times, mutates captures
    let mut y = 10;
    let mut mutable_closure = || {
        y += 1;
        println!("  FnMut closure: y = {}", y);
    };
    call_fn_mut(&mut mutable_closure);
    call_fn_mut(&mut mutable_closure);

    // FnOnce - can only be called once, consumes captures
    let z = vec![1, 2, 3];
    let consuming_closure = || {
        println!("  FnOnce closure: consumed vec with {} elements", z.len());
        drop(z); // Moves z
    };
    call_fn_once(consuming_closure);
}

fn call_fn<F: Fn()>(f: &F) {
    f();
}

fn call_fn_mut<F: FnMut()>(f: &mut F) {
    f();
}

fn call_fn_once<F: FnOnce()>(f: F) {
    f();
}

// Key differences from C:
// 1. Native closure syntax: |args| expr
// 2. Automatic capture of variables
// 3. Three closure traits: Fn, FnMut, FnOnce
// 4. No manual struct + function pointer
// 5. Type inference for closures
// 6. Zero-cost abstractions
// 7. RAII: automatic cleanup
// 8. Move semantics with 'move' keyword
