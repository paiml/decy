/* K&R C Chapter 5: Pointer to Function Arrays
 * Arrays of function pointers and dispatch tables
 * Transpiled to safe Rust (using fn types)
 */

// Calculator operations
fn add(a: i32, b: i32) -> i32 { a + b }
fn subtract(a: i32, b: i32) -> i32 { a - b }
fn multiply(a: i32, b: i32) -> i32 { a * b }
fn divide(a: i32, b: i32) -> i32 { if b != 0 { a / b } else { 0 } }

// Menu actions
fn action_open() { println!("  Opening file..."); }
fn action_save() { println!("  Saving file..."); }
fn action_close() { println!("  Closing file..."); }
fn action_quit() { println!("  Quitting..."); }

// State machine
#[derive(Debug)]
enum State {
    Idle,
    Running,
    Paused,
    Stopped,
}

fn idle_handler() { println!("  Idle state"); }
fn running_handler() { println!("  Running state"); }
fn paused_handler() { println!("  Paused state"); }
fn stopped_handler() { println!("  Stopped state"); }

fn main() {
    // Array of function pointers - calculator
    let operations: [fn(i32, i32) -> i32; 4] = [add, subtract, multiply, divide];
    let op_names = ["+", "-", "*", "/"];

    let x = 10;
    let y = 5;

    println!("Calculator operations on {} and {}:", x, y);
    for (i, &op) in operations.iter().enumerate() {
        let result = op(x, y);
        println!("  {} {} {} = {}", x, op_names[i], y, result);
    }

    // Array of function pointers - menu
    let menu: [fn(); 4] = [action_open, action_save, action_close, action_quit];
    let menu_names = ["Open", "Save", "Close", "Quit"];

    println!("\nMenu dispatch:");
    for (i, &action) in menu.iter().enumerate() {
        println!("Action {} ({}):", i, menu_names[i]);
        action();
    }

    // Dispatch table - state machine
    let state_handlers: [fn(); 4] = [
        idle_handler,
        running_handler,
        paused_handler,
        stopped_handler,
    ];

    println!("\nState machine:");
    for (i, &handler) in state_handlers.iter().enumerate() {
        println!("State {}:", i);
        handler();
    }

    // Function pointer table lookup
    println!("\nDirect function calls:");
    let choice = 2;  // Multiply
    if choice < operations.len() {
        println!("Selected operation: {}", op_names[choice]);
        let result = operations[choice](x, y);
        println!("Result: {}", result);
    }

    // Slice of function pointers
    let op_slice: &[fn(i32, i32) -> i32] = &operations;
    println!("\nUsing slice of functions:");
    println!("First operation: {} {} {} = {}",
             x, op_names[0], y, op_slice[0](x, y));
}

// Alternative: using enum for dispatch (more idiomatic)
#[allow(dead_code)]
enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl Operation {
    fn apply(&self, a: i32, b: i32) -> i32 {
        match self {
            Operation::Add => a + b,
            Operation::Subtract => a - b,
            Operation::Multiply => a * b,
            Operation::Divide => if b != 0 { a / b } else { 0 },
        }
    }

    fn symbol(&self) -> &str {
        match self {
            Operation::Add => "+",
            Operation::Subtract => "-",
            Operation::Multiply => "*",
            Operation::Divide => "/",
        }
    }
}

// Key differences from C:
// 1. fn(A, B) -> R instead of R (*f)(A, B)
// 2. No void* needed
// 3. Type-safe function arrays
// 4. Enum-based dispatch (more idiomatic)
// 5. Bounds checking on array access
// 6. Cannot have null function pointers
