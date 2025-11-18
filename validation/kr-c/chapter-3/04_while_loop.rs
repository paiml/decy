/* K&R C Chapter 3.5: While and For Loops
 * Page 57-58
 * While loop example - string trim
 * Transpiled to safe Rust
 */

// trim: remove trailing blanks, tabs, newlines
fn trim(s: &mut String) -> usize {
    let original_len = s.len();

    while !s.is_empty() {
        let last_char = s.chars().last().unwrap();
        if last_char != ' ' && last_char != '\t' && last_char != '\n' {
            break;
        }
        s.pop();
    }

    original_len - s.len()
}

fn main() {
    let mut str = String::from("hello world   \n");

    println!("Before: '{}'", str);
    let n = trim(&mut str);
    println!("After: '{}' (removed {} chars)", str, n);
}
