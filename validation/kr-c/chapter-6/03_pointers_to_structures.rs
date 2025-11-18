/* K&R C Chapter 6.4: Pointers to Structures
 * Page 131-132
 * Pointer to structure and arrow operator
 * Transpiled to safe Rust using references
 */

#[derive(Copy, Clone)]
struct Point {
    x: i32,
    y: i32,
}

struct Rect {
    pt1: Point,
    pt2: Point,
}

fn main() {
    let pt = Point { x: 10, y: 20 };
    let pp: &Point;

    pp = &pt;

    // Two ways to access structure members via reference
    println!("pt.x = {}, pt.y = {}", pt.x, pt.y);
    println!("(*pp).x = {}, (*pp).y = {}", (*pp).x, (*pp).y);
    println!("pp.x = {}, pp.y = {}", pp.x, pp.y);  // Rust auto-dereferences

    // Nested structures
    let mut screen = Rect {
        pt1: Point { x: 0, y: 0 },
        pt2: Point { x: 0, y: 0 },
    };
    screen.pt1.x = 0;
    screen.pt1.y = 0;
    screen.pt2.x = 100;
    screen.pt2.y = 100;

    println!("Rectangle: ({},{}) to ({},{})",
           screen.pt1.x, screen.pt1.y,
           screen.pt2.x, screen.pt2.y);
}
