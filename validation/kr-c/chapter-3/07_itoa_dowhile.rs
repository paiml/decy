/* K&R C Chapter 3.6: Loops - Do-While
 * Page 63
 * Convert integer to string using do-while
 * Transpiled to safe Rust
 */

// itoa: convert n to characters in s (do-while version)
fn itoa(mut n: i32) -> String {
    let mut s = String::new();
    let sign: i32;

    sign = n;
    if sign < 0 {
        n = -n;
    }

    // Do-while loop transpiled to loop with explicit break
    loop {
        s.push(char::from_digit((n % 10) as u32, 10).unwrap());
        n /= 10;
        if n <= 0 {
            break;
        }
    }

    if sign < 0 {
        s.push('-');
    }

    // Reverse string
    s.chars().rev().collect()
}

fn main() {
    let buffer: String;

    buffer = itoa(12345);
    println!("12345 -> \"{}\"", buffer);

    let buffer = itoa(-6789);
    println!("-6789 -> \"{}\"", buffer);

    let buffer = itoa(0);
    println!("0 -> \"{}\"", buffer);
}
