/* K&R C Chapter 5.7: Multi-dimensional Arrays
 * Page 110
 * Two-dimensional array example
 * Transpiled to safe Rust
 */

fn main() {
    let mut i: usize;
    let mut j: usize;
    let matrix: [[i32; 4]; 3] = [
        [1, 2, 3, 4],
        [5, 6, 7, 8],
        [9, 10, 11, 12],
    ];

    println!("Matrix:");
    for i in 0..3 {
        for j in 0..4 {
            print!("{:3} ", matrix[i][j]);
        }
        println!();
    }
}
