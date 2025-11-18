/* K&R C Chapter 1.3: Fahrenheit-Celsius Table (for loop)
 * Page 13
 * Print Fahrenheit-Celsius table using for loop
 * Transpiled to safe Rust
 */

fn main() {
    let mut fahr: i32 = 0;

    while fahr <= 300 {
        println!("{:3} {:6.1}", fahr, (5.0 / 9.0) * ((fahr - 32) as f64));
        fahr = fahr + 20;
    }
}
