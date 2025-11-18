/* K&R C Chapter 5.1: Pointers and Addresses
 * Page 93-94
 * Basic pointer operations
 * Transpiled to safe Rust using references
 */

fn main() {
    let mut x: i32 = 1;
    let mut y: i32 = 2;
    let mut z: [i32; 10] = [0; 10];
    let ip: &mut i32;    // ip is a mutable reference to int

    ip = &mut x;    // ip now points to x
    y = *ip;        // y is now 1
    *ip = 0;        // x is now 0

    // For array element, we need a new binding
    let ip2 = &mut z[0];  // ip2 now points to z[0]

    println!("x = {}, y = {}", x, y);
    println!("*ip2 = {}", *ip2);
}
