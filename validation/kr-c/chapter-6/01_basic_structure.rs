/* K&R C Chapter 6.1: Basics of Structures
 * Page 127-128
 * Simple structure definition and usage
 * Transpiled to safe Rust
 */

struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let mut pt: Point;

    pt = Point { x: 10, y: 20 };

    println!("Point: ({}, {})", pt.x, pt.y);

    // Structure initialization
    let origin = Point { x: 0, y: 0 };
    println!("Origin: ({}, {})", origin.x, origin.y);
}
