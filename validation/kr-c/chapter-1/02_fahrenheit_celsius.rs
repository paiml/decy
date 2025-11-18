/* K&R C Chapter 1.2: Fahrenheit-Celsius Table
 * Page 8-9
 * Print Fahrenheit-Celsius table using while loop
 * Transpiled to safe Rust
 */

fn main() {
    let mut fahr: i32;
    let mut celsius: i32;
    let lower: i32;
    let upper: i32;
    let step: i32;

    lower = 0;    // lower limit of temperature table
    upper = 300;  // upper limit
    step = 20;    // step size

    fahr = lower;
    while fahr <= upper {
        celsius = 5 * (fahr - 32) / 9;
        println!("{}\t{}", fahr, celsius);
        fahr = fahr + step;
    }
}
