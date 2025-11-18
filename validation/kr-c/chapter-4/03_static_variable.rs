/* K&R C Chapter 4.4: Static Variables
 * Page 83
 * Static variable example
 * Transpiled to safe Rust using static mut
 */

static mut COUNTER: i32 = 0;

fn increment() {
    unsafe {
        COUNTER += 1;
    }
}

fn main() {
    increment();
    increment();
    unsafe {
        println!("counter = {}", COUNTER);
    }
}
