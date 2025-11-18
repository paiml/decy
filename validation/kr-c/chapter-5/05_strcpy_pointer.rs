/* K&R C Chapter 5.3: Pointers and Arrays
 * Page 100-101
 * String copy using pointers
 * Transpiled to safe Rust using String methods
 */

// strcpy: copy t to s (returns new String)
fn strcpy1(t: &str) -> String {
    t.to_string()
}

// strcpy: copy t to s (alternative using chars)
fn strcpy2(t: &str) -> String {
    let mut result = String::new();
    for ch in t.chars() {
        result.push(ch);
    }
    result
}

// strcpy: copy t to s (using iterator collect)
fn strcpy3(t: &str) -> String {
    t.chars().collect()
}

fn main() {
    let t = "hello, world";

    let s = strcpy1(t);
    println!("strcpy1: {}", s);

    let s = strcpy2(t);
    println!("strcpy2: {}", s);

    let s = strcpy3(t);
    println!("strcpy3: {}", s);
}
