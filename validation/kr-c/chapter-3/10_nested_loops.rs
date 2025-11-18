/* K&R C Chapter 3 Exercise: Nested loops
 * Based on Chapter 3 concepts
 * Multiplication table using nested loops
 * Transpiled to safe Rust
 */

fn main() {
    let mut i: i32;
    let mut j: i32;
    let size: i32 = 10;

    println!("Multiplication Table (1-10):");
    print!("   ");

    // Print header
    i = 1;
    while i <= size {
        print!("{:4}", i);
        i += 1;
    }
    println!();

    // Print separator
    print!("   ");
    i = 1;
    while i <= size {
        print!("----");
        i += 1;
    }
    println!();

    // Print table
    i = 1;
    while i <= size {
        print!("{:2}|", i);
        j = 1;
        while j <= size {
            print!("{:4}", i * j);
            j += 1;
        }
        println!();
        i += 1;
    }
}
