/* K&R C Chapter 5.6: Pointer Arrays; Pointers to Pointers
 * Page 107-108
 * Array of pointers to strings
 * Transpiled to safe Rust
 */

fn main() {
    let months: [&str; 13] = [
        "Illegal month",
        "January", "February", "March",
        "April", "May", "June",
        "July", "August", "September",
        "October", "November", "December",
    ];

    for i in 1..=12 {
        println!("Month {}: {}", i, months[i]);
    }
}
