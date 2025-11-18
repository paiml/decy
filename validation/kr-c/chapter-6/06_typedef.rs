/* K&R C Chapter 6.7: Typedef
 * Page 146-147
 * Type aliases with typedef
 * Transpiled to safe Rust using type aliases
 */

// Simple type alias
type Length = i32;

// Structure
#[derive(Copy, Clone)]
struct Point {
    x: i32,
    y: i32,
}

// Pointer type alias (using reference)
type PointPtr<'a> = &'a Point;

// Function pointer type alias
type CompareFunc = fn(*const std::ffi::c_void, *const std::ffi::c_void) -> i32;

fn main() {
    let width: Length;
    let height: Length;
    let mut origin: Point;
    let pt: Point;
    let pp: PointPtr;

    width = 100;
    height = 200;

    origin = Point { x: 0, y: 0 };

    pt = Point { x: 10, y: 20 };

    pp = &pt;

    println!("Width: {}, Height: {}", width, height);
    println!("Origin: ({}, {})", origin.x, origin.y);
    println!("Point via pointer: ({}, {})", pp.x, pp.y);
}
