/* K&R C Chapter 5: Const Pointers and Pointers to Const
 * Different const pointer variations
 * Transpiled to safe Rust (using references and mut)
 */

fn main() {
    let mut x = 10;
    let mut y = 20;

    // Regular mutable reference - can change value
    let p1 = &mut x;
    println!("p1 = {}", p1);
    *p1 = 15;
    println!("After *p1 = 15: x = {}", x);

    // Can rebind to different variable
    let p1 = &mut y;
    println!("After rebinding: *p1 = {}", p1);

    // Immutable reference - cannot change value (like const T*)
    let p2 = &x;
    println!("\nImmutable reference:");
    println!("*p2 = {}", p2);
    // *p2 = 25;  // ERROR: cannot mutate through immutable reference

    // Can rebind to point to different variable
    let p2 = &y;
    println!("After rebinding: *p2 = {}", p2);

    // Mutable reference (Rust doesn't have "const pointer")
    // In Rust, the reference itself can always be rebound
    let p3 = &mut x;
    *p3 = 30;  // OK: can modify value
    println!("\nMutable reference:");
    println!("After *p3 = 30: x = {}", x);

    // Array of string slices (immutable by default)
    let messages = [
        "Hello",
        "World",
        "Const",
        "Pointers"
    ];

    println!("\nArray of string slices:");
    for (i, &msg) in messages.iter().enumerate() {
        println!("  messages[{}] = \"{}\"", i, msg);
    }
}

// Demonstrate different reference patterns
fn demonstrate_reference_patterns() {
    let mut value = 42;

    // Immutable borrow
    let r1 = &value;
    let r2 = &value;  // Multiple immutable borrows OK
    println!("{} {}", r1, r2);

    // Mutable borrow (exclusive)
    let r3 = &mut value;
    *r3 += 1;
    // let r4 = &value;  // ERROR: cannot borrow as immutable while mutable borrow exists
    println!("{}", r3);
}

// Key differences from C:
// 1. &T = immutable reference (like const T*)
// 2. &mut T = mutable reference (like T*)
// 3. No "const reference" - references can always rebind
// 4. Borrow checker enforces aliasing rules at compile time
// 5. Cannot have &T and &mut T simultaneously
// 6. No need for const correctness annotations - enforced by type system
