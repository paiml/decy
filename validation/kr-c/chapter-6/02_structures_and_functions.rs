/* K&R C Chapter 6.2: Structures and Functions
 * Page 129-130
 * Passing structures to functions
 * Transpiled to safe Rust
 */

#[derive(Copy, Clone)]
struct Point {
    x: i32,
    y: i32,
}

// makepoint: make a point from x and y components
fn makepoint(x: i32, y: i32) -> Point {
    Point { x, y }
}

// addpoint: add two points
fn addpoint(mut p1: Point, p2: Point) -> Point {
    p1.x += p2.x;
    p1.y += p2.y;
    p1
}

fn main() {
    let origin: Point;
    let pt1: Point;
    let pt2: Point;
    let result: Point;

    origin = makepoint(0, 0);
    pt1 = makepoint(10, 20);
    pt2 = makepoint(30, 40);

    result = addpoint(pt1, pt2);

    println!("pt1: ({}, {})", pt1.x, pt1.y);
    println!("pt2: ({}, {})", pt2.x, pt2.y);
    println!("sum: ({}, {})", result.x, result.y);
}
