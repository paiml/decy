/* K&R C Chapter 4.3: External Variables
 * Page 80-82
 * Simple global variable example
 * Transpiled to safe Rust
 */

static mut MAX: i32 = 0;  // Global variable (static)

fn main() {
    unsafe {
        MAX = 100;
        println!("max = {}", MAX);
    }
}
