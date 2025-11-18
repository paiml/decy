/* K&R C Chapter 4: External and Static Variables Interaction
 * Demonstrates interaction between external and static variables
 * Transpiled to safe Rust
 */

// Static module-level variable (visible only in this module)
static mut INTERNAL: i32 = 10;

// Public module-level variable (visible externally)
static mut EXTERNAL: i32 = 20;

fn modify_vars() {
    unsafe {
        INTERNAL += 5;
        EXTERNAL += 5;
    }
}

fn main() {
    unsafe {
        println!("Before: internal={}, external={}", INTERNAL, EXTERNAL);
        modify_vars();
        println!("After: internal={}, external={}", INTERNAL, EXTERNAL);
    }
}

// Safer Rust alternative using Cell/RefCell:
mod safe_alternative {
    use std::cell::Cell;

    // Thread-local statics with interior mutability
    thread_local! {
        static INTERNAL: Cell<i32> = Cell::new(10);
        static EXTERNAL: Cell<i32> = Cell::new(20);
    }

    fn modify_vars() {
        INTERNAL.with(|val| val.set(val.get() + 5));
        EXTERNAL.with(|val| val.set(val.get() + 5));
    }

    pub fn demo() {
        println!("\nSafe alternative using thread_local + Cell:");
        INTERNAL.with(|i| EXTERNAL.with(|e| {
            println!("Before: internal={}, external={}", i.get(), e.get());
        }));

        modify_vars();

        INTERNAL.with(|i| EXTERNAL.with(|e| {
            println!("After: internal={}, external={}", i.get(), e.get());
        }));
    }
}
