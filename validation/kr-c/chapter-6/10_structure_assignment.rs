/* K&R C Chapter 6.2: Structure Assignment
 * Page 129
 * Transpiled to safe Rust (Copy and Clone traits)
 */

#[derive(Debug, Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone)]
struct Person {
    name: String,
    age: i32,
}

fn main() {
    // Initialize first structure
    let mut pt1 = Point { x: 10, y: 20 };

    // Structure assignment (Copy trait - bitwise copy)
    let mut pt2 = pt1;

    println!("pt1: ({}, {})", pt1.x, pt1.y);
    println!("pt2: ({}, {})", pt2.x, pt2.y);

    // Modify copy doesn't affect original
    pt2.x = 30;
    println!("After pt2.x = 30:");
    println!("pt1: ({}, {})", pt1.x, pt1.y);
    println!("pt2: ({}, {})", pt2.x, pt2.y);

    // Structure with String (Clone, not Copy)
    let p1 = Person {
        name: "Bob".to_string(),
        age: 25,
    };

    // Explicit clone (String is not Copy)
    let p2 = p1.clone();

    println!("p1: {}, age {}", p1.name, p1.age);
    println!("p2: {}, age {}", p2.name, p2.age);

    // Move semantics (without clone)
    let p3 = Person {
        name: "Alice".to_string(),
        age: 30,
    };

    let p4 = p3; // p3 is moved, no longer accessible
    // println!("{}", p3.name); // ERROR: value borrowed after move
    println!("p4: {}, age {}", p4.name, p4.age);
}

// Copy vs Clone
#[allow(dead_code)]
fn demonstrate_copy_vs_clone() {
    // Copy: Implicit bitwise copy (stack-only types)
    #[derive(Copy, Clone)]
    struct SmallData {
        x: i32,
        y: i32,
    }

    let a = SmallData { x: 1, y: 2 };
    let b = a; // Implicit copy
    println!("a: ({}, {})", a.x, a.y); // Still valid

    // Clone: Explicit deep copy (heap types)
    #[derive(Clone)]
    struct LargeData {
        buffer: Vec<i32>,
    }

    let c = LargeData {
        buffer: vec![1, 2, 3],
    };
    let d = c.clone(); // Explicit clone
    // let e = c; // This would move c
    println!("c buffer len: {}", c.buffer.len());
    println!("d buffer len: {}", d.buffer.len());
}

// Partial moves
#[allow(dead_code)]
fn demonstrate_partial_moves() {
    struct Data {
        id: i32,
        name: String,
    }

    let data = Data {
        id: 1,
        name: "Test".to_string(),
    };

    let id = data.id; // Copy (i32 is Copy)
    let name = data.name; // Move (String is not Copy)
    // println!("{}", data.name); // ERROR: partial move
    println!("ID: {}, Name: {}", id, name);
}

// Key differences from C:
// 1. Copy trait for implicit copy (stack-only types)
// 2. Clone trait for explicit deep copy
// 3. Move semantics by default (no copy)
// 4. Arrays in structs are copied with Copy trait
// 5. String is Clone, not Copy (heap-allocated)
// 6. Compiler tracks moves and prevents use-after-move
// 7. No shallow copy issues (String clones deep)
// 8. References (&T) to avoid copies
