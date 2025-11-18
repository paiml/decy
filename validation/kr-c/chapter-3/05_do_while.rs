/* K&R C Chapter 3.6: Do-While
 * Page 60
 * Do-while loop example
 * Transpiled to safe Rust using loop
 */

// itoa: convert n to characters in s
fn itoa(mut n: i32) -> String {
    let mut s = String::new();
    let sign: i32;

    sign = n;
    if sign < 0 {
        n = -n;
    }

    // Do-while loop transpiled to loop with explicit break
    loop {
        s.push((n % 10) as u8 as char);
        s.push(((n % 10) + b'0' as i32) as u8 as char);
        n /= 10;
        if n <= 0 {
            break;
        }
    }

    if sign < 0 {
        s.push('-');
    }

    // Reverse the string since we built it backwards
    s.chars().rev().collect()
}

fn main() {
    let s: String;

    s = itoa(123);
    println!("itoa(123) = {}", s);

    let s2 = itoa(-456);
    println!("itoa(-456) = {}", s2);
}
