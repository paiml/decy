/* K&R C Chapter 5.3: Pointers and Arrays
 * Page 99
 * String length using pointer arithmetic
 * Transpiled to safe Rust using slices and iterators
 */

// strlen: return length of string s
fn strlen_ptr(s: &str) -> i32 {
    let mut n: i32 = 0;

    for _ in s.chars() {
        n += 1;
    }
    return n;
}

// Alternative version using Rust's built-in length
fn strlen_ptr2(s: &str) -> i32 {
    return s.len() as i32;
}

fn main() {
    let str: &str = "hello";

    println!("strlen_ptr(\"{}\") = {}", str, strlen_ptr(str));
    println!("strlen_ptr2(\"{}\") = {}", str, strlen_ptr2(str));
}
