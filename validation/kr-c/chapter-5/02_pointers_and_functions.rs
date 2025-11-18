/* K&R C Chapter 5.2: Pointers and Function Arguments
 * Page 95-96
 * Pass-by-reference using pointers
 * Transpiled to safe Rust using mutable references
 */

// swap: interchange *px and *py
fn swap(px: &mut i32, py: &mut i32) {
    let temp: i32;

    temp = *px;
    *px = *py;
    *py = temp;
}

fn main() {
    let mut a: i32 = 10;
    let mut b: i32 = 20;

    println!("Before swap: a = {}, b = {}", a, b);
    swap(&mut a, &mut b);
    println!("After swap: a = {}, b = {}", a, b);
}
