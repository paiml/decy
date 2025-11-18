/* K&R C Chapter 5.3: Pointers and Arrays
 * Page 97-99
 * Pointer arithmetic with arrays
 * Transpiled to safe Rust using slices and indexing
 */

fn main() {
    let mut a: [i32; 10] = [0; 10];
    let mut i: usize;

    // Initialize array
    for i in 0..10 {
        a[i] = (i * 10) as i32;
    }

    // Safe slice indexing instead of pointer arithmetic
    println!("a[0] = {}", a[0]);
    println!("a[1] = {}", a[1]);

    // Array indexing with slices
    for i in 0..10 {
        println!("a[{}] = {}", i, a[i]);
    }
}
