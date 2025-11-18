/* K&R C Chapter 5.3: Pointers and Arrays
 * Page 101
 * String compare using pointers
 * Transpiled to safe Rust using iterator comparison
 */

// strcmp: return <0 if s<t, 0 if s==t, >0 if s>t
fn strcmp_ptr(s: &str, t: &str) -> i32 {
    let mut s_chars = s.chars();
    let mut t_chars = t.chars();

    loop {
        match (s_chars.next(), t_chars.next()) {
            (None, None) => return 0,
            (None, Some(_)) => return -1,
            (Some(_), None) => return 1,
            (Some(sc), Some(tc)) => {
                if sc != tc {
                    return (sc as i32) - (tc as i32);
                }
            }
        }
    }
}

fn main() {
    println!("strcmp(\"abc\", \"abc\") = {}", strcmp_ptr("abc", "abc"));
    println!("strcmp(\"abc\", \"def\") = {}", strcmp_ptr("abc", "def"));
    println!("strcmp(\"def\", \"abc\") = {}", strcmp_ptr("def", "abc"));
}
