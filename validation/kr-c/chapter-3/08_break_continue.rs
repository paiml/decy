/* K&R C Chapter 3.7: Break and Continue
 * Page 64-65
 * String trimming with break and continue
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

    s.len()
}

// count_nonwhite: count non-whitespace characters using continue
fn count_nonwhite(s: &str) -> i32 {
    let mut count: i32 = 0;

    for ch in s.chars() {
        if ch == ' ' || ch == '\t' || ch == '\n' {
            continue;  // skip whitespace
        }
        count += 1;
    }

    count
}

fn main() {
    let mut str1 = String::from("hello world    \t\n");
    let str2 = "  test  string  ";

    println!("Before trim: \"{}\" (length={})", str1, str1.len());
    trim(&mut str1);
    println!("After trim:  \"{}\" (length={})", str1, str1.len());

    println!("\nNon-whitespace chars in \"{}\": {}", str2, count_nonwhite(str2));
}
