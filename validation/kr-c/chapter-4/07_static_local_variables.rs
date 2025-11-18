/* K&R C Chapter 4.3: External Variables
 * Page 80-83
 * Static local variables - preserving state across calls
 * Transpiled to safe Rust using Cell for interior mutability
 */

use std::cell::Cell;

// counter: function with static local variable
fn counter() -> i32 {
    thread_local! {
        static COUNT: Cell<i32> = Cell::new(0);
    }

    COUNT.with(|count| {
        let new_val = count.get() + 1;
        count.set(new_val);
        new_val
    })
}

// generate_id: generate unique IDs
fn generate_id() -> i32 {
    thread_local! {
        static NEXT_ID: Cell<i32> = Cell::new(1000);
    }

    NEXT_ID.with(|id| {
        let current = id.get();
        id.set(current + 1);
        current
    })
}

// running_sum: keep running total
fn running_sum(value: i32) -> i32 {
    thread_local! {
        static TOTAL: Cell<i32> = Cell::new(0);
    }

    TOTAL.with(|total| {
        let new_total = total.get() + value;
        total.set(new_total);
        new_total
    })
}

fn main() {
    println!("Counter function (static local):");
    for i in 0..5 {
        println!("Call {}: count = {}", i + 1, counter());
    }

    println!("\nID generator:");
    println!("ID 1: {}", generate_id());
    println!("ID 2: {}", generate_id());
    println!("ID 3: {}", generate_id());

    println!("\nRunning sum:");
    println!("Add 10: total = {}", running_sum(10));
    println!("Add 20: total = {}", running_sum(20));
    println!("Add 30: total = {}", running_sum(30));
    println!("Add -15: total = {}", running_sum(-15));
}
