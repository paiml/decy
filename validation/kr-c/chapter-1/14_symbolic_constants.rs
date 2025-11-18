/* K&R C Chapter 1.4: Symbolic Constants
 * Page 14-15
 * Using #define for constants
 * Transpiled to safe Rust
 */

const LOWER: i32 = 0;
const UPPER: i32 = 300;
const STEP: i32 = 20;

fn main() {
    let mut fahr: i32;

    println!("Fahrenheit-Celsius table (using symbolic constants)");
    println!("=================================================");

    fahr = LOWER;
    while fahr <= UPPER {
        println!("{:3} {:6.1}", fahr, (5.0 / 9.0) * ((fahr - 32) as f64));
        fahr = fahr + STEP;
    }
}
