/* K&R C Chapter 6: Structure Copy vs Reference
 * Pass by value vs pass by pointer
 * Transpiled to safe Rust (ownership and borrowing)
 */

#[derive(Debug, Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone)]
struct Config {
    name: String,
    value: i32,
}

// Pass by value - structure copied (Copy trait)
fn modify_by_value(mut p: Point) {
    p.x = 999;
    p.y = 999;
    println!("  Inside modify_by_value: ({}, {})", p.x, p.y);
}

// Pass by mutable reference - modifies original
fn modify_by_reference(p: &mut Point) {
    p.x = 888;
    p.y = 888;
    println!("  Inside modify_by_reference: ({}, {})", p.x, p.y);
}

// Pass by immutable reference - read-only
fn read_by_reference(p: &Point) {
    println!("  Inside read_by_reference: ({}, {})", p.x, p.y);
    // p.x = 100; // ERROR: cannot mutate
}

// Return by value - structure moved/copied
fn create_point(x: i32, y: i32) -> Point {
    Point { x, y }
}

// Large structure example
struct LargeStruct {
    buffer: [u8; 1000],
    numbers: [i32; 100],
}

fn process_by_value(s: LargeStruct) {
    // Expensive: entire structure moved
    println!(
        "  Processing by value (moved {} bytes)",
        std::mem::size_of::<LargeStruct>()
    );
}

fn process_by_reference(s: &LargeStruct) {
    // Efficient: only reference passed
    println!(
        "  Processing by reference (passed {} bytes)",
        std::mem::size_of::<&LargeStruct>()
    );
}

fn process_by_mut_reference(s: &mut LargeStruct) {
    // Efficient and can modify
    s.buffer[0] = 42;
    println!(
        "  Processing by mut reference (passed {} bytes, writable)",
        std::mem::size_of::<&mut LargeStruct>()
    );
}

fn main() {
    println!("=== Structure Copy vs Reference ===\n");

    // Pass by value (Copy)
    let p1 = Point { x: 10, y: 20 };
    println!("Original: ({}, {})", p1.x, p1.y);
    modify_by_value(p1);
    println!("After modify_by_value: ({}, {}) (unchanged)\n", p1.x, p1.y);

    // Pass by mutable reference
    let mut p1 = Point { x: 10, y: 20 };
    println!("Original: ({}, {})", p1.x, p1.y);
    modify_by_reference(&mut p1);
    println!("After modify_by_reference: ({}, {}) (changed)\n", p1.x, p1.y);

    // Pass by immutable reference
    read_by_reference(&p1);
    println!();

    // Return by value
    let p2 = create_point(100, 200);
    println!("Created point: ({}, {})\n", p2.x, p2.y);

    // Structure assignment (copy for Copy types)
    let p3 = p2;
    println!(
        "p2: ({}, {}), p3: ({}, {}) (independent copies)\n",
        p2.x, p2.y, p3.x, p3.y
    );

    // Non-Copy type (String)
    let c1 = Config {
        name: "Config1".to_string(),
        value: 100,
    };

    // This moves c1, not copies
    let c2 = c1;
    // println!("{}", c1.name); // ERROR: value borrowed after move
    println!("c2: {}, value {}\n", c2.name, c2.value);

    // Clone for explicit copy
    let c3 = c2.clone();
    println!("c2: {}, value {}", c2.name, c2.value);
    println!("c3: {}, value {} (explicit clone)\n", c3.name, c3.value);

    // Large structure performance
    let mut large = LargeStruct {
        buffer: [0; 1000],
        numbers: [0; 100],
    };

    println!(
        "Large structure size: {} bytes\n",
        std::mem::size_of::<LargeStruct>()
    );

    // Cannot use process_by_value here without moving
    // process_by_value(large); // This would move large
    process_by_reference(&large);
    process_by_mut_reference(&mut large);

    // Array of structures - contiguous memory
    let configs = vec![
        Config {
            name: "Config1".to_string(),
            value: 100,
        },
        Config {
            name: "Config2".to_string(),
            value: 200,
        },
        Config {
            name: "Config3".to_string(),
            value: 300,
        },
    ];

    println!("\nArray of structures:");
    println!(
        "  sizeof(Config) = {} bytes (stack)",
        std::mem::size_of::<Config>()
    );
    println!(
        "  Vec overhead = {} bytes",
        std::mem::size_of::<Vec<Config>>()
    );
    println!("  Addresses:");
    for (i, config) in configs.iter().enumerate() {
        println!("    configs[{}] at {:p}", i, config);
    }
}

// Demonstrate borrowing rules
#[allow(dead_code)]
fn demonstrate_borrowing_rules() {
    let mut p = Point { x: 10, y: 20 };

    // Multiple immutable borrows OK
    let r1 = &p;
    let r2 = &p;
    println!("r1: ({}, {}), r2: ({}, {})", r1.x, r1.y, r2.x, r2.y);

    // Mutable borrow (exclusive)
    let r3 = &mut p;
    r3.x = 30;
    // Cannot use r1, r2, or p here while r3 is active
    println!("r3: ({}, {})", r3.x, r3.y);

    // Now p is usable again
    println!("p: ({}, {})", p.x, p.y);
}

// Key differences from C:
// 1. Copy trait for implicit copy (stack-only)
// 2. Move semantics by default
// 3. Borrowing: &T (immutable), &mut T (mutable)
// 4. Compiler enforces exclusive mutable access
// 5. No dangling references (lifetime tracking)
// 6. Clone trait for explicit deep copy
// 7. No pointer arithmetic
// 8. References are always valid (no NULL)
